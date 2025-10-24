//! Contains the messages for saving projects and their handling systems.
use crate::persistence::Document;
use anyhow::Context;
use bevy::prelude::{BevyError, Commands, Entity, Message, MessageReader, Query, default};
use drs_data::{ElementQuery, LayerQuery, LevelQuery, ProjectQuery};
use drs_serialization::serialize_to;
use drs_utils::{AsyncComponent, report_progress};
use std::fs::File;

/// When this message is sent, the associated `project` will be fetched and saved.
/// As a reaction to this message, a system will build a [`bevy::prelude::Query`] that attempts to
/// fetch all [`drs_data::Project`]'s [`drs_data::Level`]s (and their descendant [`drs_data::Layer`]s along
/// all their descendants) and then (attempts) to persist them to disk.
///
/// The [`drs_data::Project`] fetched is queried by the [`SaveProjectMessage::project`] [`Entity`],
/// the user is responsible for only emitting "valid" entities, as this crate will assume they are,
/// and according to Bevy's own documentation, this can lead to undefined behaviour if not respected.
#[derive(Message, Debug)]
pub struct SaveProjectMessage {
    /// The [`Entity`] of the [`drs_data::Project`] to save.
    pub(crate) project: Entity,
}

/// This message indicates that the work of a [`SaveProjectMessage`] has completed.
#[derive(Message, Debug)]
pub struct SaveProjectCompleteMessage {
    /// The [`Entity`] of the [`drs_data::Project`] that was saved.
    pub project: Entity,
}

/// This message indicates that the work of a [`SaveProjectMessage`] has failed.
#[derive(Message, Debug)]
pub struct SaveProjectFailedMessage {
    /// The [`Entity`] of the [`drs_data::Project`] that failed to save.
    pub project: Entity,
    /// The error that prmessageed the project from being saved.
    pub error: BevyError,
}

impl SaveProjectMessage {
    /// Generate a new [`SaveProjectMessage`] that can be dispatched.
    #[must_use = "This message does nothing unless you dispatch it"]
    pub fn new(project: Entity) -> Self {
        Self { project }
    }
}

/// Bevy system that handles [`SaveProjectMessage`] messages.
#[drs_utils::bevy_system]
pub fn handle_save_project(
    mut commands: Commands,
    mut messages: MessageReader<SaveProjectMessage>,
    project_query: Query<ProjectQuery>,
    level_query: Query<LevelQuery>,
    layer_query: Query<LayerQuery>,
    object_query: Query<ElementQuery>,
) -> Result<(), BevyError> {
    let Some(message) = messages.read().next() else {
        return Ok(());
    };

    // Attempt to fetch the project. If this fails, we dispatch a failed message and return.
    let project = match project_query.get(message.project) {
        Ok(project) => project,
        Err(error) => {
            commands.write_message(SaveProjectFailedMessage {
                project: message.project,
                error: error.into(),
            });

            return Err(error.into());
        }
    };

    let entity = message.project;
    let output = project.project.file.clone();
    let document = Document::new(&project, level_query, layer_query, object_query);
    commands.spawn(AsyncComponent::new_io(
        async move |sender| {
            let file = File::create(output.clone()).with_context(|| {
                format!("Failed to open {} for writing savefile", output.display())
            })?;
            serialize_to(&document, &default(), file)?;

            // Report completion
            report_progress(&sender, SaveProjectCompleteMessage { project: entity })?;
            Ok(())
        },
        move |error, sender| {
            let _ = report_progress(
                &sender,
                SaveProjectFailedMessage {
                    project: entity,
                    error,
                },
            );
        },
    ));

    Ok(())
}
