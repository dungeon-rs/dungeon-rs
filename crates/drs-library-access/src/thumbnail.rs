//! `Thumbnail`: the thumbnail cache of every Asset Folder on this device, its generator, and the
//! asset source thumbnails are read through.

mod generate;
mod pack;
mod source;

pub use source::{ThumbnailTable, register_thumbnail_source};

use crate::{LibraryDirectories, LibraryError};
use bevy_math::UVec2;
use drs_model::{CaughtPanics, FolderKey};
use generate::Made;
use pack::{Digest, Record, Writer};
use std::any::Any;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::path::{Path, PathBuf};
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
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.folder.as_str().as_bytes());
        hasher.update(&[0]);
        hasher.update(self.place.as_bytes());
        hasher.update(&[0]);
        hasher.update(&self.byte_size.to_le_bytes());
        hasher.update(&self.modified.as_secs().to_le_bytes());
        hasher.update(&self.modified.subsec_nanos().to_le_bytes());
        let mut digest = [0; 16];
        digest.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
        digest
    }
}

/// What the cache holds for an Asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbnailLookup {
    /// A thumbnail this many pixels wide and high.
    Ready(UVec2),
    /// A record that the file could not be decoded as an image.
    Broken,
    /// Nothing: the thumbnail is still to be generated.
    Absent,
}

/// What is shared between the cache, its generator's threads, and nobody else.
struct Shared {
    /// Every record served, by the digest of its key.
    records: RwLock<HashMap<Digest, Record>>,
    /// What appends to the pack and the index.
    writer: Mutex<Writer>,
    /// The pack, open for positional reads.
    reader: Arc<File>,
    /// The pack's path, for errors.
    pack_path: PathBuf,
    /// What the `thumb://` source serves.
    table: ThumbnailTable,
}

impl Shared {
    /// Writes out what was appended, serves it, and hands the completions back; or, when it
    /// cannot be written, hands back that the cache failed.
    fn flush(
        &self,
        writer: &mut Writer,
        pending: &mut Vec<Completed>,
        completions: &Sender<ThumbnailCompletion>,
    ) -> bool {
        let written = writer.flush();
        let done = std::mem::take(pending);
        match written {
            Ok(records) => {
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
                        outcome: completed.outcome,
                    });
                }
                true
            }
            Err(error) => {
                let _ = completions.send(ThumbnailCompletion::CacheFailed(error.to_string()));
                false
            }
        }
    }
}

/// A thumbnail appended but not yet written out.
struct Completed {
    /// The Asset.
    key: ThumbnailKey,
    /// Where its thumbnail lies.
    record: Record,
    /// What it came to.
    outcome: ThumbnailOutcome,
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
    /// an empty one without a word, since nothing is lost but time.
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
        let reader = Arc::new(opened.reader);
        table.attach(Arc::clone(&reader));
        Ok(Self {
            shared: Arc::new(Shared {
                records: RwLock::new(opened.records),
                writer: Mutex::new(opened.writer),
                reader,
                pack_path: directory.join(pack::PACK_FILE),
                table: table.clone(),
            }),
        })
    }

    /// What the cache holds for the Asset under `key`, the last record appended for it winning.
    /// From now on the `thumb://` source serves the Asset at that place from what was found,
    /// or nothing when there is no thumbnail.
    #[must_use]
    pub fn lookup(&self, key: &ThumbnailKey) -> ThumbnailLookup {
        let record = self
            .shared
            .records
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&key.digest())
            .copied();
        self.shared.table.serve(&key.folder, &key.place, record);
        match record {
            Some(record) if record.is_broken() => ThumbnailLookup::Broken,
            Some(record) => {
                ThumbnailLookup::Ready(UVec2::new(record.width.into(), record.height.into()))
            }
            None => ThumbnailLookup::Absent,
        }
    }

    /// The encoded thumbnail of the Asset under `key`, a PNG or a JPEG, or `None` when it has no
    /// thumbnail.
    ///
    /// # Errors
    ///
    /// [`LibraryError::Io`] when the pack cannot be read.
    pub fn read(&self, key: &ThumbnailKey) -> Result<Option<Vec<u8>>, LibraryError> {
        let record = self
            .shared
            .records
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&key.digest())
            .copied();
        let Some(record) = record.filter(|record| !record.is_broken()) else {
            return Ok(None);
        };
        pack::read_entry(&self.shared.reader, record)
            .map(Some)
            .map_err(|source| LibraryError::Io {
                action: "read",
                path: self.shared.pack_path.clone(),
                source,
            })
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

/// What generating an Asset's thumbnail came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThumbnailOutcome {
    /// The thumbnail is kept and served, this many pixels wide and high.
    Ready(UVec2),
    /// The file could not be decoded as an image, which is recorded.
    Broken,
    /// The file could not be read; nothing is recorded, so it is tried again at the next start.
    Unreadable,
}

/// What the generator hands back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThumbnailCompletion {
    /// An Asset was finished.
    Finished {
        /// The Asset.
        key: ThumbnailKey,
        /// What came of it.
        outcome: ThumbnailOutcome,
    },
    /// The cache could not be written, for this reason; the generator has stopped, and the
    /// thumbnails it had not written out are lost.
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
    /// The digests of the Assets handed out.
    taken: HashSet<Digest>,
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

    /// The next Asset to generate, front first.
    fn next(&mut self) -> Option<ThumbnailJob> {
        self.settle();
        let job = self.front.pop_front().or_else(|| self.rest.pop_front())?;
        self.taken.insert(job.key.digest());
        Some(job)
    }

    /// Whether nothing is waiting.
    fn is_empty(&mut self) -> bool {
        self.settle();
        self.front.is_empty() && self.rest.is_empty()
    }

    /// Drops Assets already handed out from the head of the front and of the rest.
    fn settle(&mut self) {
        for jobs in [&mut self.front, &mut self.rest] {
            while jobs
                .front()
                .is_some_and(|job| self.taken.contains(&job.key.digest()))
            {
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
    /// [`LibraryError::Io`] when not a single thread can be started.
    pub fn start(cache: &ThumbnailCache, caught: CaughtPanics) -> Result<Self, LibraryError> {
        let count =
            std::thread::available_parallelism().map_or(1, |cores| (cores.get() / 2).max(1));
        Self::start_with(cache, count, generate::make, caught)
    }

    /// Starts `count` threads that make each thumbnail with `make`.
    ///
    /// # Errors
    ///
    /// [`LibraryError::Io`] when not a single thread can be started.
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
            return Err(LibraryError::Io {
                action: "start a thread for",
                path: PathBuf::from(THUMBNAIL_DIRECTORY),
                source,
            });
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
                    let record = writer.append(digest, Some((&bytes, width, height)));
                    pending.push(Completed {
                        key: job.key,
                        record,
                        outcome: ThumbnailOutcome::Ready(UVec2::new(width.into(), height.into())),
                    });
                }
                Ok(Made::Broken(reason)) => {
                    log::info!("{} has no thumbnail: {reason}", job.file.display());
                    let record = writer.append(digest, None);
                    pending.push(Completed {
                        key: job.key,
                        record,
                        outcome: ThumbnailOutcome::Broken,
                    });
                }
                Err(error) => {
                    log::warn!(
                        "{} could not be read for its thumbnail: {error}",
                        job.file.display()
                    );
                    let _ = self.completions.send(ThumbnailCompletion::Finished {
                        key: job.key,
                        outcome: ThumbnailOutcome::Unreadable,
                    });
                }
            }
            let drained = self.work().queue.is_empty();
            if (drained || writer.due())
                && !self
                    .shared
                    .flush(&mut writer, &mut pending, &self.completions)
            {
                self.work().stopping = true;
                self.signal.arrived.notify_all();
                return;
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
            if let Some(job) = work.queue.next() {
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
        ThumbnailKey, ThumbnailOutcome, ThumbnailTable,
    };
    use crate::LibraryDirectories;
    use bevy_math::UVec2;
    use drs_model::{CaughtPanics, FolderKey};
    use std::cell::Cell;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};
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
        std::iter::from_fn(|| queue.next())
            .map(|job| job.key.place)
            .collect()
    }

    /// The Assets last named as wanted that are still pending are generated before any other.
    #[test]
    fn wanted_assets_come_first() {
        let mut queue = Queue::default();
        queue.enqueue(["a", "b", "c", "d", "e", "f"].map(job));
        assert_eq!(queue.next().map(|job| job.key.place), Some("a".to_owned()));

        queue.want(vec![job("e"), job("a")]);
        queue.want(vec![job("d"), job("a"), job("f")]);

        assert_eq!(drain(&mut queue), vec!["d", "f", "b", "c", "e"]);
        assert!(queue.is_empty());
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
                if let ThumbnailCompletion::Finished { key, outcome } = completion {
                    finished.push((key.place, outcome));
                }
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        finished.sort_by(|a, b| a.0.cmp(&b.0));
        let ready = ThumbnailOutcome::Ready(UVec2::ONE);
        assert_eq!(
            finished,
            vec![
                ("a.png".to_owned(), ready.clone()),
                ("b.png".to_owned(), ready),
                ("panics.png".to_owned(), ThumbnailOutcome::Broken),
            ]
        );
        assert!(MARKED_WHEN_PANICKING.load(Ordering::SeqCst));
        assert!(!MARKED.get(), "the test thread was never marked");
    }
}
