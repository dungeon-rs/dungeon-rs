#![doc = include_str!("../README.md")]
#![warn(
    clippy::pedantic,
    clippy::suspicious,
    clippy::complexity
)]

mod plugin;

use crate::plugin::EditorPlugin;
use bevy::prelude::*;
use core::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EditorPlugin))
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

#[allow(clippy::needless_pass_by_value)]
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Name::new("Project"),
        Project::new(Rect::from_center_size(Vec2::ZERO, Vec2::splat(1000.))),
        children![(
            Name::new("Root"),
            Level,
            children![
                (
                    Name::new("Default Layer"),
                    Layer,
                    children![
                        (
                            Name::new("Logo"),
                            Texture {
                                size: Rectangle::from_size(Vec2::splat(256.))
                            },
                            Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(256.)))),
                            generate_image(&mut materials, &asset_server),
                            Transform::from_xyz(0., 0., 0.),
                        ),
                        (
                            Name::new("Logo 2"),
                            Texture {
                                size: Rectangle::from_size(Vec2::splat(256.))
                            },
                            Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(256.)))),
                            generate_image(&mut materials, &asset_server),
                            Transform::from_xyz(250., 0., 0.),
                        ),
                    ]
                ),
                (
                    Name::new("Background Layer"),
                    Layer,
                    Transform::from_xyz(0., 0., -1.),
                    children![
                        (
                            Name::new("Logo 3"),
                            Texture {
                                size: Rectangle::from_size(Vec2::splat(256.))
                            },
                            Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(256.)))),
                            generate_image(&mut materials, &asset_server),
                            Transform::from_xyz(0., 250., 0.),
                        ),
                        (
                            Texture {
                                size: Rectangle::from_size(Vec2::splat(256.))
                            },
                            Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(256.)))),
                            generate_image(&mut materials, &asset_server),
                            Transform::from_xyz(250., 250., 0.),
                        ),
                    ]
                ),
            ]
        )],
    ));
}

fn update(
    mut progress: EventReader<ExportProgress>,
    mut completed: EventReader<ExportCompleted>,
    mut gizmos: Gizmos,
) {
    gizmos.rect_2d(Vec2::ZERO, Vec2::splat(256.), Color::srgb(1.0, 0., 0.));

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

fn generate_image(
    materials: &mut ResMut<Assets<ColorMaterial>>,
    asset_server: &Res<AssetServer>,
) -> MeshMaterial2d<ColorMaterial> {
    MeshMaterial2d(materials.add(ColorMaterial {
        texture: Some(asset_server.load("logo.png")),
        ..default()
    }))
}
