mod plugin;

use crate::plugin::EditorPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EditorPlugin))
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn(Sprite::from_image(asset_server.load("logo.png")));
}
