//! The Manifest: the editor's description of an Asset Folder, kept in the configuration directory.

use crate::{LibraryDirectories, LibraryError};
use drs_model::{CanonicalName, FolderKey};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The extension of Manifest files.
const MANIFEST_EXTENSION: &str = "json";

/// What the editor records about an Asset Folder on this device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// The folder's device-local key, which names this file.
    pub key: FolderKey,
    /// The folder's path as the Author gave it.
    pub path: PathBuf,
    /// The name Projects know the folder by.
    pub name: CanonicalName,
    /// The folder's version: the day it was added, until the Author can set it.
    pub version: String,
    /// Assets renamed since the folder was added. None yet.
    #[serde(default)]
    pub renames: Vec<Rename>,
}

/// An Asset known under a new place in the folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rename {
    /// The place the Asset had.
    pub from: String,
    /// The place it has now.
    pub to: String,
}

impl Manifest {
    /// A Manifest for a folder added today, with a key no other folder has.
    #[must_use]
    pub fn new(path: PathBuf, name: CanonicalName) -> Self {
        Self {
            key: FolderKey(uuid::Uuid::new_v4().simple().to_string()),
            path,
            name,
            version: today(),
            renames: Vec::new(),
        }
    }
}

/// Today's date in UTC as `YYYY-MM-DD`.
fn today() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let (year, month, day) = civil_from_days(i64::try_from(seconds / 86_400).unwrap_or(0));
    format!("{year:04}-{month:02}-{day:02}")
}

/// The proleptic Gregorian date of a day count since 1970-01-01 (Howard Hinnant's algorithm).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = u32::try_from(day_of_year - (153 * shifted_month + 2) / 5 + 1).unwrap_or(1);
    let month = u32::try_from(if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    })
    .unwrap_or(1);
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// `WriteManifest`: records `manifest` in the configuration directory, replacing any earlier one
/// for the same key.
///
/// # Errors
///
/// [`LibraryError::UnrecordablePath`] when the folder's path is not valid Unicode, or
/// [`LibraryError::Io`] when the directory or the file cannot be written.
pub fn write_manifest(
    directories: &LibraryDirectories,
    manifest: &Manifest,
) -> Result<(), LibraryError> {
    let text =
        serde_json::to_string_pretty(manifest).map_err(|error| LibraryError::UnrecordablePath {
            path: manifest.path.clone(),
            reason: error.to_string(),
        })?;
    std::fs::create_dir_all(&directories.configuration).map_err(|source| LibraryError::Io {
        action: "create",
        path: directories.configuration.clone(),
        source,
    })?;
    write_atomically(&directories.manifest_file(&manifest.key), text.as_bytes())
}

/// Writes a file through a sibling temporary file and a rename, so a reader never sees half of it.
///
/// # Errors
///
/// [`LibraryError::Io`] when the temporary file cannot be written or renamed.
pub(crate) fn write_atomically(file: &Path, bytes: &[u8]) -> Result<(), LibraryError> {
    let temporary = file.with_extension("tmp");
    std::fs::write(&temporary, bytes).map_err(|source| LibraryError::Io {
        action: "write",
        path: temporary.clone(),
        source,
    })?;
    std::fs::rename(&temporary, file).map_err(|source| LibraryError::Io {
        action: "replace",
        path: file.to_path_buf(),
        source,
    })
}

/// `WriteManifest`, forgetting: removes the Manifest of the folder with `key`, if there is one.
///
/// # Errors
///
/// [`LibraryError::Io`] when the file exists and cannot be removed.
pub fn remove_manifest(
    directories: &LibraryDirectories,
    key: &FolderKey,
) -> Result<(), LibraryError> {
    let file = directories.manifest_file(key);
    match std::fs::remove_file(&file) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(LibraryError::Io {
            action: "remove",
            path: file,
            source,
        }),
    }
}

/// Every Manifest in the configuration directory, and the files there that are not Manifests.
#[derive(Debug, Default)]
pub struct ManifestsRead {
    /// The Manifests, ordered by key.
    pub manifests: Vec<Manifest>,
    /// Why each file that could not be read as a Manifest was skipped.
    pub skipped: Vec<LibraryError>,
}

/// Reads every Manifest in the configuration directory. A directory that does not exist holds none.
///
/// # Errors
///
/// [`LibraryError::Io`] when the directory exists but cannot be listed.
pub fn read_manifests(directories: &LibraryDirectories) -> Result<ManifestsRead, LibraryError> {
    let entries = match std::fs::read_dir(&directories.configuration) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ManifestsRead::default());
        }
        Err(source) => {
            return Err(LibraryError::Io {
                action: "list",
                path: directories.configuration.clone(),
                source,
            });
        }
    };
    let mut read = ManifestsRead::default();
    for entry in entries {
        let path = match entry {
            Ok(entry) => entry.path(),
            Err(source) => {
                read.skipped.push(LibraryError::Io {
                    action: "list",
                    path: directories.configuration.clone(),
                    source,
                });
                continue;
            }
        };
        if path
            .extension()
            .is_none_or(|extension| extension != MANIFEST_EXTENSION)
        {
            continue;
        }
        match read_manifest(&path) {
            Ok(manifest) => read.manifests.push(manifest),
            Err(error) => read.skipped.push(error),
        }
    }
    read.manifests.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(read)
}

/// Reads one Manifest file, whose name must match the key inside it.
///
/// # Errors
///
/// [`LibraryError::Io`] when the file cannot be read, or [`LibraryError::BadManifest`] when it
/// does not hold a Manifest for the key it is named after.
fn read_manifest(path: &Path) -> Result<Manifest, LibraryError> {
    let text = std::fs::read_to_string(path).map_err(|source| LibraryError::Io {
        action: "read",
        path: path.to_path_buf(),
        source,
    })?;
    let manifest: Manifest =
        serde_json::from_str(&text).map_err(|error| LibraryError::BadManifest {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;
    let named = path.file_stem().and_then(|stem| stem.to_str());
    if named != Some(manifest.key.as_str()) {
        return Err(LibraryError::BadManifest {
            path: path.to_path_buf(),
            reason: format!("it holds the key {} but is named otherwise", manifest.key),
        });
    }
    Ok(manifest)
}
