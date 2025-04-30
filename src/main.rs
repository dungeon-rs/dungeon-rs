use bevy::prelude::*;
use dungeon_rs::{DungeonRsPlugin, ToolbarAction, dialog};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DungeonRsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_file_cmds)
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn(Sprite::from_image(asset_server.load("branding.png")));
}

fn handle_file_cmds(mut reader: EventReader<ToolbarAction>) {
    for event in reader.read() {
        dbg!(event);
    }
}
