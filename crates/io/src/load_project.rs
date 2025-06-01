use crate::document::Document;
use anyhow::Context;
use bevy::prelude::{BevyError, Commands, Event, EventReader, Transform, default};
use data::{Layer, Level, Project};
use serialization::deserialize;
use std::fs::read;
use std::path::PathBuf;

#[derive(Event, Debug)]
pub struct LoadProjectEvent {
    pub input: PathBuf,
}

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
                                .spawn(Layer::new_with_transform(
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
