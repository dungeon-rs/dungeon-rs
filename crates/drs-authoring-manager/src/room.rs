//! Rooms as outline hosts: a Room's closed outline of edges read and written as the shape
//! Engine's outline, combined with the Rooms of its Layer, drawn as its floor and the stroke of
//! the Walls in its look.

use crate::OutlineKind;
use crate::outline::OutlineHost;
use bevy_math::Rect;
use drs_model::{Edge, ElementId, ElementKindName, ROOM, Room, RoomShape, Stretch, StrokeMesh};
use drs_shape_engine::{CombinedOutline, Path};

impl OutlineHost for Room {
    const KIND: ElementKindName = ROOM;
    const OUTLINE: OutlineKind = OutlineKind::Room;
    const FEWEST_POINTS: usize = 3;
    const COLOUR: &'static str = "wall_colour";
    const FLOOR_COLOUR: Option<&'static str> = Some("floor_colour");
    const CUTS: Option<&'static str> = Some("cuts");
    const COMBINES: bool = true;

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

    fn cuts(&self) -> bool {
        self.cuts
    }

    fn shape(
        combined: &CombinedOutline,
        mesh: StrokeMesh,
        stretches: Vec<Stretch>,
        drawn_at: ElementId,
    ) -> RoomShape {
        RoomShape {
            outline: combined.line.clone(),
            walls: combined
                .walls
                .iter()
                .map(|wall| wall.line.clone())
                .collect(),
            walled: combined.walled.clone(),
            stretches,
            mesh,
            drawn_at,
            floor: combined.floor.clone(),
        }
    }
}
