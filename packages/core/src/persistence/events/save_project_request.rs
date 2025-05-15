use bevy::prelude::Event;
use std::path::PathBuf;

#[derive(Event)]
pub struct SaveProjectRequest {
    pub path: PathBuf,
}

impl SaveProjectRequest {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}
