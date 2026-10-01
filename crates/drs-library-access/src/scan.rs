//! Scanning an Asset Folder: a walk over directory entries and file metadata, diffed against the
//! folder's index cache.

use crate::manifest::write_atomically;
use crate::{LibraryDirectories, LibraryError, LibraryTable};
use drs_model::{FolderKey, ScanSkips};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

/// One file found in an Asset Folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannedFile {
    /// The file's relative path in the folder with `/` separators, as spelled on disk.
    pub place: String,
    /// The size of the file in bytes.
    pub byte_size: u64,
    /// When the file was last modified, as time since the Unix epoch.
    pub modified: Duration,
}

/// What changed in a folder since its index cache was written, by place.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanDiff {
    /// Files that are new.
    pub added: Vec<String>,
    /// Files that are gone.
    pub removed: Vec<String>,
    /// Files whose size or modification time differs.
    pub changed: Vec<String>,
}

impl ScanDiff {
    /// Whether nothing changed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }
}

/// The result of scanning an Asset Folder.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scan {
    /// Every file found, ordered by place, whether or not it is an Asset: the scan and the index
    /// cache it is diffed against know files by their metadata alone, and which files are
    /// Assets is decided afterwards by classifying them. Hidden entries and symbolic links are
    /// left out.
    pub files: Vec<ScannedFile>,
    /// What was skipped.
    pub skips: ScanSkips,
    /// What changed since the folder's index cache was written; everything is added on the first
    /// scan.
    pub diff: ScanDiff,
}

/// `ScanFolder`: walks the folder at `path` reading directory entries and file metadata only,
/// diffs the result against the folder's index cache, rewrites the cache, and registers the
/// folder with the `lib://` source so that its Assets can be loaded.
///
/// Hidden entries (names beginning with a dot) are not scanned, symbolic links are neither
/// indexed nor followed, and names that are not valid Unicode are counted and skipped. Nothing
/// is written into the folder.
///
/// # Errors
///
/// [`LibraryError::Io`] when the folder itself cannot be listed or the cache cannot be written,
/// or [`LibraryError::UnencodableIndex`] when the cache cannot be encoded.
pub fn scan_folder(
    directories: &LibraryDirectories,
    table: &LibraryTable,
    key: &FolderKey,
    path: &Path,
) -> Result<Scan, LibraryError> {
    let mut scan = Scan::default();
    walk(path, path, &mut scan)?;
    scan.files.sort_by(|a, b| a.place.cmp(&b.place));

    let cache_file = directories.index_cache_file(key);
    let cached = read_cache(&cache_file);
    scan.diff = diff(&cached, &scan.files);

    std::fs::create_dir_all(&directories.cache).map_err(|source| LibraryError::Io {
        action: "create",
        path: directories.cache.clone(),
        source,
    })?;
    let text =
        serde_json::to_string(&scan.files).map_err(|error| LibraryError::UnencodableIndex {
            path: cache_file.clone(),
            reason: error.to_string(),
        })?;
    write_atomically(&cache_file, text.as_bytes())?;

    table.insert(key, path.to_path_buf());
    Ok(scan)
}

/// Lists `directory` into `scan`, descending into its folders.
///
/// # Errors
///
/// [`LibraryError::Io`] when `directory` cannot be listed; for folders below the root this is
/// counted instead.
fn walk(root: &Path, directory: &Path, scan: &mut Scan) -> Result<(), LibraryError> {
    let entries = std::fs::read_dir(directory).map_err(|source| LibraryError::Io {
        action: "list",
        path: directory.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let Ok(entry) = entry else {
            scan.skips.unreadable_entries += 1;
            continue;
        };
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            scan.skips.non_unicode_names += 1;
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let Ok(file_type) = entry.file_type() else {
            scan.skips.unreadable_entries += 1;
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            if walk(root, &path, scan).is_err() {
                scan.skips.unlisted_folders += 1;
            }
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            scan.skips.unreadable_entries += 1;
            continue;
        };
        let Some(place) = place_of(root, &path) else {
            scan.skips.non_unicode_names += 1;
            continue;
        };
        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .unwrap_or_default();
        scan.files.push(ScannedFile {
            place,
            byte_size: metadata.len(),
            modified,
        });
    }
    Ok(())
}

/// The place of `file` in the folder at `root`: its relative path with `/` separators.
fn place_of(root: &Path, file: &Path) -> Option<String> {
    let relative = file.strip_prefix(root).ok()?;
    let parts: Option<Vec<&str>> = relative
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect();
    Some(parts?.join("/"))
}

/// The index cache, or nothing when there is none or it cannot be read: it is only a cache.
fn read_cache(file: &Path) -> Vec<ScannedFile> {
    std::fs::read_to_string(file)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// What differs between the cached and the scanned files.
fn diff(cached: &[ScannedFile], scanned: &[ScannedFile]) -> ScanDiff {
    let before: BTreeMap<&str, &ScannedFile> = cached
        .iter()
        .map(|file| (file.place.as_str(), file))
        .collect();
    let after: BTreeMap<&str, &ScannedFile> = scanned
        .iter()
        .map(|file| (file.place.as_str(), file))
        .collect();
    let mut result = ScanDiff::default();
    for (place, file) in &after {
        match before.get(place) {
            None => result.added.push((*place).to_owned()),
            Some(old) if old.byte_size != file.byte_size || old.modified != file.modified => {
                result.changed.push((*place).to_owned());
            }
            Some(_) => {}
        }
    }
    for place in before.keys() {
        if !after.contains_key(place) {
            result.removed.push((*place).to_owned());
        }
    }
    result
}
