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
    ///             Level::new("Ground Floor")
    ///         ]
    ///     ));
    /// # }
    /// ```
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(name: impl Into<Cow<'static, str>>) -> impl Bundle {
        (Name::new(name), Level {})
    }
}
