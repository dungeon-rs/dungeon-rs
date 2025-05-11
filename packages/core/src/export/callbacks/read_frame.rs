use crate::export::ongoing_export::{ExportState, OngoingExport};
use crate::export::{callbacks, ExportProgress, ExportStatus};
use bevy::prelude::{Commands, EventWriter, ResMut, Trigger};
use bevy::render::gpu_readback::ReadbackComplete;
use bevy::tasks::AsyncComputeTaskPool;
use std::mem;

/// Reads the texture from the GPU, set as an observer callback from [crate::export::systems::attach_readback]
pub fn read_frame(
    mut trigger: Trigger<ReadbackComplete>,
    mut commands: Commands,
    export: Option<ResMut<OngoingExport>>,
    mut progress: EventWriter<ExportProgress>,
) {
    let Some(mut export) = export else {
        return;
    };

    let data = mem::take(&mut trigger.0);
    trigger.propagate(false);
    match export.push_extracted(data) {
        Ok(_) => {
            progress.write(ExportProgress {
                status: ExportStatus::Exporting,
                progress: (export.current_step as f32 / export.total_steps as f32) * 100.0,
                current: export.current_step,
                total: export.total_steps,
            });
        }
        Err(_) => {
            commands.entity(trigger.target()).despawn();
            export.state = ExportState::Processing;

            let (buffer, total, current, sender) = export.consume();
            let task = AsyncComputeTaskPool::get()
                .spawn(async move { callbacks::process_frames(buffer, total, current, sender) });

            export.set_processing_task(task);
        }
    }
}
