mod components;
mod constants;
mod export;

use crate::export::ExportPlugin;
use bevy::app::App;
use bevy::prelude::Plugin;

pub mod prelude {
    pub use crate::{components::*, export::events::*};
}

#[derive(Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ExportPlugin,));
    }
}
