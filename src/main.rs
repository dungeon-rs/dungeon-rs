use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, add_camera)
        .run();
}

fn add_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
