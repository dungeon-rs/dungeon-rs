use crate::export::ongoing_export::{ExportState, OngoingExport};
use crate::export::{callbacks, ExportProgress, ExportStatus};
use bevy::prelude::{Camera, Commands, EventWriter, ResMut, Single, Transform, Trigger, Vec3, With};
use bevy::render::camera::RenderTarget;
use bevy::render::gpu_readback::ReadbackComplete;
use bevy::tasks::AsyncComputeTaskPool;
use bevy::window::WindowRef;
use std::mem;
use std::ops::DerefMut;

/// Reads the texture from the GPU, set as an observer callback from [crate::export::systems::attach_readback]
pub fn read_frame(
    mut trigger: Trigger<ReadbackComplete>,
    mut commands: Commands,
    mut camera: Single<(&mut Camera, &mut Transform), With<Camera>>,
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
            let (camera, transform) = camera.deref_mut();
            // Reset camera position and target
            transform.translation = Vec3::ZERO;
            camera.target = RenderTarget::Window(WindowRef::Primary);
            // despawn the readback to prevent additional frames from being read
            commands.entity(trigger.target()).despawn();
            // advance export state
            export.state = ExportState::Processing;

            // Generate an asynchronous task to handle the processing of the raw image data.
            let data = export.consume();
            let task =
                AsyncComputeTaskPool::get().spawn(async move { callbacks::process_frames(data) });

            export.set_processing_task(task);
        }
    }
}
