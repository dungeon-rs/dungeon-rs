mod constants;
pub mod export;

use crate::export::ExportPlugin;
use bevy::app::App;
use bevy::prelude::Plugin;

#[derive(Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ExportPlugin,));
    }
}
