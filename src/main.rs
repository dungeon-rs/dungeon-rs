use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::prelude::*;
use bevy::text::FontSmoothing;
use dungeon_rs::DungeonRsPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            DungeonRsPlugin,
            #[cfg(feature = "dev")]
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        font_size: 20.0,
                        font: default(),
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: Color::srgb(0.0, 1.0, 0.0),
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                },
            },
        ))
        .add_systems(Startup, setup)
        // .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn(Sprite::from_image(asset_server.load("branding.png")));
}
