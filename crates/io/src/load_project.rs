//! Contains the [`LoadProjectEvent`] and it's handler systems.
use crate::document::Document;
use anyhow::Context;
use bevy::prelude::{BevyError, Commands, Event, EventReader, Transform, default};
use data::{Layer, Level, Project};
use serialization::deserialize;
use std::fs::read;
use std::path::PathBuf;

/// Emitting this event will cause the software to attempt loading a project file at the given `input`.
///
/// The progress, result or failure of this event's actions will also be emitted as events.
/// TODO: add events to indicate progress, success or failure
#[derive(Event, Debug)]
pub struct LoadProjectEvent {
    /// The path to the project file to load.
    pub input: PathBuf,
}

/// Bevy system that handles `LoadProjectEvent` events that were fired.
#[utils::bevy_system]
pub fn handle_load_project_event(
    mut events: EventReader<LoadProjectEvent>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let Some(event) = events.read().next() else {
        return Ok(());
    };

    let content = read(event.input.clone())
        .with_context(|| format!("Failed to open project file: '{}'", event.input.display()))?;
    let project = deserialize::<Document>(&content, &default())
        .with_context(|| format!("Failed to parse project file '{}'", event.input.display()))?;

    commands
        .spawn(Project::new(project.name))
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
        });

    Ok(())
}
