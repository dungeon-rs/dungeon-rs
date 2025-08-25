//! Helper types and functions for querying the `DungeonRS` hierarchy.
//!
//! This module provides helper types and functions for querying the `DungeonRS`
//! hierarchy, including types for representing the hierarchy levels and
//! components, and functions for navigating between levels and layers.

use crate::{Element, Layer, Level, Project};
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Children, Entity, Name, Query, Transform, Visibility};

/// A query for project entities, containing all necessary components to work with projects
/// in the `DungeonRS` hierarchy.
///
/// Projects are the top-level containers in the hierarchy and contain one or more levels.
#[derive(QueryData)]
pub struct ProjectQuery {
    /// The entity ID of the project
    pub entity: Entity,
    /// The human-readable name of the project
    pub name: &'static Name,
    /// Child entities (levels) belonging to this project
    pub children: &'static Children,
    /// The project-specific component data
    pub project: &'static Project,
}

/// A query for level entities, containing all necessary components to work with levels
/// in the `DungeonRS` hierarchy.
///
/// Levels represent individual floors, areas, or scenes within a project and contain
/// one or more layers for organising content.
#[derive(QueryData)]
pub struct LevelQuery {
    /// The entity ID of the level
    entity: Entity,
    /// The level-specific component data
    level: &'static Level,
    /// The human-readable name of the level
    pub name: &'static Name,
    /// Child entities (layers) belonging to this level
    pub children: &'static Children,
    /// Whether this level is currently visible/enabled
    visibility: &'static Visibility,
}

/// A query for layer entities, containing all necessary components to work with layers
/// in the `DungeonRS` hierarchy.
///
/// Layers are used to organise elements within a level, similar to layers in image
/// editing software. Each layer can be transformed and made visible/invisible independently.
#[derive(QueryData)]
pub struct LayerQuery {
    /// The entity ID of the layer
    entity: Entity,
    /// The layer-specific component data
    layer: &'static Layer,
    /// The human-readable name of the layer
    pub name: &'static Name,
    /// The spatial transformation (position, rotation, scale) of the layer
    pub transform: &'static Transform,
    /// Child entities (elements) belonging to this layer
    pub children: &'static Children,
    /// Whether this layer is currently visible/enabled
    visibility: &'static Visibility,
}

/// A query for element entities, containing all necessary components to work with elements
/// in the `DungeonRS` hierarchy.
///
/// Elements are the actual content items (sprites, tiles, objects, etc.) placed within
/// layers. They represent the lowest level of the hierarchy and contain no children.
#[derive(QueryData)]
pub struct ElementQuery {
    /// The entity ID of the element
    entity: Entity,
    /// The element-specific component data
    pub element: &'static Element,
    /// The human-readable name of the element
    pub name: &'static Name,
    /// The spatial transformation (position, rotation, scale) of the element
    pub transform: &'static Transform,
    /// Whether this element is currently visible/enabled
    pub visibility: &'static Visibility,
}

/// A system parameter that provides convenient access to all dungeon hierarchy queries.
///
/// This struct bundles together queries for all levels of the `DungeonRS` hierarchy
/// (Projects → Levels → Layers → Elements) and provides helper methods for navigating
/// between hierarchy levels.
///
/// # Usage
///
/// ```rust
/// use bevy::prelude::*;
/// use data::{DungeonQueries, Project};
///
/// fn my_system(queries: DungeonQueries, project_query: Query<Entity, With<Project>>) {
///     // Get all levels in a project
///     if let Ok(project_entity) = project_query.single() {
///         for level in queries.levels_for_project(project_entity) {
///             println!("Level: {}", level.name);
///         }
///     }
/// }
/// ```
#[derive(SystemParam)]
pub struct DungeonQueries<'w, 's> {
    /// Query for all project entities in the world
    pub projects: Query<'w, 's, ProjectQuery>,
    /// Query for all level entities in the world
    pub levels: Query<'w, 's, LevelQuery>,
    /// Query for all layer entities in the world
    pub layers: Query<'w, 's, LayerQuery>,
    /// Query for all element entities in the world
    pub elements: Query<'w, 's, ElementQuery>,
}

impl LevelQueryItem<'_> {
    /// Returns whether the layer is visible or not.
    #[inline]
    #[must_use]
    pub fn is_visible(&self) -> bool {
        match self.visibility {
            Visibility::Inherited | Visibility::Hidden => false,
            Visibility::Visible => true,
        }
    }
}

impl DungeonQueries<'_, '_> {
    /// Get all levels that belong to a specific project.
    ///
    /// This method traverses the hierarchy by looking up the project entity,
    /// then iterating through its children to find level entities.
    ///
    /// # Arguments
    ///
    /// * `project_entity` - The entity ID of the project to query
    ///
    /// # Returns
    ///
    /// An iterator over `LevelQueryItem` instances for all levels in the project.
    /// Returns an empty iterator if the project doesn't exist or has no level children.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy::prelude::*;
    /// use data::{DungeonQueries, Project};
    ///
    /// fn example_system(queries: DungeonQueries, project_query: Query<Entity, With<Project>>) {
    ///     if let Ok(project_entity) = project_query.single() {
    ///         for level in queries.levels_for_project(project_entity) {
    ///             println!("Found level: {}", level.name);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn levels_for_project(
        &self,
        project_entity: Entity,
    ) -> impl Iterator<Item = LevelQueryItem<'_>> + '_ {
        self.projects
            .get(project_entity)
            .ok()
            .into_iter()
            .flat_map(|project| project.children.iter())
            .filter_map(|&child| self.levels.get(child).ok())
    }

    /// Get all layers that belong to a specific level.
    ///
    /// This method traverses the hierarchy by looking up the level entity,
    /// then iterating through its children to find layer entities.
    ///
    /// # Arguments
    ///
    /// * `level_entity` - The entity ID of the level to query
    ///
    /// # Returns
    ///
    /// An iterator over `LayerQueryItem` instances for all layers in the level.
    /// Returns an empty iterator if the level doesn't exist or has no layer children.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy::prelude::*;
    /// use data::{DungeonQueries, Level};
    ///
    /// fn example_system(queries: DungeonQueries, level_query: Query<Entity, With<Level>>) {
    ///     if let Ok(level_entity) = level_query.single() {
    ///         for layer in queries.layers_for_level(level_entity) {
    ///             println!("Found layer: {} at {:?}", layer.name, layer.transform);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn layers_for_level(
        &self,
        level_entity: Entity,
    ) -> impl Iterator<Item = LayerQueryItem<'_>> + '_ {
        self.levels
            .get(level_entity)
            .ok()
            .into_iter()
            .flat_map(|level| level.children.iter())
            .filter_map(|&child| self.layers.get(child).ok())
    }

    /// Get all elements that belong to a specific layer.
    ///
    /// This method traverses the hierarchy by looking up the layer entity,
    /// then iterating through its children to find element entities.
    ///
    /// # Arguments
    ///
    /// * `layer_entity` - The entity ID of the layer to query
    ///
    /// # Returns
    ///
    /// An iterator over `ElementQueryItem` instances for all elements in the layer.
    /// Returns an empty iterator if the layer doesn't exist or has no element children.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy::prelude::*;
    /// use data::{DungeonQueries, Layer};
    ///
    /// fn example_system(queries: DungeonQueries, layer_query: Query<Entity, With<Layer>>) {
    ///     if let Ok(layer_entity) = layer_query.single() {
    ///         for element in queries.elements_for_layer(layer_entity) {
    ///             println!("Found element: {} at {:?}", element.name, element.transform);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn elements_for_layer(
        &self,
        layer_entity: Entity,
    ) -> impl Iterator<Item = ElementQueryItem<'_>> + '_ {
        self.layers
            .get(layer_entity)
            .ok()
            .into_iter()
            .flat_map(|layer| layer.children.iter())
            .filter_map(|&child| self.elements.get(child).ok())
    }
}

#[cfg(test)]
#[allow(clippy::missing_panics_doc)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::{Commands, World};
    use std::path::PathBuf;

    fn create_test_hierarchy(world: &mut World) -> (Entity, Entity, Entity, Entity) {
        let mut system_state: SystemState<Commands> = SystemState::new(world);
        let mut commands = system_state.get_mut(world);

        // Create project
        let project_entity = commands
            .spawn(Project::new(PathBuf::new(), "Test Project"))
            .id();

        // Create level
        let level_entity = commands
            .spawn((Level::new("Test Level"), Children::default()))
            .id();

        // Create layer
        let layer_entity = commands
            .spawn((
                Layer::new("Test Layer", Transform::default()),
                Children::default(),
            ))
            .id();

        // Create element
        let element_entity = commands
            .spawn((
                Name::new("Test Element"),
                Element::new_object("test-asset-id".to_string()),
                Transform::default(),
            ))
            .id();

        system_state.apply(world);

        // Set up parent-child relationships
        let mut system_state: SystemState<Commands> = SystemState::new(world);
        let mut commands = system_state.get_mut(world);

        // Add level as child of project
        commands.entity(project_entity).add_child(level_entity);
        // Add layer as child of level
        commands.entity(level_entity).add_child(layer_entity);
        // Add element as child of layer
        commands.entity(layer_entity).add_child(element_entity);

        system_state.apply(world);

        (project_entity, level_entity, layer_entity, element_entity)
    }

    #[test]
    fn test_project_query() {
        let mut world = World::new();
        let (project_entity, _, _, _) = create_test_hierarchy(&mut world);

        let mut system_state: SystemState<Query<ProjectQuery>> = SystemState::new(&mut world);
        let query = system_state.get(&world);

        let project = query.get(project_entity).expect("Project should exist");
        assert_eq!(project.entity, project_entity);
        assert_eq!(project.name.as_str(), "Test Project");
        assert!(!project.children.is_empty(), "Project should have children");
    }

    #[test]
    fn test_level_query() {
        let mut world = World::new();
        let (_, level_entity, _, _) = create_test_hierarchy(&mut world);

        let mut system_state: SystemState<Query<LevelQuery>> = SystemState::new(&mut world);
        let query = system_state.get(&world);

        let level = query.get(level_entity).expect("Level should exist");
        assert_eq!(level.entity, level_entity);
        assert_eq!(level.name.as_str(), "Test Level");
        assert!(!level.children.is_empty(), "Level should have children");
    }

    #[test]
    fn test_layer_query() {
        let mut world = World::new();
        let (_, _, layer_entity, _) = create_test_hierarchy(&mut world);

        let mut system_state: SystemState<Query<LayerQuery>> = SystemState::new(&mut world);
        let query = system_state.get(&world);

        let layer = query.get(layer_entity).expect("Layer should exist");
        assert_eq!(layer.entity, layer_entity);
        assert_eq!(layer.name.as_str(), "Test Layer");
        assert!(!layer.children.is_empty(), "Layer should have children");
    }

    #[test]
    fn test_element_query() {
        let mut world = World::new();
        let (_, _, _, element_entity) = create_test_hierarchy(&mut world);

        let mut system_state: SystemState<Query<ElementQuery>> = SystemState::new(&mut world);
        let query = system_state.get(&world);

        let element = query.get(element_entity).expect("Element should exist");
        assert_eq!(element.entity, element_entity);
        assert_eq!(element.name.as_str(), "Test Element");
    }

    #[test]
    fn test_dungeon_queries_levels_for_project() {
        let mut world = World::new();
        let (project_entity, level_entity, _, _) = create_test_hierarchy(&mut world);

        let mut system_state: SystemState<DungeonQueries> = SystemState::new(&mut world);
        let queries = system_state.get(&world);

        let levels: Vec<_> = queries.levels_for_project(project_entity).collect();
        assert_eq!(levels.len(), 1, "Should find exactly one level");
        assert_eq!(levels[0].entity, level_entity);
        assert_eq!(levels[0].name.as_str(), "Test Level");
    }

    #[test]
    fn test_dungeon_queries_layers_for_level() {
        let mut world = World::new();
        let (_, level_entity, layer_entity, _) = create_test_hierarchy(&mut world);

        let mut system_state: SystemState<DungeonQueries> = SystemState::new(&mut world);
        let queries = system_state.get(&world);

        let layers: Vec<_> = queries.layers_for_level(level_entity).collect();
        assert_eq!(layers.len(), 1, "Should find exactly one layer");
        assert_eq!(layers[0].entity, layer_entity);
        assert_eq!(layers[0].name.as_str(), "Test Layer");
    }

    #[test]
    fn test_dungeon_queries_elements_for_layer() {
        let mut world = World::new();
        let (_, _, layer_entity, element_entity) = create_test_hierarchy(&mut world);

        let mut system_state: SystemState<DungeonQueries> = SystemState::new(&mut world);
        let queries = system_state.get(&world);

        let elements: Vec<_> = queries.elements_for_layer(layer_entity).collect();
        assert_eq!(elements.len(), 1, "Should find exactly one element");
        assert_eq!(elements[0].entity, element_entity);
        assert_eq!(elements[0].name.as_str(), "Test Element");
    }

    #[test]
    fn test_dungeon_queries_nonexistent_entities() {
        let mut world = World::new();
        let _ = create_test_hierarchy(&mut world);

        let mut system_state: SystemState<DungeonQueries> = SystemState::new(&mut world);
        let queries = system_state.get(&world);

        let fake_entity = Entity::from_raw(9999);

        // Test with nonexistent entities should return empty iterators
        let levels: Vec<_> = queries.levels_for_project(fake_entity).collect();
        assert_eq!(
            levels.len(),
            0,
            "Should find no levels for nonexistent project"
        );

        let layers: Vec<_> = queries.layers_for_level(fake_entity).collect();
        assert_eq!(
            layers.len(),
            0,
            "Should find no layers for nonexistent level"
        );

        let elements: Vec<_> = queries.elements_for_layer(fake_entity).collect();
        assert_eq!(
            elements.len(),
            0,
            "Should find no elements for nonexistent layer"
        );
    }

    #[test]
    fn test_multiple_children() {
        let mut world = World::new();

        let mut system_state: SystemState<Commands> = SystemState::new(&mut world);
        let mut commands = system_state.get_mut(&mut world);

        // Create a project with multiple levels
        let project_entity = commands
            .spawn(Project::new(PathBuf::new(), "Multi-Level Project"))
            .id();
        let level1_entity = commands
            .spawn((Level::new("Level 1"), Children::default()))
            .id();
        let level2_entity = commands
            .spawn((Level::new("Level 2"), Children::default()))
            .id();

        system_state.apply(&mut world);

        // Set up parent-child relationships
        let mut system_state: SystemState<Commands> = SystemState::new(&mut world);
        let mut commands = system_state.get_mut(&mut world);

        commands.entity(project_entity).add_child(level1_entity);
        commands.entity(project_entity).add_child(level2_entity);

        system_state.apply(&mut world);

        // Test querying multiple children
        let mut system_state: SystemState<DungeonQueries> = SystemState::new(&mut world);
        let queries = system_state.get(&world);

        let levels: Vec<_> = queries.levels_for_project(project_entity).collect();
        assert_eq!(levels.len(), 2, "Should find exactly two levels");

        let level_names: Vec<&str> = levels.iter().map(|l| l.name.as_str()).collect();
        assert!(level_names.contains(&"Level 1"));
        assert!(level_names.contains(&"Level 2"));
    }
}
