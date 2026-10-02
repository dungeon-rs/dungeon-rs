#![doc = include_str!("../README.md")]

mod bindings;
mod browser;
#[cfg(feature = "dev")]
mod crash_test;
mod diagnostics;
mod export;
mod files;
mod outcomes;
mod panels;
#[cfg(feature = "dev")]
mod screenshot;
#[cfg(feature = "dev")]
mod script;
mod state;
mod viewport;
mod window;

use bevy::app::{App, Plugin, Startup, Update};
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::window::{Window, WindowPlugin};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use drs_diagnostics::Started;

/// The window the Editor runs in, for the Host to `set` on Bevy's default plugins: it is not
/// closed on request, because the Editor answers the window's close request itself, asking
/// about unsaved changes before it quits, and it carries the editor's name until a Project
/// names it.
#[must_use]
pub fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: window::EDITOR.to_owned(),
            ..Window::default()
        }),
        close_when_requested: false,
        ..WindowPlugin::default()
    }
}

/// The egui interface: the panels, the menu, the dialogs, and the viewport's interaction, in
/// the window [`window_plugin`] describes.
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
            .init_resource::<browser::Browser>()
            .init_resource::<panels::Layout>()
            .add_systems(EguiPrimaryContextPass, panels::draw)
            .add_systems(Startup, diagnostics::report_bundled_files)
            .add_systems(
                Update,
                (
                    outcomes::report,
                    diagnostics::announce_crash,
                    window::close_requested,
                    window::title,
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
