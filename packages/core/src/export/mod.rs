mod events;
mod screenshot;
mod systems;

use crate::export::screenshot::{Screenshot, ScreenshotStatus};
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
                attach_readback.run_if(in_state(ScreenshotStatus::Preparing)),
                advance_camera.run_if(in_state(ScreenshotStatus::Capturing)),
            ),
        );
        app.add_systems(
            FixedPostUpdate,
            systems::on_export_request.run_if(not(resource_exists::<Screenshot>)),
        );
    }
}

fn in_state(_state: ScreenshotStatus) -> impl FnMut(Option<Res<Screenshot>>) -> bool {
    |screenshot: Option<Res<Screenshot>>| {
        let Some(screenshot) = screenshot else {
            return false;
        };

        matches!(&screenshot.status, _state)
    }
}
