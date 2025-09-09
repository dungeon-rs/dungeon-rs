//! Defines the [`Level`] struct and it's implementations.
use bevy::prelude::{Bundle, Component, Name, Transform, Visibility};
use std::borrow::Cow;

/// A [`Level`] indicates a single visible subset of a map, most commonly used to represent
/// distinct parts of the same logical project.
///
/// An example would be a 'project' that represents a full roadside tavern which might have a ground
/// floor, a basement and an upper floor for lodging. Each physical floor would be represented under
/// a [`Level`] so that all floors are grouped in the same savefile (project) "Roadside Tavern" but
/// the levels are separated so that the user can edit/export each physical floor separately.
#[derive(Component, Default)]
#[component(immutable)]
#[cfg_attr(feature = "dev", derive(bevy::prelude::Reflect))]
#[require(Transform::from_xyz(0.0, 0.0, 0.0), Visibility::default())]
pub struct Level;

impl Level {
    /// Generates a new [`Bundle`] with a level to indicate the start of a hierarchy under which
    /// a map will be set.
    ///
    /// # Examples
    ///
    /// Here's how to spawn a simple `Level` for the ground floor.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use data::Project;
    /// # use data::Level;
    /// # use std::path::PathBuf;
    /// #
    /// # fn main() {
    /// #   App::new()
    /// #       .add_systems(Startup, spawn_project)
    /// #       .run();
    /// # }
    /// #
    /// # fn spawn_project(mut commands: Commands) {
    /// #   let output = PathBuf::new();
    ///     commands.spawn((
    ///         Project::new(output, "Roadside Inn"),
    ///         children![
    ///             Level::new("Ground Floor")
    ///         ]
    ///     ));
    /// # }
    /// ```
    #[allow(clippy::new_ret_no_self)]
    #[must_use = "Level won't be added to the world unless spawned"]
    pub fn new(name: impl Into<Cow<'static, str>>) -> impl Bundle {
        (Name::new(name), Level {})
    }
}
