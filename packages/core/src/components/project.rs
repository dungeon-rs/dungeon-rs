use bevy::prelude::{Component, Rect, Transform, Visibility};

/// Represents a project in `DungeonRS`, which can be compared to a workspace or folder.
///
/// A project is the marker component for the ECS hierarchy that contains everything you want saved
/// when the user "saves" a map.
///
/// Typically, a [Project] will contain [`crate::components::Layer`]s which in turn contain the elements that the
/// user sees (images, paths, patterns, ...).
#[derive(Component)]
#[component(immutable)]
#[require(Transform::from_xyz(0.0, 0.0, 0.0), Visibility::default())]
pub struct Project {
    pub size: Rect,
}

impl Project {
    /// Generate a new [Project] instance.
    /// The [Rect] passed in determines the exported region.
    #[must_use] pub fn new(size: Rect) -> Self {
        Self { size }
    }
}
