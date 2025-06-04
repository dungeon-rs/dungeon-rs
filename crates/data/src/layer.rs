//! Defines the [`Layer`] struct and it's implementations.
use bevy::prelude::{Bundle, Component, Name, Transform, Visibility};
use std::borrow::Cow;

/// A [`Layer`] represents a distinct editing plane within a [`crate::Level`], allowing elements to be
/// grouped by function or intent (e.g., "Walls", "Objects", "Lighting").
///
/// Layers are most commonly used for selective visibility, per-layer export, and to control edit
/// interactions (such as locking or isolating layers during drawing or object placement).
#[derive(Component, Default)]
#[component(immutable)]
#[require(Visibility::default())]
pub struct Layer;

impl Layer {
    /// Generates a new [`Bundle`] with a layer under which graphic items can be grouped.
    /// The `transform` determines how the layer is positioned relative to other layers.
    ///
    /// # Examples
    ///
    /// Here's how to spawn a simple `Layer` for the Walls.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use data::Project;
    /// # use data::Level;
    /// # use data::Layer;
    /// #
    /// # fn main() {
    /// #   App::new()
    /// #       .add_systems(Startup, spawn_project)
    /// #       .run();
    /// # }
    /// #
    /// # fn spawn_project(mut commands: Commands) {
    ///     commands.spawn((
    ///         Project::new("Roadside Inn"),
    ///         children![
    ///             Level::new("Ground Floor"),
    ///             children![
    ///                 Layer::new("Walls", Transform::IDENTITY),
    ///             ]
    ///         ]
    ///     ));
    /// # }
    /// ```
    #[allow(clippy::new_ret_no_self)]
    #[must_use = "Layer won't be added to the world unless spawned"]
    pub fn new(name: impl Into<Cow<'static, str>>, transform: Transform) -> impl Bundle {
        (Name::new(name), Layer {}, transform)
    }
}
