//! Contains the [`CorePlugin`] that registers the listeners for all messages exposed by this crate.
use crate::persistence::{handle_load_project_message, handle_save_project};
use crate::project::handle_create_project_message;
use crate::{
    CreateProjectMessage, LoadProjectCompleteMessage, LoadProjectFailedMessage, LoadProjectMessage,
    SaveProjectCompleteMessage, SaveProjectFailedMessage, SaveProjectMessage,
};
use bevy::prelude::{App, FixedPostUpdate, Plugin};

/// The [`CorePlugin`] is responsible for registering all the systems and messages
/// that the `drs-core` crate exposes.
///
/// This includes, but is not limited to, saving and loading projects, ...
#[derive(Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveProjectMessage>()
            .add_message::<SaveProjectCompleteMessage>()
            .add_message::<SaveProjectFailedMessage>()
            .add_message::<SaveProjectMessage>()
            .add_systems(FixedPostUpdate, handle_save_project);

        app.add_message::<LoadProjectMessage>()
            .add_message::<LoadProjectCompleteMessage>()
            .add_message::<LoadProjectFailedMessage>()
            .add_systems(FixedPostUpdate, handle_load_project_message);

        app.add_message::<CreateProjectMessage>()
            .add_systems(FixedPostUpdate, handle_create_project_message);
    }
}
