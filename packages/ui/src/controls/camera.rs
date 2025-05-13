use std::ops::DerefMut;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::{ButtonInput, Camera, KeyCode, Local, MouseButton, Projection, Query, Res, Transform, With, Result};
use bevy::prelude::Projection::Orthographic;

/// Tracks the state of the camera controls, used to see if the user is trying to move the camera
/// so we don't have the camera move with the mouse all the time.
pub struct CameraState {
    moving: bool,
}

/// System that allows the user to control the camera.
pub fn camera(
    mut state: Local<Option<CameraState>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera: Query<&mut Transform, With<Camera>>,
    mut projection: Query<&mut Projection, With<Camera>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
) -> Result {
    let state = state.get_or_insert(CameraState { moving: false });

    if mouse_input.just_pressed(MouseButton::Right) {
        state.moving = true;
    } else if mouse_input.just_released(MouseButton::Right) {
        state.moving = false;
    }

    let mut projection = projection.single_mut()?;
    if let Orthographic(projection) = projection.deref_mut() {
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            projection.scale *= 1.1;
        } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            projection.scale /= 1.1;
        } else if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            projection.scale = 1.;
        }
    }

    if !state.moving {
        return Ok(());
    }

    let mut transform = camera.single_mut()?;

    transform.translation.x -= mouse_motion.delta.x;
    transform.translation.y += mouse_motion.delta.y;
    Ok(())
}
