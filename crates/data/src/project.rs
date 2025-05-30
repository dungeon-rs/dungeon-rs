use bevy::prelude::{Bundle, Component, Name, Transform, Visibility};
use std::borrow::Cow;

/// Top level component in the hierarchy used by `DungeonRS` to identify components that relate to the
/// map the user is editing. In short, only components under this [`Project`] component will be considered
/// when saving/loading a map or exporting.
///
/// This allows editing tools to spawn additional components (like Gizmo's, temporary stamps, ...)
/// without having them accidentally included in any export or persistence operations.
///
/// This distinction also allows a (minor) performance increase since queries can be run on a subset
/// of the ECS hierarchy rather than all components available.
#[derive(Component)]
#[component(immutable)]
#[require(Transform::from_xyz(0.0, 0.0, 0.0), Visibility::default())]
pub struct Project;

impl Project {
    /// Generates a new [`Bundle`] with a project to indicate the start of a hierarchy under which
    /// the map (often referred to as 'project', hence the name) will be set.
    ///
    /// # Examples
    ///
    /// Here's how to spawn a simple `Project` named "Roadside Inn"
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use data::Project;
    /// #
    /// # fn main() {
    /// #   App::new()
    /// #       .add_systems(Startup, spawn_project)
    /// #       .run();
    /// # }
    /// #
    /// # fn spawn_project(mut commands: Commands) {
    ///     commands.spawn(Project::new("Roadside Inn"));
    /// # }
    /// ```
    #[allow(clippy::new_ret_no_self)]
    #[must_use = "Project won't be added to the world unless spawned"]
    pub fn new(name: impl Into<Cow<'static, str>>) -> impl Bundle {
        (Name::new(name), Project {})
    }
}
