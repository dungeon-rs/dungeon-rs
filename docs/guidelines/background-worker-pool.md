# Background worker pool in a ResourceAccess

**Use when**: a ResourceAccess runs a queue of jobs that a Manager feeds, on threads of its own, and hands the results back for the Manager to write into the World (generating thumbnails). **Not when**: the asset system can do the work in its own load tasks behind an asset source (decoding a thumbnail for display through `thumb://`), or the work is short enough to run inside the Manager's handler.
**Exemplar**: `crates/drs-library-access/src/thumbnail.rs`

## Rules

- The pool is one public struct (`ThumbnailGenerator`) whose `start` spawns `std::thread::Builder` threads named `<pool>-<index>`, half of `available_parallelism()` and at least one, and returns the crate's own variant (`LibraryError::ThreadNotStarted`) only when not a single thread started. The Manager keeps it in its `Resource` as an `Option` and stops it by setting that to `None`, on `AppExit` in `Last` and when the pool reports that it failed.
- The queue and the `stopping` flag live together in one `Mutex<Work>` beside a `Condvar` in an `Arc<Signal>`; `enqueue`, `want`, and `Drop` call `notify_all` after changing them, and a thread's `next()` checks `stopping` before taking a job and waits on the `Condvar` in a loop. Every lock is taken with `.unwrap_or_else(PoisonError::into_inner)`. A job is taken out under the lock and made after it is released.
- Results go back as one `pub enum` (`ThumbnailCompletion`) through `std::sync::mpsc`; the `Receiver` sits in a `Mutex` so the pool can live in a `Resource`, and `completions()` drains it with `try_iter()`. The Manager calls it once a frame in its `Commands` set, before any Command is handled. A failed `send` is ignored with a comment saying nobody is listening any more.
- Each job runs under `std::panic::catch_unwind`, between `(self.caught.0)(true)` and `(self.caught.0)(false)`, where `caught` is the `CaughtPanics` the Manager took from the World with `get_resource::<CaughtPanics>().copied().unwrap_or_default()` and passed to `start`. A panic becomes that job's ordinary failure, logged at `warn` with its `panic_reason`.
- The job itself is a `fn` pointer field of the worker (`Maker`), set by a private `start_with(cache, count, make, caught)` that `start` calls with the real one, so a unit test can start one thread whose job panics.
- `Drop` sets `stopping`, notifies, joins every thread (logging a failed join at `warn`), then writes out what the threads left behind; it never waits for the queue. A failure that makes every later job pointless (the cache cannot be written) is sent once as its own variant, sets `stopping`, and ends the thread.
- A failure of one job that a library may hold by the thousand (a file that is not an image) is logged at `debug` and counted, and the count is logged at `info` when the queue drains and when the pool stops.

## Example

```rust
/// The queue and whether the generator is stopping, guarded together.
#[derive(Default)]
struct Work {
    /// What waits.
    queue: Queue,
    /// Whether the threads are to stop.
    stopping: bool,
}

/// What the generator's threads wait on.
#[derive(Default)]
struct Signal {
    /// The queue.
    work: Mutex<Work>,
    /// Woken when work arrives or the generator stops.
    arrived: Condvar,
}

impl ThumbnailGenerator {
    /// Starts the threads, idle until Assets are enqueued; each marks itself with `caught` while
    /// it makes a thumbnail, since it catches a panic in doing so itself.
    ///
    /// # Errors
    ///
    /// [`LibraryError::ThreadNotStarted`] when not a single thread can be started.
    pub fn start(cache: &ThumbnailCache, caught: CaughtPanics) -> Result<Self, LibraryError> {
        let count =
            std::thread::available_parallelism().map_or(1, |cores| (cores.get() / 2).max(1));
        Self::start_with(cache, count, generate::make, caught)
    }

    /// Starts `count` threads that make each thumbnail with `make`.
    ///
    /// # Errors
    ///
    /// [`LibraryError::ThreadNotStarted`] when not a single thread can be started.
    fn start_with(
        cache: &ThumbnailCache,
        count: usize,
        make: Maker,
        caught: CaughtPanics,
    ) -> Result<Self, LibraryError> {
        let signal = Arc::new(Signal::default());
        let (sender, receiver) = channel();
        let pending = Arc::new(Mutex::new(Vec::new()));
        let mut threads = Vec::with_capacity(count);
        let mut failure = None;
        for index in 0..count {
            let worker = Worker {
                signal: Arc::clone(&signal),
                shared: Arc::clone(&cache.shared),
                pending: Arc::clone(&pending),
                completions: sender.clone(),
                make,
                caught,
            };
            match std::thread::Builder::new()
                .name(format!("thumbnails-{index}"))
                .spawn(move || worker.run())
            {
                Ok(thread) => threads.push(thread),
                Err(error) => failure = Some(error),
            }
        }
        if threads.is_empty()
            && let Some(source) = failure
        {
            return Err(LibraryError::ThreadNotStarted(source));
        }
        Ok(Self {
            signal,
            threads,
            shared: Arc::clone(&cache.shared),
            pending,
            completions: Mutex::new(receiver),
            sender,
        })
    }

    /// Adds Assets to the back of the queue, in order.
    pub fn enqueue(&self, jobs: impl IntoIterator<Item = ThumbnailJob>) {
        self.work().queue.enqueue(jobs);
        self.signal.arrived.notify_all();
    }

    /// What came of the Assets finished since the last call.
    #[must_use]
    pub fn completions(&self) -> Vec<ThumbnailCompletion> {
        self.completions
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .try_iter()
            .collect()
    }
}

impl Drop for ThumbnailGenerator {
    fn drop(&mut self) {
        self.work().stopping = true;
        self.signal.arrived.notify_all();
        for thread in self.threads.drain(..) {
            if thread.join().is_err() {
                log::warn!("a thumbnail thread stopped unexpectedly");
            }
        }
        let mut writer = self
            .shared
            .writer
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        let _ = self.shared.flush(&mut writer, &mut pending, &self.sender);
        self.shared.log_broken();
    }
}

/// What makes one thumbnail out of an image file.
type Maker = fn(&Path) -> std::io::Result<Made>;

impl Worker {
    /// Makes the thumbnail of `file`; a panic in doing so is caught and makes the Asset broken,
    /// so that one file that brings its decoder down costs neither the thread nor the queue.
    ///
    /// # Errors
    ///
    /// The error of reading the file.
    fn make_caught(&self, file: &Path) -> std::io::Result<Made> {
        let make = self.make;
        (self.caught.0)(true);
        let outcome = std::panic::catch_unwind(|| make(file));
        (self.caught.0)(false);
        outcome.unwrap_or_else(|payload| {
            let reason = panic_reason(payload.as_ref());
            log::warn!(
                "{} has no thumbnail: decoding it panicked: {reason}",
                file.display()
            );
            Ok(Made::Broken(format!("decoding it panicked: {reason}")))
        })
    }

    /// The next Asset to generate, waiting for one; `None` once the generator stops.
    fn next(&self) -> Option<ThumbnailJob> {
        let mut work = self.work();
        loop {
            if work.stopping {
                return None;
            }
            if let Some(job) = work.queue.next(&|digest| self.shared.is_served(digest)) {
                return Some(job);
            }
            work = self
                .signal
                .arrived
                .wait(work)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }
}
```

## Pitfalls

- Catching the panic without marking the thread: the pool survives, yet the crash handler writes a crash report and the Author is told the editor crashed.
- Making the job while holding the queue's lock: every other thread, and the main thread's `enqueue` and `want`, wait for the slowest file.
- Taking the pool's locks in another order on another path: the exemplar always takes the writer before the queue and the queue before the served records, and a path that reverses two of them deadlocks under load.
- Waiting on the `Condvar` without checking `stopping` first in the same loop: a `Drop` that notified before the thread began to wait is missed, and `join` never returns.
