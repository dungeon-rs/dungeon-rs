use bevy::prelude::Event;
use std::path::PathBuf;

#[derive(Event)]
pub struct LoadProjectRequest {
    pub path: PathBuf,
}

impl LoadProjectRequest {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}
