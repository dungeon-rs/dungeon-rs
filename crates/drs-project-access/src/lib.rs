#![doc = include_str!("../README.md")]

use drs_model::{FORMAT_VERSION, ProjectFile, SerialisationError, SerialisationRegistry};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// What can go wrong while reading or writing a Project file.
#[derive(Debug, thiserror::Error)]
pub enum ProjectAccessError {
    /// The file could not be read, written, or replaced.
    #[error("cannot {action} {}: {source}", path.display())]
    Io {
        /// What was attempted, such as `read` or `write`.
        action: &'static str,
        /// The file.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// The file does not hold a Project.
    #[error("{} is not a Project file: {reason}", path.display())]
    NotAProject {
        /// The file.
        path: PathBuf,
        /// Why it is not one.
        reason: String,
    },
    /// The file was written in a newer shape than this editor knows.
    #[error(
        "{} was saved in format version {version}, newer than version {known} this editor knows",
        path.display()
    )]
    NewerFormat {
        /// The file.
        path: PathBuf,
        /// The format version in the file.
        version: u32,
        /// The format version this editor reads.
        known: u32,
    },
    /// A component in the file is newer than this editor knows, or malformed.
    #[error(transparent)]
    Serialisation(#[from] SerialisationError),
}

/// The one field read before anything else, so a file in a newer shape is refused for its
/// version rather than for whatever else differs.
#[derive(Deserialize)]
struct FormatHeader {
    /// The version of the file's shape.
    format: u32,
}

/// `ReadProject`: reads the Project file at `path`.
///
/// The format version and, through `registry`, the version of every known component are checked;
/// envelopes the registry does not know are kept as they are.
///
/// # Errors
///
/// [`ProjectAccessError::Io`] when the file cannot be read, [`ProjectAccessError::NotAProject`]
/// when it is not JSON in the Project's shape, [`ProjectAccessError::NewerFormat`] when its
/// format version is newer than [`FORMAT_VERSION`], or [`ProjectAccessError::Serialisation`]
/// when a component's version is newer than the registry knows or its envelope is malformed.
pub fn read_project(
    path: &Path,
    registry: &SerialisationRegistry,
) -> Result<ProjectFile, ProjectAccessError> {
    let text = std::fs::read_to_string(path).map_err(|source| ProjectAccessError::Io {
        action: "read",
        path: path.to_path_buf(),
        source,
    })?;
    let not_a_project = |error: serde_json::Error| ProjectAccessError::NotAProject {
        path: path.to_path_buf(),
        reason: error.to_string(),
    };
    let header: FormatHeader = serde_json::from_str(&text).map_err(not_a_project)?;
    if header.format > FORMAT_VERSION {
        return Err(ProjectAccessError::NewerFormat {
            path: path.to_path_buf(),
            version: header.format,
            known: FORMAT_VERSION,
        });
    }
    let file: ProjectFile = serde_json::from_str(&text).map_err(not_a_project)?;
    registry.check_versions(&file.project)?;
    for level in &file.levels {
        registry.check_versions(&level.components)?;
        for layer in &level.layers {
            registry.check_versions(&layer.components)?;
        }
    }
    for envelopes in file.elements.values() {
        registry.check_versions(envelopes)?;
    }
    Ok(file)
}

/// `WriteProject`: writes `file` to `path`, pretty-printed in a fixed key order.
///
/// The text goes into a temporary file beside the target first and is renamed over it once
/// complete, so a write that fails leaves whatever was at `path` as it was and never a partial
/// file.
///
/// # Errors
///
/// [`ProjectAccessError::NotAProject`] when the file cannot be encoded, or
/// [`ProjectAccessError::Io`] when the temporary file cannot be written or renamed; the
/// temporary file is removed either way.
pub fn write_project(path: &Path, file: &ProjectFile) -> Result<(), ProjectAccessError> {
    let mut text =
        serde_json::to_string_pretty(file).map_err(|error| ProjectAccessError::NotAProject {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;
    text.push('\n');
    let temporary = temporary_beside(path);
    std::fs::write(&temporary, text.as_bytes()).map_err(|source| ProjectAccessError::Io {
        action: "write",
        path: temporary.clone(),
        source,
    })?;
    std::fs::rename(&temporary, path).map_err(|source| {
        // Whether or not the half-written file can be removed, the error that matters is the
        // rename that failed.
        drop(std::fs::remove_file(&temporary));
        ProjectAccessError::Io {
            action: "replace",
            path: path.to_path_buf(),
            source,
        }
    })
}

/// A file beside `path` that is replaced over it once written.
fn temporary_beside(path: &Path) -> PathBuf {
    let mut name = path.file_name().map(ToOwned::to_owned).unwrap_or_default();
    name.push(".tmp");
    path.with_file_name(name)
}
