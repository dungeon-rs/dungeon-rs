mod state;

use crate::camera_controls::state::CameraControlsState;
use bevy::app::{App, Plugin};
use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::prelude::{
    Camera, Commands, CursorMoved, EventReader, Local, MouseButton, Node, Projection, Query, Res,
    Startup, Text, Transform, Update, Val, default,
};
use bevy::ui::PositionType;

/// A Plugin that allows the user to move the camera with the middle mouse button.
#[derive(Default)]
pub struct CameraControlsPlugin;

impl Plugin for CameraControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, camera_controls);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Text::new("Hold [MID MOUSE] to move camera"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.),
            right: Val::Px(20.),
            ..default()
        },
    ));
}

fn camera_controls(
    mut state: Local<Option<CameraControlsState>>,
    mut cursor_events: EventReader<CursorMoved>,
    input: Res<ButtonInput<MouseButton>>,
    mut camera: Query<(&mut Camera, &mut Transform, &mut Projection)>,
) {
    let state = state.get_or_insert(CameraControlsState {
        locked: false,
        moving: false,
    });
    if input.just_pressed(MouseButton::Middle) {
        state.moving = true;
    } else if input.just_released(MouseButton::Middle) {
        state.moving = false;
    }

    if state.locked || !state.moving {
        return;
    }

    let Ok((_camera, mut transform, _projection)) = camera.single_mut() else {
        return;
    };

    let delta = cursor_events
        .read()
        .filter_map(|event| event.delta) // take only the events with Some(delta)
        .reduce(|a, b| a + b) // sum them
        .unwrap_or(Vec2::ZERO);

    transform.translation.x -= delta.x;
    transform.translation.y += delta.y;
}
