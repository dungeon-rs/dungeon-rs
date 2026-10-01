#![doc = include_str!("../README.md")]

use bevy::DefaultPlugins;
use bevy::app::{App, AppExit, PluginGroup};
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::window::{Window, WindowPlugin};
use drs_authoring_manager::AuthoringManagerPlugin;
use drs_editor::EditorPlugin;
use drs_history::HistoryPlugin;
use drs_library_access::{LibraryAccessPlugin, register_library_source};
use drs_library_manager::LibraryManagerPlugin;
use drs_model::ModelPlugin;
use drs_project_manager::ProjectManagerPlugin;
use drs_render_engine::RenderEnginePlugin;

/// Builds the editor out of every crate's plugin and runs it.
///
/// The `lib://` asset source is registered before Bevy's `AssetPlugin` builds, since sources
/// freeze then, and `.meta` lookups are off because Asset Folders never hold them.
fn main() -> AppExit {
    let mut app = App::new();
    register_library_source(&mut app);
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..AssetPlugin::default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "DungeonRS".to_owned(),
                    ..Window::default()
                }),
                ..WindowPlugin::default()
            }),
    );
    app.add_plugins((
        ModelPlugin,
        HistoryPlugin,
        LibraryAccessPlugin,
        LibraryManagerPlugin,
        ProjectManagerPlugin,
        AuthoringManagerPlugin,
        RenderEnginePlugin,
        EditorPlugin,
    ));
    app.run()
}
