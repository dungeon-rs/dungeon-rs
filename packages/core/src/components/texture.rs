use bevy::prelude::{Component, Rectangle};
use serde::{Deserialize, Serialize};

/// Marker component for a texture ("plain" image).
///
/// We store the size [Rectangle] since we can't reconstruct it from a mesh later.
#[derive(Component, Serialize, Deserialize)]
pub struct Texture {
    pub size: Rectangle,
}
