#![doc = include_str!("../README.md")]

use drs_model::{
    ElementId, Envelopes, LayerSnapshot, LevelSnapshot, ProjectSnapshot, SerialisationError,
    SerialisationRegistry,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{Builder, NamedTempFile};

/// The version of the file's own shape; the components inside carry versions of their own.
pub const FORMAT_VERSION: u32 = 1;

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

/// The shape of a Project file at [`FORMAT_VERSION`]: one JSON document holding the format
/// version, the Project entity's envelopes, the Levels with their Layers in order, and every
/// Element by identity. Keys are written in this order and Elements sorted by identity, so the
/// same snapshot always writes the same file.
#[derive(Serialize, Deserialize)]
struct ProjectFile {
    /// The version of this shape.
    format: u32,
    /// The components of the Project entity.
    project: Envelopes,
    /// The Levels, in order.
    levels: Vec<LevelRecord>,
    /// Every Element, by identity.
    elements: BTreeMap<ElementId, Envelopes>,
}

/// One Level in a Project file.
#[derive(Serialize, Deserialize)]
struct LevelRecord {
    /// The components of the Level entity.
    components: Envelopes,
    /// The Layers, in order.
    layers: Vec<LayerRecord>,
}

/// One Layer in a Project file.
#[derive(Serialize, Deserialize)]
struct LayerRecord {
    /// The components of the Layer entity.
    components: Envelopes,
    /// The Elements on the Layer in stacking order, the first drawn first.
    elements: Vec<ElementId>,
}

impl From<&ProjectSnapshot> for ProjectFile {
    /// The snapshot laid out in the current shape; the envelopes are copied.
    fn from(snapshot: &ProjectSnapshot) -> Self {
        Self {
            format: FORMAT_VERSION,
            project: snapshot.project.clone(),
            levels: snapshot
                .levels
                .iter()
                .map(|level| LevelRecord {
                    components: level.components.clone(),
                    layers: level
                        .layers
                        .iter()
                        .map(|layer| LayerRecord {
                            components: layer.components.clone(),
                            elements: layer.elements.clone(),
                        })
                        .collect(),
                })
                .collect(),
            elements: snapshot.elements.clone(),
        }
    }
}

impl From<ProjectFile> for ProjectSnapshot {
    /// The snapshot the file holds, whatever shape it was read in.
    fn from(file: ProjectFile) -> Self {
        Self {
            project: file.project,
            levels: file
                .levels
                .into_iter()
                .map(|level| LevelSnapshot {
                    components: level.components,
                    layers: level
                        .layers
                        .into_iter()
                        .map(|layer| LayerSnapshot {
                            components: layer.components,
                            elements: layer.elements,
                        })
                        .collect(),
                })
                .collect(),
            elements: file.elements,
        }
    }
}

/// `ReadProject`: reads the Project file at `path` into a snapshot.
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
) -> Result<ProjectSnapshot, ProjectAccessError> {
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
    Ok(file.into())
}

/// `WriteProject`: writes `snapshot` to `path` in the current shape, pretty-printed in a fixed
/// key order.
///
/// The text goes into a temporary file beside the target first, is flushed to the disk, and is
/// renamed over the target once complete, so a write that fails, or a crash during it, leaves
/// whatever was at `path` as it was and never a partial file; the temporary file is removed
/// when the write fails.
///
/// # Errors
///
/// [`ProjectAccessError::NotAProject`] when the snapshot cannot be encoded, or
/// [`ProjectAccessError::Io`] when the temporary file cannot be created, written, flushed, or
/// renamed.
pub fn write_project(path: &Path, snapshot: &ProjectSnapshot) -> Result<(), ProjectAccessError> {
    let file = ProjectFile::from(snapshot);
    let mut text =
        serde_json::to_string_pretty(&file).map_err(|error| ProjectAccessError::NotAProject {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;
    text.push('\n');
    let io = |action: &'static str| {
        move |source: std::io::Error| ProjectAccessError::Io {
            action,
            path: path.to_path_buf(),
            source,
        }
    };
    let beside = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temporary = file_beside(beside).map_err(io("create a file beside"))?;
    temporary.write_all(text.as_bytes()).map_err(io("write"))?;
    temporary.as_file().sync_all().map_err(io("flush"))?;
    temporary
        .persist(path)
        .map_err(|error| io("rename")(error.error))?;
    Ok(())
}

/// A temporary file in `directory`, removed when dropped unless it is persisted.
///
/// The file is created as readable as any file the Author makes: a temporary file is private by
/// default, but a saved Project is for sharing, so it is created with the usual mode, which the
/// process's umask narrows as it does for every new file.
///
/// # Errors
///
/// The error of creating the file.
fn file_beside(directory: &Path) -> std::io::Result<NamedTempFile> {
    let mut builder = Builder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        builder.permissions(std::fs::Permissions::from_mode(0o666));
    }
    builder.tempfile_in(directory)
}
