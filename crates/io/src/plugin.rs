use crate::{LoadProjectEvent, SaveProjectEvent};
use crate::save_project::handle_save_project;
use bevy::prelude::{App, FixedPostUpdate, Plugin};
use crate::load_project::handle_load_project_event;

/// Sets up listening for events to persist or load data from the disk.
pub struct IOPlugin;

impl Plugin for IOPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveProjectEvent>()
            .add_systems(FixedPostUpdate, handle_save_project);

        app.add_event::<LoadProjectEvent>()
            .add_systems(FixedPostUpdate, handle_load_project_event);
    }
}
