#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::suspicious, clippy::complexity)]

mod async_command;
pub mod components;
mod constants;
mod export;
mod log_plugin;
mod persistence;
mod states;
pub mod utils;

use crate::export::ExportPlugin;
use crate::persistence::PersistencePlugin;
use crate::states::DungeonRsState;
use bevy::app::App;
use bevy::prelude::{AppExtStates, Plugin, PostUpdate};
use core_assets::AssetsPlugin;

pub mod prelude {
    pub use crate::{
        async_command::AsyncCommand, components::*, export::events::*, log_plugin::log_plugin,
        persistence::events::load_project_request::*, persistence::events::save_project_request::*,
        states::*,
    };
    pub use core_assets::{AssetLibraryBuilder, AssetPack};
}

#[derive(Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ExportPlugin, PersistencePlugin, AssetsPlugin))
            .init_state::<DungeonRsState>()
            .add_systems(PostUpdate, async_command::execute_async_commands);

        #[cfg(feature = "dev")]
        app.add_systems(
            bevy::prelude::FixedPreUpdate,
            bevy::dev_tools::states::log_transitions::<DungeonRsState>,
        );
    }
}
