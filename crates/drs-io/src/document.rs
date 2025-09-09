//! The [`Document`] structs are used for writing and loading [`drs_data::Project`] entities to and
//! from storage. This module is intentionally not made public, since these structs have no use
//! beyond persistence and should not be used outside this scope.

use bevy::ecs::{relationship::RelationshipTarget, system::Query};
use bevy::prelude::{Quat, Vec3};
use drs_data::{
    Element, ElementQuery, ElementQueryItem, LayerQuery, LayerQueryItem, LevelQuery,
    LevelQueryItem, ProjectQueryItem,
};
use drs_serialization::{Deserialize, Serialize};

/// A [`Document`] represents a [`drs_data::Project`] (and it's children) that is written to or read from storage.
///
/// It's an intentionally simplified representation of the ECS data structure optimised for serialisation.
#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    /// See `name` in [`drs_data::Project::new`].
    pub name: String,
    /// All [`DocumentLevel`] constructed from the [`drs_data::Project`]'s children.
    pub levels: Vec<DocumentLevel>,
}

/// A [`DocumentLevel`] represents a [`drs_data::Level`] (and it's children) that is written to or read
/// from storage.
///
/// It's an intentionally simplified representation of the ECS data structure optimised for serialisation.
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentLevel {
    /// See `name` in [`drs_data::Level::new`].
    pub name: String,
    /// Whether the level is visible in the editor (e.g. active).
    pub visible: bool,
    /// All [`DocumentLayer`] constructed from the [`drs_data::Level`]'s children.
    pub layers: Vec<DocumentLayer>,
}

/// A [`DocumentLayer`] represents a [`drs_data::Layer`] (and it's children) that is written to or read
/// from storage.
///
/// It's an intentionally simplified representation of the ECS data structure optimised for serialisation.
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentLayer {
    /// See `name` in [`drs_data::Layer::new`].
    pub name: String,
    /// Whether the layer is visible in the editor (e.g. active).
    pub visible: bool,
    /// The order of the [`drs_data::Layer`] (determined by it's [`bevy::prelude::Transform`]).
    pub order: f32,
    /// The [`DocumentItem`] constructed from the [`drs_data::Layer`]'s children.
    pub items: Vec<DocumentItem>,
}

/// Represents the lowest level of a [`Document`]; these are the items that are 'visible' on the
/// screen for the user (objects, paths, patterns, textures, ...).
///
/// They represent a [`drs_data::Element`] at its core.
#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DocumentItem {
    /// Captures the metadata of an [`drs_data::Element::Object`].
    Object {
        /// The ID of the asset being captured.
        id: String,
        /// The translation of the object in the world.
        translation: Vec3,
        /// The rotation of the object in the world.
        rotation: Quat,
        /// The scale of the object in the world.
        scale: Vec3,
    },
}

impl Document {
    /// Generate a new [`Document`] and it's related children based on the current state (fetched
    /// through the `level_query` and `layer_query` queries).
    ///
    /// The `value` parameter is the result of a [`drs_data::ProjectQuery`] to fetch the relevant
    /// data for the [`drs_data::Project`] component.
    ///
    /// See [`DocumentLevel::new`] for `level_query`.
    ///
    /// See [`DocumentLayer::new`] for `layer_query`.
    pub fn new(
        value: &ProjectQueryItem,
        level_query: Query<LevelQuery>,
        layer_query: Query<LayerQuery>,
        object_query: Query<ElementQuery>,
    ) -> Self {
        let levels: Vec<DocumentLevel> = value
            .children
            .iter()
            .flat_map(|pc| level_query.get(pc))
            .map(|lvl| DocumentLevel::new(&lvl, layer_query, object_query))
            .collect();

        Self {
            name: value.name.to_string(),
            levels,
        }
    }
}

impl DocumentLevel {
    /// Generate a new [`DocumentLevel`] and it's related children based on the current state
    /// (fetched through the `layer_query`).
    ///
    /// The `value` parameter is the result of a [`drs_data::LevelQuery`] to fetch the relevant
    /// data for the [`drs_data::Level`], and is usually passed from [`Document::new`].
    ///
    /// See [`Document::new`] for how this is called.
    ///
    /// See [`DocumentLayer::new`] for `layer_query`.
    pub fn new(
        value: &LevelQueryItem,
        layer_query: Query<LayerQuery>,
        object_query: Query<ElementQuery>,
    ) -> Self {
        let layers = value
            .children
            .iter()
            .flat_map(|c| layer_query.get(c))
            .map(|value| DocumentLayer::new(&value, object_query))
            .collect();
        Self {
            name: value.name.to_string(),
            visible: value.is_visible(),
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
    pub fn new(value: &LayerQueryItem, object_query: Query<ElementQuery>) -> Self {
        let items: Vec<DocumentItem> = value
            .children
            .iter()
            .flat_map(|c| object_query.get(c))
            .map(|item| DocumentItem::new(&item))
            .collect();

        Self {
            name: value.name.to_string(),
            visible: value.is_visible(),
            order: value.transform.translation.z,
            items,
        }
    }
}

impl DocumentItem {
    /// Create a new [`DocumentItem`] from the given [`drs_data::Element`] and it's meta components.
    ///
    /// # Panics
    /// This method can panic if the [`drs_data::Element`] is an unsupported type.
    pub fn new(value: &ElementQueryItem) -> Self {
        match value.element {
            Element::Object(object) => DocumentItem::Object {
                id: object.clone(),
                translation: value.transform.translation,
                rotation: value.transform.rotation,
                scale: value.transform.scale,
            },
            _ => panic!("DocumentItem::new called with unsupported Element type"),
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
    use drs_data::{Layer, Level, Project, ProjectQuery};
    use std::path::PathBuf;

    #[test]
    pub fn document_new() -> anyhow::Result<()> {
        let mut world = World::default();
        world.spawn((
            Project::new(PathBuf::new(), "Example Project"),
            children![(
                Level::new("First Level"),
                children![(
                    Layer::new("First Layer", Transform::IDENTITY),
                    children![(
                        Element::Object(String::new()),
                        Name::new("First Object"),
                        Transform::IDENTITY,
                    )]
                )]
            )],
        ));

        let mut system_state: SystemState<(
            Query<ProjectQuery>,
            Query<LevelQuery>,
            Query<LayerQuery>,
            Query<ElementQuery>,
        )> = SystemState::new(&mut world);
        let (project_query, level_query, layer_query, object_query) = system_state.get(&world);
        let project = project_query.single()?;

        let document = Document::new(&project, level_query, layer_query, object_query);
        assert_eq!(document.name, String::from("Example Project"));
        assert_eq!(document.levels.len(), 1);
        assert_eq!(document.levels[0].name, String::from("First Level"));
        assert_eq!(document.levels[0].layers.len(), 1);
        assert_eq!(document.levels[0].layers[0].name, "First Layer");
        assert_eq!(document.levels[0].layers[0].order, 0.0);
        assert_eq!(document.levels[0].layers[0].items.len(), 1);

        Ok(())
    }
}
