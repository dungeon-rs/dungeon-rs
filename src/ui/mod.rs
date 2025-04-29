use bevy::app::App;
use bevy::input_focus::InputFocus;
use bevy::prelude::Plugin;

mod button;
mod toolbar;

pub use toolbar::ToolbarAction;

#[derive(Default)]
pub struct UI;

impl Plugin for UI {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFocus>();

        app.add_plugins(toolbar::Toolbar);
    }
}
