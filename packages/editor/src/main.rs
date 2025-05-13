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
}

fn update(
    mut progress: EventReader<ExportProgress>,
    mut completed: EventReader<ExportCompleted>,
    mut gizmos: Gizmos,
) {
    gizmos
        .grid_2d(
            Isometry2d::IDENTITY,
            UVec2::splat(50),
            Vec2::splat(100.),
            Color::WHITE.with_alpha(0.2),
        )
        .outer_edges();

    for progress in progress.read() {
        info!("Exporting: {:?}", progress);
    }

    for completed in completed.read() {
        info!("Export completed: {:#?}", completed);
    }
}
