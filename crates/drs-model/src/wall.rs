//! Walls: lines of straight and curved segments drawn at a thickness, and the shape derived from
//! them for drawing and picking.

use crate::{ElementKindName, Serialisable, SerialisationError, Tier, read_only_version};
use bevy_ecs::component::Component;
use bevy_ecs::reflect::ReflectComponent;
use bevy_math::{Rect, Vec2};
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

/// The Wall kind.
pub const WALL: ElementKindName = ElementKindName::new("wall");

/// An opaque colour, its channels in sRGB.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Colour {
    /// The red channel.
    pub red: u8,
    /// The green channel.
    pub green: u8,
    /// The blue channel.
    pub blue: u8,
}

impl Colour {
    /// A colour of three sRGB channels.
    #[must_use]
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

/// One segment of a Wall, from one of its points to the next: straight, or curved as the
/// quadratic Bézier curve through its control point.
#[derive(Reflect, Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    /// The control point in Grid cells, or `None` for a straight segment.
    pub control: Option<Vec2>,
}

/// A boundary along a line: an ordered list of two or more points in Grid cells with a segment
/// between each point and the next, drawn at a thickness in an opaque colour.
///
/// The segment from the first point to the second is the first segment, and so on: what is set
/// into a Wall is anchored by segment number, and only adding or removing a point renumbers the
/// segments.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Wall {
    /// The points, in order.
    pub points: Vec<Vec2>,
    /// One entry per segment, one fewer than there are points.
    pub segments: Vec<Segment>,
    /// How wide the Wall is drawn, in Grid cells.
    pub thickness: f32,
    /// The colour the Wall is drawn in.
    pub colour: Colour,
}

impl Wall {
    /// A Wall through `points` whose every segment is straight.
    #[must_use]
    pub fn straight(points: Vec<Vec2>, thickness: f32, colour: Colour) -> Self {
        let segments = vec![Segment::default(); points.len().saturating_sub(1)];
        Self {
            points,
            segments,
            thickness,
            colour,
        }
    }

    /// The box an Element of this Wall has: the smallest box around its points and control
    /// points, grown by half the thickness on every side.
    #[must_use]
    pub fn element_box(&self) -> Rect {
        let controls = self.segments.iter().filter_map(|segment| segment.control);
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

    /// Why the Wall is not one, if it is not: fewer than two points, a segment count that does
    /// not match the points, a thickness not above zero, or a coordinate that is not finite.
    #[must_use]
    pub fn malformation(&self) -> Option<String> {
        if self.points.len() < 2 {
            return Some(format!(
                "a Wall needs two or more points, not {}",
                self.points.len()
            ));
        }
        if self.segments.len() + 1 != self.points.len() {
            return Some(format!(
                "a Wall of {} points has {} segments, not {}",
                self.points.len(),
                self.points.len() - 1,
                self.segments.len()
            ));
        }
        if !(self.thickness > 0.0 && self.thickness.is_finite()) {
            return Some(format!(
                "a Wall's thickness must be above zero, not {}",
                self.thickness
            ));
        }
        let controls = self.segments.iter().filter_map(|segment| segment.control);
        if !self
            .points
            .iter()
            .copied()
            .chain(controls)
            .all(Vec2::is_finite)
        {
            return Some("a Wall's points must be finite".to_owned());
        }
        None
    }
}

impl Serialisable for Wall {
    const NAME: &'static str = "wall";
    const VERSION: u32 = 1;
    const TIER: Tier = Tier::Element;

    fn read(version: u32, data: &RawValue) -> Result<Self, SerialisationError> {
        let wall: Self = read_only_version(version, data)?;
        match wall.malformation() {
            Some(reason) => Err(SerialisationError::Malformed {
                component: Self::NAME.to_owned(),
                reason,
            }),
            None => Ok(wall),
        }
    }
}

/// A point of a Wall's flattened line, with the segment it lies on and the parameter along that
/// segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinePoint {
    /// Where the point is, in Grid cells.
    pub position: Vec2,
    /// The number of the segment it lies on, counted from zero.
    pub segment: usize,
    /// The parameter along that segment, from zero at its first point to one at its second.
    pub t: f32,
}

/// The triangles a Wall is drawn with, in Grid cells.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StrokeMesh {
    /// The corners of the triangles.
    pub vertices: Vec<Vec2>,
    /// For each vertex, how far along the Wall's line it sits, in Grid cells from its start.
    pub arc_lengths: Vec<f32>,
    /// Three vertex indices per triangle.
    pub indices: Vec<u32>,
}

/// The shape derived from a [`Wall`]: its line flattened into chords, and the stroke it is drawn
/// with. It is never saved; the authoring Manager derives it whenever the Wall changes, and
/// whoever draws or picks a Wall reads it.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct WallShape {
    /// The flattened line, from the first point to the last; each chord is no farther than the
    /// flattening tolerance from the curve it stands for.
    pub line: Vec<LinePoint>,
    /// The stroke: the line at the Wall's thickness, with round joins and caps.
    pub mesh: StrokeMesh,
}
