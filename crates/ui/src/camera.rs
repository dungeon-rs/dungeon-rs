//! Contains data structures and configuration related to the UI camera(s).

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::Projection::Orthographic;
use bevy::prelude::{
    Bundle, ButtonInput, Camera2d, Commands, Component, Local, MouseButton, Name, Projection, Res,
    Single, Transform, With,
};
use bevy_egui::PrimaryEguiContext;

/// Marker component for the UI's camera bundle.
#[derive(Default, Component)]
pub struct UICamera;

/// Maintains the UI camera state (moving, zooming, ...).
#[derive(Default)]
pub(crate) struct UICameraState {
    /// Whether the camera is following mouse movement.
    pub moving: bool,
}

impl UICamera {
    /// Generates a `Bundle` with a [`UICamera`].
    #[must_use]
    pub fn bundle() -> impl Bundle {
        (Name::new("UI Camera"), Self, PrimaryEguiContext, Camera2d)
    }
}

/// Spawns the [`UICamera`] bundle.
#[utils::bevy_system]
pub(crate) fn setup_ui_camera(mut commands: Commands) {
    commands.spawn(UICamera::bundle());
}

/// The system that controls the UI camera.
///
/// It handles moving, zooming and other functionality in response to user input.
#[utils::bevy_system]
pub(crate) fn camera_control_system(
    mut state: Local<UICameraState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut camera: Single<(&mut Transform, &mut Projection), With<UICamera>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
) {
    if mouse_input.just_pressed(MouseButton::Right) {
        state.moving = true;
    } else if mouse_input.just_released(MouseButton::Right) {
        state.moving = false;
    }

    let (transform, projection) = &mut *camera;

    if state.moving {
        transform.translation.x -= mouse_motion.delta.x;
        transform.translation.y += mouse_motion.delta.y;
    }

    if let Orthographic(projection) = &mut **projection {
        projection.scale /= 1.0 + mouse_scroll.delta.y * 0.01;
    }
}
