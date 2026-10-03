//! The Grid in the viewport: a thin, faint line along every edge of every cell in view, drawn
//! over the Elements as an overlay of the Editor's own, so no Export ever holds it.

use crate::state::EditorState;
use bevy::color::Color;
use bevy::ecs::system::Res;
use bevy::gizmos::config::{GizmoConfig, GizmoConfigGroup, GizmoLineConfig};
use bevy::gizmos::gizmos::Gizmos;
use bevy::math::Vec2;
use bevy::reflect::Reflect;
use drs_model::Viewport;

/// The fewest screen pixels a cell spans for the Grid to be drawn; below it the lines would wash
/// the view out.
const SMALLEST_CELL: f32 = 8.0;
/// The colour of the Grid's lines: dark and faint, over the background and the Elements alike.
const LINES: Color = Color::srgba(0.0, 0.0, 0.0, 0.2);

/// The gizmos the Grid is drawn with, whose lines are thinner than the other overlays'.
#[derive(Default, Reflect, GizmoConfigGroup)]
pub(crate) struct GridGizmos;

/// How the Grid's lines are drawn: one pixel wide.
pub(crate) fn config() -> GizmoConfig {
    GizmoConfig {
        line: GizmoLineConfig {
            width: 1.0,
            ..GizmoLineConfig::default()
        },
        ..GizmoConfig::default()
    }
}

/// Draws a line at every whole number of cells across the visible part of the viewport in each
/// direction, while a cell is at least eight pixels across, whether or not snapping is on.
/// Nothing is drawn while an Export runs.
pub(crate) fn draw(
    mut gizmos: Gizmos<GridGizmos>,
    state: Res<EditorState>,
    viewport: Res<Viewport>,
) {
    if state.exporting || viewport.zoom < SMALLEST_CELL {
        return;
    }
    let (a, b) = (
        viewport.cells_at(viewport.area.min),
        viewport.cells_at(viewport.area.max),
    );
    let (low, high) = (a.min(b), a.max(b));
    let mut x = low.x.ceil();
    while x <= high.x {
        gizmos.line_2d(Vec2::new(x, low.y), Vec2::new(x, high.y), LINES);
        x += 1.0;
    }
    let mut y = low.y.ceil();
    while y <= high.y {
        gizmos.line_2d(Vec2::new(low.x, y), Vec2::new(high.x, y), LINES);
        y += 1.0;
    }
}
