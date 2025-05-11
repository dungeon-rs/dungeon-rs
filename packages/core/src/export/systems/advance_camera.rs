use crate::export::ongoing_export::{OngoingExport, ExportState};
use crate::export::{ExportProgress, ExportStatus};
use bevy::prelude::{Camera, EventWriter, ResMut, Single, Transform, With};

/// The system responsible for updating the camera based on [OngoingExport]'s state.
pub fn advance_camera(
    mut transform: Single<&mut Transform, With<Camera>>,
    mut export: ResMut<OngoingExport>,
    mut progress: EventWriter<ExportProgress>,
) {
    if !matches!(export.state, ExportState::Capturing) {
        return;
    }

    // Attempt to pop a coordinate from the queue if we can't advance to extracting.
    let Some(coordinates) = export.pop_pending() else {
        export.state = ExportState::Extracting;

        return;
    };

    transform.translation.x = coordinates.x;
    transform.translation.y = coordinates.y;
    progress.write(ExportProgress {
        status: ExportStatus::Exporting,
        progress: (export.current_step as f32 / export.total_steps as f32) * 100.0,
        current: export.current_step,
        total: export.total_steps,
    });
}
