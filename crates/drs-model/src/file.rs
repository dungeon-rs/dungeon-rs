//! The shape of a Project file, and what the editor remembers about the file a Project is in.

use crate::{ElementId, Envelopes};
use bevy_ecs::resource::Resource;
use drs_history::Position;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The version of the file's own shape; the components inside carry versions of their own.
pub const FORMAT_VERSION: u32 = 1;

/// The extension of Project files.
pub const PROJECT_EXTENSION: &str = "dungeon";

/// `path` with the Project extension added when its name lacks it, in any letter case.
#[must_use]
pub fn with_project_extension(path: PathBuf) -> PathBuf {
    let has_extension = path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(PROJECT_EXTENSION));
    if has_extension {
        return path;
    }
    let mut name = path.file_name().map(ToOwned::to_owned).unwrap_or_default();
    name.push(".");
    name.push(PROJECT_EXTENSION);
    path.with_file_name(name)
}

/// The name a Project takes from its file: the file name without its extension.
#[must_use]
pub fn project_name_of(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// A whole Project as one file: the Project's own components, its Levels with their Layers in
/// order, and every Element keyed by its identity.
///
/// Every component is an envelope under its stable name; an Element's kind component sits beside
/// its common `element` envelope. Elements are sorted by identity and each Layer lists its
/// Elements in stacking order, so the same Project always writes the same file.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectFile {
    /// The version of this shape.
    pub format: u32,
    /// The components of the Project entity: the Project, its Grid, its Bounds, and its Asset
    /// Reference table.
    pub project: Envelopes,
    /// The Levels, in order.
    pub levels: Vec<LevelRecord>,
    /// Every Element, by identity.
    pub elements: BTreeMap<ElementId, Envelopes>,
}

/// One Level in a Project file.
#[derive(Debug, Serialize, Deserialize)]
pub struct LevelRecord {
    /// The components of the Level entity.
    pub components: Envelopes,
    /// The Layers, in order.
    pub layers: Vec<LayerRecord>,
}

/// One Layer in a Project file.
#[derive(Debug, Serialize, Deserialize)]
pub struct LayerRecord {
    /// The components of the Layer entity.
    pub components: Envelopes,
    /// The Elements on the Layer in stacking order, the first drawn first.
    pub elements: Vec<ElementId>,
}

/// The file the Project was last saved to or opened from, and where the history stood then.
///
/// Written only by the project Manager. The Project has unsaved changes exactly when the
/// history's position differs from `position`.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct SavedMark {
    /// The Project's file, or `None` for a Project that was never saved.
    pub file: Option<PathBuf>,
    /// The history's position at the last save or open.
    pub position: Position,
}
