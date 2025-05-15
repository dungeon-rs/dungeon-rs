use bevy::prelude::{Component, Rectangle};
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize)]
pub struct Texture {
    pub size: Rectangle,
}
