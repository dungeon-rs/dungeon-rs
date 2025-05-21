use crate::export::ongoing::OngoingExport;
use crate::export::state::ExportState;
use crate::export::{ExportCompleted, ExportFailed, ExportProgress};
use bevy::prelude::{EventWriter, ResMut};

/// Waits for [`crate::export::tasks::process_image_data`] to finish processing the received data.
///
/// It monitors for updates and completion of the processing and moves that information back into ECS.
pub fn wait_for_image_processing(
    mut ongoing_export: ResMut<OngoingExport>,
    mut progress: EventWriter<ExportProgress>,
    mut finished: EventWriter<ExportCompleted>,
    mut failed: EventWriter<ExportFailed>,
) {
    if let Some(events) = ongoing_export.poll_processing_progress() {
        progress.write_batch(events);
    }

    if let Some(completed) = ongoing_export.poll_processing_completed() {
        if let Ok(completed) = completed {
            finished.write(completed);
        } else {
            failed.write(ExportFailed {
                error: completed.unwrap_err(),
            });
        }

        ongoing_export.state = ExportState::Cleanup;
    }
}
