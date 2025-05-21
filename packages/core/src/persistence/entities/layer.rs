use crate::persistence::entities::image::Image;
use serde::{Deserialize, Serialize};

/// Counterpart of [`crate::components::Layer`].
#[derive(Debug, Serialize, Deserialize)]
pub struct Layer {
    /// The name that identifies this [Layer] in the UI.
    pub name: String,
    /// The weight determines the order in which layers are rendered.
    /// A higher weight will render on top, making this an abstraction of the Z index.
    pub weight: f32,
    /// A list of all [Image]s in this layer.
    pub images: Vec<Image>,
}

impl Layer {
    /// Generates a new [Layer] with a given name and [Image]s.
    pub fn new(name: impl Into<String>, weight: f32, images: Vec<Image>) -> Self {
        Self {
            name: name.into(),
            weight,
            images,
        }
    }
}
