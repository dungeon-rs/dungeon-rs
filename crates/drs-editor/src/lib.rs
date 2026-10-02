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
use drs_diagnostics::Started;

/// The egui interface: the panels, the menu, and the viewport's interaction.
pub struct EditorPlugin {
    /// What the diagnostics Utility set up and found at start.
    started: Started,
}

impl EditorPlugin {
    /// The Editor over what the diagnostics Utility set up and found at start, which it keeps
    /// for Show Logs and the status line.
    #[must_use]
    pub fn new(started: Started) -> Self {
        Self { started }
    }
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .insert_resource(diagnostics::Diagnostics(self.started.clone()))
            .init_resource::<state::EditorState>()
            .init_resource::<panels::Layout>()
            .add_systems(EguiPrimaryContextPass, panels::draw)
            .add_systems(Startup, diagnostics::report_bundled_files)
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
            if let Some(path) = screenshot::ScreenshotPath::from_environment() {
                app.insert_resource(path)
                    .add_systems(Update, screenshot::screenshot);
            }
            if let Some(forced) = crash_test::ForcedCrash::from_environment() {
                app.insert_resource(forced)
                    .add_systems(Update, crash_test::on_second_frame);
            }
            if let Some(script) = script::Script::from_environment() {
                app.insert_resource(script).add_systems(
                    bevy::app::PreUpdate,
                    script::drive.before(bevy::input::InputSystems),
                );
            }
        }
    }
}
