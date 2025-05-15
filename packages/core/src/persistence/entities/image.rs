use bevy::prelude::{Rectangle, Transform};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub name: String,
    pub path: PathBuf,
    pub alpha: f32,
    pub size: Rectangle,
    pub transform: Transform,
}
