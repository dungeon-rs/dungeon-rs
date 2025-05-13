mod callbacks;
mod events;
mod ongoing;
mod state;
mod systems;
mod tasks;

use crate::{
    export::ongoing::OngoingExport,
    export::state::ExportState,
    export::systems::{check_for_requests, prepare_and_advance_camera, wait_for_image_processing},
};
use bevy::app::App;
use bevy::prelude::{IntoScheduleConfigs, Plugin, PostUpdate, Res, Update, not, resource_exists};

use crate::export::systems::clean_up;
pub use events::*;

#[derive(Default)]
pub struct ExportPlugin;

impl Plugin for ExportPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExportRequest>()
            .add_event::<ExportProgress>()
            .add_event::<ExportCompleted>()
            .add_event::<ExportFailed>();

        // If there are no ongoing exports, check for requests.
        app.add_systems(
            PostUpdate,
            (
                check_for_requests.run_if(not(resource_exists::<OngoingExport>)),
                wait_for_image_processing.run_if(in_state(ExportState::ProcessFrames)),
                clean_up.run_if(in_state(ExportState::Cleanup)),
            ),
        )
        .add_systems(
            Update,
            prepare_and_advance_camera.run_if(resource_exists::<OngoingExport>),
        );
    }
}

/// Generates a [bevy::prelude::Condition] that validates a system should run this frame based on the current export state.
fn in_state(state: ExportState) -> impl Fn(Option<Res<OngoingExport>>) -> bool {
    move |export: Option<Res<OngoingExport>>| {
        let Some(export) = export else {
            return false;
        };

        export.state == state
    }
}
