//! `GenerateWalls` and `SplitWall` for drawn Walls: the flattened line, the stroke mesh, and the
//! exact split of a segment.
//!
//! Everything is computed in double precision and handed to the model in single precision. No
//! trigonometric function is called anywhere: the flattening counts come from square roots, and
//! a round join or cap is an arc subdivided by bisecting unit vectors, whose angle halves at each
//! step, so the only operation beyond arithmetic is the correctly rounded square root and the
//! mesh is the same on every machine.

use bevy_math::Vec2;
use drs_model::{LinePoint, Segment, StrokeMesh, Wall, WallShape};
use kurbo::{Line, ParamCurve, Point, QuadBez};

/// How far, in Grid cells, a chord of the flattened line or an arc of the stroke may stray from
/// the curve it stands for: a pixel at the highest Export resolution.
const TOLERANCE: f64 = 0.001;

/// The most chords one curved segment is flattened into, whatever its size.
const MOST_CHORDS: f64 = 4096.0;

/// The most times an arc of a join or cap is halved: 2¹⁶ pieces is far finer than any Wall needs.
const MOST_HALVINGS: u32 = 16;

/// Shorter than this, in Grid cells, two consecutive points of the line count as one for the
/// stroke, which has no direction to give them.
const COINCIDENT: f64 = 1e-9;

/// Why a Wall could not be split.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ShapeError {
    /// The Wall has no segment of that number.
    #[error("the Wall has no segment {segment}; it has {segments}")]
    NoSegment {
        /// The segment asked for.
        segment: usize,
        /// How many segments the Wall has.
        segments: usize,
    },
    /// The parameter does not lie strictly between the segment's two points.
    #[error("a segment is split strictly between its points, not at {0}")]
    OutsideSegment(f32),
}

/// `GenerateWalls`: the shape a drawn Wall is drawn and picked by.
///
/// The line runs from the first point to the last. A straight segment is one chord; a curved
/// one is cut at evenly spaced parameters, an even number of them so that its middle is a point
/// of the line, as many as keep every chord within a thousandth of a cell of the curve. Each
/// point is tagged with the segment it lies on and the parameter along it; a point shared by two
/// segments belongs to the later one, at parameter zero, and the last point to the last segment
/// at one. The stroke covers everything within half the thickness of the line, with round joins
/// at its points and round caps at its ends, and records the arc length of the line at each
/// vertex.
#[must_use]
pub fn generate_walls(wall: &Wall) -> WallShape {
    let line = flatten(wall);
    let mesh = stroke(&line, f64::from(wall.thickness) / 2.0);
    WallShape { line, mesh }
}

/// `SplitWall`: the Wall with a point added on `segment` at parameter `t`, splitting the segment
/// into two whose joined curve is exactly the one it had.
///
/// A straight segment becomes two straight segments; a curved one becomes the two halves of its
/// curve, each with its own control point. The new point sits after the segment's first point,
/// so the segments after it are numbered one higher.
///
/// # Errors
///
/// [`ShapeError::NoSegment`] when the Wall has no such segment, or
/// [`ShapeError::OutsideSegment`] when `t` is not strictly between zero and one.
pub fn split_wall(wall: &Wall, segment: usize, t: f32) -> Result<Wall, ShapeError> {
    let Some(split) = wall.segments.get(segment) else {
        return Err(ShapeError::NoSegment {
            segment,
            segments: wall.segments.len(),
        });
    };
    if !(t > 0.0 && t < 1.0) {
        return Err(ShapeError::OutsideSegment(t));
    }
    let (Some(&start), Some(&end)) = (wall.points.get(segment), wall.points.get(segment + 1))
    else {
        return Err(ShapeError::NoSegment {
            segment,
            segments: wall.points.len().saturating_sub(1),
        });
    };
    let t = f64::from(t);
    let (middle, first, second) = match split.control {
        None => {
            let middle = Line::new(point(start), point(end)).eval(t);
            (middle, None, None)
        }
        Some(control) => {
            let curve = QuadBez::new(point(start), point(control), point(end));
            let first = curve.subsegment(0.0..t);
            let second = curve.subsegment(t..1.0);
            (first.p2, Some(vector(first.p1)), Some(vector(second.p1)))
        }
    };
    let mut split_wall = wall.clone();
    split_wall.points.insert(segment + 1, vector(middle));
    split_wall.segments[segment] = Segment { control: first };
    split_wall
        .segments
        .insert(segment + 1, Segment { control: second });
    Ok(split_wall)
}

/// A model position as a kurbo point.
fn point(position: Vec2) -> Point {
    Point::new(f64::from(position.x), f64::from(position.y))
}

/// A kurbo point as a model position.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the model keeps positions in single precision"
)]
fn vector(point: Point) -> Vec2 {
    Vec2::new(point.x as f32, point.y as f32)
}

/// The length of a vector, through the correctly rounded square root only.
fn length(vector: kurbo::Vec2) -> f64 {
    (vector.x * vector.x + vector.y * vector.y).sqrt()
}

/// How many chords keep a quadratic segment within the tolerance: a chord over a parameter
/// interval `h` strays at most `|p0 - 2c + p1| h² / 4` from the curve, as its second derivative
/// is constant. The count is even, so the segment's middle is one of the cuts.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the count is a positive whole number of at most `MOST_CHORDS`"
)]
fn chords_of(start: Point, control: Point, end: Point) -> usize {
    let bend = length(start.to_vec2() - control.to_vec2() * 2.0 + end.to_vec2());
    let chords = (bend / (4.0 * TOLERANCE))
        .sqrt()
        .ceil()
        .clamp(2.0, MOST_CHORDS) as usize;
    chords + chords % 2
}

/// The Wall's line flattened into points tagged with their segment and parameter.
#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    reason = "chord counts are far below where f64 loses whole numbers; parameters are kept in \
              single precision"
)]
fn flatten(wall: &Wall) -> Vec<LinePoint> {
    let mut line = Vec::new();
    let last = wall.segments.len().saturating_sub(1);
    for (index, (segment, ends)) in wall.segments.iter().zip(wall.points.windows(2)).enumerate() {
        let (start, end) = (point(ends[0]), point(ends[1]));
        let (chords, curve) = match segment.control {
            None => (1, None),
            Some(control) => {
                let control = point(control);
                (
                    chords_of(start, control, end),
                    Some(QuadBez::new(start, control, end)),
                )
            }
        };
        let cuts = if index == last { chords + 1 } else { chords };
        for cut in 0..cuts {
            let t = cut as f64 / chords as f64;
            let at = match curve {
                Some(curve) => curve.eval(t),
                None => Line::new(start, end).eval(t),
            };
            line.push(LinePoint {
                position: vector(at),
                segment: index,
                t: t as f32,
            });
        }
    }
    line
}

/// The stroke being built.
#[derive(Default)]
struct Builder {
    /// The mesh so far.
    mesh: StrokeMesh,
}

impl Builder {
    /// Adds a vertex and returns its index.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the model keeps the mesh in single precision"
    )]
    fn vertex(&mut self, at: Point, arc_length: f64) -> u32 {
        // A Wall would need a billion chords to come near; the index saturates rather than wraps.
        debug_assert!(
            u32::try_from(self.mesh.vertices.len()).is_ok(),
            "a stroke of more vertices than a mesh index reaches"
        );
        let index = u32::try_from(self.mesh.vertices.len()).unwrap_or(u32::MAX);
        self.mesh.vertices.push(vector(at));
        self.mesh.arc_lengths.push(arc_length as f32);
        index
    }

    /// Adds a triangle.
    fn triangle(&mut self, a: u32, b: u32, c: u32) {
        self.mesh.indices.extend_from_slice(&[a, b, c]);
    }

    /// Adds the band of one chord, from `start` to `end`, `normal` scaled to half the thickness.
    fn band(&mut self, start: (Point, f64), end: (Point, f64), normal: kurbo::Vec2) {
        let a = self.vertex(start.0 + normal, start.1);
        let b = self.vertex(start.0 - normal, start.1);
        let c = self.vertex(end.0 + normal, end.1);
        let d = self.vertex(end.0 - normal, end.1);
        self.triangle(a, b, c);
        self.triangle(c, b, d);
    }

    /// Adds a fan around `centre` through the radii in `rim`, the first and last included.
    fn fan(&mut self, centre: (Point, f64), radius: f64, rim: &[kurbo::Vec2]) {
        let hub = self.vertex(centre.0, centre.1);
        let mut previous = None;
        for direction in rim {
            let corner = self.vertex(centre.0 + *direction * radius, centre.1);
            if let Some(previous) = previous {
                self.triangle(hub, previous, corner);
            }
            previous = Some(corner);
        }
    }
}

/// The unit vector halfway between two unit vectors less than half a turn apart.
fn halfway(a: kurbo::Vec2, b: kurbo::Vec2) -> kurbo::Vec2 {
    let sum = a + b;
    sum / length(sum)
}

/// How many times the arc from `a` to `b`, less than half a turn, is halved so that each piece's
/// chord strays no more than the tolerance from a circle of `radius`.
///
/// A chord across an angle φ strays `radius · (1 − cos(φ / 2))`, and the cosine of half an angle
/// is `√((1 + cos) / 2)`, so each halving is one square root.
fn halvings(a: kurbo::Vec2, b: kurbo::Vec2, radius: f64) -> u32 {
    let mut cosine_of_half = f64::midpoint(1.0, a.dot(b).clamp(-1.0, 1.0)).sqrt();
    let mut halvings = 0;
    while radius * (1.0 - cosine_of_half) > TOLERANCE && halvings < MOST_HALVINGS {
        halvings += 1;
        cosine_of_half = f64::midpoint(1.0, cosine_of_half).sqrt();
    }
    halvings
}

/// Pushes the radii of the arc from `a` to `b`, less than half a turn, halved `times` times,
/// leaving out `a` itself.
fn arc(a: kurbo::Vec2, b: kurbo::Vec2, times: u32, rim: &mut Vec<kurbo::Vec2>) {
    if times == 0 {
        rim.push(b);
        return;
    }
    let middle = halfway(a, b);
    arc(a, middle, times - 1, rim);
    arc(middle, b, times - 1, rim);
}

/// The radii of a half turn from `from` through `through` to the opposite of `from`, as two
/// quarter turns.
fn half_turn(from: kurbo::Vec2, through: kurbo::Vec2, radius: f64) -> Vec<kurbo::Vec2> {
    let times = halvings(from, through, radius);
    let mut rim = vec![from];
    arc(from, through, times, &mut rim);
    arc(through, -from, times, &mut rim);
    rim
}

/// The stroke of a flattened line at half-thickness `radius`: a band per chord, a round join on
/// the outer side of each bend, and a round cap at each end.
fn stroke(line: &[LinePoint], radius: f64) -> StrokeMesh {
    // The line's points with their arc lengths, coincident points merged.
    let mut points: Vec<(Point, f64)> = Vec::with_capacity(line.len());
    let mut travelled = 0.0;
    for line_point in line {
        let at = point(line_point.position);
        match points.last() {
            None => points.push((at, 0.0)),
            Some(&(previous, _)) => {
                let step = length(at - previous);
                if step > COINCIDENT {
                    travelled += step;
                    points.push((at, travelled));
                }
            }
        }
    }
    let mut builder = Builder::default();
    let Some(&first) = points.first() else {
        return builder.mesh;
    };
    if points.len() == 1 {
        // A Wall whose points all coincide is drawn as a dot of its thickness.
        let east = kurbo::Vec2::new(1.0, 0.0);
        let north = kurbo::Vec2::new(0.0, 1.0);
        builder.fan(first, radius, &half_turn(north, -east, radius));
        builder.fan(first, radius, &half_turn(-north, east, radius));
        return builder.mesh;
    }
    let directions: Vec<kurbo::Vec2> = points
        .windows(2)
        .map(|pair| {
            let along = pair[1].0 - pair[0].0;
            along / length(along)
        })
        .collect();
    let left = |direction: kurbo::Vec2| kurbo::Vec2::new(-direction.y, direction.x);

    for (pair, direction) in points.windows(2).zip(&directions) {
        builder.band(pair[0], pair[1], left(*direction) * radius);
    }
    for (index, turn) in directions.windows(2).enumerate() {
        let (before, after) = (turn[0], turn[1]);
        let bend = before.cross(after);
        let (from, to) = if bend > 0.0 {
            (-left(before), -left(after))
        } else {
            (left(before), left(after))
        };
        let centre = points[index + 1];
        // `from` and `to` are `before` and `after` turned by the same quarter turn, so their dot
        // product is the same.
        if before.dot(after) <= -1.0 + f64::EPSILON {
            // The line turns right back on itself: the outer side is the whole way round the
            // front.
            builder.fan(centre, radius, &half_turn(from, before, radius));
        } else if from.dot(to) < 1.0 {
            let mut rim = vec![from];
            arc(from, to, halvings(from, to, radius), &mut rim);
            builder.fan(centre, radius, &rim);
        }
    }
    if let (Some(&start), Some(&end), Some(&last)) =
        (directions.first(), directions.last(), points.last())
    {
        builder.fan(first, radius, &half_turn(left(start), -start, radius));
        builder.fan(last, radius, &half_turn(-left(end), end, radius));
    }
    builder.mesh
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        clippy::float_cmp,
        reason = "a test stops at the first thing that is not as expected, and the middle of a \
                  segment is exactly a half"
    )]

    use super::*;
    use drs_model::Colour;

    /// A grey Wall a quarter of a cell thick through `points`, curved where a control is given.
    fn wall(points: &[Vec2], controls: &[Option<Vec2>]) -> Wall {
        Wall {
            points: points.to_vec(),
            segments: controls
                .iter()
                .map(|control| Segment { control: *control })
                .collect(),
            thickness: 0.25,
            colour: Colour::rgb(60, 60, 60),
        }
    }

    /// The distance from `p` to the chord from `a` to `b`.
    fn to_chord(p: Vec2, a: Vec2, b: Vec2) -> f32 {
        let along = b - a;
        let t = if along.length_squared() == 0.0 {
            0.0
        } else {
            ((p - a).dot(along) / along.length_squared()).clamp(0.0, 1.0)
        };
        p.distance(a + along * t)
    }

    /// The distance from `p` to the nearest chord of a flattened line.
    fn to_line(p: Vec2, line: &[LinePoint]) -> f32 {
        line.windows(2)
            .map(|pair| to_chord(p, pair[0].position, pair[1].position))
            .fold(f32::INFINITY, f32::min)
    }

    /// Whether `p` lies in a triangle of the mesh, edges included.
    fn covered(p: Vec2, mesh: &StrokeMesh) -> bool {
        mesh.indices.chunks(3).any(|corners| {
            let [a, b, c] = [0, 1, 2].map(|i| mesh.vertices[corners[i] as usize]);
            let side = |u: Vec2, v: Vec2| (v - u).perp_dot(p - u);
            let (ab, bc, ca) = (side(a, b), side(b, c), side(c, a));
            (ab >= 0.0 && bc >= 0.0 && ca >= 0.0) || (ab <= 0.0 && bc <= 0.0 && ca <= 0.0)
        })
    }

    /// A straight segment is flattened to its two points and nothing between them.
    #[test]
    fn a_straight_segment_flattens_to_its_ends() {
        let points = [Vec2::ZERO, Vec2::new(4.0, 0.0), Vec2::new(4.0, 3.0)];
        let shape = generate_walls(&wall(&points, &[None, None]));

        let tags: Vec<(Vec2, usize, f32)> = shape
            .line
            .iter()
            .map(|point| (point.position, point.segment, point.t))
            .collect();
        assert_eq!(
            tags,
            vec![
                (points[0], 0, 0.0),
                (points[1], 1, 0.0),
                (points[2], 1, 1.0)
            ]
        );
    }

    /// No chord of a curved segment strays more than the tolerance from the curve it stands for.
    #[test]
    fn the_chord_stays_within_tolerance() {
        let (start, control, end) = (Vec2::ZERO, Vec2::new(3.0, 7.0), Vec2::new(9.0, -1.0));
        let shape = generate_walls(&wall(&[start, end], &[Some(control)]));
        let curve = QuadBez::new(point(start), point(control), point(end));

        assert!(shape.line.len() > 10, "a strong curve takes many chords");
        for pair in shape.line.windows(2) {
            for step in 0..=20 {
                let t = f64::from(pair[0].t)
                    + (f64::from(pair[1].t) - f64::from(pair[0].t)) * f64::from(step) / 20.0;
                let on_curve = vector(curve.eval(t));
                let off = to_chord(on_curve, pair[0].position, pair[1].position);
                assert!(f64::from(off) <= TOLERANCE + 1e-6, "{off} off at t = {t}");
            }
        }
        let middle = shape
            .line
            .iter()
            .find(|point| point.t == 0.5)
            .expect("the middle of the segment is a point of the line");
        assert_eq!(middle.position, vector(curve.eval(0.5)));
    }

    /// The stroke covers every point within half the thickness of the line, round at the joins
    /// and the ends, and nothing beyond.
    #[test]
    fn the_mesh_covers_the_thickness() {
        let shape = generate_walls(&wall(
            &[
                Vec2::ZERO,
                Vec2::new(3.0, 0.0),
                Vec2::new(3.0, 3.0),
                Vec2::new(0.5, 1.0),
            ],
            &[None, Some(Vec2::new(5.0, 1.5)), None],
        ));
        let half = 0.125;
        let margin = 2.0 * 0.001;
        let mut inside = 0;
        let mut outside = 0;
        for x in -40..=80 {
            for y in -40..=80 {
                #[expect(clippy::cast_precision_loss, reason = "small whole numbers")]
                let p = Vec2::new(x as f32, y as f32) / 20.0 + Vec2::new(0.013, 0.007);
                let distance = to_line(p, &shape.line);
                if distance <= half - margin {
                    inside += 1;
                    assert!(covered(p, &shape.mesh), "{p} at {distance} is not covered");
                } else if distance >= half + margin {
                    outside += 1;
                    assert!(!covered(p, &shape.mesh), "{p} at {distance} is covered");
                }
            }
        }
        assert!(
            inside > 300 && outside > 10_000,
            "{inside} in, {outside} out"
        );
        assert_eq!(shape.mesh.vertices.len(), shape.mesh.arc_lengths.len());
        let longest = shape.mesh.arc_lengths.iter().copied().fold(0.0, f32::max);
        let length: f32 = shape
            .line
            .windows(2)
            .map(|pair| pair[0].position.distance(pair[1].position))
            .sum();
        assert!(
            (longest - length).abs() < 1e-4,
            "{longest} against {length}"
        );
    }

    /// A split segment's two halves together trace exactly the curve it had, straight or curved,
    /// and the segments after it move up by one.
    #[test]
    fn a_split_keeps_the_shape() {
        let points = [Vec2::ZERO, Vec2::new(4.0, 0.0), Vec2::new(4.0, 5.0)];
        let control = Vec2::new(7.0, 2.0);
        let before = wall(&points, &[None, Some(control)]);

        let straight = split_wall(&before, 0, 0.25).expect("a straight split");
        assert_eq!(straight.points[1], Vec2::new(1.0, 0.0));
        assert_eq!(straight.segments.len(), 3);
        assert_eq!(straight.segments[0].control, None);
        assert_eq!(straight.segments[1].control, None);
        assert_eq!(straight.segments[2].control, Some(control));

        let t = 0.3;
        let curved = split_wall(&before, 1, t).expect("a curved split");
        assert_eq!(curved.points.len(), 4);
        let original = QuadBez::new(point(points[1]), point(control), point(points[2]));
        let halves = [1, 2].map(|segment| {
            QuadBez::new(
                point(curved.points[segment]),
                point(curved.segments[segment].control.expect("both halves curve")),
                point(curved.points[segment + 1]),
            )
        });
        for step in 0..=100 {
            let u = f64::from(step) / 100.0;
            let traced = if u <= f64::from(t) {
                halves[0].eval(u / f64::from(t))
            } else {
                halves[1].eval((u - f64::from(t)) / (1.0 - f64::from(t)))
            };
            let expected = original.eval(u);
            assert!(
                length(traced - expected) < 1e-5,
                "{traced:?} against {expected:?} at {u}"
            );
        }

        assert_eq!(
            split_wall(&before, 2, 0.5),
            Err(ShapeError::NoSegment {
                segment: 2,
                segments: 2
            })
        );
        assert_eq!(
            split_wall(&before, 0, 1.0),
            Err(ShapeError::OutsideSegment(1.0))
        );
    }
}
