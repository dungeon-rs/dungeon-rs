//! Contains the events for saving projects and their handling systems.
use crate::document::Document;
use anyhow::Context;
use bevy::prelude::{BevyError, Commands, Entity, Event, EventReader, Query, default};
use drs_data::{ElementQuery, LayerQuery, LevelQuery, ProjectQuery};
use serialization::serialize_to;
use std::fs::File;
use utils::{AsyncComponent, report_progress};

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

impl SaveProjectEvent {
    /// Generate a new [`SaveProjectEvent`] that can be dispatched.
    #[must_use = "This event does nothing unless you dispatch it"]
    pub fn new(project: Entity) -> Self {
        Self { project }
    }
}

/// Bevy system that handles [`SaveProjectEvent`] events.
///
/// TODO: add error reporting
#[utils::bevy_system]
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

    let project = project_query.get(event.project)?;

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
        |_, _| {
            // TODO: handle errors.
        },
    ));

    Ok(())
}
