use bevy::prelude::Event;
use std::path::PathBuf;

/// Event fired when the user requests to save the project.
#[derive(Event)]
pub struct SaveProjectRequest {
    /// The path to save the project to.
    pub path: PathBuf,
}

impl SaveProjectRequest {
    /// Creates a new [SaveProjectRequest] event.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}
