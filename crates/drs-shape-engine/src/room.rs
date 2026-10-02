//! `CombineOutlines`, `GenerateWalls`, and `SplitWall` for Rooms: the closed line of a Room's
//! outline, the floor it winds around, the stroke of its Walls, and the exact split of an edge.
//!
//! A Room's outline is read as an open path of segments from its first point round to its first
//! point again, so it is flattened, measured, and split exactly as a drawn Wall's line is, its
//! edges numbered as the path's segments. The floor is filled by `lyon_tessellation` over the
//! same chords the Walls are stroked along, under the non-zero rule; the tessellator is given
//! straight chords only, which it fills with arithmetic and comparisons, so the floor too comes
//! out the same on every machine.

use crate::wall::{Measured, ShapeError, TOLERANCE, flatten, split_wall, stroke_closed};
use bevy_math::Vec2;
use drs_model::{Edge, FillMesh, LinePoint, Room, RoomShape, Segment, Stretch, Wall, WallShape};
use lyon_tessellation::geometry_builder::{BuffersBuilder, VertexBuffers};
use lyon_tessellation::math::point;
use lyon_tessellation::path::Path;
use lyon_tessellation::{FillOptions, FillTessellator, FillVertex};

/// What `CombineOutlines` derives from a Room's outline: the closed line and the floor.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Outline {
    /// The outline flattened into a closed line, from the first point round to the first point
    /// again, each point tagged with the edge it lies on as its segment and the parameter along
    /// that edge.
    pub line: Vec<LinePoint>,
    /// What the line winds around, filled up to the line under the non-zero rule.
    pub floor: FillMesh,
}

/// A Room's outline as an open path of segments from its first point round to its first point
/// again: segment `k` is edge `k`.
pub(crate) fn path_of(room: &Room) -> Wall {
    let mut points = room.points.clone();
    if let Some(first) = room.points.first() {
        points.push(*first);
    }
    Wall {
        points,
        segments: room
            .edges
            .iter()
            .map(|edge| Segment {
                control: edge.control,
            })
            .collect(),
        thickness: room.thickness,
        colour: room.wall_colour,
    }
}

/// The closed line of a Room's outline, its last point exactly its first.
pub(crate) fn closed_line(room: &Room) -> Vec<LinePoint> {
    let mut line = flatten(&path_of(room));
    if let (Some(last), Some(first)) = (line.last_mut(), room.points.first()) {
        last.position = *first;
    }
    line
}

/// `CombineOutlines` for one Room: its outline flattened into a closed line, within a thousandth
/// of a cell of every curved edge as a Wall's line is, and the floor filling everything the line
/// winds around, every part of an outline whose edges cross included.
///
/// An outline the tessellator cannot fill, which takes coordinates far beyond any Level, has no
/// floor rather than a wrong one.
#[must_use]
pub fn combine_outlines(room: &Room) -> Outline {
    let line = closed_line(room);
    let floor = fill(&line);
    Outline { line, floor }
}

/// `GenerateWalls` for a Room: its closed line and floor, and the stroke of its Walls centred on
/// the line at its thickness, with a round join at every point, the first included, and no caps,
/// left out along `stretches`. A stretch whose start lies after its end runs on past the first
/// point; the stroke ends squarely across the line at each end of a stretch. The stretches are
/// kept on the shape as given.
#[must_use]
pub fn generate_room_walls(room: &Room, stretches: &[Stretch]) -> RoomShape {
    let Outline { line, floor } = combine_outlines(room);
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
    let mesh = stroke_closed(&line, f64::from(room.thickness) / 2.0, &gaps);
    RoomShape {
        walls: WallShape {
            line,
            stretches: stretches.to_vec(),
            mesh,
        },
        floor,
    }
}

/// `SplitWall` for a Room: the Room with a point added on `edge` at parameter `t`, splitting the
/// edge into two whose joined curve is the one it had, to single precision. The new point comes
/// after the edge's first point, so a point added on the last edge becomes the last point, and
/// the edges after it are numbered one higher.
///
/// # Errors
///
/// [`ShapeError::NoEdge`] when the Room has no such edge, or [`ShapeError::OutsideEdge`] when
/// `t` is not strictly between zero and one.
pub fn split_room(room: &Room, edge: usize, t: f32) -> Result<Room, ShapeError> {
    if edge >= room.edges.len() || edge >= room.points.len() {
        return Err(ShapeError::NoEdge {
            edge,
            edges: room.edges.len(),
        });
    }
    if !(t > 0.0 && t < 1.0) {
        return Err(ShapeError::OutsideEdge(t));
    }
    let split = split_wall(&path_of(room), edge, t)?;
    let mut points = split.points;
    points.pop();
    Ok(Room {
        points,
        edges: split
            .segments
            .iter()
            .map(|segment| Edge {
                control: segment.control,
            })
            .collect(),
        ..room.clone()
    })
}

/// The triangles filling what a closed line winds around, under the non-zero rule.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the tessellator works in single precision, as the model does"
)]
fn fill(line: &[LinePoint]) -> FillMesh {
    // The closing point is the first again, which closing the path adds.
    let Some((first, rest)) = line.split_last().and_then(|(_, open)| open.split_first()) else {
        return FillMesh::default();
    };
    let mut path = Path::builder();
    path.begin(point(first.position.x, first.position.y));
    for line_point in rest {
        path.line_to(point(line_point.position.x, line_point.position.y));
    }
    path.end(true);
    let path = path.build();
    let mut buffers: VertexBuffers<Vec2, u32> = VertexBuffers::new();
    let options = FillOptions::non_zero().with_tolerance(TOLERANCE as f32);
    let filled = FillTessellator::new().tessellate_path(
        &path,
        &options,
        &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
            let at = vertex.position();
            Vec2::new(at.x, at.y)
        }),
    );
    match filled {
        Ok(()) => FillMesh {
            vertices: buffers.vertices,
            indices: buffers.indices,
        },
        Err(_) => FillMesh::default(),
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        reason = "a test stops at the first thing that is not as expected"
    )]

    use super::*;
    use drs_model::{Colour, StrokeMesh};

    /// A Room a quarter of a cell thick through `points`, every edge straight.
    fn room(points: &[Vec2]) -> Room {
        Room::straight(
            points.to_vec(),
            0.25,
            Colour::rgb(60, 60, 60),
            Colour::rgb(200, 200, 200),
        )
    }

    /// Whether `p` lies in a triangle of `vertices` and `indices`, edges included.
    fn inside(p: Vec2, vertices: &[Vec2], indices: &[u32]) -> bool {
        indices.chunks(3).any(|corners| {
            let [a, b, c] = [0, 1, 2].map(|i| vertices[corners[i] as usize]);
            let side = |u: Vec2, v: Vec2| (v - u).perp_dot(p - u);
            let (ab, bc, ca) = (side(a, b), side(b, c), side(c, a));
            (ab >= 0.0 && bc >= 0.0 && ca >= 0.0) || (ab <= 0.0 && bc <= 0.0 && ca <= 0.0)
        })
    }

    /// Whether `p` lies in the stroke.
    fn stroked(p: Vec2, mesh: &StrokeMesh) -> bool {
        inside(p, &mesh.vertices, &mesh.indices)
    }

    /// The floor covers both lobes of an outline whose edges cross, and nothing outside it.
    #[test]
    fn the_floor_fills_a_crossed_outline() {
        // A bow tie: the edges from the second point to the third and from the fourth back to
        // the first cross in the middle.
        let bow = room(&[
            Vec2::ZERO,
            Vec2::new(4.0, 0.0),
            Vec2::new(0.0, 4.0),
            Vec2::new(4.0, 4.0),
        ]);
        let floor = combine_outlines(&bow).floor;

        assert!(inside(Vec2::new(2.0, 0.5), &floor.vertices, &floor.indices));
        assert!(inside(Vec2::new(2.0, 3.5), &floor.vertices, &floor.indices));
        assert!(!inside(
            Vec2::new(0.5, 2.0),
            &floor.vertices,
            &floor.indices
        ));
        assert!(!inside(
            Vec2::new(5.0, 2.0),
            &floor.vertices,
            &floor.indices
        ));
    }

    /// The stroke of a closed outline joins its last edge to its first round the first point, as
    /// at every other point, and ends squarely, with no cap, where a stretch leaves it out.
    #[test]
    fn a_closed_stroke_joins_its_first_point() {
        let square = room(&[
            Vec2::ZERO,
            Vec2::new(4.0, 0.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(0.0, 4.0),
        ]);
        let shape = generate_room_walls(&square, &[]);
        let mesh = &shape.walls.mesh;
        let half = 0.125;
        let corner = half * std::f32::consts::FRAC_1_SQRT_2;

        for (point, outward) in [
            (Vec2::ZERO, Vec2::new(-1.0, -1.0)),
            (Vec2::new(4.0, 0.0), Vec2::new(1.0, -1.0)),
            (Vec2::new(4.0, 4.0), Vec2::new(1.0, 1.0)),
        ] {
            let near = point + outward * (corner - 0.01);
            assert!(stroked(near, mesh), "the join at {point} reaches {near}");
            let past = point + outward * (corner + 0.01);
            assert!(!stroked(past, mesh), "nothing past the join at {point}");
        }
        assert!(
            stroked(Vec2::new(-(half - 0.01), 2.0), mesh),
            "the last edge"
        );

        let gap = Stretch {
            start: drs_model::LinePlace {
                segment: 0,
                t: 0.25,
            },
            end: drs_model::LinePlace {
                segment: 0,
                t: 0.75,
            },
        };
        let mesh = generate_room_walls(&square, &[gap]).walls.mesh;
        assert!(!stroked(Vec2::new(2.0, 0.0), &mesh), "the gap");
        assert!(stroked(Vec2::new(0.99, half - 0.01), &mesh), "a square end");
        assert!(
            !stroked(Vec2::new(1.05, 0.0), &mesh),
            "no cap reaching into the gap"
        );
        assert!(
            stroked(Vec2::new(3.01, -(half - 0.01)), &mesh),
            "a square end"
        );
        assert!(
            !stroked(Vec2::new(2.95, 0.0), &mesh),
            "no cap reaching into the gap"
        );
    }
}
