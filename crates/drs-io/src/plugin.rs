//! Contains the [`IOPlugin`] that registers the required systems and data in the bevy `App`.
use crate::load_project::handle_load_project_event;
use crate::save_project::{SaveProjectCompleteEvent, handle_save_project};
use crate::{LoadProjectEvent, SaveProjectEvent};
use bevy::prelude::{App, FixedPostUpdate, Plugin};

/// Sets up listening for events to persist or load data from the disk.
pub struct IOPlugin;

impl Plugin for IOPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveProjectEvent>()
            .add_event::<SaveProjectCompleteEvent>()
            .add_systems(FixedPostUpdate, handle_save_project);

        app.add_event::<LoadProjectEvent>()
            .add_systems(FixedPostUpdate, handle_load_project_event);
    }
}
