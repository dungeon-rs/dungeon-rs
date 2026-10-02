//! `GenerateWalls` and `SplitWall` for drawn Walls: the flattened line, the stroke mesh, and the
//! exact split of a segment.
//!
//! Everything is computed in double precision and handed to the model in single precision. No
//! trigonometric function is called anywhere: the flattening counts come from square roots, and
//! a round join or cap is an arc subdivided by bisecting unit vectors, whose angle halves at each
//! step, so the only operation beyond arithmetic is the correctly rounded square root and the
//! mesh is the same on every machine.

use bevy_math::Vec2;
use drs_model::{LinePlace, LinePoint, Segment, Stretch, StrokeMesh, Wall, WallShape};
use kurbo::{Line, ParamCurve, Point, QuadBez};

/// How far, in Grid cells, a chord of the flattened line or an arc of the stroke may stray from
/// the curve it stands for: a pixel at the highest Export resolution.
///
/// A curved segment so strongly bent that it would take more than [`MOST_CHORDS`] chords is
/// flattened into that many, and its chords then stray farther; that takes a control point more
/// than thirty thousand cells from the middle of its segment.
pub(crate) const TOLERANCE: f64 = 0.001;

/// The fewest chords a curved segment is flattened into, so that its middle is always a point of
/// the line.
const FEWEST_CHORDS: f64 = 2.0;

/// The most chords one curved segment is flattened into, whatever its size; past it the
/// [`TOLERANCE`] no longer holds.
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
    /// The Room has no edge of that number.
    #[error("the Room has no edge {edge}; it has {edges}")]
    NoEdge {
        /// The edge asked for.
        edge: usize,
        /// How many edges the Room has.
        edges: usize,
    },
    /// The parameter does not lie strictly between the edge's two points.
    #[error("an edge is split strictly between its points, not at {0}")]
    OutsideEdge(f32),
}

/// `GenerateWalls`: the shape a drawn Wall is drawn and picked by, left out along `stretches`.
///
/// The line runs from the first point to the last. A straight segment is one chord; a curved
/// one is cut at evenly spaced parameters, an even number of them so that its middle is a point
/// of the line, as many as keep every chord within a thousandth of a cell of the curve. Each
/// point is tagged with the segment it lies on and the parameter along it; a point shared by two
/// segments belongs to the later one, at parameter zero, and the last point to the last segment
/// at one. The stroke covers everything within half the thickness of the line, with round joins
/// at its points and round caps at its ends, and records the arc length of the line at each
/// vertex.
///
/// The stroke leaves out every stretch, measured along the flattened line: it ends squarely
/// across the line at each end of a stretch, and an end of the Wall that a stretch reaches has
/// no cap. Overlapping stretches leave out what either covers. The stretches are kept on the
/// shape as given.
#[must_use]
pub fn generate_walls(wall: &Wall, stretches: &[Stretch]) -> WallShape {
    let line = flatten(wall);
    let measured = Measured::of(&line);
    let gaps: Vec<(f64, f64)> = stretches
        .iter()
        .map(|stretch| {
            (
                measured.length_at(stretch.start),
                measured.length_at(stretch.end),
            )
        })
        .collect();
    let mesh = stroke(&line, f64::from(wall.thickness) / 2.0, &gaps);
    WallShape {
        line,
        stretches: stretches.to_vec(),
        mesh,
    }
}

/// A flattened line with the arc length at each of its points, for measuring along it.
pub(crate) struct Measured<'a> {
    /// The line.
    line: &'a [LinePoint],
    /// The arc length at each point of the line, from its start, summing its chords.
    lengths: Vec<f64>,
}

impl<'a> Measured<'a> {
    /// Measures `line`.
    pub(crate) fn of(line: &'a [LinePoint]) -> Self {
        let mut lengths = Vec::with_capacity(line.len());
        let mut travelled = 0.0;
        let mut previous: Option<Point> = None;
        for line_point in line {
            let at = point(line_point.position);
            if let Some(previous) = previous {
                travelled += length(at - previous);
            }
            lengths.push(travelled);
            previous = Some(at);
        }
        Self { line, lengths }
    }

    /// The length of the whole line.
    pub(crate) fn total(&self) -> f64 {
        self.lengths.last().copied().unwrap_or(0.0)
    }

    /// The chords of the line: for each, the index of its first point, the segment it lies on,
    /// and the parameters its two ends have along that segment.
    fn chords(&self) -> impl Iterator<Item = (usize, usize, f32, f32)> + '_ {
        self.line.windows(2).enumerate().map(|(index, pair)| {
            let (from, to) = (pair[0], pair[1]);
            let end = if to.segment == from.segment {
                to.t
            } else {
                1.0
            };
            (index, from.segment, from.t, end)
        })
    }

    /// The arc length at a place of the line, interpolated along the chord the place lies on; a
    /// place past the line's segments is at its end.
    pub(crate) fn length_at(&self, place: LinePlace) -> f64 {
        let found = self.chords().find(|(_, segment, from, to)| {
            *segment == place.segment && *from <= place.t && place.t <= *to
        });
        let Some((index, _, from, to)) = found else {
            let before = self
                .line
                .first()
                .is_some_and(|first| place.segment < first.segment);
            return if before { 0.0 } else { self.total() };
        };
        let span = f64::from(to) - f64::from(from);
        let share = if span > 0.0 {
            (f64::from(place.t) - f64::from(from)) / span
        } else {
            0.0
        };
        self.lengths[index] + share * (self.lengths[index + 1] - self.lengths[index])
    }

    /// The place at an arc length, clamped to the line, its parameter interpolated along the
    /// chord the length falls in.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the model keeps parameters in single precision"
    )]
    pub(crate) fn place_at(&self, along: f64) -> LinePlace {
        let along = along.clamp(0.0, self.total());
        let found = self
            .chords()
            .find(|(index, ..)| along <= self.lengths[index + 1]);
        let Some((index, segment, from, to)) = found else {
            return self
                .line
                .last()
                .map_or(LinePlace { segment: 0, t: 0.0 }, |last| LinePlace {
                    segment: last.segment,
                    t: last.t,
                });
        };
        let span = self.lengths[index + 1] - self.lengths[index];
        let share = if span > 0.0 {
            (along - self.lengths[index]) / span
        } else {
            0.0
        };
        LinePlace {
            segment,
            t: (f64::from(from) + share * (f64::from(to) - f64::from(from))) as f32,
        }
    }
}

/// `SplitWall`: the Wall with a point added on `segment` at parameter `t`, splitting the segment
/// into two whose joined curve is the one it had, to single precision.
///
/// A straight segment becomes two straight segments; a curved one becomes the two pieces of its
/// curve on either side of `t`, each with its own control point. The new point sits after the segment's first point,
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
pub(crate) fn point(position: Vec2) -> Point {
    Point::new(f64::from(position.x), f64::from(position.y))
}

/// A kurbo point as a model position.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the model keeps positions in single precision"
)]
pub(crate) fn vector(point: Point) -> Vec2 {
    Vec2::new(point.x as f32, point.y as f32)
}

/// The length of a vector, through the correctly rounded square root only.
pub(crate) fn length(vector: kurbo::Vec2) -> f64 {
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
        .clamp(FEWEST_CHORDS, MOST_CHORDS) as usize;
    chords + chords % 2
}

/// The Wall's line flattened into points tagged with their segment and parameter.
#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    reason = "chord counts are far below where f64 loses whole numbers; parameters are kept in \
              single precision"
)]
pub(crate) fn flatten(wall: &Wall) -> Vec<LinePoint> {
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

/// The stroke of a flattened line at half-thickness `radius`, left out along `gaps`, each a
/// range of arc length: a band per chord, a round join on the outer side of each bend, a round
/// cap at each end of the line that no gap reaches, and a square end at each end of a gap.
fn stroke(line: &[LinePoint], radius: f64, gaps: &[(f64, f64)]) -> StrokeMesh {
    let (points, travelled) = distinct(line);
    let mut builder = Builder::default();
    let Some(&first) = points.first() else {
        return builder.mesh;
    };
    if gaps.is_empty() && points.len() == 1 {
        // A Wall whose points all coincide is drawn as a dot of its thickness.
        dot(&mut builder, first, radius);
        return builder.mesh;
    }
    for (start, end) in runs(gaps, travelled) {
        if end - start <= COINCIDENT {
            continue;
        }
        let run = cut(&points, start, end);
        stroke_run(&mut builder, &run, radius, start <= 0.0, end >= travelled);
    }
    builder.mesh
}

/// The points of a flattened line with their arc lengths, coincident points merged, and the
/// length of the whole line.
fn distinct(line: &[LinePoint]) -> (Vec<(Point, f64)>, f64) {
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
    (points, travelled)
}

/// Adds a dot of half-thickness `radius` at `centre`: what a line whose points all coincide is
/// drawn as.
fn dot(builder: &mut Builder, centre: (Point, f64), radius: f64) {
    let east = kurbo::Vec2::new(1.0, 0.0);
    let north = kurbo::Vec2::new(0.0, 1.0);
    builder.fan(centre, radius, &half_turn(north, -east, radius));
    builder.fan(centre, radius, &half_turn(-north, east, radius));
}

/// The stroke of a closed flattened line, whose last point is its first again, at half-thickness
/// `radius`, left out along `gaps`, each a range of arc length from where it starts to where it
/// ends, a gap that starts after it ends running on past the first point: a band per chord and a
/// round join on the outer side of every bend, the first point's included, with no caps; the
/// stroke ends squarely across the line at each end of a gap.
pub(crate) fn stroke_closed(line: &[LinePoint], radius: f64, gaps: &[(f64, f64)]) -> StrokeMesh {
    let (points, total) = distinct(line);
    let mut builder = Builder::default();
    // The ring of distinct points: the closing point is the first again, so it is left out.
    let mut ring = points.clone();
    if ring.len() > 1
        && let (Some(&(first, _)), Some(&(last, _))) = (ring.first(), ring.last())
        && length(last - first) <= COINCIDENT
    {
        ring.pop();
    }
    if ring.len() < 2 {
        if let (Some(&first), true) = (ring.first(), gaps.is_empty()) {
            dot(&mut builder, first, radius);
        }
        return builder.mesh;
    }
    let runs = ring_runs(gaps, total);
    if gaps.is_empty() || runs == [(0.0, total)] {
        stroke_ring(&mut builder, &ring, total, radius);
        return builder.mesh;
    }
    for (start, end) in runs {
        if end - start <= COINCIDENT {
            continue;
        }
        let run = if end <= total {
            cut(&points, start, end)
        } else {
            // The run goes on past the first point, where the arc length starts again at zero.
            let mut run = cut(&points, start, total);
            run.extend(cut(&points, 0.0, end - total).into_iter().skip(1));
            run
        };
        stroke_run(&mut builder, &run, radius, false, false);
    }
    builder.mesh
}

/// The ranges of arc length round a closed line of length `total` that no gap covers, each from
/// where it starts to where it ends; a range that runs on past the first point ends beyond
/// `total`, by the length it runs on.
fn ring_runs(gaps: &[(f64, f64)], total: f64) -> Vec<(f64, f64)> {
    let mut covered = Vec::with_capacity(gaps.len() + 1);
    for &(from, to) in gaps {
        if from <= to {
            covered.push((from, to));
        } else {
            // A gap that starts after it ends covers the end of the line and its start.
            covered.push((from, total));
            covered.push((0.0, to));
        }
    }
    let mut runs = runs(&covered, total);
    if runs.len() >= 2
        && runs.first().is_some_and(|first| first.0 <= 0.0)
        && runs.last().is_some_and(|last| last.1 >= total)
    {
        // The run at the end and the run at the start meet at the first point: they are one.
        let first = runs.remove(0);
        if let Some(last) = runs.last_mut() {
            last.1 = total + first.1;
        }
    }
    runs
}

/// Adds the stroke of a whole closed ring of two or more distinct points, `total` long: a band
/// per chord, the last back to the first, and a round join on the outer side of every bend, the
/// first point's included.
fn stroke_ring(builder: &mut Builder, ring: &[(Point, f64)], total: f64, radius: f64) {
    let count = ring.len();
    let next = |index: usize| (index + 1) % count;
    let directions: Vec<kurbo::Vec2> = (0..count)
        .map(|index| {
            let along = ring[next(index)].0 - ring[index].0;
            along / length(along)
        })
        .collect();
    for (index, direction) in directions.iter().enumerate() {
        let end = if next(index) == 0 {
            (ring[0].0, total)
        } else {
            ring[next(index)]
        };
        builder.band(ring[index], end, left(*direction) * radius);
    }
    for index in 0..count {
        let before = directions[(index + count - 1) % count];
        join(builder, ring[index], before, directions[index], radius);
    }
}

/// The unit vector a quarter turn to the left of `direction`.
fn left(direction: kurbo::Vec2) -> kurbo::Vec2 {
    kurbo::Vec2::new(-direction.y, direction.x)
}

/// Adds the round join at `centre` on the outer side of the bend from `before` to `after`, unit
/// directions of the chords meeting there.
fn join(
    builder: &mut Builder,
    centre: (Point, f64),
    before: kurbo::Vec2,
    after: kurbo::Vec2,
    radius: f64,
) {
    let bend = before.cross(after);
    let (from, to) = if bend > 0.0 {
        (-left(before), -left(after))
    } else {
        (left(before), left(after))
    };
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

/// The ranges of arc length from zero to `total` that no gap covers, in order.
fn runs(gaps: &[(f64, f64)], total: f64) -> Vec<(f64, f64)> {
    let mut covered: Vec<(f64, f64)> = gaps
        .iter()
        .map(|&(from, to)| (from.min(to).max(0.0), from.max(to).min(total)))
        .collect();
    covered.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut runs = Vec::new();
    let mut start = 0.0;
    for (from, to) in covered {
        if from > start {
            runs.push((start, from));
        }
        start = f64::max(start, to);
    }
    if start < total {
        runs.push((start, total));
    }
    runs
}

/// The points of the line between two arc lengths, the ends interpolated along their chords.
fn cut(points: &[(Point, f64)], start: f64, end: f64) -> Vec<(Point, f64)> {
    let at = |along: f64| -> (Point, f64) {
        let index = points
            .windows(2)
            .position(|pair| along <= pair[1].1)
            .unwrap_or(points.len().saturating_sub(2));
        let (from, to) = (points[index], points[(index + 1).min(points.len() - 1)]);
        let span = to.1 - from.1;
        let share = if span > 0.0 {
            ((along - from.1) / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (from.0 + (to.0 - from.0) * share, along)
    };
    let mut run = vec![at(start)];
    run.extend(
        points
            .iter()
            .copied()
            .filter(|&(_, along)| along > start + COINCIDENT && along < end - COINCIDENT),
    );
    run.push(at(end));
    run
}

/// Adds the stroke of one run of two or more distinct points: a band per chord, a round join
/// on the outer side of each bend, and a round cap at the start or the end where asked; an end
/// without a cap is square.
fn stroke_run(
    builder: &mut Builder,
    points: &[(Point, f64)],
    radius: f64,
    cap_start: bool,
    cap_end: bool,
) {
    let directions: Vec<kurbo::Vec2> = points
        .windows(2)
        .map(|pair| {
            let along = pair[1].0 - pair[0].0;
            along / length(along)
        })
        .collect();

    for (pair, direction) in points.windows(2).zip(&directions) {
        builder.band(pair[0], pair[1], left(*direction) * radius);
    }
    for (index, turn) in directions.windows(2).enumerate() {
        join(builder, points[index + 1], turn[0], turn[1], radius);
    }
    if let (Some(&first), Some(&start)) = (points.first(), directions.first())
        && cap_start
    {
        builder.fan(first, radius, &half_turn(left(start), -start, radius));
    }
    if let (Some(&last), Some(&end)) = (points.last(), directions.last())
        && cap_end
    {
        builder.fan(last, radius, &half_turn(-left(end), end, radius));
    }
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
        let shape = generate_walls(&wall(&points, &[None, None]), &[]);

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
        let shape = generate_walls(&wall(&[start, end], &[Some(control)]), &[]);
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
        let shape = generate_walls(
            &wall(
                &[
                    Vec2::ZERO,
                    Vec2::new(3.0, 0.0),
                    Vec2::new(3.0, 3.0),
                    Vec2::new(0.5, 1.0),
                ],
                &[None, Some(Vec2::new(5.0, 1.5)), None],
            ),
            &[],
        );
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

    /// A stretch leaves its part of the line out of the stroke, which ends squarely across the
    /// line at either side of it and keeps its round caps at the Wall's own ends; a stretch that
    /// reaches an end leaves that end without a cap.
    #[test]
    fn a_stretch_ends_the_stroke_squarely() {
        let straight = wall(&[Vec2::ZERO, Vec2::new(10.0, 0.0)], &[None]);
        let half = 0.125;
        let gap = Stretch {
            start: LinePlace { segment: 0, t: 0.4 },
            end: LinePlace { segment: 0, t: 0.6 },
        };
        let shape = generate_walls(&straight, &[gap]);
        assert_eq!(shape.stretches, vec![gap]);
        let mesh = &shape.mesh;

        assert!(
            covered(Vec2::new(2.0, 0.0), mesh),
            "the stroke before the gap"
        );
        assert!(
            covered(Vec2::new(8.0, 0.0), mesh),
            "the stroke after the gap"
        );
        assert!(!covered(Vec2::new(5.0, 0.0), mesh), "the gap");
        assert!(
            covered(Vec2::new(3.99, half - 0.01), mesh),
            "the square corner before the gap"
        );
        assert!(
            !covered(Vec2::new(4.05, 0.0), mesh),
            "no cap reaching into the gap"
        );
        assert!(
            covered(Vec2::new(6.01, -(half - 0.01)), mesh),
            "the square corner after the gap"
        );
        assert!(
            covered(Vec2::new(-0.1, 0.0), mesh),
            "the round cap at the start"
        );

        let to_the_end = Stretch {
            start: LinePlace { segment: 0, t: 0.9 },
            end: LinePlace { segment: 0, t: 1.0 },
        };
        let shape = generate_walls(&straight, &[gap, to_the_end]);
        assert!(
            !covered(Vec2::new(10.05, 0.0), &shape.mesh),
            "no cap at the end"
        );
        assert!(covered(Vec2::new(8.99, half - 0.01), &shape.mesh));
    }
}
