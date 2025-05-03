use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .run();
}
