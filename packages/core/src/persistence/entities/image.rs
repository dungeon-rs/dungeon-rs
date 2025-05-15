use bevy::prelude::Transform;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub path: PathBuf,
    pub alpha: f32,
    pub transform: Transform,
}
