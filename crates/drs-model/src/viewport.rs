//! Where the Author is looking at the Level.

use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;
use bevy_math::{Rect, Vec2};
use bevy_reflect::Reflect;

/// The part of the Level shown on screen: presentation state the Editor steers by panning and
/// zooming, which the render Engine's projection follows. It is never a Command and never a
/// history step.
///
/// Two coordinate systems meet here. Level positions are in Grid cells, `x` growing to the right
/// and `y` growing upwards, as the Bounds' origin at the lower-left corner implies; the render
/// Engine draws one cell as one world unit. Screen points are logical pixels of the window,
/// the origin at the top-left corner and `y` growing downwards, as the window and egui report
/// them. The conversions below are the only place the two meet, so picking and drawing agree.
#[derive(Resource, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Resource)]
pub struct Viewport {
    /// The cell at the centre of [`area`](Self::area).
    pub centre: Vec2,
    /// How many logical screen pixels one cell spans.
    pub zoom: f32,
    /// The rectangle of the window, in logical pixels, the Level is shown in.
    pub area: Rect,
}

impl Default for Viewport {
    /// Centred on the origin at 64 pixels per cell, with no area until the Editor lays one out.
    fn default() -> Self {
        Self {
            centre: Vec2::ZERO,
            zoom: 64.0,
            area: Rect::default(),
        }
    }
}

impl Viewport {
    /// The smallest zoom: a cell is four pixels.
    pub const MIN_ZOOM: f32 = 4.0;
    /// The largest zoom: a cell is 1024 pixels, four times the pixel size of a cell.
    pub const MAX_ZOOM: f32 = 1024.0;

    /// The cell under a screen point.
    #[must_use]
    pub fn cells_at(&self, point: Vec2) -> Vec2 {
        let offset = point - self.area.center();
        self.centre + Vec2::new(offset.x, -offset.y) / self.zoom
    }

    /// The screen point a cell is shown at: the inverse of [`cells_at`](Self::cells_at).
    #[must_use]
    pub fn screen_at(&self, cells: Vec2) -> Vec2 {
        let offset = (cells - self.centre) * self.zoom;
        self.area.center() + Vec2::new(offset.x, -offset.y)
    }

    /// The rectangle of cells shown: the area's size at the zoom, around the centre; of no size
    /// before the Editor lays the area out.
    #[must_use]
    pub fn view(&self) -> Rect {
        Rect::from_center_half_size(self.centre, self.area.half_size() / self.zoom)
    }

    /// Whether a screen point lies in the area the Level is shown in.
    #[must_use]
    pub fn contains(&self, point: Vec2) -> bool {
        self.area.contains(point)
    }

    /// Moves the view along with a pointer that moved by `screen_delta`, so the cell under the
    /// pointer stays under it.
    pub fn pan_by(&mut self, screen_delta: Vec2) {
        self.centre -= Vec2::new(screen_delta.x, -screen_delta.y) / self.zoom;
    }

    /// Multiplies the zoom by `factor`, within the limits, keeping the cell under `point` where
    /// it is on screen.
    pub fn zoom_by(&mut self, factor: f32, point: Vec2) {
        let before = self.cells_at(point);
        self.zoom = (self.zoom * factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        let after = self.cells_at(point);
        self.centre += before - after;
    }
}
