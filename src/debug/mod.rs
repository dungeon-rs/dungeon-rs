use bevy::app::App;
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::prelude::{
    ButtonInput, Color, FixedUpdate, KeyCode, Plugin, Res, ResMut, TextFont, UiDebugOptions,
    default,
};
use bevy::text::FontSmoothing;

#[derive(Default)]
pub struct DebugPlugin;

/// Adds controls for debug options for dev builds this module will not compile for release builds.
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextFont {
                    font_size: 20.0,
                    font: default(),
                    font_smoothing: FontSmoothing::default(),
                    ..default()
                },
                text_color: Color::srgb(0.0, 1.0, 0.0),
                refresh_interval: core::time::Duration::from_millis(100),
                enabled: true,
            },
        });

        app.add_systems(FixedUpdate, toggle_ui_debug);
    }
}

/// Toggles the ui debug options when pressing F1.
fn toggle_ui_debug(input: Res<ButtonInput<KeyCode>>, mut debug_options: ResMut<UiDebugOptions>) {
    if input.just_pressed(KeyCode::F1) {
        debug_options.toggle();
    }
}
