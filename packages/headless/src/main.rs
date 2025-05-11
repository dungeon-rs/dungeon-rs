use bevy::window::ExitCondition;
use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use dungeonrs_core::CorePlugin;
use dungeonrs_core::export::{ExportCompleted, ExportProgress, ExportRequest};
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            }),
            CorePlugin,
        ))
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::ZERO))
        // .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(100)))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut event_writer: EventWriter<ExportRequest>,
) {
    commands.spawn(Camera2d);

    commands.spawn(Sprite::from_image(asset_server.load("logo.png")));

    let Ok(request) = ExportRequest::new(PathBuf::from("output.png"), 128, (512, 512)) else {
        return;
    };

    event_writer.write(request);
}

fn update(
    mut progress: EventReader<ExportProgress>,
    mut completed: EventReader<ExportCompleted>,
    mut app_exit: EventWriter<AppExit>,
) {
    for progress in progress.read() {
        info!("Exporting: {:?}", progress);
    }

    for completed in completed.read() {
        info!("Export completed: {:#?}", completed);

        app_exit.write(AppExit::Success);
    }
}
