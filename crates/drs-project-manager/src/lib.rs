#![doc = include_str!("../README.md")]

use bevy_app::{App, Plugin, Startup};
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::system::Commands;
use drs_model::{Layer, Level, Project};

/// Creates the new Project the editor opens on.
pub struct ProjectManagerPlugin;

impl Plugin for ProjectManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_new_project);
    }
}

/// Spawns a new, unsaved Project with one Level holding one Layer.
///
/// The Grid, the Bounds, and the Asset Reference table come with the Project by default.
fn create_new_project(mut commands: Commands) {
    let project = commands
        .spawn(Project {
            name: "Untitled".to_owned(),
        })
        .id();
    let level = commands
        .spawn((
            Level {
                name: "Level 1".to_owned(),
            },
            ChildOf(project),
        ))
        .id();
    commands.spawn((
        Layer {
            name: "Layer 1".to_owned(),
        },
        ChildOf(level),
    ));
}
