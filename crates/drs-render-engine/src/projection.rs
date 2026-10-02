//! The projection: a 2D camera that shows the part of the Level the [`Viewport`] names.

use bevy_camera::{Camera, Camera2d, ClearColorConfig, OrthographicProjection, Projection};
use bevy_color::Color;
use bevy_ecs::component::Component;
use bevy_ecs::query::{Changed, With};
use bevy_ecs::system::{Commands, Query, Res, Single};
use bevy_math::Vec3;
use bevy_transform::components::Transform;
use drs_model::Viewport;

/// The camera that draws the Level into the window.
#[derive(Component)]
pub(crate) struct LevelCamera;

/// What shows where nothing is drawn: a mid grey, so the dark grey a Wall is drawn in by default
/// and the editor's dark panels both stand apart from it.
const BACKDROP: Color = Color::srgb(0.52, 0.52, 0.55);

/// Depth beyond which Elements are clipped, either way; stacking assigns one unit per Element.
pub(crate) const DEPTH: f32 = 1_000_000.0;

/// Spawns the camera the Level is drawn with.
///
/// Its projection is orthographic with one cell per world unit before zoom, and deep enough
/// that every Element's own depth stays in view.
pub(crate) fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(BACKDROP),
            ..Camera::default()
        },
        Projection::Orthographic(OrthographicProjection {
            near: -DEPTH,
            far: DEPTH,
            ..OrthographicProjection::default_2d()
        }),
        LevelCamera,
    ));
}

/// Whether the window the camera draws into changed size, which moves the Viewport's area
/// relative to the window's centre.
pub(crate) fn target_resized(cameras: Query<(), (With<LevelCamera>, Changed<Camera>)>) -> bool {
    !cameras.is_empty()
}

/// Moves and scales the camera so that the Viewport's centre sits at the centre of its area,
/// one cell spanning `zoom` logical pixels.
///
/// The camera covers the whole window, so the cell at the window's centre is the one the
/// Viewport puts there; the Viewport's own conversion gives it.
pub(crate) fn follow_viewport(
    viewport: Res<Viewport>,
    camera: Single<(&Camera, &mut Projection, &mut Transform), With<LevelCamera>>,
) {
    let (camera, mut projection, mut transform) = camera.into_inner();
    let window_centre = camera
        .logical_target_size()
        .map_or(viewport.area.center(), |size| size / 2.0);
    let centre = viewport.cells_at(window_centre);
    let translation = Vec3::new(centre.x, centre.y, 0.0);
    if transform.translation != translation {
        transform.translation = translation;
    }
    if let Projection::Orthographic(orthographic) = &mut *projection {
        orthographic.scale = 1.0 / viewport.zoom;
    }
}
