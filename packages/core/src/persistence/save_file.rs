use crate::persistence::entities::layer::Layer;
use crate::persistence::entities::level::Level;
use bevy::prelude::Rect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveFile {
    pub version: &'static str,
    pub size: Rect,
    pub levels: Vec<Level>,
}

impl SaveFile {
    pub fn new() -> Self {
        Self {
            version: "0.0.1",
            size: Rect::new(0.0, 0.0, 100.0, 100.0),
            levels: vec![
                Level::new(
                    "default",
                    vec![Layer::new("default", 0), Layer::new("background", 0)],
                ),
                Level::new("basement", vec![]),
            ],
        }
    }
}
