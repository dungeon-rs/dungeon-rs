use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::Projection::Orthographic;
use bevy::prelude::{
    ButtonInput, Camera, KeyCode, Local, MouseButton, Projection, Query, Res, Result, Transform,
    With, default,
};
use std::ops::DerefMut;

/// Tracks the state of the camera controls, used to see if the user is trying to move the camera
/// so we don't have the camera move with the mouse all the time.
#[derive(Default)]
pub struct CameraState {
    /// Whether the camera is being dragged around
    moving: bool,
    /// Whether the camera is being zoomed.
    scrolling: bool,
}

/// System that allows the user to control the camera.
pub fn camera(
    mut state: Local<Option<CameraState>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera: Query<&mut Transform, With<Camera>>,
    mut projection: Query<&mut Projection, With<Camera>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
) -> Result {
    let state = state.get_or_insert(default());

    if mouse_input.just_pressed(MouseButton::Right) {
        state.moving = true;
    } else if mouse_input.just_released(MouseButton::Right) {
        state.moving = false;
    }

    if keyboard_input.just_pressed(KeyCode::ControlLeft) {
        state.scrolling = true;
    } else if keyboard_input.just_released(KeyCode::ControlLeft) {
        state.scrolling = false;
    }

    if state.scrolling {
        let mut projection = projection.single_mut()?;
        if let Orthographic(projection) = projection.deref_mut() {
            projection.scale /= 1.0 + mouse_scroll.delta.y * 0.01;
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
