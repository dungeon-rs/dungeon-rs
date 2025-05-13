use crate::export::ongoing::OngoingExport;
use crate::export::state::ExportState;
use crate::export::{ExportCompleted, ExportProgress};
use bevy::prelude::{EventWriter, ResMut, warn};

/// Waits for [crate::export::tasks::process_image_data] to finish processing the received data.
///
/// It monitors for updates and completion of the processing and moves that information back into ECS.
pub fn wait_for_image_processing(
    mut ongoing_export: ResMut<OngoingExport>,
    mut progress: EventWriter<ExportProgress>,
    mut finished: EventWriter<ExportCompleted>,
) {
    if let Some(events) = ongoing_export.poll_processing_progress() {
        progress.write_batch(events);
    }

    if let Some(completed) = ongoing_export.poll_processing_completed() {
        if let Ok(completed) = completed {
            finished.write(completed);
        } else {
            warn!("An error caused failure in image processing");
        }

        ongoing_export.state = ExportState::Cleanup;
    }
}
