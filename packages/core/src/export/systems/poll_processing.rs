use crate::export::ongoing_export::OngoingExport;
use crate::export::{ExportCompleted, ExportProgress};
use bevy::prelude::{Commands, EventWriter, ResMut};
use bevy::tasks::{block_on, poll_once};

/// Polls the progression of the asynchronous image processing.
pub fn poll_processing(
    mut commands: Commands,
    mut export: ResMut<OngoingExport>,
    mut progress: EventWriter<ExportProgress>,
    mut completed: EventWriter<ExportCompleted>,
) {
    if let Some(receiver) = &export.receiver {
        for event in receiver.try_iter() {
            progress.write(event);
        }
    }

    if let Some(task) = &mut export.processing_task {
        if task.is_finished() {
            if let Some(result) = block_on(poll_once(task)) {
                completed.write(result);

                commands.remove_resource::<OngoingExport>();
            }
        }
    }
}
