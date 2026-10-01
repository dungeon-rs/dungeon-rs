#![doc = include_str!("../README.md")]

mod browser;
mod outcomes;
mod panels;
mod state;
mod viewport;

use bevy::app::{App, Plugin, Update};
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

/// The egui interface: the panels, the menu, and the viewport's interaction.
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<state::EditorState>()
            .init_resource::<panels::Layout>()
            .add_systems(EguiPrimaryContextPass, panels::draw)
            .add_systems(
                Update,
                (
                    outcomes::report,
                    viewport::pointer,
                    viewport::keys,
                    viewport::outline_selection,
                )
                    .chain(),
            );
    }
}
