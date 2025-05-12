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
    let mut current = export.current_step;
    let mut total = export.total_steps;
    if let Some(receiver) = &mut export.receiver {
        for event in receiver.try_iter() {
            current = event.current;
            total = event.total;

            progress.write(event);
        }
    }
    export.current_step = current;
    export.total_steps = total;

    if let Some(task) = &mut export.processing_task {
        if task.is_finished() {
            if let Some(result) = block_on(poll_once(task)) {
                completed.write(result);

                commands.remove_resource::<OngoingExport>();
            }
        }
    }
}
