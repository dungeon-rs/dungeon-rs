use crate::export::screenshot::{Screenshot, ScreenshotStatus};
use crate::export::{ExportProgress, ExportStatus};
use bevy::prelude::{Commands, EventWriter, ResMut, Trigger};
use bevy::render::gpu_readback::ReadbackComplete;
use std::mem;

/// Reads the texture from the GPU, set as an observer callback from [crate::export::systems::attach_readback]
pub fn read_frame(
    mut trigger: Trigger<ReadbackComplete>,
    mut commands: Commands,
    screenshot: Option<ResMut<Screenshot>>,
    mut progress: EventWriter<ExportProgress>,
) {
    let Some(mut screenshot) = screenshot else {
        return;
    };

    let data = mem::take(&mut trigger.0);
    trigger.propagate(false);
    match screenshot.push_extracted(data) {
        Ok(_) => {
            progress.write(ExportProgress {
                status: ExportStatus::Exporting,
                progress: (screenshot.current_step as f32 / screenshot.total_steps as f32) * 100.0,
                current: screenshot.current_step,
                total: screenshot.total_steps,
            });
        }
        Err(_) => {
            commands.entity(trigger.target()).despawn();
            screenshot.status = ScreenshotStatus::Processing;
        }
    }
}
