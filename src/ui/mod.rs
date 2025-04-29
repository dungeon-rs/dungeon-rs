use bevy::app::App;
use bevy::prelude::Plugin;

mod new_map;

#[derive(Default)]
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(new_map::NewMapPlugin);
    }
}
