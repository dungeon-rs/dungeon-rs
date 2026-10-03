//! Erasing and editing the strokes of a Terrain through the headless editor: the real plugins of
//! the model, the history, `LibraryAccess`, `LibraryManager`, `ProjectManager`, and
//! `AuthoringManager`, with no window and no render Engine, over one Asset Folder of two texture
//! images and a table for Props, driven by Apply, Undo, and Redo messages and asserted on the
//! Terrain, its Element, its derived coverage, the Layer's children, the answers, and the
//! history.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

mod support;

use bevy::math::Vec2;
use drs_model::{
    Apply, BrushSettings, EditElement, ElementChange, ElementId, Gesture, Stroke, StrokeChange,
};
use support::terrain::{FLAGSTONES, Fixture, GRASS, HALF_PIXEL, SOFT, Tiles, at, brush, stroke};

/// The Edit Element change `change` of the stroke of number `stroke`.
fn edit_of(stroke: usize, change: StrokeChange) -> ElementChange {
    ElementChange::Stroke { stroke, change }
}

/// A stroke that erases through `points` with `brush`.
fn erasing(points: &[Vec2], brush: BrushSettings) -> Stroke {
    Stroke {
        points: points.to_vec(),
        brush,
        erase: true,
    }
}

/// The pixel-wise smallest of several coverages that hold the same tiles.
fn smallest(coverages: &[Tiles]) -> Tiles {
    let mut smallest = coverages.first().cloned().unwrap_or_default();
    for tiles in &coverages[1..] {
        for (key, into) in &mut smallest {
            let pixels = tiles.get(key).expect("the same tiles");
            for (into, pixel) in into.iter_mut().zip(pixels) {
                *into = (*into).min(*pixel);
            }
        }
    }
    smallest
}

/// The value a coverage of `coverage` is held as: times 255, rounded half up.
fn byte(coverage: f32) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a coverage is from 0 to 1"
    )]
    let value = (coverage * 255.0 + 0.5).floor() as u8;
    value
}

/// The coverage a stroke laid with `brush` has at `distance` cells from its path: its strength
/// within the hardness of its radius, nothing from the radius on, and the smooth falloff between.
fn shaped(brush: BrushSettings, distance: f32) -> f32 {
    let radius = brush.size / 2.0;
    let inner = brush.hardness * radius;
    if distance <= inner {
        brush.strength
    } else if distance >= radius {
        0.0
    } else {
        let t = (distance - inner) / (radius - inner);
        brush.strength * (1.0 - 3.0 * t * t + 2.0 * t * t * t)
    }
}

impl Fixture {
    /// Lays the erase `stroke` on the Layer naming no image, failing the test on a refusal.
    fn erase(&mut self, stroke: Stroke) {
        assert!(stroke.erase, "an erase");
        self.paint(stroke, None);
    }

    /// Sends an Edit Element on its own, failing the test on a refusal.
    fn edit(&mut self, element: ElementId, change: ElementChange) {
        let refused = self.try_edit(element, change);
        assert!(refused.is_empty(), "the edit was refused: {refused:?}");
    }

    /// Sends `changes` as one gesture, its first the Begin and its last the End, one update
    /// each, failing the test on a refusal.
    fn gesture(&mut self, element: ElementId, changes: Vec<ElementChange>) {
        let last = changes.len() - 1;
        for (index, change) in changes.into_iter().enumerate() {
            let gesture = match index {
                0 => Gesture::Begin,
                index if index == last => Gesture::End,
                _ => Gesture::Continue,
            };
            self.apply(Apply::EditElement(EditElement {
                element,
                change,
                gesture,
            }));
        }
    }

    /// The one Terrain's strokes.
    fn strokes(&mut self) -> Vec<Stroke> {
        self.terrain().2.strokes
    }
}

/// A Brush that covers fully and sharply: `size` cells across, hard, at full strength.
fn hard(size: f32) -> BrushSettings {
    brush(size, 1.0, 1.0)
}

/// Ground covered fully from (-2, -2) to (22, 12), laid as the first stroke of the Layer.
fn ground(fixture: &mut Fixture) {
    fixture.flagstones(stroke(
        &[Vec2::new(0.0, 5.0), Vec2::new(20.0, 5.0)],
        hard(14.0),
    ));
}

/// An erase lowers a Terrain's coverage at each point to one minus its own coverage there where
/// that is lower and leaves it where it is already as low: a full-strength erase leaves nothing
/// within its hardness, and one of strength `s` leaves at most `1 - s`.
#[test]
fn erasing_caps_what_remains() {
    let mut fixture = Fixture::new();
    // A soft stroke along a row of pixel centres, so a pixel `j` rows above lies `j`
    // thirty-seconds of a cell from its path.
    let y = 4.0 + HALF_PIXEL;
    let soft = brush(4.0, 0.5, 1.0);
    fixture.flagstones(stroke(&[Vec2::new(0.0, y), Vec2::new(20.0, y)], soft));
    let full = brush(2.0, 0.5, 1.0);
    let x = 4.0 + HALF_PIXEL;
    fixture.erase(erasing(&[Vec2::new(x, 0.0), Vec2::new(x, 8.0)], full));

    assert_eq!(
        fixture.coverage_at(Vec2::new(x, y)),
        0,
        "on the erase's path"
    );
    assert_eq!(
        fixture.coverage_at(Vec2::new(x + 0.5, y)),
        0,
        "at the erase's hardness"
    );
    let tiles = fixture.tiles();
    for across in 0..48_u8 {
        for up in 0..72_u8 {
            let (dx, dy) = (f32::from(across) / 32.0, f32::from(up) / 32.0);
            let expected = byte(shaped(soft, dy)).min(byte(1.0 - shaped(full, dx)));
            assert_eq!(
                at(&tiles, Vec2::new(x + dx, y + dy)),
                expected,
                "{dx} across and {dy} up"
            );
        }
    }

    let half = brush(2.0, 1.0, 0.5);
    let x = 10.0 + HALF_PIXEL;
    fixture.erase(erasing(&[Vec2::new(x, 0.0), Vec2::new(x, 8.0)], half));
    assert_eq!(
        fixture.coverage_at(Vec2::new(x, y)),
        byte(0.5),
        "half left of full ground"
    );

    let mut halves = Fixture::new();
    halves.flagstones(stroke(&[Vec2::new(0.0, y), Vec2::new(20.0, y)], half));
    let before = halves.coverage_at(Vec2::new(x, y));
    halves.erase(erasing(&[Vec2::new(x, 0.0), Vec2::new(x, 8.0)], half));
    assert_eq!(before, byte(0.5));
    assert_eq!(
        halves.coverage_at(Vec2::new(x, y)),
        before,
        "half over half is left as it was"
    );
}

/// An erase lowers the coverage at a point by the same however many of its segments pass near
/// the point, at a joint or a self-crossing alike, and passing over ground again with an erase
/// never lowers it below what the strongest erase there leaves.
#[test]
fn no_build_up_along_an_erase() {
    let mut fixture = Fixture::new();
    ground(&mut fixture);
    let half = brush(3.0, 0.2, 0.5);
    let alone = |fixture: &mut Fixture, points: &[Vec2]| {
        fixture.erase(erasing(points, half));
        let tiles = fixture.tiles();
        fixture.undo();
        tiles
    };

    let sharp = [
        Vec2::new(2.0, 2.0),
        Vec2::new(4.0, 8.0),
        Vec2::new(6.0, 2.0),
    ];
    let bent = alone(&mut fixture, &sharp);
    let first = alone(&mut fixture, &sharp[..2]);
    let second = alone(&mut fixture, &sharp[1..]);
    assert_eq!(bent, smallest(&[first.clone(), second.clone()]));
    let inside = Vec2::new(4.0 + HALF_PIXEL, 6.5 + HALF_PIXEL);
    let (one, other) = (at(&first, inside), at(&second, inside));
    assert!(
        one < 255 && other < 255,
        "both segments reach inside the joint, so a product would show: {one} and {other}"
    );
    assert_eq!(at(&bent, inside), one.min(other));
    assert_eq!(at(&bent, sharp[1]), byte(0.5), "the joint left at half");

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
    assert_eq!(crossed, smallest(&pieces));
    assert_eq!(
        at(&crossed, Vec2::new(13.0, 5.0)),
        byte(0.5),
        "the self-crossing left at half"
    );

    let across = [Vec2::new(1.0, 10.0), Vec2::new(19.0, 10.0)];
    fixture.erase(erasing(&across, brush(2.0, 1.0, 0.5)));
    fixture.erase(erasing(&across, brush(2.0, 1.0, 0.5)));
    fixture.erase(erasing(
        &[Vec2::new(8.0, 9.0), Vec2::new(8.0, 11.0)],
        brush(2.0, 1.0, 0.25),
    ));
    assert_eq!(
        fixture.coverage_at(Vec2::new(8.0 + HALF_PIXEL, 10.0 + HALF_PIXEL)),
        byte(0.5),
        "erased three times, left at what the strongest leaves"
    );
}

/// An erase lowers only the coverage the strokes laid before it give: a stroke laid after it
/// paints over the erased ground as over bare ground, and an erase where no earlier stroke covers
/// anything changes nothing.
#[test]
fn an_erase_takes_from_what_lies_before_it() {
    let mut fixture = Fixture::new();
    let y = 4.0 + HALF_PIXEL;
    let x = 6.0 + HALF_PIXEL;
    fixture.flagstones(stroke(&[Vec2::new(1.0, y), Vec2::new(12.0, y)], hard(2.0)));
    fixture.erase(erasing(&[Vec2::new(x, 1.0), Vec2::new(x, 8.0)], hard(2.0)));
    assert_eq!(fixture.coverage_at(Vec2::new(x, y)), 0, "erased");
    assert_eq!(fixture.coverage_at(Vec2::new(x + 2.0, y)), 255, "beyond it");

    fixture.paint(
        stroke(
            &[Vec2::new(x, 2.0), Vec2::new(x, 7.0)],
            brush(1.0, 1.0, 1.0),
        ),
        None,
    );
    assert_eq!(
        fixture.coverage_at(Vec2::new(x, y)),
        255,
        "laid after the erase, on its path"
    );
    assert_eq!(
        fixture.coverage_at(Vec2::new(x, 6.0 + HALF_PIXEL)),
        255,
        "laid after the erase, where only it paints"
    );

    let tiles = fixture.tiles();
    let (_, element, _) = fixture.terrain();
    fixture.erase(erasing(
        &[Vec2::new(40.0, 30.0), Vec2::new(60.0, 50.0)],
        hard(6.0),
    ));
    fixture.erase(erasing(&[Vec2::new(12.0, 9.0)], hard(3.0)));
    assert_eq!(fixture.tiles(), tiles, "no tile added and none changed");
    assert_ne!(
        fixture.terrain().1,
        element,
        "the box still holds every stroke, erases included"
    );
}

/// A Paint that erases on a Layer with no Terrain is answered with the reason, makes no Terrain,
/// and records no history step, whether it names an image or none.
#[test]
fn nothing_to_erase() {
    let mut fixture = Fixture::new();
    let erase = erasing(&[Vec2::new(2.0, 2.0)], SOFT);
    for place in [Some(FLAGSTONES), None] {
        let refused = fixture.try_paint(erase.clone(), place);
        assert_eq!(refused.len(), 1, "{refused:?}");
        assert!(refused[0].contains("nothing to erase"), "{}", refused[0]);
    }
    fixture.prop(Vec2::new(3.0, 3.0));
    let refused = fixture.try_paint(erase, None);
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert!(fixture.terrains().is_empty(), "no Terrain is made");
    assert_eq!(
        fixture.references().assets.len(),
        1,
        "only the table is recorded"
    );
    assert_eq!(fixture.steps(), 1, "the Prop alone");
}

/// A Paint that erases adds its stroke to the topmost Terrain on its Layer whatever image it
/// names, or none.
#[test]
fn an_erase_needs_no_image() {
    let mut fixture = Fixture::new();
    fixture.flagstones(stroke(&[Vec2::new(1.0, 1.0), Vec2::new(8.0, 1.0)], SOFT));
    let other = erasing(&[Vec2::new(3.0, 1.0)], SOFT);
    let none = erasing(&[Vec2::new(5.0, 1.0)], SOFT);
    fixture.paint(other.clone(), Some(GRASS));
    fixture.paint(none.clone(), None);

    let (_, _, terrain) = fixture.terrain();
    assert_eq!(terrain.strokes[1..], [other, none]);
    let references = fixture.references();
    assert_eq!(references.assets.len(), 1, "the grass is never recorded");
    assert_eq!(references.assets[0].name, "flagstones");
    assert_eq!(fixture.steps(), 3);
}

/// A Paint that erases appends its stroke, whether it erases with it, at the end of the
/// Terrain's strokes as one history step; undo takes exactly that stroke away and redo puts it
/// back at the same place in the order.
#[test]
fn an_erase_is_a_stroke() {
    let mut fixture = Fixture::new();
    let first = stroke(&[Vec2::new(1.0, 1.0), Vec2::new(6.0, 1.0)], SOFT);
    let erase = erasing(
        &[Vec2::new(3.0, 0.0), Vec2::new(3.0, 3.0)],
        brush(1.0, 0.5, 0.7),
    );
    fixture.flagstones(first.clone());
    fixture.erase(erase.clone());
    assert_eq!(fixture.strokes(), vec![first.clone(), erase.clone()]);
    assert_eq!(fixture.steps(), 2);
    let tiles = fixture.tiles();

    fixture.undo();
    assert_eq!(fixture.strokes(), vec![first.clone()]);
    fixture.redo();
    assert_eq!(fixture.strokes(), vec![first, erase]);
    assert_eq!(fixture.tiles(), tiles);
    assert_eq!(fixture.steps(), 2);
}

/// Four strokes over one another, the third an erase, laid on a fresh Terrain, with the
/// Terrain's identity.
fn four_strokes(fixture: &mut Fixture) -> (ElementId, Vec<Stroke>) {
    let strokes = vec![
        stroke(&[Vec2::new(1.0, 2.0), Vec2::new(9.0, 2.0)], SOFT),
        stroke(
            &[
                Vec2::new(2.0, 0.0),
                Vec2::new(4.0, 5.0),
                Vec2::new(7.0, 1.0),
            ],
            brush(1.5, 0.8, 0.6),
        ),
        erasing(
            &[Vec2::new(5.0, -1.0), Vec2::new(6.0, 6.0)],
            brush(1.0, 0.5, 1.0),
        ),
        stroke(&[Vec2::new(6.0, 3.0)], brush(3.0, 0.0, 0.9)),
    ];
    fixture.flagstones(strokes[0].clone());
    for later in &strokes[1..] {
        fixture.paint(later.clone(), None);
    }
    assert_eq!(fixture.strokes(), strokes);
    (fixture.terrain().0, strokes)
}

/// A Terrain's first-laid stroke is its first: moving a stroke or a point of its path, changing
/// its Brush settings, and turning it to painting or erasing change no stroke's number or place
/// in the order, and removing a stroke numbers each later stroke one lower.
#[test]
fn strokes_are_numbered() {
    let mut fixture = Fixture::new();
    let (terrain, mut strokes) = four_strokes(&mut fixture);

    fixture.edit(
        terrain,
        edit_of(
            1,
            StrokeChange::Point {
                index: 2,
                position: Vec2::new(8.0, 0.0),
            },
        ),
    );
    strokes[1].points[2] = Vec2::new(8.0, 0.0);
    fixture.edit(
        terrain,
        edit_of(3, StrokeChange::Position(Vec2::new(10.0, 3.0))),
    );
    strokes[3].points[0] = Vec2::new(10.0, 3.0);
    fixture.edit(
        terrain,
        edit_of(0, StrokeChange::Brush(brush(2.5, 0.25, 0.75))),
    );
    strokes[0].brush = brush(2.5, 0.25, 0.75);
    fixture.edit(terrain, edit_of(2, StrokeChange::Erase(false)));
    strokes[2].erase = false;
    assert_eq!(fixture.strokes(), strokes, "every stroke at its number");

    fixture.edit(terrain, edit_of(1, StrokeChange::Remove));
    strokes.remove(1);
    assert_eq!(fixture.strokes(), strokes, "each later one a number lower");
    fixture.edit(
        terrain,
        edit_of(
            1,
            StrokeChange::Point {
                index: 0,
                position: Vec2::new(5.5, -2.0),
            },
        ),
    );
    strokes[1].points[0] = Vec2::new(5.5, -2.0);
    assert_eq!(
        fixture.strokes(),
        strokes,
        "the third stroke is now addressed as the second"
    );
}

/// An Edit Element moving a point of a stroke's path changes that point and nothing else: every
/// other point, the stroke's Brush settings, whether it erases, and every other stroke stay as
/// they were.
#[test]
fn a_strokes_point_moves_alone() {
    let mut fixture = Fixture::new();
    let (terrain, mut strokes) = four_strokes(&mut fixture);
    let moved = Vec2::new(3.5, 7.0);

    fixture.edit(
        terrain,
        edit_of(
            1,
            StrokeChange::Point {
                index: 1,
                position: moved,
            },
        ),
    );

    strokes[1].points[1] = moved;
    assert_eq!(fixture.strokes(), strokes);
    assert_eq!(
        fixture.coverage_at(Vec2::new(3.5 + HALF_PIXEL, 7.0 + HALF_PIXEL)),
        byte(0.6),
        "the stroke reaches its moved point"
    );
}

/// An Edit Element moving a stroke to a position moves every point of its path by the
/// difference between that position and the centre of the smallest box around its points,
/// keeping its Brush settings, whether it erases, and its place in the order; its ground goes
/// with it, and an erase moved still takes from a stroke laid before it.
#[test]
fn moving_a_stroke_moves_its_path() {
    let mut fixture = Fixture::new();
    let y = 4.0 + HALF_PIXEL;
    let first = stroke(&[Vec2::new(1.0, y), Vec2::new(20.0, y)], hard(2.0));
    let x = 4.0 + HALF_PIXEL;
    let erase = erasing(&[Vec2::new(x, 1.0), Vec2::new(x, 7.0)], hard(2.0));
    let dab = stroke(
        &[Vec2::new(10.0, 10.0), Vec2::new(12.0, 13.0)],
        brush(1.0, 1.0, 0.8),
    );
    fixture.flagstones(first.clone());
    fixture.erase(erase.clone());
    fixture.paint(dab.clone(), None);
    let (terrain, ..) = fixture.terrain();
    assert_eq!(dab.centre(), Vec2::new(11.0, 11.5));

    fixture.edit(
        terrain,
        edit_of(2, StrokeChange::Position(Vec2::new(15.0, 9.5))),
    );
    let mut moved = dab.clone();
    moved.points = vec![Vec2::new(14.0, 8.0), Vec2::new(16.0, 11.0)];
    assert_eq!(fixture.strokes(), vec![first.clone(), erase.clone(), moved]);
    assert_eq!(
        fixture.coverage_at(Vec2::new(10.0, 10.0)),
        0,
        "its old place"
    );
    assert_eq!(
        fixture.coverage_at(Vec2::new(14.0, 8.0)),
        byte(0.8),
        "its new place"
    );

    fixture.edit(
        terrain,
        edit_of(1, StrokeChange::Position(Vec2::new(x + 6.0, 4.0))),
    );
    assert!(
        fixture.strokes()[1].erase,
        "still an erase, still the second"
    );
    assert_eq!(
        fixture.coverage_at(Vec2::new(x, y)),
        255,
        "the ground back where the erase was"
    );
    assert_eq!(
        fixture.coverage_at(Vec2::new(x + 6.0, y)),
        0,
        "taken from the stroke laid before it where it is"
    );
}

/// An Edit Element setting a stroke's Brush settings changes its size, hardness, and strength
/// and nothing else of it or of any other stroke.
#[test]
fn a_strokes_brush_stays_editable() {
    let mut fixture = Fixture::new();
    let (terrain, mut strokes) = four_strokes(&mut fixture);
    let wider = brush(4.0, 1.0, 0.5);

    fixture.edit(terrain, edit_of(0, StrokeChange::Brush(wider)));

    strokes[0].brush = wider;
    assert_eq!(fixture.strokes(), strokes);
    assert_eq!(
        fixture.coverage_at(Vec2::new(1.0 + HALF_PIXEL, 3.75 + HALF_PIXEL)),
        byte(0.5),
        "the wider, weaker stroke where the old one did not reach"
    );
}

/// An Edit Element turning a stroke to erasing, or an erase to painting, changes that and
/// nothing else of it or of any other stroke, each as one step; one naming what the stroke
/// already does changes nothing and records no step.
#[test]
fn painting_or_erasing_stays_editable() {
    let mut fixture = Fixture::new();
    let y = 4.0 + HALF_PIXEL;
    let x = 6.0 + HALF_PIXEL;
    fixture.flagstones(stroke(&[Vec2::new(1.0, y), Vec2::new(12.0, y)], hard(2.0)));
    fixture.paint(
        stroke(&[Vec2::new(x, 1.0), Vec2::new(x, 8.0)], hard(1.0)),
        None,
    );
    let (terrain, _, before) = fixture.terrain();
    let steps = fixture.steps();

    fixture.edit(terrain, edit_of(1, StrokeChange::Erase(true)));
    let mut erased = before.strokes.clone();
    erased[1].erase = true;
    assert_eq!(fixture.strokes(), erased);
    assert_eq!(fixture.coverage_at(Vec2::new(x, y)), 0, "now it erases");
    assert_eq!(fixture.steps(), steps + 1);

    fixture.edit(terrain, edit_of(1, StrokeChange::Erase(true)));
    assert_eq!(fixture.strokes(), erased, "already an erase");
    assert_eq!(
        fixture.steps(),
        steps + 1,
        "no step for what it already does"
    );

    fixture.edit(terrain, edit_of(1, StrokeChange::Erase(false)));
    assert_eq!(fixture.terrain().2, before);
    assert_eq!(fixture.coverage_at(Vec2::new(x, 7.0)), 255, "paints again");
    assert_eq!(fixture.steps(), steps + 2);
}

/// Moving a point, moving a stroke, setting its Brush settings, turning it to painting or
/// erasing, and removing it are each one history step on their own, and a gesture of moves or
/// of Brush settings from its beginning to its end is one step however long, which undo returns
/// to where the gesture began.
#[test]
fn a_stroke_edit_is_one_step() {
    let mut fixture = Fixture::new();
    let (terrain, strokes) = four_strokes(&mut fixture);
    let steps = fixture.steps();

    fixture.gesture(
        terrain,
        (0..5_u8)
            .map(|step| {
                edit_of(
                    1,
                    StrokeChange::Point {
                        index: 0,
                        position: Vec2::new(2.0, -f32::from(step)),
                    },
                )
            })
            .collect(),
    );
    assert_eq!(fixture.strokes()[1].points[0], Vec2::new(2.0, -4.0));
    assert_eq!(fixture.steps(), steps + 1, "a drag of a point");
    fixture.undo();
    assert_eq!(fixture.strokes(), strokes);

    fixture.gesture(
        terrain,
        (0..5_u8)
            .map(|step| {
                edit_of(
                    0,
                    StrokeChange::Position(Vec2::new(5.0 + f32::from(step), 2.0)),
                )
            })
            .collect(),
    );
    assert_eq!(fixture.strokes()[0].centre(), Vec2::new(9.0, 2.0));
    assert_eq!(fixture.steps(), steps + 1, "a drag of a stroke");
    fixture.undo();
    assert_eq!(fixture.strokes(), strokes);

    fixture.gesture(
        terrain,
        (1..=5_u8)
            .map(|step| edit_of(3, StrokeChange::Brush(brush(f32::from(step), 0.0, 0.9))))
            .collect(),
    );
    assert_eq!(fixture.strokes()[3].brush, brush(5.0, 0.0, 0.9));
    assert_eq!(fixture.steps(), steps + 1, "a drag of the size");
    fixture.undo();
    assert_eq!(fixture.strokes(), strokes);

    for change in [
        edit_of(
            0,
            StrokeChange::Point {
                index: 1,
                position: Vec2::new(9.0, 3.0),
            },
        ),
        edit_of(1, StrokeChange::Position(Vec2::new(4.0, 4.0))),
        edit_of(2, StrokeChange::Brush(brush(2.0, 0.5, 0.5))),
        edit_of(3, StrokeChange::Erase(true)),
        edit_of(0, StrokeChange::Remove),
    ] {
        let before = fixture.strokes();
        fixture.edit(terrain, change.clone());
        assert_eq!(fixture.steps(), steps + 1, "{change:?} on its own");
        fixture.undo();
        assert_eq!(fixture.strokes(), before, "{change:?} undone");
    }
}

/// An Edit Element removing a stroke takes it out of its Terrain's strokes, leaving every other
/// stroke and their order as they were, as one history step that undo returns to the same place
/// in the order, exactly as it was.
#[test]
fn removing_a_stroke_keeps_the_rest() {
    let mut fixture = Fixture::new();
    let (terrain, strokes) = four_strokes(&mut fixture);
    let tiles = fixture.tiles();
    let steps = fixture.steps();

    for removed in [1, 2] {
        fixture.edit(terrain, edit_of(removed, StrokeChange::Remove));
        let mut rest = strokes.clone();
        rest.remove(removed);
        assert_eq!(fixture.strokes(), rest, "stroke {removed} taken out");
        assert_eq!(fixture.steps(), steps + 1);
        fixture.undo();
        assert_eq!(
            fixture.strokes(),
            strokes,
            "stroke {removed} back at its place"
        );
        assert_eq!(fixture.tiles(), tiles);
    }

    let x = 5.5 + HALF_PIXEL;
    fixture.edit(terrain, edit_of(2, StrokeChange::Remove));
    assert_eq!(
        fixture.coverage_at(Vec2::new(x, 2.0 + HALF_PIXEL)),
        255,
        "the ground the erase took comes back"
    );
}

/// Removing the only stroke of a Terrain removes the Terrain instead, as one history step that
/// undo returns with its identity, its stroke, its place in the stacking order, and its
/// coverage.
#[test]
fn the_last_stroke_takes_its_terrain() {
    let mut fixture = Fixture::new();
    let prop = fixture.prop(Vec2::new(3.0, 3.0));
    fixture.flagstones(stroke(&[Vec2::new(1.0, 1.0), Vec2::new(6.0, 4.0)], SOFT));
    let terrain = fixture.terrain();
    let tiles = fixture.tiles();
    let wall = fixture.wall(&[Vec2::new(0.0, 6.0), Vec2::new(5.0, 6.0)]);
    let steps = fixture.steps();

    fixture.edit(terrain.0, edit_of(0, StrokeChange::Remove));
    assert!(fixture.terrains().is_empty(), "the Terrain goes");
    assert_eq!(fixture.order(), vec![prop, wall]);
    assert_eq!(fixture.steps(), steps + 1);

    fixture.undo();
    assert_eq!(fixture.terrain(), terrain);
    assert_eq!(fixture.order(), vec![terrain.0, prop, wall]);
    assert_eq!(fixture.tiles(), tiles);
    fixture.redo();
    assert!(fixture.terrains().is_empty(), "gone again");
}

/// An Edit Element naming a stroke the Terrain does not have or a point its path does not have,
/// moving a point or a stroke where it is not finite, or setting Brush settings that are not a
/// Brush's, and a stroke change sent for an Element that is not a Terrain, are answered with the
/// reason, change nothing, and record no history step.
#[test]
fn malformed_stroke_edits_are_refused() {
    let mut fixture = Fixture::new();
    let (terrain, _) = four_strokes(&mut fixture);
    let prop = fixture.prop(Vec2::new(12.0, 12.0));
    let before = fixture.terrain();
    let tiles = fixture.tiles();
    let steps = fixture.steps();
    let every = |stroke: usize| {
        [
            edit_of(
                stroke,
                StrokeChange::Point {
                    index: 0,
                    position: Vec2::ONE,
                },
            ),
            edit_of(stroke, StrokeChange::Position(Vec2::ONE)),
            edit_of(stroke, StrokeChange::Brush(SOFT)),
            edit_of(stroke, StrokeChange::Erase(true)),
            edit_of(stroke, StrokeChange::Remove),
        ]
    };
    let mut cases: Vec<(ElementId, ElementChange, &str)> = every(4)
        .into_iter()
        .map(|change| (terrain, change, "stroke 4"))
        .collect();
    cases.extend(
        every(0)
            .into_iter()
            .map(|change| (prop, change, "not a Terrain")),
    );
    cases.extend([
        (
            terrain,
            edit_of(
                1,
                StrokeChange::Point {
                    index: 3,
                    position: Vec2::ONE,
                },
            ),
            "point 3",
        ),
        (
            terrain,
            edit_of(
                1,
                StrokeChange::Point {
                    index: 0,
                    position: Vec2::new(f32::NAN, 1.0),
                },
            ),
            "finite",
        ),
        (
            terrain,
            edit_of(0, StrokeChange::Position(Vec2::new(1.0, f32::INFINITY))),
            "finite",
        ),
    ]);
    for (bad, reason) in [
        (brush(0.0, 0.5, 1.0), "size"),
        (brush(-1.0, 0.5, 1.0), "size"),
        (brush(f32::NAN, 0.5, 1.0), "size"),
        (brush(f32::INFINITY, 0.5, 1.0), "size"),
        (brush(1.0, -0.1, 1.0), "hardness"),
        (brush(1.0, 1.5, 1.0), "hardness"),
        (brush(1.0, 0.5, 0.0), "strength"),
        (brush(1.0, 0.5, 1.5), "strength"),
    ] {
        cases.push((terrain, edit_of(2, StrokeChange::Brush(bad)), reason));
    }

    for (element, change, reason) in cases {
        let refused = fixture.try_edit(element, change.clone());
        assert_eq!(refused.len(), 1, "{change:?}");
        assert!(refused[0].contains(reason), "{change:?}: {}", refused[0]);
    }
    assert_eq!(fixture.terrain(), before);
    assert_eq!(fixture.tiles(), tiles);
    assert_eq!(fixture.steps(), steps);
}

/// One edit of each kind on the strokes [`four_strokes`] lays, each over others.
fn one_of_each() -> [ElementChange; 5] {
    [
        edit_of(
            1,
            StrokeChange::Point {
                index: 1,
                position: Vec2::new(5.0, 3.0),
            },
        ),
        edit_of(2, StrokeChange::Position(Vec2::new(3.0, 2.0))),
        edit_of(0, StrokeChange::Brush(brush(3.0, 0.1, 0.5))),
        edit_of(3, StrokeChange::Erase(true)),
        edit_of(2, StrokeChange::Remove),
    ]
}

/// After undoing a stroke's edit or its removal, every point of its Terrain has the coverage it
/// had before.
#[test]
fn an_edit_leaves_no_trace() {
    let mut fixture = Fixture::new();
    let (terrain, _) = four_strokes(&mut fixture);
    let before = fixture.tiles();

    for change in one_of_each() {
        fixture.edit(terrain, change.clone());
        assert_ne!(fixture.tiles(), before, "{change:?} shows");
        fixture.undo();
        assert_eq!(fixture.tiles(), before, "{change:?} undone");
    }
}

/// A Terrain's coverage is the same however its strokes came to be: painted, erased, edited,
/// and removed, or painted afresh as they ended, and after saving and opening.
#[test]
fn edited_coverage_is_the_strokes_alone() {
    let mut fixture = Fixture::new();
    let (terrain, _) = four_strokes(&mut fixture);
    fixture.erase(erasing(
        &[Vec2::new(0.0, 0.0), Vec2::new(10.0, 4.0)],
        brush(1.0, 0.0, 0.7),
    ));
    for change in one_of_each() {
        fixture.edit(terrain, change);
    }
    fixture.undo();
    fixture.redo();
    let strokes = fixture.strokes();
    let tiles = fixture.tiles();

    let mut afresh = Fixture::new();
    afresh.flagstones(strokes[0].clone());
    for stroke in &strokes[1..] {
        afresh.paint(stroke.clone(), None);
    }
    assert_eq!(afresh.strokes(), strokes);
    assert_eq!(afresh.tiles(), tiles, "painted afresh");

    fixture.save_and_open();
    assert_eq!(fixture.tiles(), tiles, "opened from the file");
}

/// Redo carries a stroke's edit or removal out exactly as it was first applied: the same
/// strokes and the same coverage.
#[test]
fn stroke_edits_redo_exactly() {
    let mut fixture = Fixture::new();
    let (terrain, _) = four_strokes(&mut fixture);
    let mut after = Vec::new();
    for change in one_of_each() {
        fixture.edit(terrain, change);
        after.push((fixture.terrain(), fixture.tiles()));
    }
    for _ in 0..after.len() {
        fixture.undo();
    }
    for (step, (terrain, tiles)) in after.into_iter().enumerate() {
        fixture.redo();
        assert_eq!(fixture.terrain(), terrain, "step {step}");
        assert_eq!(fixture.tiles(), tiles, "step {step}");
    }
}

/// Stroke edits are steps in the one history, among Paint, Place, and Edit Element, undone and
/// redone in the order they were taken.
#[test]
fn stroke_edits_share_the_history() {
    let mut fixture = Fixture::new();
    let first = stroke(&[Vec2::new(1.0, 1.0), Vec2::new(5.0, 1.0)], SOFT);
    fixture.flagstones(first.clone());
    let (terrain, ..) = fixture.terrain();
    let prop = fixture.prop(Vec2::new(3.0, 3.0));
    fixture.edit(
        terrain,
        edit_of(0, StrokeChange::Position(Vec2::new(3.0, 4.0))),
    );
    fixture.edit(prop, ElementChange::Position(Vec2::new(8.0, 8.0)));
    let erase = erasing(&[Vec2::new(3.0, 4.0)], SOFT);
    fixture.erase(erase.clone());
    fixture.edit(terrain, edit_of(0, StrokeChange::Remove));
    let moved = stroke(&[Vec2::new(1.0, 4.0), Vec2::new(5.0, 4.0)], SOFT);
    assert_eq!(fixture.strokes(), vec![erase.clone()]);

    fixture.undo();
    assert_eq!(fixture.strokes(), vec![moved.clone(), erase]);
    fixture.undo();
    assert_eq!(fixture.strokes(), vec![moved.clone()]);
    fixture.undo();
    assert_eq!(fixture.order(), vec![terrain, prop]);
    fixture.undo();
    assert_eq!(fixture.strokes(), vec![first.clone()]);
    fixture.undo();
    assert_eq!(fixture.order(), vec![terrain]);

    for _ in 0..3 {
        fixture.redo();
    }
    assert_eq!(fixture.strokes(), vec![moved]);
}
