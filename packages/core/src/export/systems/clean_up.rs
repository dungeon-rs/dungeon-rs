use crate::export::ongoing::OngoingExport;
use bevy::prelude::Projection::Orthographic;
use bevy::prelude::{Camera, Commands, Projection, Query, ResMut, Result, Transform, With};
use bevy::render::camera::{RenderTarget, ScalingMode};
use bevy::window::WindowRef;
use std::ops::DerefMut;

/// Handles cleaning up the [OngoingExport].
/// It resets the camera and projection as well as removing the resource.
pub fn clean_up(
    mut commands: Commands,
    mut camera: Query<(&mut Camera, &mut Transform, &mut Projection), With<Camera>>,
    ongoing_export: ResMut<OngoingExport>,
) -> Result {
    commands.remove_resource::<OngoingExport>();

    let (mut camera, mut transform, mut projection) = camera.single_mut()?;
    camera.target = RenderTarget::Window(WindowRef::Primary);

    if let Some(original) = ongoing_export.camera_location {
        transform.translation = original.translation;
    }

    if let Orthographic(projection) = projection.deref_mut() {
        projection.scaling_mode = ScalingMode::WindowSize;
    }

    Ok(())
}
