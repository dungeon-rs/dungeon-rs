#![doc = include_str!("../README.md")]

mod bindings;
mod browser;
#[cfg(feature = "dev")]
mod crash_test;
mod diagnostics;
mod outcomes;
mod panels;
#[cfg(feature = "dev")]
mod screenshot;
#[cfg(feature = "dev")]
mod script;
mod state;
mod viewport;

use bevy::app::{App, Plugin, Startup, Update};
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
            .add_systems(Startup, diagnostics::report_resources)
            .add_systems(
                Update,
                (
                    outcomes::report,
                    diagnostics::announce_crash,
                    viewport::pointer,
                    viewport::keys,
                    viewport::outline_selection,
                )
                    .chain(),
            );
        #[cfg(feature = "dev")]
        {
            crash_test::at_startup();
            app.add_systems(
                Update,
                (screenshot::screenshot, crash_test::on_second_frame),
            );
            if let Some(script) = script::Script::from_environment() {
                app.insert_resource(script).add_systems(
                    bevy::app::PreUpdate,
                    script::drive.before(bevy::input::InputSystems),
                );
            }
        }
    }
}
