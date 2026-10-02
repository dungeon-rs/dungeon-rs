//! The thumbnail pack and its index on disk.
//!
//! The pack is a 16-byte header (magic, version, four reserved bytes) followed by the encoded
//! thumbnails back to back. The index is a 16-byte header of its own followed by 32-byte records,
//! one per thumbnail generated: the 16-byte digest of the Asset's key, the entry's offset (`u64`)
//! and length (`u32`) in the pack, and the thumbnail's width and height (`u16` each), all
//! little-endian. A record with a length of zero says the Asset is broken. Both files are only
//! ever appended to while thumbnails are generated: the bytes go to the pack before the record
//! goes to the index, so a crash between the two leaves at worst bytes nothing refers to.

use crate::LibraryError;
use crate::manifest::write_atomically;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The pack's file name in the thumbnail directory.
pub(crate) const PACK_FILE: &str = "thumbnails.pack";
/// The index's file name in the thumbnail directory.
pub(crate) const INDEX_FILE: &str = "thumbnails.index";
/// What the pack starts with.
const PACK_MAGIC: [u8; 8] = *b"DRSTHPK\0";
/// What the index starts with.
const INDEX_MAGIC: [u8; 8] = *b"DRSTHIX\0";
/// The version of both files' layout.
const VERSION: u32 = 1;
/// The length of either file's header.
const HEADER_LEN: usize = 16;
/// The length of one index record.
const RECORD_LEN: usize = 32;
/// How many thumbnails are kept in memory before they are written out.
const FLUSH_EVERY: usize = 256;
/// How long a generated thumbnail waits in memory at most before it is written out.
const FLUSH_AFTER: Duration = Duration::from_millis(100);

/// The digest of an Asset's key: its folder key, place, byte size, and modification time.
pub(crate) type Digest = [u8; 16];

/// Where one thumbnail lies in the pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Record {
    /// Where the entry starts in the pack.
    pub offset: u64,
    /// How long the entry is; zero for an Asset that is broken.
    pub len: u32,
    /// The thumbnail's width in pixels.
    pub width: u16,
    /// The thumbnail's height in pixels.
    pub height: u16,
}

impl Record {
    /// Whether the record says the Asset is broken.
    pub(crate) fn is_broken(self) -> bool {
        self.len == 0
    }

    /// The record as the index holds it.
    fn encode(self, digest: &Digest) -> [u8; RECORD_LEN] {
        let mut bytes = [0; RECORD_LEN];
        bytes[..16].copy_from_slice(digest);
        bytes[16..24].copy_from_slice(&self.offset.to_le_bytes());
        bytes[24..28].copy_from_slice(&self.len.to_le_bytes());
        bytes[28..30].copy_from_slice(&self.width.to_le_bytes());
        bytes[30..32].copy_from_slice(&self.height.to_le_bytes());
        bytes
    }

    /// A record as the index holds it.
    fn decode(bytes: &[u8; RECORD_LEN]) -> (Digest, Self) {
        let mut digest = [0; 16];
        digest.copy_from_slice(&bytes[..16]);
        let mut offset = [0; 8];
        offset.copy_from_slice(&bytes[16..24]);
        let mut len = [0; 4];
        len.copy_from_slice(&bytes[24..28]);
        let record = Self {
            offset: u64::from_le_bytes(offset),
            len: u32::from_le_bytes(len),
            width: u16::from_le_bytes([bytes[28], bytes[29]]),
            height: u16::from_le_bytes([bytes[30], bytes[31]]),
        };
        (digest, record)
    }

    /// Whether the entry lies inside a pack this long.
    fn fits(self, pack_len: u64) -> bool {
        self.is_broken()
            || (self.offset >= HEADER_LEN as u64
                && self
                    .offset
                    .checked_add(u64::from(self.len))
                    .is_some_and(|end| end <= pack_len))
    }
}

/// A file's header.
fn header(magic: [u8; 8]) -> [u8; HEADER_LEN] {
    let mut bytes = [0; HEADER_LEN];
    bytes[..8].copy_from_slice(&magic);
    bytes[8..12].copy_from_slice(&VERSION.to_le_bytes());
    bytes
}

/// The pack and index as they were found when opened.
pub(crate) struct Opened {
    /// Every complete record, the last one appended for a digest winning.
    pub records: HashMap<Digest, Record>,
    /// What appends to the pack and the index.
    pub writer: Writer,
    /// The pack, open for positional reads.
    pub reader: File,
}

/// Opens the pack and index in `directory`, creating the directory and both files when absent.
///
/// A pack or index that does not start with its header is replaced, with the other, by an empty
/// one. A record that is incomplete or whose entry ends beyond the pack is skipped and logged,
/// and the index is written again without it, so that later appends line up and the record can
/// never come to point at a later entry.
///
/// # Errors
///
/// [`LibraryError::Io`] when the directory cannot be created, a file cannot be read or replaced,
/// or either file cannot be opened for appending.
pub(crate) fn open(directory: &Path) -> Result<Opened, LibraryError> {
    let io = |action: &'static str, path: &Path| {
        let path = path.to_path_buf();
        move |source: std::io::Error| LibraryError::Io {
            action,
            path,
            source,
        }
    };
    std::fs::create_dir_all(directory).map_err(io("create", directory))?;
    let pack_path = directory.join(PACK_FILE);
    let index_path = directory.join(INDEX_FILE);

    let pack_ok = starts_with(&pack_path, &header(PACK_MAGIC)).map_err(io("read", &pack_path))?;
    let index = match std::fs::read(&index_path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(source) => return Err(io("read", &index_path)(source)),
    };
    let index_ok = index
        .as_deref()
        .is_some_and(|bytes| bytes.starts_with(&header(INDEX_MAGIC)));
    let mut records = HashMap::new();
    if pack_ok && index_ok {
        let pack_len = std::fs::metadata(&pack_path)
            .map_err(io("read", &pack_path))?
            .len();
        let bytes = index.as_deref().unwrap_or_default();
        let kept = complete_records(bytes, pack_len, &index_path);
        if kept.len() * RECORD_LEN + HEADER_LEN != bytes.len() {
            let mut rewritten = header(INDEX_MAGIC).to_vec();
            for (digest, record) in &kept {
                rewritten.extend_from_slice(&record.encode(digest));
            }
            write_atomically(&index_path, &rewritten)?;
        }
        records.extend(kept);
    } else {
        if pack_ok || index.is_some() {
            log::info!(
                "the thumbnail cache in {} is not one this editor can read; it starts afresh",
                directory.display()
            );
        }
        write_atomically(&pack_path, &header(PACK_MAGIC))?;
        write_atomically(&index_path, &header(INDEX_MAGIC))?;
    }

    let append = |path: &Path| {
        OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(io("open for appending", path))
    };
    let pack = append(&pack_path)?;
    let index = append(&index_path)?;
    let pack_end = pack.metadata().map_err(io("read", &pack_path))?.len();
    let reader = File::open(&pack_path).map_err(io("open", &pack_path))?;
    Ok(Opened {
        records,
        writer: Writer {
            pack,
            index,
            pack_path,
            index_path,
            pack_end,
            pack_pending: Vec::new(),
            index_pending: Vec::new(),
            appended: Vec::new(),
            since: None,
        },
        reader,
    })
}

/// Whether the file at `path` starts with `expected`; a missing or shorter file does not.
///
/// # Errors
///
/// The error of opening or reading a file that exists.
fn starts_with(path: &Path, expected: &[u8]) -> std::io::Result<bool> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    let mut found = vec![0; expected.len()];
    match file.read_exact(&mut found) {
        Ok(()) => Ok(found == expected),
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => Ok(false),
        Err(error) => Err(error),
    }
}

/// The complete records of an index whose header has been checked, in the order appended,
/// leaving out and logging an incomplete last record and every record whose entry does not lie
/// inside a pack of `pack_len` bytes.
fn complete_records(index: &[u8], pack_len: u64, path: &Path) -> Vec<(Digest, Record)> {
    let body = index.get(HEADER_LEN..).unwrap_or_default();
    let (chunks, remainder) = body.as_chunks::<RECORD_LEN>();
    if !remainder.is_empty() {
        log::warn!(
            "the last record of the thumbnail index {} is incomplete and is skipped",
            path.display()
        );
    }
    let mut kept = Vec::with_capacity(chunks.len());
    let mut beyond = 0_usize;
    for chunk in chunks {
        let (digest, record) = Record::decode(chunk);
        if record.fits(pack_len) {
            kept.push((digest, record));
        } else {
            beyond += 1;
        }
    }
    if beyond > 0 {
        log::warn!(
            "{beyond} records of the thumbnail index {} point beyond the pack and are skipped",
            path.display()
        );
    }
    kept
}

/// Appends thumbnails to the pack and their records to the index, keeping them in memory until
/// they are flushed.
pub(crate) struct Writer {
    /// The pack, open for appending.
    pack: File,
    /// The index, open for appending.
    index: File,
    /// The pack's path, for errors.
    pack_path: PathBuf,
    /// The index's path, for errors.
    index_path: PathBuf,
    /// The pack's length on disk.
    pack_end: u64,
    /// Entries not yet written to the pack.
    pack_pending: Vec<u8>,
    /// Records not yet written to the index.
    index_pending: Vec<u8>,
    /// What was appended since the last flush, in order.
    appended: Vec<(Digest, Record)>,
    /// When the oldest thumbnail not yet written was appended.
    since: Option<Instant>,
}

impl Writer {
    /// Appends a thumbnail's encoded bytes and its size, or with `None` records the Asset as
    /// broken; the record is returned. Nothing reaches the files until [`Self::flush`].
    pub(crate) fn append(
        &mut self,
        digest: Digest,
        thumbnail: Option<(&[u8], u16, u16)>,
    ) -> Record {
        let offset = self.pack_end + self.pack_pending.len() as u64;
        let record = match thumbnail {
            Some((bytes, width, height)) => {
                self.pack_pending.extend_from_slice(bytes);
                Record {
                    offset,
                    len: u32::try_from(bytes.len()).unwrap_or(u32::MAX),
                    width,
                    height,
                }
            }
            None => Record {
                offset,
                len: 0,
                width: 0,
                height: 0,
            },
        };
        self.index_pending
            .extend_from_slice(&record.encode(&digest));
        self.appended.push((digest, record));
        self.since.get_or_insert_with(Instant::now);
        record
    }

    /// Whether enough is kept in memory, or for long enough, that it should be flushed.
    pub(crate) fn due(&self) -> bool {
        self.appended.len() >= FLUSH_EVERY
            || self
                .since
                .is_some_and(|since| since.elapsed() >= FLUSH_AFTER)
    }

    /// Writes what was appended to the pack and then to the index, and returns it.
    ///
    /// # Errors
    ///
    /// [`LibraryError::Io`] when either file cannot be written; what was appended is then
    /// dropped.
    pub(crate) fn flush(&mut self) -> Result<Vec<(Digest, Record)>, LibraryError> {
        self.since = None;
        let pack = std::mem::take(&mut self.pack_pending);
        let index = std::mem::take(&mut self.index_pending);
        let appended = std::mem::take(&mut self.appended);
        if appended.is_empty() {
            return Ok(appended);
        }
        self.pack
            .write_all(&pack)
            .and_then(|()| self.pack.flush())
            .map_err(|source| LibraryError::Io {
                action: "write",
                path: self.pack_path.clone(),
                source,
            })?;
        self.pack_end += pack.len() as u64;
        self.index
            .write_all(&index)
            .and_then(|()| self.index.flush())
            .map_err(|source| LibraryError::Io {
                action: "write",
                path: self.index_path.clone(),
                source,
            })?;
        Ok(appended)
    }
}

/// Reads one entry's bytes from the pack at its record's offset, without moving any cursor that
/// another reader relies on.
///
/// # Errors
///
/// The error of reading the file, including an entry that ends beyond it.
pub(crate) fn read_entry(pack: &File, record: Record) -> std::io::Result<Vec<u8>> {
    let mut bytes = vec![0; record.len as usize];
    read_at(pack, &mut bytes, record.offset)?;
    Ok(bytes)
}

/// A positional read of exactly `buffer.len()` bytes at `offset`.
///
/// # Errors
///
/// The error of reading, including the end of the file coming first.
#[cfg(unix)]
fn read_at(file: &File, buffer: &mut [u8], offset: u64) -> std::io::Result<()> {
    use std::os::unix::fs::FileExt;
    file.read_exact_at(buffer, offset)
}

/// A positional read of exactly `buffer.len()` bytes at `offset`.
///
/// # Errors
///
/// The error of reading, including the end of the file coming first.
#[cfg(windows)]
fn read_at(file: &File, buffer: &mut [u8], offset: u64) -> std::io::Result<()> {
    use std::os::windows::fs::FileExt;
    let mut done = 0;
    while done < buffer.len() {
        let read = file.seek_read(&mut buffer[done..], offset + done as u64)?;
        if read == 0 {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        done += read;
    }
    Ok(())
}
