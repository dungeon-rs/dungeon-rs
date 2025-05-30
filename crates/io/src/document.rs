//! The [`Document`] structs are used for writing and loading [`data::Project`] entities to and
//! from storage. This module is intentionally not made public, since these structs have no use
//! beyond persistence and should not be used outside this scope.

use bevy::{
    ecs::{relationship::RelationshipTarget, system::Query},
    prelude::{Children, Name}, transform::components::Transform,
};
use data::{Layer, Level};
use serde::Serialize;

/// A [`Document`] represents a [`data::Project`] (and it's children) that is written to or read from storage.
///
/// It's an intentionally simplified representation of the ECS datastructure optimized for serialisation.
#[derive(Debug, Serialize)]
pub struct Document {
    name: String,
    levels: Vec<DocumentLevel>,
}

#[derive(Debug, Serialize)]
pub struct DocumentLevel {
    name: String,
    layers: Vec<DocumentLayer>,
}

#[derive(Debug, Serialize)]
pub struct DocumentLayer {
    name: String,
    order: f32,
    items: Vec<DocumentItem>,
}

#[derive(Debug, Serialize)]
pub enum DocumentItem {
    // TODO: actually rendered items and their metadata should be included here
}

impl Document {
    pub fn new(
        value: (&Name, &Children),
        level_query: Query<(&Level, &Name, &Children)>,
        layer_query: Query<(&Layer, &Name, &Transform, &Children)>,
    ) -> Self {
        let levels: Vec<DocumentLevel> = value
            .1
            .iter()
            .flat_map(|pc| level_query.get(pc))
            .map(|lvl| DocumentLevel::new(lvl, layer_query))
            .collect();

        Self {
            name: value.0.to_string(),
            levels,
        }
    }
}

impl DocumentLevel {
    pub fn new(
        value: (&Level, &Name, &Children),
        layer_query: Query<(&Layer, &Name, &Transform, &Children)>,
    ) -> Self {
        let layers = value
            .2
            .iter()
            .flat_map(|c| layer_query.get(c))
            .map(DocumentLayer::new)
            .collect();
        Self {
            name: value.1.to_string(),
            layers,
        }
    }
}

impl DocumentLayer {
    pub fn new(value: (&Layer, &Name, &Transform, &Children)) -> Self {
        Self {
            name: value.1.to_string(),
            order: value.2.translation.z,
            items: Vec::new(),
        }
    }
}
