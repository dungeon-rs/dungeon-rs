//! `CombineOutlines`: an outline flattened into its line, and the floor a closed one winds
//! around.
//!
//! The line is flattened as every operation of the Engine flattens it, so the floor's edge, the
//! Walls' centre line, and the lengths Portals are measured by are the same chords. The floor is
//! filled by `lyon_tessellation` over those chords under the non-zero rule; the tessellator is
//! given straight chords only, which it fills with arithmetic and comparisons, so the floor too
//! comes out the same on every machine.

use crate::path::{Path, TOLERANCE, flatten};
use bevy_math::Vec2;
use drs_model::{FillMesh, LinePoint};
use lyon_tessellation::geometry_builder::{BuffersBuilder, VertexBuffers};
use lyon_tessellation::math::point;
use lyon_tessellation::{FillOptions, FillTessellator, FillVertex};

/// What `CombineOutlines` derives from an outline: its line and, for a closed one, its floor.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CombinedOutline {
    /// The outline flattened into a line, from the first point to the last, or round to the
    /// first again on a closed outline, each point tagged with the part it lies on as its
    /// segment and the parameter along that part.
    pub line: Vec<LinePoint>,
    /// Whether the line closes from its last point back to its first.
    pub closed: bool,
    /// What a closed line winds around, filled up to the line under the non-zero rule; an open
    /// line encloses nothing.
    pub floor: FillMesh,
}

/// `CombineOutlines` for one outline: its line, within a thousandth of a cell of every curved
/// part, and, when it is closed, the floor filling everything the line winds around, every part
/// of an outline whose edges cross included. It is given one outline at a time and combines it
/// with none.
///
/// An outline the tessellator cannot fill, which takes coordinates far beyond any Level, has no
/// floor rather than a wrong one.
#[must_use]
pub fn combine_outlines(path: &Path) -> CombinedOutline {
    let line = flatten(path);
    let floor = if path.closed {
        fill(&line)
    } else {
        FillMesh::default()
    };
    CombinedOutline {
        line,
        closed: path.closed,
        floor,
    }
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
    let mut path = lyon_tessellation::path::Path::builder();
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
    use crate::generate_walls;
    use drs_model::{Stretch, StrokeMesh, WallShape};

    /// A Room's closed outline through `points`, every edge straight.
    fn room(points: &[Vec2]) -> Path {
        Path {
            points: points.to_vec(),
            controls: vec![None; points.len()],
            closed: true,
        }
    }

    /// The shape of the Walls round `path`, a quarter of a cell thick, left out along `stretches`.
    fn walls(path: &Path, stretches: &[Stretch]) -> WallShape {
        generate_walls(&combine_outlines(path), 0.25, stretches)
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
        let shape = walls(&square, &[]);
        let mesh = &shape.mesh;
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
        let mesh = walls(&square, &[gap]).mesh;
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
