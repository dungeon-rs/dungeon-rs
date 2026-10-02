//! What the editor knows about the file a Project is in: its extension, the name it gives the
//! Project, and where the history stood when the file was last written or read.

use bevy_ecs::resource::Resource;
use drs_history::Position;
use std::path::{Path, PathBuf};

/// The extension of Project files.
pub const PROJECT_EXTENSION: &str = "dungeon";

/// `path` with `.extension` added to its name when the name lacks that extension in any letter
/// case, as a file chosen in a dialog often does.
#[must_use]
pub fn with_extension_if_missing(path: PathBuf, extension: &str) -> PathBuf {
    let has_extension = path
        .extension()
        .is_some_and(|present| present.eq_ignore_ascii_case(extension));
    if has_extension {
        return path;
    }
    let mut name = path.file_name().map(ToOwned::to_owned).unwrap_or_default();
    name.push(".");
    name.push(extension);
    path.with_file_name(name)
}

/// The name a Project takes from its file: the file name without its extension.
#[must_use]
pub fn project_name_of(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
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

impl SavedMark {
    /// The name the Author knows the Project by: its file's name without the extension, or
    /// `Untitled` while it has no file.
    #[must_use]
    pub fn name(&self) -> String {
        self.file
            .as_deref()
            .map_or_else(|| "Untitled".to_owned(), project_name_of)
    }
}
