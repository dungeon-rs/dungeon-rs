//! Contains events and systems related to handling [`drs_data::Project`] instances.
//! Things like creating, updating and unloading projects.

use bevy::ecs::children;
use bevy::prelude::SpawnRelated;
use bevy::prelude::{Commands, Event, EventReader, Single, Transform, debug, info_span};
use drs_data::{Layer, Level, Project, ProjectQuery};
use std::path::PathBuf;

/// Event emitted when a new project should be created.
#[derive(Event, Debug)]
pub struct CreateProjectEvent {
    /// See the `file` argument of [`drs_data::Project::new`].
    pub file: PathBuf,
    /// See the `name` argument of [`drs_data::Project::new`].
    pub name: String,
}

impl CreateProjectEvent {
    /// Creates a new [`CreateProjectEvent`].
    #[must_use]
    pub fn new(file: PathBuf, name: String) -> Self {
        Self { file, name }
    }
}

/// When creating a new project we need to ensure that the previous `Project` (if any) is unloaded.
///
/// Only then we spawn the new project alongside a [`drs_data::Level`] and [`drs_data::Layer`].
#[drs_utils::bevy_system]
pub fn handle_create_project_event(
    mut commands: Commands,
    projects: Option<Single<ProjectQuery>>,
    mut events: EventReader<CreateProjectEvent>,
) {
    let Some(event) = events.read().next() else {
        return;
    };

    let _ = info_span!("create_project").entered();

    if let Some(project) = projects {
        debug!("despawning previous project");

        commands.entity(project.entity).despawn();
    }

    debug!("Spawning new project '{name}'", name = event.name);
    commands.spawn((
        Project::new(event.file.clone(), event.name.clone()),
        children![(
            Level::new("default"),
            children![(Layer::new("default", Transform::default()), children![],)]
        )],
    ));
}
