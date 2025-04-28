use bevy::app::App;
use bevy::prelude::{ButtonInput, FixedUpdate, KeyCode, Plugin, Res, ResMut, UiDebugOptions};

#[derive(Default)]
pub struct DebugPlugin;

/// Adds controls for debug options for dev builds this module will not compile for release builds.
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, toggle_ui_debug);
    }
}

/// Toggles the ui debug options when pressing F1.
fn toggle_ui_debug(input: Res<ButtonInput<KeyCode>>, mut debug_options: ResMut<UiDebugOptions>) {
    if input.just_pressed(KeyCode::F1) {
        debug_options.toggle();
    }
}
