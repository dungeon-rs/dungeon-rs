//! A Room's floor: what its closed line, or the contours left of it once later Rooms cut it,
//! winds around, filled by `lyon_tessellation` under the non-zero rule.
//!
//! The tessellator is given straight chords only, which it fills with arithmetic and
//! comparisons, so the floor comes out the same on every machine.

use crate::path::TOLERANCE;
use bevy_math::Vec2;
use drs_model::{FillMesh, LinePoint};
use lyon_tessellation::geometry_builder::{BuffersBuilder, VertexBuffers};
use lyon_tessellation::math::point;
use lyon_tessellation::{FillOptions, FillTessellator, FillVertex};

/// The triangles filling what a closed line, its last point its first again, winds around,
/// under the non-zero rule. A line the tessellator cannot fill, which takes coordinates far
/// beyond any Level, has no floor rather than a wrong one.
pub(crate) fn fill(line: &[LinePoint]) -> FillMesh {
    // The closing point is the first again, which closing the contour adds.
    let Some((_, open)) = line.split_last() else {
        return FillMesh::default();
    };
    let contour: Vec<Vec2> = open.iter().map(|point| point.position).collect();
    fill_contours(&[contour])
}

/// The triangles filling what `contours`, each closed from its last point back to its first,
/// wind around together, under the non-zero rule; a contour of fewer than three points adds
/// nothing.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the tessellator works in single precision, as the model does"
)]
pub(crate) fn fill_contours(contours: &[Vec<Vec2>]) -> FillMesh {
    let mut path = lyon_tessellation::path::Path::builder();
    let mut any = false;
    for contour in contours {
        let Some((first, rest)) = contour.split_first() else {
            continue;
        };
        if rest.len() < 2 {
            continue;
        }
        path.begin(point(first.x, first.y));
        for at in rest {
            path.line_to(point(at.x, at.y));
        }
        path.end(true);
        any = true;
    }
    if !any {
        return FillMesh::default();
    }
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

    use crate::{Outline, Path, combine_outlines, generate_walls};
    use bevy_math::Vec2;
    use drs_model::{LinePlace, Stretch, StrokeMesh};

    /// A Room's closed outline through `points`, every edge straight.
    fn room(points: &[Vec2]) -> Path {
        Path {
            points: points.to_vec(),
            controls: vec![None; points.len()],
            closed: true,
        }
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

    /// The stroke of the Walls round `path` alone, a quarter of a cell thick, left out along
    /// `stretches`.
    fn walls(path: &Path, stretches: &[Stretch]) -> StrokeMesh {
        let combined = combine_outlines(&[Outline {
            path: path.clone(),
            cuts: false,
        }]);
        generate_walls(&combined.outlines[0], 0.25, stretches)
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
        let combined = combine_outlines(&[Outline {
            path: bow,
            cuts: false,
        }]);
        let floor = &combined.outlines[0].floor;

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
        let mesh = walls(&square, &[]);
        let half = 0.125;
        let corner = half * std::f32::consts::FRAC_1_SQRT_2;

        for (point, outward) in [
            (Vec2::ZERO, Vec2::new(-1.0, -1.0)),
            (Vec2::new(4.0, 0.0), Vec2::new(1.0, -1.0)),
            (Vec2::new(4.0, 4.0), Vec2::new(1.0, 1.0)),
        ] {
            let near = point + outward * (corner - 0.01);
            assert!(stroked(near, &mesh), "the join at {point} reaches {near}");
            let past = point + outward * (corner + 0.01);
            assert!(!stroked(past, &mesh), "nothing past the join at {point}");
        }
        assert!(
            stroked(Vec2::new(-(half - 0.01), 2.0), &mesh),
            "the last edge"
        );

        let gap = Stretch {
            start: LinePlace {
                segment: 0,
                t: 0.25,
            },
            end: LinePlace {
                segment: 0,
                t: 0.75,
            },
        };
        let mesh = walls(&square, &[gap]);
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
