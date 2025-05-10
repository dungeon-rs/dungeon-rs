use crate::export::screenshot::Screenshot;
use crate::export::{ExportCompleted, ExportProgress, ExportStatus};
use bevy::prelude::{
    Camera, Commands, Entity, EventWriter, Query, ResMut, Transform, Trigger, With, info, warn_once,
};
use bevy::render::gpu_readback::{Readback, ReadbackComplete};

/// The system responsible for updating the camera based on [Screenshot]'s state.
/// It also emits the [ExportProgress] and [ExportCompleted] events.
pub fn export_frame(
    screenshot: Option<ResMut<Screenshot>>,
    mut camera: Query<(Entity, &mut Transform), With<Camera>>,
    mut commands: Commands,
    mut progress: EventWriter<ExportProgress>,
    mut completed: EventWriter<ExportCompleted>,
) {
    let Some(mut screenshot) = screenshot else {
        return;
    };

    let Ok((camera, mut transform)) = camera.single_mut() else {
        warn_once!("Failed to acquire camera entity");
        return;
    };

    // if we're in the preparation stage (screenshot was first initiated a frame ago),
    // we attach the readback to capture frames and update the state.
    if matches!(screenshot.state, ExportStatus::Preparing) {
        let target = screenshot.render_target.clone_weak();
        commands.entity(camera).with_children(|spawner| {
            spawner.spawn(Readback::texture(target)).observe(on_readback_complete);
        });

        screenshot.state = ExportStatus::Exporting;
    }

    let Some(coordinates) = screenshot.coordinates.pop_front() else {
        commands.entity(camera).remove::<Readback>();
        completed.write(ExportCompleted {
            path: screenshot.output.clone(),
        });

        return;
    };

    progress.write(ExportProgress {
        total: 0,
        current: 0,
        progress: 0.,
        status: screenshot.state,
    });

    transform.translation.x = coordinates.x;
    transform.translation.y = coordinates.y;
}

/// Callback for the GPU texture is copied and available for reading.
fn on_readback_complete(_event: Trigger<ReadbackComplete>) {
}
