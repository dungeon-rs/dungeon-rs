//! Contains the [`CorePlugin`] that registers the listeners for all events exposed by this crate.
use crate::persistence::{handle_load_project_event, handle_save_project};
use crate::project::handle_create_project_event;
use crate::{
    CreateProjectEvent, LoadProjectCompleteEvent, LoadProjectEvent, LoadProjectFailedEvent,
    SaveProjectCompleteEvent, SaveProjectEvent, SaveProjectFailedEvent,
};
use bevy::prelude::{App, FixedPostUpdate, Plugin};

/// The [`CorePlugin`] is responsible for registering all the systems and events
/// that the `drs-core` crate exposes.
///
/// This includes, but is not limited to, saving and loading projects, ...
#[derive(Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveProjectEvent>()
            .add_event::<SaveProjectCompleteEvent>()
            .add_event::<SaveProjectFailedEvent>()
            .add_event::<SaveProjectEvent>()
            .add_systems(FixedPostUpdate, handle_save_project);

        app.add_event::<LoadProjectEvent>()
            .add_event::<LoadProjectCompleteEvent>()
            .add_event::<LoadProjectFailedEvent>()
            .add_systems(FixedPostUpdate, handle_load_project_event);

        app.add_event::<CreateProjectEvent>()
            .add_systems(FixedPostUpdate, handle_create_project_event);
    }
}
