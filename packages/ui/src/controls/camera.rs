use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::{ButtonInput, Camera, Local, MouseButton, Query, Res, Transform, With};

/// Tracks the state of the camera controls, used to see if the user is trying to move the camera
/// so we don't have the camera move with the mouse all the time.
pub struct CameraState {
    moving: bool,
}

/// System that allows the user to control the camera.
pub fn camera(
    mut state: Local<Option<CameraState>>,
    input: Res<ButtonInput<MouseButton>>,
    mut camera: Query<&mut Transform, With<Camera>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
) {
    let state = state.get_or_insert(CameraState { moving: false });

    if input.just_pressed(MouseButton::Middle) {
        state.moving = true;
    } else if input.just_released(MouseButton::Middle) {
        state.moving = false;
    }

    if !state.moving {
        return;
    }

    let Ok(mut transform) = camera.single_mut() else {
        return;
    };

    transform.translation.x -= mouse_motion.delta.x;
    transform.translation.y += mouse_motion.delta.y;
}
