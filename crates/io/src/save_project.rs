use crate::document::Document;
use anyhow::Context;
use bevy::prelude::default;
use bevy::prelude::{
    BevyError, Children, Entity, Event, EventReader, Name, Query, Transform, With,
};
use data::{Layer, Level, Project};
use serialization::serialize_to;
use std::{fs::File, path::PathBuf};

/// When this event is sent, the associated `project` will be fetched and saved.
/// As a reaction to this event, a system will build a [`bevy::prelude::Query`] that attempts to
/// fetch all [`data::Project`]'s [`data::Level`]s (and their descendant [`data::Layer`]s)
/// and then (attempts) to persist them to disk.
///
/// The [`data::Project`] fetched is queried by the [`SaveProjectEvent::project`] [`Entity`],
/// the user is responsible for only emitting "valid" entities, as this crate will assume they are,
/// and according to Bevy's own documentation, this can lead to undefined behaviour if not respected.
#[derive(Event, Debug)]
#[allow(rustdoc::private_intra_doc_links)]
pub struct SaveProjectEvent {
    /// The [`Entity`] of the [`data::Project`] to save.
    pub(crate) project: Entity,
    /// The output path of the savefile that will be created.
    pub(crate) output: PathBuf,
}

impl SaveProjectEvent {
    /// Generate a new [`SaveProjectEvent`] that can be dispatched.
    #[must_use = "This event does nothing unless you dispatch it"]
    pub fn new(project: Entity, output: PathBuf) -> Self {
        Self { project, output }
    }
}

/// Bevy system that handles [`SaveProjectEvent`] events.
#[allow(clippy::needless_pass_by_value)]
pub fn handle_save_project(
    mut events: EventReader<SaveProjectEvent>,
    project_query: Query<(&Name, &Children), With<Project>>,
    level_query: Query<(&Level, &Name, &Children)>,
    layer_query: Query<(&Layer, &Name, &Transform, &Children)>,
) -> Result<(), BevyError> {
    let Some(event) = events.read().next() else {
        return Ok(());
    };

    let project = project_query.get(event.project)?;
    let document = Document::new(project, level_query, layer_query);
    // TODO: we should probably write asynchronously to files
    let file = File::create(event.output.clone()).with_context(|| {
        format!(
            "Failed to open {} for writing savefile",
            event.output.display()
        )
    })?;
    serialize_to(&document, &default(), file)?;

    Ok(())
}
