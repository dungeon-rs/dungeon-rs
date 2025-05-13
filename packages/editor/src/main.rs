mod plugin;

use crate::plugin::EditorPlugin;
use bevy::prelude::*;
use dungeonrs_core::export::{ExportCompleted, ExportProgress};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EditorPlugin))
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_image(asset_server.load("logo.png")),
        Transform::from_xyz(0., 0., 0.),
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("logo.png")),
        Transform::from_xyz(512., 512., 0.),
    ));
}

fn update(
    mut progress: EventReader<ExportProgress>,
    mut completed: EventReader<ExportCompleted>,
    mut app_exit: EventWriter<AppExit>,
    mut gizmos: Gizmos,
) {
    gizmos.rect_2d(
        Isometry2d::IDENTITY,
        Vec2::splat(2048.),
        Color::srgb(1., 0., 0.),
    );

    gizmos
        .grid_2d(
            Isometry2d::IDENTITY,
            UVec2::splat(11),
            Vec2::splat(100.),
            Color::WHITE,
        )
        .outer_edges();

    gizmos.axes_2d(Transform::IDENTITY, 512.);

    for progress in progress.read() {
        info!("Exporting: {:?}", progress);
    }

    for completed in completed.read() {
        info!("Export completed: {:#?}", completed);

        app_exit.write(AppExit::Success);
    }
}
