//! Rooms as outline hosts: a Room's closed outline of edges read and written as the shape
//! Engine's outline, drawn as its floor under the stroke of its Walls.

use crate::OutlineKind;
use crate::outline::OutlineHost;
use bevy_math::Rect;
use drs_model::{Edge, ElementKindName, FillMesh, ROOM, Room, RoomShape, WallShape};
use drs_shape_engine::Path;

impl OutlineHost for Room {
    const KIND: ElementKindName = ROOM;
    const OUTLINE: OutlineKind = OutlineKind::Room;
    const FEWEST_POINTS: usize = 3;
    const COLOUR: &'static str = "wall_colour";
    const FLOOR_COLOUR: Option<&'static str> = Some("floor_colour");
    const CUTS: Option<&'static str> = Some("cuts");

    type Shape = RoomShape;

    fn control_path(part: usize) -> String {
        format!("edges[{part}].control")
    }

    fn path(&self) -> Path {
        Path::of_room(self)
    }

    fn with_path(&self, path: Path) -> Self {
        Self {
            points: path.points,
            edges: path
                .controls
                .into_iter()
                .map(|control| Edge { control })
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

    fn shape(walls: WallShape, floor: FillMesh) -> RoomShape {
        RoomShape { walls, floor }
    }
}
