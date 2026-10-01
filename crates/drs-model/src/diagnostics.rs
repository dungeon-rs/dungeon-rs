//! What diagnostics found at start, for the Editor to show.

use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;
use bevy_reflect::Reflect;
use std::path::PathBuf;

/// Where the editor's own diagnostic files are and what was found about its bundled resources.
///
/// The Host fills it in before the App runs; the Editor reads it for Show Logs and the status
/// line. Nothing writes it afterwards.
#[derive(Resource, Reflect, Debug, Clone, PartialEq, Eq)]
#[reflect(Resource)]
pub struct Diagnostics {
    /// The log directory, where log files and crash reports are.
    pub logs: PathBuf,
    /// The current log file; `None` when logging goes to the terminal only.
    pub log_file: Option<PathBuf>,
    /// The resource directory, or the locations tried when none is marked as the editor's.
    pub resources: ResourceDirectory,
}

/// Where the editor's bundled resources are, or why they are nowhere.
#[derive(Reflect, Debug, Clone, PartialEq, Eq)]
pub enum ResourceDirectory {
    /// The directory marked as the editor's.
    Found(PathBuf),
    /// No location is marked.
    Missing {
        /// Every location tried, in the order tried.
        tried: Vec<PathBuf>,
    },
}
