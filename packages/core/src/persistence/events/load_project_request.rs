use bevy::prelude::Event;
use std::path::PathBuf;

/// Event fired when the user requests to load a project.
#[derive(Event)]
pub struct LoadProjectRequest {
    /// The path to the savefile to load.
    pub path: PathBuf,
}

impl LoadProjectRequest {
    /// Creates a new [LoadProjectRequest] event.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}
