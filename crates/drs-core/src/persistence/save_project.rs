//! Contains the events for saving projects and their handling systems.
use crate::persistence::Document;
use anyhow::Context;
use bevy::prelude::{default, BevyError, Commands, Entity, Event, EventReader, Query};
use drs_data::{ElementQuery, LayerQuery, LevelQuery, ProjectQuery};
use drs_serialization::serialize_to;
use drs_utils::{report_progress, AsyncComponent};
use std::fs::File;

/// When this event is sent, the associated `project` will be fetched and saved.
/// As a reaction to this event, a system will build a [`bevy::prelude::Query`] that attempts to
/// fetch all [`drs_data::Project`]'s [`drs_data::Level`]s (and their descendant [`drs_data::Layer`]s along
/// all their descendants) and then (attempts) to persist them to disk.
///
/// The [`drs_data::Project`] fetched is queried by the [`SaveProjectEvent::project`] [`Entity`],
/// the user is responsible for only emitting "valid" entities, as this crate will assume they are,
/// and according to Bevy's own documentation, this can lead to undefined behaviour if not respected.
#[derive(Event, Debug)]
pub struct SaveProjectEvent {
    /// The [`Entity`] of the [`drs_data::Project`] to save.
    pub(crate) project: Entity,
}

/// This event indicates that the work of a [`SaveProjectEvent`] has completed.
#[derive(Event, Debug)]
pub struct SaveProjectCompleteEvent {
    /// The [`Entity`] of the [`drs_data::Project`] that was saved.
    pub project: Entity,
}

/// This event indicates that the work of a [`SaveProjectEvent`] has failed.
#[derive(Event, Debug)]
pub struct SaveProjectFailedEvent {
    /// The [`Entity`] of the [`drs_data::Project`] that failed to save.
    pub project: Entity,
    /// The error that prevented the project from being saved.
    pub error: BevyError,
}

impl SaveProjectEvent {
    /// Generate a new [`SaveProjectEvent`] that can be dispatched.
    #[must_use = "This event does nothing unless you dispatch it"]
    pub fn new(project: Entity) -> Self {
        Self { project }
    }
}

/// Bevy system that handles [`SaveProjectEvent`] events.
#[drs_utils::bevy_system]
pub fn handle_save_project(
    mut commands: Commands,
    mut events: EventReader<SaveProjectEvent>,
    project_query: Query<ProjectQuery>,
    level_query: Query<LevelQuery>,
    layer_query: Query<LayerQuery>,
    object_query: Query<ElementQuery>,
) -> Result<(), BevyError> {
    let Some(event) = events.read().next() else {
        return Ok(());
    };

    // Attempt to fetch the project. If this fails, we dispatch a failed event and return.
    let project = match project_query.get(event.project) {
        Ok(project) => project,
        Err(error) => {
            commands.send_event(SaveProjectFailedEvent {
                project: event.project,
                error: error.into(),
            });

            return Err(error.into());
        }
    };

    let entity = event.project;
    let output = project.project.file.clone();
    let document = Document::new(&project, level_query, layer_query, object_query);
    commands.spawn(AsyncComponent::new_io(
        async move |sender| {
            let file = File::create(output.clone()).with_context(|| {
                format!("Failed to open {} for writing savefile", output.display())
            })?;
            serialize_to(&document, &default(), file)?;

            // Report completion
            report_progress(&sender, SaveProjectCompleteEvent { project: entity })?;
            Ok(())
        },
        move |error, sender| {
            let _ = report_progress(&sender, SaveProjectFailedEvent {
                project: entity,
                error,
            });
        },
    ));

    Ok(())
}
