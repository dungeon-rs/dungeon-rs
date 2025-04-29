use bevy::app::{App, Plugin};
use bevy::input_focus::InputFocus;
use bevy::prelude::{AssetServer, Font, Handle};
use widgets::toolbar;

mod colors;
mod widgets;

use crate::theme::widgets::dialog;
pub use widgets::toolbar::ToolbarAction;

#[derive(Default)]
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFocus>();

        app.add_plugins((toolbar::Toolbar, dialog::DialogPlugin));
    }
}

/// Get the font for the user interface of DungeonRS.
pub fn font(assets: &AssetServer) -> Handle<Font> {
    assets.load("fonts/opensans.ttf")
}
