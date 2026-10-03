//! Walls as outline hosts: a Wall's open line of segments read and written as the shape Engine's
//! outline, drawn as its stroke alone.

use crate::OutlineKind;
use crate::outline::OutlineHost;
use bevy_math::Rect;
use drs_model::{ElementId, ElementKindName, Segment, Stretch, StrokeMesh, WALL, Wall, WallShape};
use drs_shape_engine::{CombinedOutline, Path};

impl OutlineHost for Wall {
    const KIND: ElementKindName = WALL;
    const OUTLINE: OutlineKind = OutlineKind::Wall;
    const FEWEST_POINTS: usize = 2;
    const COLOUR: &'static str = "colour";
    const FLOOR_COLOUR: Option<&'static str> = None;
    const CUTS: Option<&'static str> = None;
    const COMBINES: bool = false;

    type Shape = WallShape;

    fn control_path(part: usize) -> String {
        format!("segments[{part}].control")
    }

    fn path(&self) -> Path {
        Path::of_wall(self)
    }

    fn with_path(&self, path: Path) -> Self {
        Self {
            points: path.points,
            segments: path
                .controls
                .into_iter()
                .map(|control| Segment { control })
                .collect(),
            ..self.clone()
        }
    }

    fn thickness(&self) -> f32 {
        self.thickness
    }

    fn with_thickness(&self, thickness: f32) -> Self {
        Self {
            thickness,
            ..self.clone()
        }
    }

    fn element_box(&self) -> Rect {
        Self::element_box(self)
    }

    fn malformation(&self) -> Option<String> {
        Self::malformation(self)
    }

    fn cuts(&self) -> bool {
        false
    }

    /// A Wall's line encloses nothing and combines with nothing, so its shape is its line and
    /// its stroke.
    fn shape(
        combined: &CombinedOutline,
        mesh: StrokeMesh,
        stretches: Vec<Stretch>,
        _drawn_at: ElementId,
    ) -> WallShape {
        WallShape {
            line: combined.line.clone(),
            stretches,
            mesh,
        }
    }
}
