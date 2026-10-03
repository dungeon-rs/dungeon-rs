//! The headless editor with offscreen rendering the seam tests that draw share: Bevy's default
//! plugins without a window or winit, the `lib://` asset source, and every plugin of the editor
//! but the Editor's own panels, the paint Engine's rasterizing on the GPU and the render Engine
//! included.

use bevy::app::{App, PluginGroup, PluginsState};
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::render::RenderPlugin;
use bevy::window::{ExitCondition, WindowPlugin};
use bevy::winit::WinitPlugin;
use drs_authoring_manager::AuthoringManagerPlugin;
use drs_history::HistoryPlugin;
use drs_library_access::{LibraryAccessPlugin, register_library_source};
use drs_library_manager::LibraryManagerPlugin;
use drs_model::{EditorDirectories, ModelPlugin};
use drs_paint_engine::PaintEnginePlugin;
use drs_project_manager::ProjectManagerPlugin;
use drs_render_engine::RenderEnginePlugin;
use std::path::Path;
use std::time::{Duration, Instant};

/// How long the renderer may take to initialise before the test gives up.
const RENDERER_START: Duration = Duration::from_secs(60);

/// A headless editor with offscreen rendering whose configuration and cache directories live
/// under `root`, started once: Bevy's default plugins without a window or winit, the `lib://`
/// asset source, and every plugin of the editor but the Editor's own panels, the paint Engine's
/// rasterizing on the GPU included.
///
/// The renderer initialises asynchronously and the render thread is set up when the plugins
/// are finished and cleaned up, which `App::run` would do; a test driving `update` by hand
/// does it here.
pub fn editor(root: &Path) -> App {
    let mut app = App::new();
    app.insert_resource(EditorDirectories::under(root));
    register_library_source(&mut app);
    app.add_plugins(
        bevy::DefaultPlugins
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..AssetPlugin::default()
            })
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                close_when_requested: false,
                ..WindowPlugin::default()
            })
            .set(RenderPlugin {
                synchronous_pipeline_compilation: true,
                ..RenderPlugin::default()
            })
            .disable::<WinitPlugin>(),
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
    ));
    let deadline = Instant::now() + RENDERER_START;
    while app.plugins_state() == PluginsState::Adding {
        assert!(
            Instant::now() < deadline,
            "the renderer did not initialise within {RENDERER_START:?}; is there a GPU adapter?"
        );
        bevy::tasks::tick_global_task_pools_on_main_thread();
    }
    app.finish();
    app.cleanup();
    app.update();
    app
}
