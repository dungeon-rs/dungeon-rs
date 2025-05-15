mod components;
mod constants;
mod export;
mod persistence;
mod utils;

use crate::export::ExportPlugin;
use crate::persistence::PersistencePlugin;
use bevy::app::App;
use bevy::prelude::Plugin;

pub mod prelude {
    pub use crate::{
        components::*, export::events::*, persistence::events::load_project_request::*,
        persistence::events::save_project_request::*,
    };
}

#[derive(Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ExportPlugin, PersistencePlugin));
    }
}
