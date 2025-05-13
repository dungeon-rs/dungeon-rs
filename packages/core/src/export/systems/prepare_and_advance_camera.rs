use crate::export::callbacks;
use crate::export::ongoing::OngoingExport;
use crate::export::state::ExportState;
use bevy::prelude::{
    BevyError, Camera, Commands, Projection, Query, ResMut, Transform, With, info,
};
use bevy::prelude::{Entity, Result};
use std::ops::DerefMut;

/// Handles the following [ExportState]s:
/// * [ExportState::PrepareTargetAndCamera]: attaches render target, readback and moves camera to the first location.
/// * [ExportState::SkipFirstFrame]: simply skip a frame and advance to the next stage
/// * [ExportState::Capturing]: move the camera to the next location for the readback to capture.
///
/// This system will not handle any other states.
/// This system assumes the presence of an [OngoingExport] resource.
pub fn prepare_and_advance_camera(
    mut ongoing_export: ResMut<OngoingExport>,
    mut camera: Query<(&mut Camera, &mut Transform, &mut Projection, Entity), With<Camera>>,
    mut commands: Commands,
) -> Result {
    let (mut camera, mut transform, mut projection, entity) = camera.single_mut()?;
    if ongoing_export.state == ExportState::PrepareTargetAndCamera {
        let Projection::Orthographic(projection) = projection.deref_mut() else {
            return Err(BevyError::from(
                "Export only works for orthographic projections",
            ));
        };

        let readback = ongoing_export.attach_to_camera(&mut camera, &transform, projection);
        commands.entity(entity).with_children(|parent| {
            parent
                .spawn(readback) // Whenever the readback reads a frame, we call [on_readback_complete].
                .observe(callbacks::on_readback_complete);
        });

        ongoing_export.state = ExportState::SkipFirstFrame;
    } else if ongoing_export.state == ExportState::SkipFirstFrame {
        ongoing_export.state = ExportState::Capturing;

        return Ok(());
    } else if ongoing_export.state != ExportState::Capturing {
        return Ok(());
    }

    let Some(coordinates) = ongoing_export.pop_camera_movement() else {
        ongoing_export.state = ExportState::AwaitReadbacks;

        return Ok(());
    };

    info!("Updating camera to {}x{}", coordinates.x, coordinates.y);
    transform.translation.x = coordinates.x;
    transform.translation.y = coordinates.y;
    Ok(())
}
