//! Painting Terrain through the headless editor: the real plugins of the model, the history,
//! `LibraryAccess`, `LibraryManager`, `ProjectManager`, and `AuthoringManager`, with no window and
//! no render Engine, over one Asset Folder of two texture images of known pixel size and a table
//! for the Props a Terrain is stacked with, driven by Apply, Undo, and Redo messages and asserted
//! on the Terrain, its Element, its derived coverage, the Layer's children, the answers, and the
//! history.
#![expect(
    clippy::missing_panics_doc,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

mod support;

use bevy::math::{UVec2, Vec2};
use drs_model::{
    Apply, CanonicalName, Colour, ElementChange, PlaceElement, Placement, RemoveElement, Side,
    StrokeChange, TERRAIN, Terrain, TileKey,
};
use support::terrain::{FLAGSTONES, Fixture, GRASS, HALF_PIXEL, SOFT, Tiles, at, brush, stroke};

/// The pixel-wise largest of several coverages.
fn largest(coverages: &[Tiles]) -> Tiles {
    let mut largest = Tiles::new();
    for tiles in coverages {
        for (key, pixels) in tiles {
            let into = largest.entry(*key).or_insert_with(|| vec![0; pixels.len()]);
            for (into, pixel) in into.iter_mut().zip(pixels) {
                *into = (*into).max(*pixel);
            }
        }
    }
    largest
}

/// The value a coverage of `coverage` is held as: times 255, rounded half up.
fn byte(coverage: f64) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a coverage is from 0 to 1"
    )]
    let value = (coverage * 255.0 + 0.5).floor() as u8;
    value
}

/// A Terrain holds one Material, which shows an image Asset, and an ordered list of strokes,
/// each a path of one or more points in Grid cells with the Brush settings it was laid with.
#[test]
fn terrain_is_its_strokes() {
    let mut fixture = Fixture::new();
    let first = stroke(
        &[
            Vec2::new(1.0, 1.0),
            Vec2::new(4.0, 2.5),
            Vec2::new(6.0, 1.0),
        ],
        brush(1.5, 0.2, 0.7),
    );
    let dab = stroke(&[Vec2::new(3.0, 5.0)], brush(4.0, 1.0, 1.0));
    fixture.flagstones(first.clone());
    fixture.paint(dab.clone(), None);

    let (_, element, terrain) = fixture.terrain();
    assert_eq!(element.kind, TERRAIN);
    let row = fixture.row(FLAGSTONES);
    assert_eq!(
        terrain,
        Terrain {
            image: row,
            strokes: vec![first, dab],
        }
    );
    let references = fixture.references();
    let image = references.get(row).expect("the Material's image");
    assert_eq!(image.places, vec![FLAGSTONES.to_owned()]);
    assert_eq!(image.pixel_size, Some(UVec2::splat(512)));
}

/// A stroke's coverage at a point is its strength within the hardness of its radius of its path,
/// nothing beyond the radius, and between them the smooth falloff of the distance, a hard
/// Brush's full strength reaching its radius exactly; a one-point path is a round dab.
#[test]
fn shaped_by_a_soft_round_brush() {
    let mut fixture = Fixture::new();
    // Along a row of pixel centres, so the pixel `j` rows above lies `j` thirty-seconds away.
    let y = 4.0 + HALF_PIXEL;
    fixture.flagstones(stroke(
        &[
            Vec2::new(2.0 + HALF_PIXEL, y),
            Vec2::new(10.0 + HALF_PIXEL, y),
        ],
        SOFT,
    ));
    let x = 6.0 + HALF_PIXEL;
    let above = |distance: f32| Vec2::new(x, y + distance);

    assert_eq!(fixture.coverage_at(above(0.0)), 255, "on the path");
    assert_eq!(fixture.coverage_at(above(0.25)), 255, "within the hardness");
    assert_eq!(fixture.coverage_at(above(0.5)), 255, "at the hardness");
    assert_eq!(
        fixture.coverage_at(above(0.75)),
        byte(0.5),
        "halfway through the falloff"
    );
    let t: f64 = 0.25;
    assert_eq!(
        fixture.coverage_at(above(0.625)),
        byte(1.0 - 3.0 * t * t + 2.0 * t * t * t),
        "a quarter through the falloff"
    );
    assert_eq!(fixture.coverage_at(above(1.0)), 0, "at the radius");
    assert_eq!(fixture.coverage_at(above(1.25)), 0, "beyond the radius");
    assert_eq!(
        fixture.coverage_at(Vec2::new(10.0 + 0.75 + HALF_PIXEL, y)),
        byte(0.5),
        "round past the end of the path"
    );

    let dab = Vec2::new(20.0 + HALF_PIXEL, y);
    fixture.paint(stroke(&[dab], brush(1.0, 0.0, 1.0)), None);
    assert_eq!(fixture.coverage_at(dab), 255, "the dab's centre");
    for offset in [
        Vec2::new(0.25, 0.0),
        Vec2::new(-0.25, 0.0),
        Vec2::new(0.0, 0.25),
        Vec2::new(0.0, -0.25),
    ] {
        assert_eq!(
            fixture.coverage_at(dab + offset),
            byte(0.5),
            "halfway out at {offset}"
        );
    }
    assert_eq!(fixture.coverage_at(dab + Vec2::new(0.5, 0.0)), 0, "its rim");

    let y = 8.0 + HALF_PIXEL;
    fixture.paint(
        stroke(
            &[
                Vec2::new(2.0 + HALF_PIXEL, y),
                Vec2::new(10.0 + HALF_PIXEL, y),
            ],
            brush(2.0, 1.0, 0.75),
        ),
        None,
    );
    assert_eq!(
        fixture.coverage_at(Vec2::new(x, y + 1.0)),
        byte(0.75),
        "a hard Brush's pixel exactly at its radius"
    );
    assert_eq!(
        fixture.coverage_at(Vec2::new(x, y + 1.0 + 1.0 / 32.0)),
        0,
        "the next one beyond"
    );
}

/// A stroke's coverage at a point is the same however many of its segments pass near it: at a
/// sharp joint and at a self-crossing it is what the nearest segment alone gives, never their
/// sum.
#[test]
fn no_build_up_along_a_stroke() {
    let mut fixture = Fixture::new();
    let half = brush(3.0, 0.2, 0.5);
    // The first stroke of a Layer makes its Terrain; it lies far from the strokes compared.
    fixture.flagstones(stroke(&[Vec2::new(-40.0, -40.0)], half));
    let alone = |fixture: &mut Fixture, points: &[Vec2]| {
        fixture.paint(stroke(points, half), None);
        let tiles = fixture.tiles();
        fixture.undo();
        tiles
    };
    let before = fixture.tiles();

    let sharp = [
        Vec2::new(2.0, 2.0),
        Vec2::new(4.0, 8.0),
        Vec2::new(6.0, 2.0),
    ];
    let bent = alone(&mut fixture, &sharp);
    let first = alone(&mut fixture, &sharp[..2]);
    let second = alone(&mut fixture, &sharp[1..]);
    assert_eq!(
        bent,
        largest(&[before.clone(), first.clone(), second.clone()])
    );
    let inside = Vec2::new(4.0 + HALF_PIXEL, 6.5 + HALF_PIXEL);
    let (one, other) = (at(&first, inside), at(&second, inside));
    assert!(
        one > 0 && other > 0,
        "both segments reach inside the joint, so their sum would show: {one} and {other}"
    );
    assert_eq!(at(&bent, inside), one.max(other));
    assert_eq!(at(&bent, sharp[1]), byte(0.5), "the joint at half strength");

    let crossing = [
        Vec2::new(10.0, 2.0),
        Vec2::new(16.0, 8.0),
        Vec2::new(16.0, 2.0),
        Vec2::new(10.0, 8.0),
    ];
    let crossed = alone(&mut fixture, &crossing);
    let pieces: Vec<Tiles> = crossing
        .windows(2)
        .map(|pair| alone(&mut fixture, pair))
        .collect();
    let mut all = vec![before];
    all.extend(pieces);
    assert_eq!(crossed, largest(&all));
    assert_eq!(
        at(&crossed, Vec2::new(13.0, 5.0)),
        byte(0.5),
        "the self-crossing at half strength"
    );
}

/// A Terrain's coverage at a point is the largest coverage any of its strokes has there: two
/// overlapping half-strength strokes stay at half, and a weaker stroke over a stronger one leaves
/// the stronger.
#[test]
fn strokes_composite_by_the_strongest() {
    let mut fixture = Fixture::new();
    let y = 4.0 + HALF_PIXEL;
    let half = brush(2.0, 1.0, 0.5);
    fixture.flagstones(stroke(&[Vec2::new(1.0, y), Vec2::new(5.0, y)], half));
    fixture.paint(
        stroke(&[Vec2::new(3.0, y - 3.0), Vec2::new(3.0, y + 3.0)], half),
        None,
    );
    fixture.paint(stroke(&[Vec2::new(1.0, y), Vec2::new(5.0, y)], half), None);
    let overlap = Vec2::new(3.0 + HALF_PIXEL, y);
    assert_eq!(
        fixture.coverage_at(overlap),
        byte(0.5),
        "passed over three times"
    );

    let x = 10.0 + HALF_PIXEL;
    fixture.paint(
        stroke(
            &[Vec2::new(x, 1.0), Vec2::new(x, 7.0)],
            brush(2.0, 1.0, 1.0),
        ),
        None,
    );
    fixture.paint(
        stroke(
            &[Vec2::new(8.0, y), Vec2::new(12.0, y)],
            brush(2.0, 1.0, 0.25),
        ),
        None,
    );
    assert_eq!(
        fixture.coverage_at(Vec2::new(x, y)),
        255,
        "the stronger stroke stays"
    );
    assert_eq!(
        fixture.coverage_at(Vec2::new(8.0 + HALF_PIXEL, y)),
        byte(0.25),
        "the weaker stroke where it is alone"
    );
}

/// A Terrain's position is the centre and its size the extent of the smallest box holding every
/// point of every stroke grown by that stroke's radius on every side.
#[test]
fn the_box_follows_the_strokes() {
    let mut fixture = Fixture::new();
    fixture.flagstones(stroke(&[Vec2::new(1.0, 2.0), Vec2::new(5.0, 3.0)], SOFT));
    let (_, element, _) = fixture.terrain();
    assert_eq!(element.position, Vec2::new(3.0, 2.5));
    assert_eq!(element.size, Vec2::new(6.0, 3.0));

    fixture.paint(stroke(&[Vec2::new(10.0, -2.0)], brush(4.0, 0.5, 1.0)), None);
    let (_, element, _) = fixture.terrain();
    assert_eq!(element.position, Vec2::new(6.0, 0.0));
    assert_eq!(element.size, Vec2::new(12.0, 8.0));

    fixture.undo();
    let (_, element, _) = fixture.terrain();
    assert_eq!(element.position, Vec2::new(3.0, 2.5));
    assert_eq!(element.size, Vec2::new(6.0, 3.0));
}

/// A Terrain made by a Paint is placed below every Element already on its Layer, and an Element
/// placed after it goes on top.
#[test]
fn terrain_goes_under() {
    let mut fixture = Fixture::new();
    let prop = fixture.prop(Vec2::new(3.0, 3.0));
    let wall = fixture.wall(&[Vec2::new(1.0, 1.0), Vec2::new(6.0, 1.0)]);
    fixture.flagstones(stroke(&[Vec2::new(2.0, 2.0), Vec2::new(8.0, 4.0)], SOFT));
    let (terrain, ..) = fixture.terrain();
    assert_eq!(fixture.order(), vec![terrain, prop, wall]);

    fixture.paint(stroke(&[Vec2::new(1.0, 6.0)], SOFT), None);
    let later = fixture.prop(Vec2::new(4.0, 4.0));
    assert_eq!(fixture.order(), vec![terrain, prop, wall, later]);
}

/// A Paint on a Layer adds its stroke to the topmost Terrain on that Layer, and makes a Terrain
/// only when the Layer has none.
#[test]
fn one_terrain_per_layer() {
    let mut fixture = Fixture::new();
    fixture.flagstones(stroke(&[Vec2::new(2.0, 2.0)], SOFT));
    let (terrain, ..) = fixture.terrain();
    fixture.prop(Vec2::new(3.0, 3.0));
    fixture.flagstones(stroke(&[Vec2::new(6.0, 2.0)], SOFT));
    fixture.paint(stroke(&[Vec2::new(9.0, 2.0)], SOFT), None);

    let terrains = fixture.terrains();
    assert_eq!(terrains.len(), 1, "one Terrain");
    assert_eq!(terrains[0].0, terrain);
    assert_eq!(terrains[0].2.strokes.len(), 3);
    assert_eq!(fixture.order().len(), 2, "the Terrain and the Prop");

    // Only a file can hold a second Terrain on a Layer, here one of no strokes on top of the
    // Prop: a Paint adds to the topmost.
    let path = fixture.save();
    let mut file: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("the saved Project"))
            .expect("the saved Project is JSON");
    let mut second = file["elements"][terrain.as_raw().to_string()].clone();
    second["terrain"]["data"]["strokes"] = serde_json::json!([]);
    file["elements"]["77"] = second;
    file["levels"][0]["layers"][0]["elements"]
        .as_array_mut()
        .expect("the Layer's order")
        .push(serde_json::json!("77"));
    std::fs::write(&path, file.to_string()).expect("the Project rewritten");
    fixture.open(path);

    fixture.paint(stroke(&[Vec2::new(12.0, 2.0)], SOFT), None);
    let terrains = fixture.terrains();
    assert_eq!(terrains.len(), 2, "both Terrains of the file");
    assert_eq!(terrains[0].0, terrain);
    assert_eq!(
        terrains[0].2.strokes.len(),
        3,
        "the lower one keeps its strokes"
    );
    assert_eq!(terrains[1].0.as_raw(), 77);
    assert_eq!(
        terrains[1].2.strokes.len(),
        1,
        "the topmost takes the stroke"
    );
}

/// A Paint adds one stroke at the end of its Terrain's strokes as one history step; undo takes
/// exactly that stroke away and redo puts it back, the same stroke at the same place in the order.
#[test]
fn a_stroke_is_one_step() {
    let mut fixture = Fixture::new();
    let first = stroke(&[Vec2::new(1.0, 1.0), Vec2::new(4.0, 1.0)], SOFT);
    let second = stroke(
        &[Vec2::new(2.0, 3.0), Vec2::new(2.0, 6.0)],
        brush(1.0, 0.8, 0.6),
    );
    let third = stroke(&[Vec2::new(5.0, 5.0)], SOFT);
    fixture.flagstones(first.clone());
    fixture.paint(second.clone(), None);
    assert_eq!(fixture.steps(), 2);
    fixture.paint(third.clone(), None);
    assert_eq!(fixture.steps(), 3);

    fixture.undo();
    assert_eq!(
        fixture.terrain().2.strokes,
        vec![first.clone(), second.clone()]
    );
    fixture.undo();
    assert_eq!(fixture.terrain().2.strokes, vec![first.clone()]);
    fixture.redo();
    assert_eq!(
        fixture.terrain().2.strokes,
        vec![first.clone(), second.clone()]
    );
    fixture.redo();
    assert_eq!(fixture.terrain().2.strokes, vec![first, second, third]);
    assert_eq!(fixture.steps(), 3);
}

/// A Paint on a Layer with no Terrain places a Terrain of the given image holding that stroke, in
/// the same history step; undo takes the Terrain away whole and redo brings it back with the
/// identity it had.
#[test]
fn the_first_stroke_makes_the_terrain() {
    let mut fixture = Fixture::new();
    let prop = fixture.prop(Vec2::new(3.0, 3.0));
    let first = stroke(&[Vec2::new(2.0, 2.0), Vec2::new(5.0, 2.0)], SOFT);
    fixture.flagstones(first.clone());
    let (id, element, terrain) = fixture.terrain();
    let tiles = fixture.tiles();
    assert_eq!(terrain.strokes, vec![first]);
    assert_eq!(fixture.steps(), 2);

    fixture.undo();
    assert!(fixture.terrains().is_empty(), "the Terrain is gone");
    assert_eq!(fixture.order(), vec![prop]);
    assert_eq!(fixture.steps(), 1);

    fixture.redo();
    assert_eq!(fixture.terrain(), (id, element, terrain));
    assert_eq!(fixture.order(), vec![id, prop]);
    assert_eq!(fixture.tiles(), tiles);
}

/// A Paint naming no image paints with its Terrain's Material and is refused on a Layer with no
/// Terrain; a Paint naming an image other than the one its Terrain's Material shows is refused,
/// with the reason naming both.
#[test]
fn painted_with_its_material() {
    let mut fixture = Fixture::new();
    let dab = |x: f32| stroke(&[Vec2::new(x, 2.0)], SOFT);

    let refused = fixture.try_paint(dab(1.0), None);
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert!(refused[0].contains("choose an Asset"), "{}", refused[0]);
    assert!(fixture.terrains().is_empty());
    assert_eq!(fixture.steps(), 0);

    fixture.flagstones(dab(2.0));
    fixture.paint(dab(4.0), None);
    fixture.flagstones(dab(6.0));
    assert_eq!(fixture.terrain().2.strokes.len(), 3);

    let refused = fixture.try_paint(dab(8.0), Some(GRASS));
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert!(
        refused[0].contains("grass") && refused[0].contains("flagstones"),
        "{}",
        refused[0]
    );
    let (_, _, terrain) = fixture.terrain();
    assert_eq!(terrain.strokes.len(), 3);
    assert_eq!(terrain.image, fixture.row(FLAGSTONES));
    assert_eq!(fixture.steps(), 3);
}

/// After undoing a stroke, every point of its Terrain has the coverage it had before the stroke
/// was laid.
#[test]
fn undo_leaves_no_trace() {
    let mut fixture = Fixture::new();
    fixture.flagstones(stroke(&[Vec2::new(1.0, 1.0), Vec2::new(14.0, 3.0)], SOFT));
    fixture.paint(
        stroke(
            &[Vec2::new(15.0, 0.0), Vec2::new(17.0, 9.0)],
            brush(3.0, 0.1, 0.6),
        ),
        None,
    );
    let before = fixture.tiles();
    fixture.paint(
        stroke(
            &[
                Vec2::new(0.0, 2.5),
                Vec2::new(16.5, 2.5),
                Vec2::new(16.0, 6.0),
            ],
            brush(2.5, 0.3, 0.9),
        ),
        None,
    );
    assert_ne!(fixture.tiles(), before, "the stroke shows");

    fixture.undo();
    assert_eq!(fixture.tiles(), before);
}

/// A Terrain's coverage is the same however its strokes came to be: laid one by one, undone and
/// redone, or opened from a file.
#[test]
fn coverage_is_the_strokes_alone() {
    let mut fixture = Fixture::new();
    fixture.flagstones(stroke(&[Vec2::new(-3.0, 1.0), Vec2::new(20.0, 4.0)], SOFT));
    fixture.paint(stroke(&[Vec2::new(5.0, 5.0)], brush(6.0, 0.0, 0.5)), None);
    fixture.paint(
        stroke(
            &[Vec2::new(16.0, -1.0), Vec2::new(15.0, 17.0)],
            brush(1.0, 1.0, 0.8),
        ),
        None,
    );
    let laid = fixture.tiles();
    assert!(laid.len() >= 4, "{} tiles", laid.len());

    fixture.undo();
    fixture.undo();
    fixture.redo();
    fixture.redo();
    assert_eq!(fixture.tiles(), laid, "undone and redone");

    fixture.save_and_open();
    assert_eq!(fixture.tiles(), laid, "opened from the file");
}

/// A Paint with no point, a point that is not finite, a size not above zero or not finite, a
/// hardness outside 0 to 1, or a strength not above 0 or above 1 is answered with the reason,
/// changes nothing, and records no history step.
#[test]
fn malformed_strokes_are_refused() {
    let mut fixture = Fixture::new();
    let point = [Vec2::new(2.0, 2.0)];
    let malformed = [
        (stroke(&[], SOFT), "one or more points"),
        (stroke(&[Vec2::new(f32::NAN, 1.0)], SOFT), "finite"),
        (stroke(&[Vec2::new(1.0, f32::INFINITY)], SOFT), "finite"),
        (stroke(&point, brush(0.0, 0.5, 1.0)), "size"),
        (stroke(&point, brush(-1.0, 0.5, 1.0)), "size"),
        (stroke(&point, brush(f32::NAN, 0.5, 1.0)), "size"),
        (stroke(&point, brush(f32::INFINITY, 0.5, 1.0)), "size"),
        (stroke(&point, brush(1.0, -0.1, 1.0)), "hardness"),
        (stroke(&point, brush(1.0, 1.1, 1.0)), "hardness"),
        (stroke(&point, brush(1.0, f32::NAN, 1.0)), "hardness"),
        (stroke(&point, brush(1.0, 0.5, 0.0)), "strength"),
        (stroke(&point, brush(1.0, 0.5, -0.5)), "strength"),
        (stroke(&point, brush(1.0, 0.5, 1.5)), "strength"),
        (stroke(&point, brush(1.0, 0.5, f32::NAN)), "strength"),
    ];
    for (bad, reason) in &malformed {
        let refused = fixture.try_paint(bad.clone(), Some(FLAGSTONES));
        assert_eq!(refused.len(), 1, "{bad:?} on a Layer without Terrain");
        assert!(refused[0].contains(reason), "{}", refused[0]);
    }
    assert!(fixture.terrains().is_empty());
    assert_eq!(fixture.steps(), 0);

    fixture.flagstones(stroke(&point, SOFT));
    let terrain = fixture.terrain();
    let tiles = fixture.tiles();
    for (bad, reason) in malformed {
        let refused = fixture.try_paint(bad.clone(), None);
        assert_eq!(refused.len(), 1, "{bad:?} on the Terrain");
        assert!(refused[0].contains(reason), "{}", refused[0]);
    }
    assert_eq!(fixture.terrain(), terrain);
    assert_eq!(fixture.tiles(), tiles);
    assert_eq!(fixture.steps(), 1);
}

/// An Edit Element setting a Terrain's Material to another image Asset makes every stroke show
/// that image, keeping every stroke's path and Brush settings, as one history step that undo
/// returns to the image it had; one naming the image it already shows records no step.
#[test]
fn the_material_stays_editable() {
    let mut fixture = Fixture::new();
    fixture.flagstones(stroke(&[Vec2::new(1.0, 1.0), Vec2::new(5.0, 2.0)], SOFT));
    fixture.paint(stroke(&[Vec2::new(3.0, 4.0)], brush(3.0, 0.2, 0.5)), None);
    let (id, element, before) = fixture.terrain();
    let tiles = fixture.tiles();
    let grass = fixture.asset(GRASS);

    let same = fixture.try_edit(id, ElementChange::Material(fixture.asset(FLAGSTONES)));
    assert!(same.is_empty(), "{same:?}");
    assert_eq!(fixture.terrain().2, before, "the image it already shows");
    assert_eq!(fixture.steps(), 2, "no step for the image it already shows");

    let refused = fixture.try_edit(id, ElementChange::Material(grass));
    assert!(refused.is_empty(), "{refused:?}");
    let (_, changed_element, changed) = fixture.terrain();
    assert_eq!(changed.image, fixture.row(GRASS));
    assert_eq!(changed.strokes, before.strokes);
    assert_eq!(changed_element, element);
    assert_eq!(fixture.tiles(), tiles);
    assert_eq!(fixture.steps(), 3);

    fixture.paint(stroke(&[Vec2::new(7.0, 4.0)], SOFT), Some(GRASS));
    assert_eq!(
        fixture.terrain().2.strokes.len(),
        3,
        "painted with its new image"
    );
    fixture.undo();

    fixture.undo();
    assert_eq!(fixture.terrain().2, before);
    fixture.redo();
    assert_eq!(fixture.terrain().2, changed);
}

/// An Edit Element of a Terrain that changes anything but its Material or one of its strokes, its
/// position and every change only a Wall, a Room, or a Portal has included, and an Edit Element
/// setting the Material or changing a stroke of a Prop, a Wall, a Room, or a Portal, none of them a
/// Terrain, are answered with the reason, change nothing, and record no history step.
#[test]
fn terrain_changes_only_its_material_and_strokes() {
    let mut fixture = Fixture::new();
    let prop = fixture.prop(Vec2::new(8.0, 8.0));
    let wall = fixture.wall(&[Vec2::new(20.0, 0.0), Vec2::new(24.0, 0.0)]);
    let room = fixture.room(&[
        Vec2::new(30.0, 0.0),
        Vec2::new(34.0, 0.0),
        Vec2::new(34.0, 4.0),
    ]);
    let portal = fixture.portal(Vec2::new(40.0, 2.0));
    fixture.flagstones(stroke(&[Vec2::new(1.0, 1.0), Vec2::new(5.0, 2.0)], SOFT));
    let terrain = fixture.terrain();
    let depth = fixture.steps();

    for change in [
        ElementChange::Position(Vec2::new(10.0, 10.0)),
        ElementChange::MoveBy(Vec2::new(1.0, 0.0)),
        ElementChange::Point {
            index: 0,
            position: Vec2::new(2.0, 2.0),
        },
        ElementChange::Control {
            segment: 0,
            position: Some(Vec2::new(3.0, 5.0)),
        },
        ElementChange::AddPoint { segment: 0, t: 0.5 },
        ElementChange::RemovePoint { index: 0 },
        ElementChange::Thickness(0.5),
        ElementChange::Colour(Colour::rgb(1, 2, 3)),
        ElementChange::FloorColour(Colour::rgb(4, 5, 6)),
        ElementChange::Width(2.0),
        ElementChange::Rotation(1.0),
        ElementChange::Mirrored(true),
        ElementChange::Side(Side::Right),
        ElementChange::Along { segment: 0, t: 0.5 },
    ] {
        let refused = fixture.try_edit(terrain.0, change.clone());
        assert_eq!(refused.len(), 1, "{change:?}");
        assert!(refused[0].contains("Terrain"), "{}", refused[0]);
    }
    let grass = fixture.asset(GRASS);
    for change in [
        ElementChange::Material(grass),
        ElementChange::Stroke {
            stroke: 0,
            change: StrokeChange::Point {
                index: 0,
                position: Vec2::ONE,
            },
        },
        ElementChange::Stroke {
            stroke: 0,
            change: StrokeChange::Position(Vec2::ONE),
        },
        ElementChange::Stroke {
            stroke: 0,
            change: StrokeChange::Brush(SOFT),
        },
        ElementChange::Stroke {
            stroke: 0,
            change: StrokeChange::Erase(true),
        },
        ElementChange::Stroke {
            stroke: 0,
            change: StrokeChange::Remove,
        },
    ] {
        for (element, what) in [
            (prop, "Prop"),
            (wall, "Wall"),
            (room, "Room"),
            (portal, "Portal"),
        ] {
            let refused = fixture.try_edit(element, change.clone());
            assert_eq!(refused.len(), 1, "{change:?} of the {what}");
            assert!(refused[0].contains("not a Terrain"), "{}", refused[0]);
        }
    }

    assert_eq!(fixture.terrain(), terrain);
    assert_eq!(fixture.steps(), depth);
    assert_eq!(
        fixture.references().assets.len(),
        2,
        "the table and the flagstones"
    );
}

/// A Remove Element of a Terrain takes it off its Layer as one step, and undo restores it with its
/// identity, its strokes, its place among the Layer's children, and its coverage derived afresh.
#[test]
fn terrain_removal_is_reversible_in_place() {
    let mut fixture = Fixture::new();
    let prop = fixture.prop(Vec2::new(3.0, 3.0));
    fixture.flagstones(stroke(&[Vec2::new(1.0, 1.0), Vec2::new(14.0, 3.0)], SOFT));
    fixture.paint(stroke(&[Vec2::new(17.0, 2.0)], brush(3.0, 0.2, 0.6)), None);
    let terrain = fixture.terrain();
    let tiles = fixture.tiles();
    let steps = fixture.steps();

    fixture.apply(Apply::RemoveElement(RemoveElement { element: terrain.0 }));
    assert!(fixture.terrains().is_empty());
    assert_eq!(fixture.order(), vec![prop]);
    assert_eq!(fixture.steps(), steps + 1);

    fixture.undo();
    assert_eq!(fixture.terrain(), terrain);
    assert_eq!(fixture.order(), vec![terrain.0, prop]);
    assert_eq!(fixture.tiles(), tiles);
}

/// A Paint that makes a Terrain, and an Edit Element setting a Terrain's Material, record the
/// image's Asset Reference as a placement does, and no second one for an Asset already recorded.
#[test]
fn painting_records_a_reference() {
    let mut fixture = Fixture::new();
    fixture.flagstones(stroke(&[Vec2::new(1.0, 1.0)], SOFT));
    let references = fixture.references();
    assert_eq!(references.assets.len(), 1);
    let flagstones = &references.assets[0];
    assert_eq!(flagstones.name, "flagstones");
    assert_eq!(flagstones.places, vec![FLAGSTONES.to_owned()]);
    assert_eq!(flagstones.pixel_size, Some(UVec2::splat(512)));
    assert!(flagstones.fingerprint.as_str().starts_with("blake3:"));

    fixture.flagstones(stroke(&[Vec2::new(3.0, 1.0)], SOFT));
    fixture.undo();
    fixture.undo();
    fixture.flagstones(stroke(&[Vec2::new(5.0, 1.0)], SOFT));
    assert_eq!(fixture.references().assets.len(), 1, "recorded once");

    let (id, ..) = fixture.terrain();
    let grass = fixture.asset(GRASS);
    assert!(
        fixture
            .try_edit(id, ElementChange::Material(grass))
            .is_empty()
    );
    assert_eq!(fixture.references().assets.len(), 2);
    let flagstones = fixture.asset(FLAGSTONES);
    assert!(
        fixture
            .try_edit(id, ElementChange::Material(flagstones))
            .is_empty()
    );
    let layer = fixture.layer();
    let asset = fixture.asset(FLAGSTONES);
    fixture.apply(Apply::PlaceElement(PlaceElement {
        layer,
        placement: Placement::Prop {
            position: Vec2::new(2.0, 2.0),
            asset,
        },
    }));
    assert_eq!(
        fixture.references().assets.len(),
        2,
        "the flagstones and the grass, each once"
    );
    assert_eq!(fixture.terrain().2.image, fixture.row(FLAGSTONES));
}

/// The first Terrain image taken from an Asset Folder records the folder, once.
#[test]
fn painting_records_the_folder() {
    let mut fixture = Fixture::new();
    fixture.flagstones(stroke(&[Vec2::new(1.0, 1.0)], SOFT));
    let folders = fixture.references().folders;
    assert_eq!(folders.len(), 1);
    assert_eq!(folders[0].name, CanonicalName("Fixtures".to_owned()));

    let (id, ..) = fixture.terrain();
    let grass = fixture.asset(GRASS);
    assert!(
        fixture
            .try_edit(id, ElementChange::Material(grass))
            .is_empty()
    );
    fixture.undo();
    fixture.undo();
    assert_eq!(
        fixture.references().folders,
        folders,
        "kept and not repeated"
    );
}

/// A stroke may lie anywhere on the Level, outside the Bounds and at negative cells, and its
/// coverage is there.
#[test]
fn strokes_lie_anywhere_on_the_level() {
    let mut fixture = Fixture::new();
    let outside = Vec2::new(-20.0 + HALF_PIXEL, -10.0 + HALF_PIXEL);
    let beyond = Vec2::new(45.0 + HALF_PIXEL, 50.0 + HALF_PIXEL);
    fixture.flagstones(stroke(&[outside, outside + Vec2::new(3.0, -2.0)], SOFT));
    fixture.paint(stroke(&[beyond], SOFT), None);

    let (_, element, _) = fixture.terrain();
    assert_eq!(
        element.position,
        (outside + Vec2::new(-1.0, -3.0) + beyond + Vec2::splat(1.0)) / 2.0
    );
    let tiles = fixture.tiles();
    assert!(
        tiles.contains_key(&TileKey { x: -2, y: -1 }),
        "{:?}",
        tiles.keys()
    );
    assert!(
        tiles.contains_key(&TileKey { x: 2, y: 3 }),
        "{:?}",
        tiles.keys()
    );
    assert_eq!(at(&tiles, outside), 255);
    assert_eq!(at(&tiles, beyond), 255);
    assert_eq!(at(&tiles, Vec2::new(10.0, 10.0)), 0);
}

/// Paint is one undo step in the one history, among Place and Edit Element, undone and redone in
/// the order the steps were taken.
#[test]
fn paint_shares_the_history() {
    let mut fixture = Fixture::new();
    let prop = fixture.prop(Vec2::new(3.0, 3.0));
    fixture.flagstones(stroke(&[Vec2::new(1.0, 1.0)], SOFT));
    let (terrain, ..) = fixture.terrain();
    let wall = fixture.wall(&[Vec2::new(1.0, 5.0), Vec2::new(6.0, 5.0)]);
    fixture.paint(stroke(&[Vec2::new(4.0, 1.0)], SOFT), None);
    assert_eq!(fixture.steps(), 4);

    fixture.undo();
    assert_eq!(fixture.terrain().2.strokes.len(), 1);
    assert_eq!(fixture.order(), vec![terrain, prop, wall]);
    fixture.undo();
    assert_eq!(fixture.order(), vec![terrain, prop]);
    fixture.undo();
    assert_eq!(fixture.order(), vec![prop]);
    fixture.undo();
    assert!(fixture.order().is_empty());

    for _ in 0..4 {
        fixture.redo();
    }
    assert_eq!(fixture.order(), vec![terrain, prop, wall]);
    assert_eq!(fixture.terrain().2.strokes.len(), 2);
}

/// Redo carries a Paint out exactly as it was first applied: the same stroke, the same Terrain,
/// the same coverage.
#[test]
fn paint_redoes_exactly() {
    let mut fixture = Fixture::new();
    fixture.flagstones(stroke(&[Vec2::new(1.0, 1.0), Vec2::new(9.0, 3.0)], SOFT));
    fixture.paint(
        stroke(
            &[Vec2::new(4.0, 0.0), Vec2::new(4.5, 6.0)],
            brush(1.25, 0.75, 0.5),
        ),
        None,
    );
    let terrain = fixture.terrain();
    let tiles = fixture.tiles();

    fixture.undo();
    fixture.redo();
    assert_eq!(fixture.terrain(), terrain);
    assert_eq!(fixture.tiles(), tiles);

    fixture.undo();
    fixture.undo();
    fixture.redo();
    fixture.redo();
    assert_eq!(fixture.terrain(), terrain);
    assert_eq!(fixture.tiles(), tiles);
}

/// Without a renderer a Terrain's coverage is the base band on the CPU whatever the zoom: a zoom
/// that would show a closer band leaves it at the base, its tiles holding their pixels as before
/// and naming no image on the GPU.
#[test]
fn without_a_renderer_the_base_is_on_the_cpu() {
    let mut fixture = Fixture::new();
    fixture.look(Vec2::new(3.0, 2.0), 40.0, Vec2::new(1024.0, 768.0));
    fixture.flagstones(stroke(&[Vec2::new(1.0, 1.0), Vec2::new(9.0, 3.0)], SOFT));
    let before = fixture.coverage();
    assert_eq!(before.band, 32);

    fixture.look(Vec2::new(3.0, 2.0), 256.0, Vec2::new(1024.0, 768.0));

    let after = fixture.coverage();
    assert_eq!(after.band, 32);
    assert_eq!(after, before);
    assert!(!after.tiles.is_empty());
    for tile in after.tiles.values() {
        assert_eq!(tile.pixels().map(<[u8]>::len), Some(512 * 512));
    }
    assert!(fixture.coverage_at(Vec2::new(5.0 + HALF_PIXEL, 2.0 + HALF_PIXEL)) > 0);
}
