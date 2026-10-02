//! The location of the editor's Bundled Files by platform layout.

use std::path::{Component, Path, PathBuf};
use thiserror::Error;

/// The file that marks a directory as the editor's bundle directory; it holds the editor's
/// version.
pub const BUNDLE_MARKER: &str = "dungeon-rs.bundle";

/// The bundle directory that was found: where the editor's Bundled Files are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundledFiles {
    /// The directory.
    root: PathBuf,
    /// The version the marker names, trimmed; `None` when it names none.
    version: Option<String>,
    /// The version of the editor that looked.
    editor_version: String,
}

impl BundledFiles {
    /// The bundle directory, absolute.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The version the marker names, when it names one.
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Whether the marker names a version other than the editor's.
    #[must_use]
    pub fn version_differs(&self) -> bool {
        self.version
            .as_deref()
            .is_some_and(|version| version != self.editor_version)
    }

    /// The Bundled File called `name`, checked to exist under the directory.
    ///
    /// # Errors
    ///
    /// [`BundledFileError::NotPlainRelative`] when the name is absolute, holds `..`, carries a
    /// source prefix such as `lib://`, or uses `\` or `:`, which are not the same path on every
    /// platform, so no lookup leaves the directory;
    /// [`BundledFileError::Missing`] when nothing is there; [`BundledFileError::Unreadable`] when what
    /// is there cannot be read.
    pub fn file(&self, name: &str) -> Result<PathBuf, BundledFileError> {
        let relative = Path::new(name);
        let plain = !name.is_empty()
            && !name.contains(['\\', ':'])
            && relative
                .components()
                .all(|component| matches!(component, Component::Normal(_)));
        if !plain {
            return Err(BundledFileError::NotPlainRelative {
                name: name.to_owned(),
            });
        }
        let path = self.root.join(relative);
        match std::fs::metadata(&path) {
            Ok(metadata) if metadata.is_file() => Ok(path),
            Ok(_) => Err(BundledFileError::Unreadable {
                name: name.to_owned(),
                reason: "it is not a file".to_owned(),
            }),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Err(BundledFileError::Missing {
                    name: name.to_owned(),
                })
            }
            Err(error) => Err(BundledFileError::Unreadable {
                name: name.to_owned(),
                reason: error.to_string(),
            }),
        }
    }
}

/// No location is marked as the editor's bundle directory.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("no bundle directory is marked as the editor's; tried {}", tried_list(.tried))]
pub struct BundledFilesNotFound {
    /// Every location tried, in the order tried.
    pub tried: Vec<PathBuf>,
}

/// The locations tried, as one clause.
fn tried_list(tried: &[PathBuf]) -> String {
    if tried.is_empty() {
        return "nothing: the executable's directory is unknown".to_owned();
    }
    tried
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Why a Bundled File could not be given.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BundledFileError {
    /// The name is not a plain relative path.
    #[error("the Bundled File name {name:?} is not a plain relative path")]
    NotPlainRelative {
        /// The name as asked.
        name: String,
    },
    /// Nothing is there.
    #[error("the Bundled File {name} is missing")]
    Missing {
        /// The name as asked.
        name: String,
    },
    /// What is there cannot be read.
    #[error("the Bundled File {name} cannot be read: {reason}")]
    Unreadable {
        /// The name as asked.
        name: String,
        /// What the file system said.
        reason: String,
    },
}

/// Finds the bundle directory from the executable's location, never from the working
/// directory: the first of these that holds the marker file.
///
/// 1. `bundle` beside the executable.
/// 2. `Resources` beside the executable's directory: the macOS application layout.
/// 3. `bundle` in each ancestor of the executable's directory, nearest first: the workspace
///    layout during development.
///
/// # Errors
///
/// [`BundledFilesNotFound`] naming every location tried when none is marked.
pub fn locate_bundled_files(
    executable: &Path,
    version: &str,
) -> Result<BundledFiles, BundledFilesNotFound> {
    let mut tried = Vec::new();
    for candidate in candidates(executable) {
        tried.push(candidate.clone());
        if let Ok(marker) = std::fs::read_to_string(candidate.join(BUNDLE_MARKER)) {
            let marker = marker.trim();
            return Ok(BundledFiles {
                root: candidate,
                version: (!marker.is_empty()).then(|| marker.to_owned()),
                editor_version: version.to_owned(),
            });
        }
    }
    Err(BundledFilesNotFound { tried })
}

/// The locations to try, in order.
fn candidates(executable: &Path) -> Vec<PathBuf> {
    let Some(directory) = executable
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return Vec::new();
    };
    let mut candidates = vec![directory.join("bundle")];
    if let Some(parent) = directory.parent() {
        candidates.push(parent.join("Resources"));
    }
    candidates.extend(
        directory
            .ancestors()
            .skip(1)
            .map(|ancestor| ancestor.join("bundle")),
    );
    candidates
}
