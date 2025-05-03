use bevy::prelude::*;
use dungeonrs_ui::UIPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, UIPlugin))
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .run();
}
