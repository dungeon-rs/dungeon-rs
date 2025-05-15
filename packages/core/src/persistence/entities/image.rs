use bevy::prelude::{Color, Rectangle, Transform};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a "plain" image.
#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    /// An optional name that can be used to identify this image.
    pub name: Option<String>,
    /// The path to the texture that this [Image] should render.
    /// Should be a path that the AssetServer can find.
    pub path: PathBuf,
    /// The [Color] to render this image at, set to [Color::WHITE] for it not to be modified.
    /// You can control the transparency of the image through the alpha of this colour.
    pub colour: Color,
    /// The [Rectangle] representing the size of the mesh this texture renders on.
    pub size: Rectangle,
    /// The [Transform] determines the location, scale and rotation of the texture.
    pub transform: Transform,
}
