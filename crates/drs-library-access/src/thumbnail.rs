//! `Thumbnail`: the thumbnail cache of every Asset Folder on this device, its generator, and the
//! asset source thumbnails are read through.

mod generate;
mod pack;
mod source;

pub use source::{ThumbnailTable, register_thumbnail_source};

use crate::{LibraryDirectories, LibraryError};
use bevy_math::UVec2;
use drs_model::{CaughtPanics, FolderKey, ThumbnailState};
use generate::Made;
use pack::{Digest, Record, Writer};
use std::any::Any;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Condvar, Mutex, PoisonError, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

/// The directory under the cache directory the thumbnail pack and its index live in.
const THUMBNAIL_DIRECTORY: &str = "thumbnails";

/// What a thumbnail is kept under: the Asset as it is on disk now.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThumbnailKey {
    /// The key of the Asset Folder the Asset sits in.
    pub folder: FolderKey,
    /// Its place in that folder.
    pub place: String,
    /// The size of its file in bytes.
    pub byte_size: u64,
    /// When its file was last modified, as time since the Unix epoch.
    pub modified: Duration,
}

impl ThumbnailKey {
    /// The digest the index records the key by.
    fn digest(&self) -> Digest {
        pack::digest(&[
            self.folder.as_str().as_bytes(),
            &[0],
            self.place.as_bytes(),
            &[0],
            &self.byte_size.to_le_bytes(),
            &self.modified.as_secs().to_le_bytes(),
            &self.modified.subsec_nanos().to_le_bytes(),
        ])
    }
}

/// What is shared between the cache, its generator's threads, and nobody else.
struct Shared {
    /// Every record served, by the digest of its key.
    records: RwLock<HashMap<Digest, Record>>,
    /// What appends to the pack and the index.
    writer: Mutex<Writer>,
    /// What the `thumb://` source serves.
    table: ThumbnailTable,
    /// How many files were found broken since that was last logged.
    broken: AtomicUsize,
}

impl Shared {
    /// Writes out what was appended, serves it, hands the completions back, and returns the
    /// digests now served; or, when it cannot be written, hands back that the cache failed and
    /// returns `None`.
    fn flush(
        &self,
        writer: &mut Writer,
        pending: &mut Vec<Completed>,
        completions: &Sender<ThumbnailCompletion>,
    ) -> Option<Vec<Digest>> {
        let written = writer.flush();
        let done = std::mem::take(pending);
        match written {
            Ok(records) => {
                let digests = records.iter().map(|(digest, _)| *digest).collect();
                {
                    let mut served = self.records.write().unwrap_or_else(PoisonError::into_inner);
                    served.extend(records);
                }
                for completed in done {
                    self.table.serve(
                        &completed.key.folder,
                        &completed.key.place,
                        Some(completed.record),
                    );
                    // A closed channel only means nobody is listening any more.
                    let _ = completions.send(ThumbnailCompletion::Finished {
                        key: completed.key,
                        state: completed.state,
                    });
                }
                Some(digests)
            }
            Err(error) => {
                let _ = completions.send(ThumbnailCompletion::CacheFailed(error.to_string()));
                None
            }
        }
    }

    /// Logs at `info` how many files were found broken since this was last logged, if any;
    /// each is logged at `debug` as it is found.
    fn log_broken(&self) {
        match self.broken.swap(0, Ordering::Relaxed) {
            0 => {}
            1 => log::info!("1 file has no thumbnail: it could not be decoded as an image"),
            count => {
                log::info!("{count} files have no thumbnail: they could not be decoded as images");
            }
        }
    }

    /// Whether a record is served for the Asset under `digest`.
    fn is_served(&self, digest: &Digest) -> bool {
        self.records
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .contains_key(digest)
    }
}

/// A thumbnail appended but not yet written out.
struct Completed {
    /// The Asset.
    key: ThumbnailKey,
    /// Where its thumbnail lies.
    record: Record,
    /// What it came to: ready or broken.
    state: ThumbnailState,
}

/// The thumbnail cache: one append-only pack of encoded thumbnails and its index, in a
/// `thumbnails` directory under the cache directory, shared by every Asset Folder.
pub struct ThumbnailCache {
    /// The state its generator shares.
    shared: Arc<Shared>,
}

impl ThumbnailCache {
    /// Opens the pack and its index, creating both when absent, and serves thumbnails out of the
    /// pack through `table`, the table the `thumb://` source reads.
    ///
    /// The index is read whole. A record that is incomplete or points beyond the end of the pack
    /// is skipped and logged; a pack or index that is not one this editor can read is replaced by
    /// an empty one with a warning in the log and no word to the Author, since nothing is lost
    /// but time.
    ///
    /// # Errors
    ///
    /// [`LibraryError::Io`] when the directory or either file cannot be created, read, or opened
    /// for appending.
    pub fn open(
        directories: &LibraryDirectories,
        table: &ThumbnailTable,
    ) -> Result<Self, LibraryError> {
        let directory = directories.cache.join(THUMBNAIL_DIRECTORY);
        let opened = pack::open(&directory)?;
        table.attach(Arc::new(opened.reader), opened.pack_path);
        Ok(Self {
            shared: Arc::new(Shared {
                records: RwLock::new(opened.records),
                writer: Mutex::new(opened.writer),
                table: table.clone(),
                broken: AtomicUsize::new(0),
            }),
        })
    }

    /// Has the `thumb://` source serve the Asset at the place `key` names from what the cache
    /// holds for it as it is now, the last record appended winning, or serve nothing there when
    /// it holds no thumbnail; and says where its thumbnail stands: ready, broken, or, with no
    /// record, pending.
    #[must_use]
    pub fn serve(&self, key: &ThumbnailKey) -> ThumbnailState {
        let record = self
            .shared
            .records
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&key.digest())
            .copied();
        self.shared.table.serve(&key.folder, &key.place, record);
        match record {
            Some(record) if record.is_broken() => ThumbnailState::Broken,
            Some(record) => {
                ThumbnailState::Ready(UVec2::new(record.width.into(), record.height.into()))
            }
            None => ThumbnailState::Pending,
        }
    }
}

/// An Asset to make a thumbnail of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThumbnailJob {
    /// The Asset as it is on disk now.
    pub key: ThumbnailKey,
    /// Its file.
    pub file: PathBuf,
}

/// What the generator hands back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThumbnailCompletion {
    /// An Asset was finished.
    Finished {
        /// The Asset.
        key: ThumbnailKey,
        /// What came of it: ready, kept and served; broken, recorded as not decodable as an
        /// image; or still pending, since the file could not be read and nothing is recorded,
        /// so it is tried again at the next start and not before.
        state: ThumbnailState,
    },
    /// The cache could not be written, for this reason; the generator has stopped, the
    /// thumbnails it had not written out are lost, and nothing more is written. Handed back once.
    CacheFailed(String),
}

/// The Assets waiting for a thumbnail: a front the caller replaces at any time, served first,
/// then the rest in the order enqueued. No Asset is handed out twice.
#[derive(Default)]
struct Queue {
    /// The Assets last named as wanted.
    front: VecDeque<ThumbnailJob>,
    /// Every Asset enqueued, in order.
    rest: VecDeque<ThumbnailJob>,
    /// The digests of the Assets handed out whose records are not served yet. A copy of one
    /// that waits further back, as when a wanted Asset also waits in the rest, or that is
    /// enqueued again meanwhile, as when a redo comes while it is in flight, is skipped rather
    /// than generated twice; once the record is served, the record itself is what skips it.
    taken: HashSet<Digest>,
    /// The digests of the Assets whose file could not be read in this session: nothing is
    /// recorded for them, so they are passed over until the next start rather than read again
    /// whenever they are wanted.
    unreadable: HashSet<Digest>,
}

impl Queue {
    /// Adds Assets to the back.
    fn enqueue(&mut self, jobs: impl IntoIterator<Item = ThumbnailJob>) {
        self.rest.extend(jobs);
    }

    /// Makes `jobs` the front, in their order, in place of the front there was.
    fn want(&mut self, jobs: Vec<ThumbnailJob>) {
        self.front = jobs.into();
    }

    /// Drops every Asset of the folder with `key` that is still waiting.
    fn withdraw(&mut self, key: &FolderKey) {
        self.front.retain(|job| &job.key.folder != key);
        self.rest.retain(|job| &job.key.folder != key);
    }

    /// Forgets that the Assets under `digests` were handed out.
    fn release(&mut self, digests: impl IntoIterator<Item = Digest>) {
        for digest in digests {
            self.taken.remove(&digest);
        }
    }

    /// Forgets that the Asset under `digest` was handed out, and passes it over from now on: its
    /// file could not be read.
    fn unreadable(&mut self, digest: Digest) {
        self.taken.remove(&digest);
        self.unreadable.insert(digest);
    }

    /// The next Asset to generate, front first, passing over those `served` says have a record.
    fn next(&mut self, served: &dyn Fn(&Digest) -> bool) -> Option<ThumbnailJob> {
        self.settle(served);
        let job = self.front.pop_front().or_else(|| self.rest.pop_front())?;
        self.taken.insert(job.key.digest());
        Some(job)
    }

    /// Whether nothing is waiting but Assets handed out or `served`.
    fn is_empty(&mut self, served: &dyn Fn(&Digest) -> bool) -> bool {
        self.settle(served);
        self.front.is_empty() && self.rest.is_empty()
    }

    /// Drops Assets already handed out, unreadable, or `served` from the head of the front and of
    /// the rest.
    fn settle(&mut self, served: &dyn Fn(&Digest) -> bool) {
        for jobs in [&mut self.front, &mut self.rest] {
            while jobs.front().is_some_and(|job| {
                let digest = job.key.digest();
                self.taken.contains(&digest)
                    || self.unreadable.contains(&digest)
                    || served(&digest)
            }) {
                jobs.pop_front();
            }
        }
    }
}

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

/// Generates thumbnails in the background on a pool of threads of its own, half the available
/// cores and at least one, so that it never competes with the asset system's decoding of
/// thumbnails for display. A panic while making one thumbnail makes that Asset broken and
/// leaves the thread serving the rest. Dropping it stops its threads after the thumbnails in
/// flight and writes out what they made.
pub struct ThumbnailGenerator {
    /// The queue the threads serve.
    signal: Arc<Signal>,
    /// The threads.
    threads: Vec<JoinHandle<()>>,
    /// The cache the threads append to.
    shared: Arc<Shared>,
    /// What the threads appended but did not write out yet.
    pending: Arc<Mutex<Vec<Completed>>>,
    /// What came of each Asset, as the threads finish them.
    completions: Mutex<Receiver<ThumbnailCompletion>>,
    /// Where the last write-out on drop hands its completions.
    sender: Sender<ThumbnailCompletion>,
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

    /// Makes `jobs` the front of the queue, generated before any other, in place of those last
    /// named.
    pub fn want(&self, jobs: Vec<ThumbnailJob>) {
        self.work().queue.want(jobs);
        self.signal.arrived.notify_all();
    }

    /// Drops every waiting Asset of the folder with `key`.
    pub fn withdraw(&self, key: &FolderKey) {
        self.work().queue.withdraw(key);
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

    /// The queue, locked.
    fn work(&self) -> std::sync::MutexGuard<'_, Work> {
        self.signal
            .work
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
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

/// One of the generator's threads.
struct Worker {
    /// The queue.
    signal: Arc<Signal>,
    /// The cache.
    shared: Arc<Shared>,
    /// What was appended but not yet written out, shared by every thread.
    pending: Arc<Mutex<Vec<Completed>>>,
    /// Where completions go.
    completions: Sender<ThumbnailCompletion>,
    /// What makes each thumbnail.
    make: Maker,
    /// What tells the crash handler that the thread catches its own panics.
    caught: CaughtPanics,
}

impl Worker {
    /// Generates thumbnails until the generator stops or the cache fails.
    fn run(self) {
        while let Some(job) = self.next() {
            let made = self.make_caught(&job.file);
            let mut writer = self
                .shared
                .writer
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
            let digest = job.key.digest();
            match made {
                Ok(Made::Thumbnail {
                    bytes,
                    width,
                    height,
                }) => {
                    if let Some(record) = writer.append(digest, Some((&bytes, width, height))) {
                        pending.push(Completed {
                            key: job.key,
                            record,
                            state: ThumbnailState::Ready(UVec2::new(width.into(), height.into())),
                        });
                    }
                }
                Ok(Made::Broken(reason)) => {
                    log::debug!("{} has no thumbnail: {reason}", job.file.display());
                    self.shared.broken.fetch_add(1, Ordering::Relaxed);
                    if let Some(record) = writer.append(digest, None) {
                        pending.push(Completed {
                            key: job.key,
                            record,
                            state: ThumbnailState::Broken,
                        });
                    }
                }
                Err(error) => {
                    log::warn!(
                        "{} could not be read for its thumbnail: {error}",
                        job.file.display()
                    );
                    self.work().queue.unreadable(digest);
                    let _ = self.completions.send(ThumbnailCompletion::Finished {
                        key: job.key,
                        state: ThumbnailState::Pending,
                    });
                }
            }
            let drained = self
                .work()
                .queue
                .is_empty(&|digest| self.shared.is_served(digest));
            if drained || writer.due() {
                let Some(served) = self
                    .shared
                    .flush(&mut writer, &mut pending, &self.completions)
                else {
                    self.work().stopping = true;
                    self.signal.arrived.notify_all();
                    return;
                };
                self.work().queue.release(served);
                if drained {
                    self.shared.log_broken();
                }
            }
        }
    }

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

    /// The queue, locked.
    fn work(&self) -> std::sync::MutexGuard<'_, Work> {
        self.signal
            .work
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// What a panic said, as far as it said it in words.
fn panic_reason(payload: &(dyn Any + Send)) -> String {
    if let Some(reason) = payload.downcast_ref::<&str>() {
        (*reason).to_owned()
    } else if let Some(reason) = payload.downcast_ref::<String>() {
        reason.clone()
    } else {
        "no reason given".to_owned()
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        reason = "a test stops at the first thing that is not as expected"
    )]

    use super::{
        Made, Queue, ThumbnailCache, ThumbnailCompletion, ThumbnailGenerator, ThumbnailJob,
        ThumbnailKey, ThumbnailTable,
    };
    use crate::LibraryDirectories;
    use bevy_math::UVec2;
    use drs_model::{CaughtPanics, FolderKey, ThumbnailState};
    use std::cell::Cell;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    /// An Asset of the folder `maps` at `place`.
    fn job(place: &str) -> ThumbnailJob {
        ThumbnailJob {
            key: ThumbnailKey {
                folder: FolderKey("maps".to_owned()),
                place: place.to_owned(),
                byte_size: 7,
                modified: Duration::from_secs(1),
            },
            file: PathBuf::from(place),
        }
    }

    /// Every Asset the queue hands out, in order.
    fn drain(queue: &mut Queue) -> Vec<String> {
        std::iter::from_fn(|| queue.next(&|_| false))
            .map(|job| job.key.place)
            .collect()
    }

    /// The Assets last named as wanted that are still pending are generated before any other.
    #[test]
    fn wanted_assets_come_first() {
        let mut queue = Queue::default();
        queue.enqueue(["a", "b", "c", "d", "e", "f"].map(job));
        assert_eq!(
            queue.next(&|_| false).map(|job| job.key.place),
            Some("a".to_owned())
        );

        queue.want(vec![job("e"), job("a")]);
        queue.want(vec![job("d"), job("a"), job("f")]);

        assert_eq!(drain(&mut queue), vec!["d", "f", "b", "c", "e"]);
        assert!(queue.is_empty(&|_| false));
    }

    /// How many times the file that cannot be read was tried.
    static UNREADABLE_TRIED: AtomicUsize = AtomicUsize::new(0);

    /// Makes a one-pixel thumbnail, except for `unreadable.png`, which cannot be read.
    #[expect(
        clippy::missing_errors_doc,
        reason = "it stands in for the maker, which reads a file"
    )]
    fn make_or_fail(path: &Path) -> std::io::Result<Made> {
        if path.ends_with("unreadable.png") {
            UNREADABLE_TRIED.fetch_add(1, Ordering::SeqCst);
            return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
        }
        Ok(Made::Thumbnail {
            bytes: vec![1, 2, 3],
            width: 1,
            height: 1,
        })
    }

    /// Waits for the completion of the Asset at `place`, handing back every other on the way.
    fn finished(generator: &ThumbnailGenerator, place: &str) -> Vec<(String, ThumbnailState)> {
        let start = Instant::now();
        let mut finished = Vec::new();
        while !finished.iter().any(|(at, _)| at == place) {
            assert!(start.elapsed() < Duration::from_secs(30), "{finished:?}");
            for completion in generator.completions() {
                if let ThumbnailCompletion::Finished { key, state } = completion {
                    finished.push((key.place, state));
                }
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        finished
    }

    /// A file that cannot be read stays pending and is read once a session, however often it is
    /// wanted or enqueued again.
    #[test]
    fn an_unreadable_file_is_read_once_a_session() {
        let root = tempfile::TempDir::new().expect("temporary root");
        let directories = LibraryDirectories {
            configuration: root.path().join("configuration"),
            cache: root.path().join("cache"),
        };
        let cache =
            ThumbnailCache::open(&directories, &ThumbnailTable::default()).expect("the cache");
        let generator =
            ThumbnailGenerator::start_with(&cache, 1, make_or_fail, CaughtPanics::default())
                .expect("the thread");

        generator.enqueue([job("unreadable.png")]);
        let first = finished(&generator, "unreadable.png");
        generator.want(vec![job("unreadable.png")]);
        generator.enqueue([job("unreadable.png"), job("a.png")]);
        let then = finished(&generator, "a.png");

        assert_eq!(
            first,
            vec![("unreadable.png".to_owned(), ThumbnailState::Pending)]
        );
        assert_eq!(
            then,
            vec![("a.png".to_owned(), ThumbnailState::Ready(UVec2::ONE))]
        );
        assert_eq!(UNREADABLE_TRIED.load(Ordering::SeqCst), 1);
    }

    thread_local! {
        /// Whether the thread is marked as catching its own panics.
        static MARKED: Cell<bool> = const { Cell::new(false) };
    }

    /// Whether the thread that panicked was marked when it did.
    static MARKED_WHEN_PANICKING: AtomicBool = AtomicBool::new(false);

    /// Marks the thread as the crash handler's marker would.
    fn mark(caught: bool) {
        MARKED.set(caught);
    }

    /// Makes a one-pixel thumbnail, except for `panics.png`, whose decoder falls over.
    #[expect(
        clippy::unnecessary_wraps,
        clippy::missing_errors_doc,
        reason = "it stands in for the maker, which reads a file"
    )]
    fn make_or_panic(path: &Path) -> std::io::Result<Made> {
        if path.ends_with("panics.png") {
            MARKED_WHEN_PANICKING.store(MARKED.get(), Ordering::SeqCst);
            panic!("the decoder fell over");
        }
        Ok(Made::Thumbnail {
            bytes: vec![1, 2, 3],
            width: 1,
            height: 1,
        })
    }

    /// A panic while decoding one file makes that Asset broken, on a thread marked as catching it,
    /// and the thread carries on with the remaining thumbnails.
    #[test]
    fn a_decoder_panic_is_broken_and_the_rest_carry_on() {
        let root = tempfile::TempDir::new().expect("temporary root");
        let directories = LibraryDirectories {
            configuration: root.path().join("configuration"),
            cache: root.path().join("cache"),
        };
        let cache =
            ThumbnailCache::open(&directories, &ThumbnailTable::default()).expect("the cache");
        let generator =
            ThumbnailGenerator::start_with(&cache, 1, make_or_panic, CaughtPanics(mark))
                .expect("the thread");

        generator.enqueue(["a.png", "panics.png", "b.png"].map(job));

        let start = Instant::now();
        let mut finished = Vec::new();
        while finished.len() < 3 {
            assert!(start.elapsed() < Duration::from_secs(30), "{finished:?}");
            for completion in generator.completions() {
                if let ThumbnailCompletion::Finished { key, state } = completion {
                    finished.push((key.place, state));
                }
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        finished.sort_by(|a, b| a.0.cmp(&b.0));
        let ready = ThumbnailState::Ready(UVec2::ONE);
        assert_eq!(
            finished,
            vec![
                ("a.png".to_owned(), ready),
                ("b.png".to_owned(), ready),
                ("panics.png".to_owned(), ThumbnailState::Broken),
            ]
        );
        assert!(MARKED_WHEN_PANICKING.load(Ordering::SeqCst));
        assert!(!MARKED.get(), "the test thread was never marked");
    }
}
