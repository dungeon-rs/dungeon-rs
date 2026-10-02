#![doc = include_str!("../README.md")]

use bevy::DefaultPlugins;
use bevy::app::{App, AppExit, PluginGroup};
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::log::LogPlugin;
use bevy::window::{Window, WindowPlugin};
use drs_authoring_manager::AuthoringManagerPlugin;
use drs_diagnostics::{CrashHandler, LevelFilter};
use drs_editor::EditorPlugin;
use drs_history::HistoryPlugin;
use drs_library_access::{LibraryAccessPlugin, register_library_source};
use drs_library_manager::LibraryManagerPlugin;
use drs_model::{Diagnostics, EditorDirectories, ModelPlugin, ResourceDirectory};
use drs_project_manager::ProjectManagerPlugin;
use drs_render_engine::RenderEnginePlugin;

/// The editor's version, as the crash report and the resources marker name it.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Builds the editor out of every crate's plugin and runs it.
///
/// The crash handler goes first, so that the hook Bevy's plugins build chains it and a crash
/// before any window still leaves a report; logging next, with its layer handed to Bevy's log
/// plugin, which is added on its own so that the first entry names the log file before any
/// other plugin logs; then the bundled resources, which the default asset source is rooted at. The
/// `lib://` asset source is registered before Bevy's `AssetPlugin` builds, since sources freeze
/// then, and `.meta` lookups are off because Asset Folders never hold them.
fn main() -> AppExit {
    let directories = directories();
    let logs = drs_diagnostics::log_directory(directories.logs.as_deref());
    drs_diagnostics::install_crash_handler(CrashHandler {
        log_directory: logs.clone(),
        version: VERSION.to_owned(),
        dialogs: true,
    });
    let logging = drs_diagnostics::start_logging(&logs, LevelFilter::from_environment());
    let executable = std::env::current_exe().unwrap_or_default();
    let resources = drs_diagnostics::locate_resources(&executable, VERSION);

    let mut asset_plugin = AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..AssetPlugin::default()
    };
    if let Ok(found) = &resources {
        asset_plugin.file_path = found.root().to_string_lossy().into_owned();
    }
    let mut app = App::new();
    register_library_source(&mut app);
    app.add_plugins(LogPlugin {
        custom_layer: |_| drs_diagnostics::take_layer(),
        ..LogPlugin::default()
    });
    drs_diagnostics::announce_start(&logging, &resources);
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<LogPlugin>()
            .set(asset_plugin)
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
    app.insert_resource(directories);
    app.insert_resource(Diagnostics {
        logs: logging.directory,
        log_file: logging.file,
        resources: match resources {
            Ok(found) => ResourceDirectory::Found(found.root().to_path_buf()),
            Err(missing) => ResourceDirectory::Missing {
                tried: missing.tried,
            },
        },
    });
    drs_diagnostics::run_guarded(|| app.run())
}

/// Where the editor keeps its own files: the platform's directories, or in development builds
/// every directory under the root `DRS_DIRECTORIES` names when it is set.
fn directories() -> EditorDirectories {
    #[cfg(feature = "dev")]
    if let Some(root) = std::env::var_os("DRS_DIRECTORIES") {
        return EditorDirectories::under(std::path::Path::new(&root));
    }
    EditorDirectories::default()
}
