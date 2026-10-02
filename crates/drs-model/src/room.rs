//! Rooms: floors with Walls around a closed outline of straight and curved edges, and the shape
//! derived from them for drawing and picking.

use crate::{
    Colour, ElementKindName, Serialisable, SerialisationError, Tier, WallShape, read_only_version,
};
use bevy_ecs::component::Component;
use bevy_ecs::reflect::ReflectComponent;
use bevy_math::{Rect, Vec2};
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

/// The Room kind.
pub const ROOM: ElementKindName = ElementKindName::new("room");

/// One edge of a Room, from one of its points to the next: straight, or curved as the quadratic
/// Bézier curve through its control point.
#[derive(Reflect, Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    /// The control point in Grid cells, or `None` for a straight edge.
    pub control: Option<Vec2>,
}

/// A floor area with Walls generated around its outline: an ordered list of three or more points
/// in Grid cells with an edge from each point to the next and from the last back to the first,
/// a wall thickness, an opaque wall colour, and an opaque floor colour.
///
/// The edge from the first point to the second is the first edge, and so on, the edge from the
/// last point back to the first being the last: what is set into a Room's Walls is anchored by
/// edge number, and only adding or removing a point renumbers the edges. The points keep the
/// order the Author gave them; nothing depends on the outline's direction.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Room {
    /// The points, in order.
    pub points: Vec<Vec2>,
    /// One entry per edge, as many as there are points, the last for the edge back to the first.
    pub edges: Vec<Edge>,
    /// How wide the Walls are drawn, in Grid cells.
    pub thickness: f32,
    /// The colour the Walls are drawn in.
    pub wall_colour: Colour,
    /// The colour the floor is drawn in.
    pub floor_colour: Colour,
}

impl Room {
    /// A Room through `points` whose every edge is straight.
    #[must_use]
    pub fn straight(
        points: Vec<Vec2>,
        thickness: f32,
        wall_colour: Colour,
        floor_colour: Colour,
    ) -> Self {
        let edges = vec![Edge::default(); points.len()];
        Self {
            points,
            edges,
            thickness,
            wall_colour,
            floor_colour,
        }
    }

    /// The box an Element of this Room has: the smallest box around its points and control
    /// points, grown by half the thickness on every side.
    #[must_use]
    pub fn element_box(&self) -> Rect {
        let controls = self.edges.iter().filter_map(|edge| edge.control);
        let mut corners = self.points.iter().copied().chain(controls);
        let Some(first) = corners.next() else {
            return Rect::default();
        };
        let (min, max) = corners.fold((first, first), |(min, max), point| {
            (min.min(point), max.max(point))
        });
        let half = Vec2::splat(self.thickness / 2.0);
        Rect {
            min: min - half,
            max: max + half,
        }
    }

    /// Why the Room is not one, if it is not: fewer than three points, an edge count that does
    /// not match the points, a thickness not above zero, or a coordinate that is not finite.
    #[must_use]
    pub fn malformation(&self) -> Option<String> {
        if self.points.len() < 3 {
            return Some(format!(
                "a Room needs three or more points, not {}",
                self.points.len()
            ));
        }
        if self.edges.len() != self.points.len() {
            return Some(format!(
                "a Room of {} points needs {} edges, not {}",
                self.points.len(),
                self.points.len(),
                self.edges.len()
            ));
        }
        if !(self.thickness > 0.0 && self.thickness.is_finite()) {
            return Some(format!(
                "a Room's wall thickness must be above zero, not {}",
                self.thickness
            ));
        }
        let controls = self.edges.iter().filter_map(|edge| edge.control);
        if !self
            .points
            .iter()
            .copied()
            .chain(controls)
            .all(Vec2::is_finite)
        {
            return Some("a Room's points must be finite".to_owned());
        }
        None
    }
}

impl Serialisable for Room {
    const NAME: &'static str = "room";
    const VERSION: u32 = 1;
    const TIER: Tier = Tier::Element;

    fn read(version: u32, data: &RawValue) -> Result<Self, SerialisationError> {
        let room: Self = read_only_version(version, data)?;
        match room.malformation() {
            Some(reason) => Err(SerialisationError::Malformed {
                component: Self::NAME.to_owned(),
                reason,
            }),
            None => Ok(room),
        }
    }
}

/// The triangles a floor is filled with, in Grid cells.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FillMesh {
    /// The corners of the triangles.
    pub vertices: Vec<Vec2>,
    /// Three vertex indices per triangle.
    pub indices: Vec<u32>,
}

/// The shape derived from a [`Room`]: the Walls around its outline, drawn and picked as a Wall's
/// are, and its floor. It is never saved; the authoring Manager derives it whenever the Room or a
/// Portal set into it changes, and whoever draws or picks a Room reads it.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct RoomShape {
    /// The Walls: the outline flattened into a closed line, from the first point round to the
    /// first point again, each point tagged with the edge it lies on as its segment; the
    /// stretches the Portals set into the Room cover, a stretch whose start lies after its end
    /// running on past the first point; and the stroke centred on the line with a round join at
    /// every point and no caps, left out along the stretches.
    pub walls: WallShape,
    /// The floor: everything the closed line winds around, filled up to the line under the
    /// non-zero rule, so its edge and the Walls' centre line are the same chords.
    pub floor: FillMesh,
}
