use crate::persistence::entities::layer::Layer;
use serde::{Deserialize, Serialize};

/// Counterpart of [`crate::components::Level`].
#[derive(Debug, Serialize, Deserialize)]
pub struct Level {
    /// The name that identifies this [Level] in the UI.
    pub name: String,
    /// All [Layer]s under this [Level].
    pub layers: Vec<Layer>,
}

impl Level {
    /// Generates a new [Level] with a given name and [Layer]s.
    pub fn new(name: impl Into<String>, layers: Vec<Layer>) -> Self {
        Self {
            name: name.into(),
            layers,
        }
    }
}
