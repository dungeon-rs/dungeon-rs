use crate::persistence::entities::layer::Layer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    pub layers: Vec<Layer>,
}

impl Level {
    pub fn new(name: impl Into<String>, layers: Vec<Layer>) -> Self {
        Self {
            name: name.into(),
            layers,
        }
    }
}
