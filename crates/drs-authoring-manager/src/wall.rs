//! Walls as outline hosts: a Wall's open line of segments read and written as the shape Engine's
//! outline, drawn as its stroke alone.

use crate::OutlineKind;
use crate::outline::OutlineHost;
use bevy_math::Rect;
use drs_model::{ElementKindName, FillMesh, Segment, WALL, Wall, WallShape};
use drs_shape_engine::Path;

impl OutlineHost for Wall {
    const KIND: ElementKindName = WALL;
    const OUTLINE: OutlineKind = OutlineKind::Wall;
    const FEWEST_POINTS: usize = 2;
    const COLOUR: &'static str = "colour";
    const FLOOR_COLOUR: Option<&'static str> = None;
    const CUTS: Option<&'static str> = None;

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

    /// A Wall's line encloses nothing, so its shape is its stroke.
    fn shape(walls: WallShape, _floor: FillMesh) -> WallShape {
        walls
    }
}
