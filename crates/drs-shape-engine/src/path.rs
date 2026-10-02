//! The outline every operation of the Engine reads: a Wall's open line or a Room's closed
//! outline, its parts straight or curved, and its flattening into a line of chords.

use crate::wall::{point, vector};
use bevy_math::Vec2;
use drs_model::{LinePoint, Room, Wall};
use kurbo::{Line, ParamCurve, Point, QuadBez};

/// How far, in Grid cells, a chord of the flattened line or an arc of the stroke may stray from
/// the curve it stands for: a pixel at the highest Export resolution.
///
/// A curved part so strongly bent that it would take more than [`MOST_CHORDS`] chords is
/// flattened into that many, and its chords then stray farther; that takes a control point more
/// than thirty thousand cells from the middle of its part.
pub(crate) const TOLERANCE: f64 = 0.001;

/// The fewest chords a curved part is flattened into, so that its middle is always a point of
/// the line.
const FEWEST_CHORDS: f64 = 2.0;

/// The most chords one curved part is flattened into, whatever its size; past it the
/// [`TOLERANCE`] no longer holds.
const MOST_CHORDS: f64 = 4096.0;

/// A Wall's line or a Room's outline as the Engine reads it: points in Grid cells with a part
/// from each point to the next, straight or curved as the quadratic Bézier curve through its
/// control point, and, when the outline is closed, a last part from the last point back to the
/// first. A Wall's parts are its segments and a Room's its edges, numbered from zero.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    /// The points, in order.
    pub points: Vec<Vec2>,
    /// The control point of each part, or `None` for a straight one: one fewer than there are
    /// points on an open outline, as many on a closed one.
    pub controls: Vec<Option<Vec2>>,
    /// Whether the last part runs from the last point back to the first.
    pub closed: bool,
}

impl Path {
    /// A Wall's line: open, a segment between each point and the next.
    #[must_use]
    pub fn of_wall(wall: &Wall) -> Self {
        Self {
            points: wall.points.clone(),
            controls: wall
                .segments
                .iter()
                .map(|segment| segment.control)
                .collect(),
            closed: false,
        }
    }

    /// A Room's outline: closed, an edge from each point to the next and from the last back to
    /// the first.
    #[must_use]
    pub fn of_room(room: &Room) -> Self {
        Self {
            points: room.points.clone(),
            controls: room.edges.iter().map(|edge| edge.control).collect(),
            closed: true,
        }
    }

    /// How many parts the outline has: the segments of a Wall or the edges of a Room.
    #[must_use]
    pub fn parts(&self) -> usize {
        self.controls.len()
    }

    /// The two points the part `part` runs between, if the outline has that part.
    pub(crate) fn ends(&self, part: usize) -> Option<(Vec2, Vec2)> {
        self.controls.get(part)?;
        let next = if self.closed && part + 1 == self.points.len() {
            0
        } else {
            part + 1
        };
        Some((*self.points.get(part)?, *self.points.get(next)?))
    }

    /// The part `part` as a curve, if the outline has that part.
    pub(crate) fn curve(&self, part: usize) -> Option<Curve> {
        let (start, end) = self.ends(part)?;
        let (start, end) = (point(start), point(end));
        Some(match self.controls.get(part).copied().flatten() {
            None => Curve::Straight(Line::new(start, end)),
            Some(control) => Curve::Bent(QuadBez::new(start, point(control), end)),
        })
    }
}

/// One part of an outline as a curve.
pub(crate) enum Curve {
    /// A straight part.
    Straight(Line),
    /// A curved part.
    Bent(QuadBez),
}

impl Curve {
    /// The point at parameter `t`.
    pub(crate) fn eval(&self, t: f64) -> Point {
        match self {
            Self::Straight(line) => line.eval(t),
            Self::Bent(quad) => quad.eval(t),
        }
    }
}

/// How many chords keep a quadratic part within the tolerance: a chord over a parameter
/// interval `h` strays at most `|p0 - 2c + p1| h² / 4` from the curve, as its second derivative
/// is constant. The count is even, so the part's middle is one of the cuts.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the count is a positive whole number of at most `MOST_CHORDS`"
)]
fn chords_of(quad: &QuadBez) -> usize {
    let bend = crate::wall::length(quad.p0.to_vec2() - quad.p1.to_vec2() * 2.0 + quad.p2.to_vec2());
    let chords = (bend / (4.0 * TOLERANCE))
        .sqrt()
        .ceil()
        .clamp(FEWEST_CHORDS, MOST_CHORDS) as usize;
    chords + chords % 2
}

/// The outline flattened into points tagged with their part and parameter, from the first point
/// to the last, or round to the first again, exactly, when it is closed.
///
/// A straight part is one chord; a curved one is cut at evenly spaced parameters, an even number
/// of them so that its middle is a point of the line, as many as keep every chord within a
/// thousandth of a cell of the curve. A point shared by two parts belongs to the later one, at
/// parameter zero, and the last point to the last part at one.
#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    reason = "chord counts are far below where f64 loses whole numbers; parameters are kept in \
              single precision"
)]
pub(crate) fn flatten(path: &Path) -> Vec<LinePoint> {
    let mut line = Vec::new();
    let last = path.parts().saturating_sub(1);
    for part in 0..path.parts() {
        let Some(curve) = path.curve(part) else {
            break;
        };
        let chords = match &curve {
            Curve::Straight(_) => 1,
            Curve::Bent(quad) => chords_of(quad),
        };
        let cuts = if part == last { chords + 1 } else { chords };
        for cut in 0..cuts {
            let t = cut as f64 / chords as f64;
            line.push(LinePoint {
                position: vector(curve.eval(t)),
                segment: part,
                t: t as f32,
            });
        }
    }
    if path.closed
        && let (Some(last), Some(first)) = (line.last_mut(), path.points.first())
    {
        last.position = *first;
    }
    line
}
