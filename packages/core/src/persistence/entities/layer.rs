use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub weight: i32,
}

impl Layer {
    pub fn new(name: impl Into<String>, weight: i32) -> Self {
        Self {
            name: name.into(),
            weight,
        }
    }
}
