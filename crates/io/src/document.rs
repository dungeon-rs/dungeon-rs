//! The [`Document`] structs are used for writing and loading [`data::Project`] entities to and
//! from storage. This module is intentionally not made public, since these structs have no use
//! beyond persistence and should not be used outside this scope.

use bevy::{
    ecs::{relationship::RelationshipTarget, system::Query},
    prelude::{Children, Name},
    transform::components::Transform,
};
use data::{Layer, Level};
use serialization::{Deserialize, Serialize};

/// A [`Document`] represents a [`data::Project`] (and it's children) that is written to or read from storage.
///
/// It's an intentionally simplified representation of the ECS datastructure optimised for serialisation.
#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    /// See `name` in [`data::Project::new`].
    pub name: String,
    /// All [`DocumentLevel`] constructed from the [`data::Project`]'s children.
    pub levels: Vec<DocumentLevel>,
}

/// A [`DocumentLevel`] represents a [`data::Level`] (and it's children) that is written to or read
/// from storage.
///
/// It's an intentionally simplified representation of the ECS datastructure optimised for serialisation.
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentLevel {
    /// See `name` in [`data::Level::new`].
    pub name: String,
    /// All [`DocumentLayer`] constructed from the [`data::Level`]'s children.
    pub layers: Vec<DocumentLayer>,
}

/// A [`DocumentLayer`] represents a [`data::Layer`] (and it's children) that is written to or read
/// from storage.
///
/// It's an intentionally simplified representation of the ECS datastructure optimised for serialisation.
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentLayer {
    /// See `name` in [`data::Layer::new`].
    pub name: String,
    /// The order of the [`data::Layer`] (determined by it's [`Transform`]).
    pub order: f32,
    /// The [`DocumentItem`] constructed from the [`data::Layer`]'s children.
    pub items: Vec<DocumentItem>,
}

/// Represents the lowest level of a [`Document`], these are the items that are 'visible' on the
/// screen for the user (objects, paths, patterns, textures, ...).
#[derive(Debug, Serialize, Deserialize)]
pub enum DocumentItem {
    // TODO: actually rendered items and their metadata should be included here
}

impl Document {
    /// Generate a new [`Document`] and it's related children based on the current state (fetched
    /// through the `level_query` and `layer_query` queries).
    ///
    /// The `value` parameter is the result of a [`bevy::prelude::Query`] to fetch the relevant
    /// data for the [`data::Project`] component.
    ///
    /// See [`DocumentLevel::new`] for `level_query`.
    ///
    /// See [`DocumentLayer::new`] for `layer_query`.
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

    /// Calculates the total number of entities contained in this document.
    ///
    /// It counts itself, all children and their children recursively.
    /// This is used mostly for progress reporting when loading projects.
    pub fn count(&self) -> u32 {
        // start counting at 1 because we count ourselves.
        self.levels.iter().fold(1, |total, lvl| {
            // start counting at total + 1 to count the level along
            lvl.layers.iter().fold(total + 1, |total, layer| {
                // same deal for the items, count the layer
                layer.items.iter().fold(total + 1, |total, _item| total + 1)
            })
        })
    }
}

impl DocumentLevel {
    /// Generate a new [`DocumentLevel`] and it's related children based on the current state
    /// (fetched through the `layer_query`).
    ///
    /// The `value` parameter is the result of a [`bevy::prelude::Query`] to fetch the relevant
    /// data for the [`data::Level`], and is usually passed from [`Document::new`].
    ///
    /// See [`Document::new`] for how this is called.
    ///
    /// See [`DocumentLayer::new`] for `layer_query`.
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
    /// Generate a new [`DocumentLayer`] and it's related children based on the current state.
    ///
    /// TODO: fetch child `items`.
    ///
    /// See [`DocumentLevel::new`] for how this is called.
    pub fn new(value: (&Layer, &Name, &Transform, &Children)) -> Self {
        Self {
            name: value.1.to_string(),
            order: value.2.translation.z,
            items: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::type_complexity)]
    #![allow(clippy::missing_panics_doc)]
    #![allow(clippy::missing_errors_doc)]
    #![allow(clippy::float_cmp)]

    use super::*;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::*;
    use data::{Layer, Level, Project};

    #[test]
    pub fn document_new() -> anyhow::Result<()> {
        let mut world = World::default();
        world.spawn((
            Project::new("Example Project"),
            children![(
                Level::new("First Level"),
                children![(Layer::new("First Layer", Transform::IDENTITY), children![])]
            )],
        ));

        let mut system_state: SystemState<(
            Query<(&Name, &Children), With<Project>>,
            Query<(&Level, &Name, &Children)>,
            Query<(&Layer, &Name, &Transform, &Children)>,
        )> = SystemState::new(&mut world);
        let (project_query, level_query, layer_query) = system_state.get(&world);
        let project = project_query.single()?;

        let document = Document::new(project, level_query, layer_query);
        assert_eq!(document.name, String::from("Example Project"));
        assert_eq!(document.levels.len(), 1);
        assert_eq!(document.levels[0].name, String::from("First Level"));
        assert_eq!(document.levels[0].layers.len(), 1);
        assert_eq!(document.levels[0].layers[0].name, "First Layer");
        assert_eq!(document.levels[0].layers[0].order, 0.0);

        Ok(())
    }

    #[test]
    pub fn document_count_recursive() {
        let document = Document {
            name: String::from("test document"),
            levels: vec![
                DocumentLevel {
                    name: String::from("test level 1"),
                    layers: vec![
                        DocumentLayer {
                            name: String::from("test layer 1"),
                            order: 0.0,
                            items: vec![],
                        },
                        DocumentLayer {
                            name: String::from("test layer 2"),
                            order: 0.0,
                            items: vec![],
                        },
                    ],
                },
                DocumentLevel {
                    name: String::from("test level 2"),
                    layers: vec![
                        DocumentLayer {
                            name: String::from("test layer 3"),
                            order: 0.0,
                            items: vec![],
                        },
                        DocumentLayer {
                            name: String::from("test layer 4"),
                            order: 0.0,
                            items: vec![],
                        },
                    ],
                },
            ],
        };

        assert_eq!(document.count(), 7);
    }
}
