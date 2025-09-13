//! Contains the [`LoadProjectEvent`] and it's handler systems.
use crate::persistence::Document;
use anyhow::Context;
use bevy::log::debug;
use bevy::prelude::{
    BevyError, Commands, Entity, Event, EventReader, Single, Transform, default, info, info_span,
};
use drs_data::{Layer, Level, Project, ProjectQuery};
use drs_serialization::deserialize;
use std::fs::read;
use std::path::{Path, PathBuf};

/// Emitting this event will cause the software to attempt loading a project file at the given `input`.
///
/// The progress, result or failure of this event's actions will also be emitted as events.
/// TODO: add events to indicate progress, success or failure
#[derive(Event, Debug)]
pub struct LoadProjectEvent {
    /// The path to the project file to load.
    pub input: PathBuf,
}

/// Indicates that the project file has been loaded successfully.
#[derive(Event, Debug)]
pub struct LoadProjectCompleteEvent {
    /// The entity of the project that was loaded.
    pub project: Entity,
}

/// Indicates that the project file failed to load.
#[derive(Event, Debug)]
pub struct LoadProjectFailedEvent {
    /// The path to the project file that failed to load.
    pub input: PathBuf,
    /// The error that caused loading to fail.
    pub error: BevyError,
}

/// Bevy system that handles `LoadProjectEvent` events that were fired.
#[drs_utils::bevy_system]
pub fn handle_load_project_event(
    projects: Option<Single<ProjectQuery>>,
    mut events: EventReader<LoadProjectEvent>,
    mut commands: Commands,
) {
    // Only handle a single load event per frame, we don't want to cram too much work in a single frame.
    let Some(event) = events.read().next() else {
        return;
    };

    let _ = info_span!("load_project", path = event.input.to_str()).entered();

    if let Some(project) = projects {
        debug!("despawning previous project");

        commands.entity(project.entity).despawn();
    }

    let project = match read_and_parse(&event.input) {
        Ok(project) => project,
        Err(error) => {
            commands.send_event(LoadProjectFailedEvent {
                input: event.input.clone(),
                error,
            });

            return;
        }
    };

    info!(
        "Loaded project: {}, spawning {level_count} levels",
        project.name,
        level_count = project.levels.len()
    );
    let project = commands
        .spawn(Project::new(event.input.clone(), project.name))
        .with_children(|commands| {
            for level in project.levels {
                commands
                    .spawn(Level::new(level.name))
                    .with_children(|commands| {
                        for layer in level.layers {
                            commands
                                .spawn(Layer::new(
                                    layer.name,
                                    Transform::from_xyz(0.0, 0.0, layer.order),
                                ))
                                .with_children(|_commands| {
                                    for _item in layer.items {
                                        // TODO: spawn item
                                    }
                                });
                        }
                    });
            }
        })
        .id();

    commands.send_event(LoadProjectCompleteEvent { project });
}

/// Handles reading the input file and parsing it into a domain structure.
/// Refactored into a separate function for easier error handling in the system itself.
///
/// # Errors
/// returns an error when either the file fails to read or it's contents are not a correct format.
fn read_and_parse(input: &Path) -> Result<Document, BevyError> {
    let content = read(input)
        .with_context(|| format!("Failed to open project file: '{}'", input.display()))?;
    let project = deserialize::<Document>(&content, &default())
        .with_context(|| format!("Failed to parse project file '{}'", input.display()))?;

    Ok(project)
}
