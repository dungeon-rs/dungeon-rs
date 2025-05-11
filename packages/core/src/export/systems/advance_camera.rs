use crate::export::screenshot::{Screenshot, ScreenshotStatus};
use crate::export::{ExportProgress, ExportStatus};
use bevy::prelude::{Camera, EventWriter, ResMut, Single, Transform, With};

/// The system responsible for updating the camera based on [Screenshot]'s state.
pub fn advance_camera(
    mut transform: Single<&mut Transform, With<Camera>>,
    mut screenshot: ResMut<Screenshot>,
    mut progress: EventWriter<ExportProgress>,
) {
    if !matches!(screenshot.status, ScreenshotStatus::Capturing) {
        return;
    }

    // Attempt to pop a coordinate from the queue if we can't advance to extracting.
    let Some(coordinates) = screenshot.pop_pending() else {
        screenshot.status = ScreenshotStatus::Extracting;

        return;
    };

    transform.translation.x = coordinates.x;
    transform.translation.y = coordinates.y;
    progress.write(ExportProgress {
        status: ExportStatus::Exporting,
        progress: (screenshot.current_step as f32 / screenshot.total_steps as f32) * 100.0,
        current: screenshot.current_step,
        total: screenshot.total_steps,
    });
}
