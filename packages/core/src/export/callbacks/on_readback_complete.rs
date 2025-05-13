use crate::export::ongoing::OngoingExport;
use crate::export::state::ExportState;
use bevy::prelude::{Commands, ResMut, Trigger};
use bevy::render::gpu_readback::ReadbackComplete;
use std::mem;

/// Handles the callback when we receive a frame from the GPU.
/// We update the [OngoingExport] with the received image data.
pub fn on_readback_complete(
    mut readback: Trigger<ReadbackComplete>,
    mut commands: Commands,
    mut ongoing_export: ResMut<OngoingExport>,
) {
    // ReadbackComplete is only observed by this method, so we can take the data.
    let data = mem::take(&mut readback.0);

    if ongoing_export.push_image_data(data).is_err() {
        // We've received all frames we need, we can detach the readback to stop receiving frames.
        commands.entity(readback.target()).try_despawn();

        // Indicate we're going to process frames and launch a task to handle the processing.
        ongoing_export.state = ExportState::ProcessFrames;
        ongoing_export.process_async();
    }
}
