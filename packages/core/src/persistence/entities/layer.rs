use crate::persistence::entities::image::Image;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub weight: i32,
    pub images: Vec<Image>,
}

impl Layer {
    pub fn new(name: impl Into<String>, weight: i32, images: Vec<Image>) -> Self {
        Self {
            name: name.into(),
            weight,
            images,
        }
    }
}
