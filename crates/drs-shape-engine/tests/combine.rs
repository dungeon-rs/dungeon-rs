//! Combining a Layer of Rooms as a drag does at every frame: the floors, the Walls, and the
//! Portals' places of twenty Rooms with curved edges, shared Walls, and doors.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    reason = "a test stops at the first thing that is not as expected"
)]

use bevy_math::Vec2;
use drs_model::{Stretch, StrokeMesh};
use drs_shape_engine::{
    Combination, Outline, Path, PortalSetting, Standing, anchor_portals, combine_outlines,
    generate_walls,
};
use std::time::{Duration, Instant};

/// How many Rooms a row of the Layer holds.
const ACROSS: usize = 5;
/// How many rows of Rooms the Layer holds.
const ROWS: usize = 4;
/// How wide and tall each Room is, in cells.
const SIDE: f32 = 10.0;
/// How far apart the rows start, in cells: a cell more than a Room is tall, so the rows' curved
/// edges, bulging two cells, overlap.
const PITCH: f32 = 11.0;
/// How thick every Room's Walls are drawn.
const THICKNESS: f32 = 0.25;

/// The fastest of `runs` runs of `work`, and what the last run gave: the least disturbed run
/// is the one that says what the code costs.
fn fastest<T>(runs: usize, mut work: impl FnMut() -> T) -> (Duration, T) {
    let mut best = Duration::MAX;
    let mut last = None;
    for _ in 0..runs {
        let start = Instant::now();
        let outcome = work();
        best = best.min(start.elapsed());
        last = Some(outcome);
    }
    (best, last.expect("at least one run"))
}

/// The Rooms of the Layer in stacking order, a row at a time: each a square whose bottom and top
/// edges bulge two cells outwards as curves, its straight sides lying on its neighbours' in the
/// row, so every pair of neighbours shares an edge, and its curves overlapping those of the rows
/// above and below.
#[expect(clippy::cast_precision_loss, reason = "a handful of rows and columns")]
fn layer() -> Vec<Outline> {
    (0..ACROSS * ROWS)
        .map(|index| {
            let x = (index % ACROSS) as f32 * SIDE;
            let y = (index / ACROSS) as f32 * PITCH;
            Outline {
                path: Path {
                    points: vec![
                        Vec2::new(x, y),
                        Vec2::new(x + SIDE, y),
                        Vec2::new(x + SIDE, y + SIDE),
                        Vec2::new(x, y + SIDE),
                    ],
                    controls: vec![
                        Some(Vec2::new(x + SIDE / 2.0, y - 4.0)),
                        None,
                        Some(Vec2::new(x + SIDE / 2.0, y + SIDE + 4.0)),
                        None,
                    ],
                    closed: true,
                },
                cuts: false,
            }
        })
        .collect()
}

/// Ten doors: one in the middle of the right edge of each Room of the first row, four in the
/// Walls they share with their neighbours and one in an outer Wall, and one at the middle of the
/// bottom curve of each, in outer Walls.
fn doors() -> Vec<PortalSetting> {
    (0..ACROSS)
        .flat_map(|outline| {
            [1, 0].map(|segment| PortalSetting {
                outline,
                segment,
                t: 0.5,
                width: 1.0,
            })
        })
        .collect()
}

/// The floors, the Walls left out where the doors stand, and the doors' places, derived as the
/// authoring Manager derives a Layer.
fn derive(
    outlines: &[Outline],
    portals: &[PortalSetting],
) -> (Combination, Vec<Option<Standing>>, Vec<StrokeMesh>) {
    let combination = combine_outlines(outlines);
    let standings = anchor_portals(&combination, portals);
    let mut stretches: Vec<Vec<Stretch>> = vec![Vec::new(); outlines.len()];
    for (outline, stretch) in standings
        .iter()
        .flatten()
        .filter_map(|standing| standing.stretches.as_ref())
        .flatten()
    {
        stretches[*outline].push(*stretch);
    }
    let meshes = combination
        .outlines
        .iter()
        .zip(&stretches)
        .map(|(outline, stretches)| generate_walls(outline, THICKNESS, stretches))
        .collect();
    (combination, standings, meshes)
}

/// In the test build, deriving the floors, the Walls, and the Portals' places of a Layer of
/// twenty Rooms, each with two curved edges, with sixteen shared edges and ten Portals, takes
/// under 4 milliseconds: a drag derives its Layer again at every frame, and this leaves three
/// quarters of a 60 Hz frame for the rest.
#[test]
fn recombined_within_a_frame() {
    let outlines = layer();
    let portals = doors();
    assert_eq!(outlines.len(), 20);
    assert_eq!(portals.len(), 10);
    let curves = outlines
        .iter()
        .flat_map(|outline| &outline.path.controls)
        .filter(|control| control.is_some())
        .count();
    assert_eq!(curves, 40);

    let (took, (combination, standings, meshes)) = fastest(5, || derive(&outlines, &portals));
    let shared = combination
        .outlines
        .iter()
        .flat_map(|outline| &outline.walls)
        .filter(|wall| {
            !wall.closed
                && wall.line.len() == 2
                && (wall.line[0].position.x - wall.line[1].position.x).abs() < 1e-6
        })
        .count();
    let set = standings
        .iter()
        .filter(|standing| {
            standing
                .as_ref()
                .is_some_and(|standing| standing.stretches.is_some())
        })
        .count();
    let floors = combination
        .outlines
        .iter()
        .filter(|outline| !outline.floor.indices.is_empty())
        .count();
    let walls = meshes
        .iter()
        .filter(|mesh| !mesh.indices.is_empty())
        .count();
    eprintln!(
        "{} Rooms with {curves} curved edges, {shared} shared Walls, and {set} of {} Portals set \
         derived in {took:?}",
        combination.outlines.len(),
        portals.len()
    );
    assert_eq!(shared, 16, "every pair of neighbours shares a Wall");
    assert_eq!(set, 10, "every Portal stands in a Wall");
    assert_eq!(floors, 20, "every Room has its floor");
    assert_eq!(walls, 20, "every Room draws Walls");
    assert!(took < Duration::from_millis(4), "derived in {took:?}");
}
