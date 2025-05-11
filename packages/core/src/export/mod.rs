mod events;
mod ongoing_export;
mod systems;

use crate::export::ongoing_export::{OngoingExport, ExportState};
use crate::export::systems::{advance_camera, attach_readback};
use bevy::app::App;
use bevy::prelude::{
    FixedPostUpdate, IntoScheduleConfigs, Plugin, PostUpdate, Res, not, resource_exists,
};
pub use events::*;

#[derive(Default)]
pub struct ExportPlugin;

impl Plugin for ExportPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExportRequest>()
            .add_event::<ExportProgress>()
            .add_event::<ExportCompleted>();

        app.add_systems(
            PostUpdate,
            (
                attach_readback.run_if(in_state(ExportState::Preparing)),
                advance_camera.run_if(in_state(ExportState::Capturing)),
            ),
        );
        app.add_systems(
            FixedPostUpdate,
            systems::on_export_request.run_if(not(resource_exists::<OngoingExport>)),
        );
    }
}

fn in_state(_state: ExportState) -> impl FnMut(Option<Res<OngoingExport>>) -> bool {
    |export: Option<Res<OngoingExport>>| {
        let Some(export) = export else {
            return false;
        };

        matches!(&export.state, _state)
    }
}
