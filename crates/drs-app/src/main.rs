#![doc = include_str!("../README.md")]

use bevy::DefaultPlugins;
use bevy::app::{App, AppExit, PluginGroup};
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::log::LogPlugin;
use drs_authoring_manager::AuthoringManagerPlugin;
use drs_diagnostics::{CrashHandler, LogDirectives, Product};
use drs_editor::{EditorPlugin, window_plugin};
use drs_history::HistoryPlugin;
use drs_library_access::{LibraryAccessPlugin, register_library_source, register_thumbnail_source};
use drs_library_manager::LibraryManagerPlugin;
use drs_model::{CaughtPanics, EditorDirectories, ModelPlugin};
use drs_paint_engine::PaintEnginePlugin;
use drs_project_manager::ProjectManagerPlugin;
use drs_render_engine::RenderEnginePlugin;

/// The editor, as the crash report, the dialog, and the bundle marker name it.
const PRODUCT: Product = Product {
    name: "DungeonRS",
    version: env!("CARGO_PKG_VERSION"),
};

/// Builds the editor out of every crate's plugin and runs it.
///
/// The crash handler goes first, so that a crash before any window still leaves a report;
/// logging next, with its layer handed to Bevy's log plugin, which is added on its own so that
/// the first entry names the log file before any other plugin logs, and with the Utility's
/// default filter, so the terminal and the file start from the same directives before
/// `RUST_LOG` is laid over both; then the Bundled Files, which the default asset source is
/// rooted at. The `lib://` and `thumb://` asset sources are registered before Bevy's
/// `AssetPlugin` builds, since sources freeze then, and `.meta` lookups are off because Asset
/// Folders never hold them. The window is the one the Editor describes, so what the Editor needs
/// of it holds by construction. Background threads that catch their own panics mark themselves
/// with the crash handler's marker, so such a panic is logged rather than reported as a crash.
fn main() -> AppExit {
    let directories = directories();
    let logs = drs_diagnostics::log_directory(directories.resolve().ok().map(|found| found.logs));
    drs_diagnostics::install_crash_handler(CrashHandler {
        log_directory: logs.clone(),
        product: PRODUCT,
        dialogs: drs_diagnostics::dialogs_possible(),
    });
    let logging = drs_diagnostics::start_logging(&logs, LogDirectives::from_environment());
    let bundled_files = drs_diagnostics::locate_bundled_files_of_this_executable(PRODUCT);

    let mut app = App::new();
    register_library_source(&mut app);
    register_thumbnail_source(&mut app);
    app.add_plugins(LogPlugin {
        custom_layer: |_| drs_diagnostics::take_layer(),
        filter: drs_diagnostics::DEFAULT_FILTER.to_owned(),
        ..LogPlugin::default()
    });
    let started = drs_diagnostics::log_start(&logging, bundled_files);
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<LogPlugin>()
            .set(AssetPlugin {
                file_path: started.asset_root().to_string_lossy().into_owned(),
                meta_check: AssetMetaCheck::Never,
                ..AssetPlugin::default()
            })
            .set(window_plugin()),
    );
    app.add_plugins((
        ModelPlugin,
        HistoryPlugin,
        LibraryAccessPlugin,
        LibraryManagerPlugin,
        ProjectManagerPlugin,
        AuthoringManagerPlugin,
        PaintEnginePlugin,
        RenderEnginePlugin,
        EditorPlugin::new(started),
    ));
    app.insert_resource(directories)
        .insert_resource(CaughtPanics(drs_diagnostics::mark_panics_caught));
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
