//! Drawing and editing Walls through the headless editor: the real plugins of the model, the
//! history, `LibraryAccess`, `LibraryManager`, `ProjectManager`, and `AuthoringManager`, with no
//! window and no render Engine, over one Asset Folder of a single image for the Props a Wall is
//! stacked with, driven by Apply, Undo, and Redo messages and asserted on the Wall, its derived
//! shape, the Element's box, the Layer's children, and the history.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::float_cmp,
    reason = "a test and its fixtures stop at the first thing that is not as expected, and the \
              geometry asserted on is exact"
)]

mod support;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::math::{UVec2, Vec2};
use drs_history::History;
use drs_model::{
    Apply, AssetAddress, Bounds, Colour, EditElement, Element, ElementChange, ElementId, FolderKey,
    Gesture, PlaceElement, Placement, Prop, RemoveElement, Segment, WALL, Wall, WallShape,
};
use support::quadratic;
use tempfile::TempDir;

/// The place of the one image in the fixture folder.
const TABLE: &str = "table.png";
/// A dark grey.
const GREY: Colour = Colour::rgb(60, 60, 60);
/// A red.
const RED: Colour = Colour::rgb(200, 30, 30);
/// The flattening tolerance the shape is derived at, in cells.
const TOLERANCE: f32 = 0.001;

/// The headless editor with the fixture folder added.
struct Fixture {
    /// Keeps the temporary directory alive for the test.
    _root: TempDir,
    /// The folder's key once added.
    key: FolderKey,
    /// The headless editor.
    app: App,
}

/// What the World holds about one Element on the Layer.
#[derive(Debug, Clone, PartialEq)]
struct Placed {
    /// Its identity.
    id: ElementId,
    /// Its kind and box.
    element: Element,
    /// The Wall, when it is one.
    wall: Option<Wall>,
    /// The Wall's derived shape, once derived.
    shape: Option<WallShape>,
    /// The Prop, when it is one.
    prop: Option<Prop>,
}

impl Placed {
    /// The Wall, which it must be.
    fn wall(&self) -> &Wall {
        self.wall.as_ref().expect("the Element is a Wall")
    }

    /// The derived shape, which it must have.
    fn shape(&self) -> &WallShape {
        self.shape.as_ref().expect("the Wall has its shape")
    }
}

impl Fixture {
    /// Creates the fixture folder, starts the editor, and adds the folder.
    fn new() -> Self {
        let root = TempDir::new().expect("temporary root");
        let folder = root.path().join("fixtures");
        support::png(&folder, TABLE, UVec2::splat(256), [120, 80, 40, 255]);
        let mut app = support::editor(root.path());
        let key = support::add_folder(&mut app, &folder, "Fixtures").key;
        Self {
            _root: root,
            key,
            app,
        }
    }

    /// The one Layer of the new Project.
    fn layer(&mut self) -> Entity {
        support::first_layer(&mut self.app)
    }

    /// Sends a Command and runs one update, returning the reasons of any failure.
    fn try_apply(&mut self, command: Apply) -> Vec<String> {
        support::try_apply(&mut self.app, command)
    }

    /// Sends a Command and runs one update, failing the test if the Command was refused.
    fn apply(&mut self, command: Apply) {
        support::apply(&mut self.app, command);
    }

    /// The Place Element Command for a Wall through `points`.
    fn wall_placement(&mut self, points: &[Vec2], thickness: f32, colour: Colour) -> Apply {
        Apply::PlaceElement(PlaceElement {
            layer: self.layer(),
            placement: Placement::Wall {
                points: points.to_vec(),
                thickness,
                colour,
            },
        })
    }

    /// Places a grey Wall an eighth of a cell thick through `points`, returning its identity.
    fn wall(&mut self, points: &[Vec2]) -> ElementId {
        let command = self.wall_placement(points, 0.125, GREY);
        self.apply(command);
        self.elements()
            .last()
            .map(|placed| placed.id)
            .expect("the placed Wall is the last child of the Layer")
    }

    /// Places a Prop of the table centred on `position`, returning its identity.
    fn prop(&mut self, position: Vec2) -> ElementId {
        let layer = self.layer();
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Prop {
                position,
                asset: AssetAddress {
                    folder: self.key.clone(),
                    place: TABLE.to_owned(),
                },
            },
        }));
        self.elements()
            .last()
            .map(|placed| placed.id)
            .expect("the placed Prop is the last child of the Layer")
    }

    /// Changes an Element on its own.
    fn edit(&mut self, element: ElementId, change: ElementChange) {
        self.gesture(element, change, Gesture::Single);
    }

    /// Changes an Element as part of a gesture.
    fn gesture(&mut self, element: ElementId, change: ElementChange, gesture: Gesture) {
        self.apply(support::edit(element, change, gesture));
    }

    /// Sends Undo and runs one update.
    fn undo(&mut self) {
        support::undo(&mut self.app);
    }

    /// Sends Redo and runs one update.
    fn redo(&mut self) {
        support::redo(&mut self.app);
    }

    /// The history.
    fn history(&self) -> &History {
        support::history(&self.app)
    }

    /// The Elements on the Layer in stacking order, bottom first.
    fn elements(&mut self) -> Vec<Placed> {
        let layer = self.layer();
        let world = self.app.world_mut();
        let children: Vec<Entity> = world
            .get::<Children>(layer)
            .map(|children| children.iter().copied().collect())
            .unwrap_or_default();
        children
            .into_iter()
            .map(|entity| Placed {
                id: *world
                    .get::<ElementId>(entity)
                    .expect("an Element has an identity"),
                element: world
                    .get::<Element>(entity)
                    .expect("a child is an Element")
                    .clone(),
                wall: world.get::<Wall>(entity).cloned(),
                shape: world.get::<WallShape>(entity).cloned(),
                prop: world.get::<Prop>(entity).cloned(),
            })
            .collect()
    }

    /// The Element with an identity.
    fn element(&mut self, id: ElementId) -> Placed {
        self.elements()
            .into_iter()
            .find(|placed| placed.id == id)
            .expect("the Element is on the Layer")
    }

    /// The Wall with an identity.
    fn wall_of(&mut self, id: ElementId) -> Wall {
        self.element(id).wall().clone()
    }
}

/// The distance from `p` to the nearest chord of a flattened line.
fn to_line(p: Vec2, shape: &WallShape) -> f32 {
    shape
        .line
        .windows(2)
        .map(|pair| {
            let (a, b) = (pair[0].position, pair[1].position);
            let along = b - a;
            let t = ((p - a).dot(along) / along.length_squared()).clamp(0.0, 1.0);
            p.distance(a + along * t)
        })
        .fold(f32::INFINITY, f32::min)
}

/// The segment numbers the derived line runs through, in order, each once.
fn segments_of(shape: &WallShape) -> Vec<usize> {
    let mut segments: Vec<usize> = shape.line.iter().map(|point| point.segment).collect();
    segments.dedup();
    segments
}

/// A Wall is an ordered list of two or more points in Grid cells with a segment between each
/// point and the next, each segment either straight or curved by one control point, a thickness
/// in cells, and an opaque colour.
#[test]
fn a_wall_is_its_points() {
    let mut fixture = Fixture::new();
    let points = [
        Vec2::new(1.0, 1.0),
        Vec2::new(5.0, 1.0),
        Vec2::new(5.0, 4.5),
    ];
    let command = fixture.wall_placement(&points, 0.25, RED);
    fixture.apply(command);
    let id = fixture.elements()[0].id;
    fixture.edit(
        id,
        ElementChange::Control {
            segment: 1,
            position: Some(Vec2::new(7.0, 3.0)),
        },
    );

    let placed = fixture.element(id);
    assert_eq!(placed.element.kind, WALL);
    assert_eq!(
        placed.wall(),
        &Wall {
            points: points.to_vec(),
            segments: vec![
                Segment { control: None },
                Segment {
                    control: Some(Vec2::new(7.0, 3.0))
                }
            ],
            thickness: 0.25,
            colour: RED,
        }
    );
    assert_eq!(placed.prop, None);
}

/// The segment from the first point to the second is the first, and so on; moving the Wall, a
/// point, or a control point, bending or straightening a segment, and changing the thickness or
/// the colour change no segment's number.
#[test]
fn segments_are_numbered() {
    let mut fixture = Fixture::new();
    let id = fixture.wall(&[
        Vec2::ZERO,
        Vec2::new(4.0, 0.0),
        Vec2::new(4.0, 4.0),
        Vec2::new(0.0, 4.0),
    ]);
    let bend = Vec2::new(6.0, 2.0);
    let changes = [
        ElementChange::Control {
            segment: 1,
            position: Some(bend),
        },
        ElementChange::Position(Vec2::new(10.0, 10.0)),
        ElementChange::Point {
            index: 3,
            position: Vec2::new(8.0, 13.0),
        },
        ElementChange::Control {
            segment: 1,
            position: Some(Vec2::new(15.0, 10.0)),
        },
        ElementChange::Thickness(0.5),
        ElementChange::Colour(RED),
        ElementChange::Control {
            segment: 0,
            position: Some(Vec2::new(10.0, 7.0)),
        },
        ElementChange::Control {
            segment: 0,
            position: None,
        },
    ];
    for change in changes {
        fixture.edit(id, change.clone());

        let placed = fixture.element(id);
        let (wall, shape) = (placed.wall(), placed.shape());
        assert_eq!(wall.segments.len(), 3, "{change:?}");
        assert!(wall.segments[1].control.is_some(), "{change:?}");
        assert_eq!(wall.segments[2].control, None, "{change:?}");
        assert_eq!(segments_of(shape), vec![0, 1, 2], "{change:?}");
        for (number, start) in wall.points[..3].iter().enumerate() {
            let first = shape
                .line
                .iter()
                .find(|point| point.segment == number)
                .expect("each segment has points");
            assert_eq!(first.position, *start, "segment {number} after {change:?}");
            assert_eq!(first.t, 0.0);
        }
        let last = shape.line.last().expect("the line has an end");
        assert_eq!(
            (last.position, last.segment, last.t),
            (wall.points[3], 2, 1.0)
        );
    }
}

/// A curved segment is the quadratic Bézier curve from its first point to its second point with
/// its control point; a segment without a control point is the straight line between its points.
#[test]
fn curved_by_one_control_point() {
    let mut fixture = Fixture::new();
    let (start, end, control) = (
        Vec2::new(1.0, 2.0),
        Vec2::new(7.0, 2.0),
        Vec2::new(4.0, 8.0),
    );
    let id = fixture.wall(&[start, end]);

    let straight = fixture.element(id);
    let line: Vec<Vec2> = straight
        .shape()
        .line
        .iter()
        .map(|point| point.position)
        .collect();
    assert_eq!(line, vec![start, end]);

    fixture.edit(
        id,
        ElementChange::Control {
            segment: 0,
            position: Some(control),
        },
    );
    let curved = fixture.element(id);
    let middle = curved
        .shape()
        .line
        .iter()
        .find(|point| point.segment == 0 && point.t == 0.5)
        .expect("the middle of the curve is a point of the line");
    assert!(
        middle
            .position
            .distance(quadratic(start, control, end, 0.5))
            < 1e-5,
        "{} is not the curve's middle, (4, 5)",
        middle.position
    );
    for point in &curved.shape().line {
        let on_curve = quadratic(start, control, end, point.t);
        assert!(point.position.distance(on_curve) < 1e-5);
    }
}

/// A Wall's position is the centre of the smallest box around its points and control points,
/// and its size is that box grown by half the thickness on every side.
#[test]
fn the_box_follows_the_points() {
    let mut fixture = Fixture::new();
    let id = fixture.wall(&[Vec2::new(1.0, 1.0), Vec2::new(5.0, 2.0)]);
    let element = fixture.element(id).element;
    assert_eq!(element.position, Vec2::new(3.0, 1.5));
    assert_eq!(element.size, Vec2::new(4.125, 1.125));

    fixture.edit(
        id,
        ElementChange::Point {
            index: 1,
            position: Vec2::new(9.0, 1.0),
        },
    );
    let element = fixture.element(id).element;
    assert_eq!(element.position, Vec2::new(5.0, 1.0));
    assert_eq!(element.size, Vec2::new(8.125, 0.125));

    fixture.edit(
        id,
        ElementChange::Control {
            segment: 0,
            position: Some(Vec2::new(4.0, -3.0)),
        },
    );
    fixture.edit(id, ElementChange::Thickness(1.0));
    let element = fixture.element(id).element;
    assert_eq!(element.position, Vec2::new(5.0, -1.0));
    assert_eq!(element.size, Vec2::new(9.0, 5.0));

    fixture.undo();
    fixture.undo();
    let element = fixture.element(id).element;
    assert_eq!(element.position, Vec2::new(5.0, 1.0));
    assert_eq!(element.size, Vec2::new(8.125, 0.125));
}

/// A Place Element of a Wall puts on the given Layer a Wall with the given points, thickness,
/// and colour, every segment straight, as one history step that undo takes away whole and redo
/// brings back whole.
#[test]
fn placed_as_one_step() {
    let mut fixture = Fixture::new();
    let depth = fixture.history().undo_depth();
    let points = [Vec2::ZERO, Vec2::new(3.0, 0.0), Vec2::new(3.0, 2.0)];

    let id = fixture.wall(&points);
    let placed = fixture.element(id);
    assert_eq!(placed.wall(), &Wall::straight(points.to_vec(), 0.125, GREY));
    assert!(
        !placed.shape().mesh.indices.is_empty(),
        "the shape is derived in the frame the Wall is placed"
    );
    assert_eq!(fixture.history().undo_depth(), depth + 1);

    fixture.undo();
    assert!(fixture.elements().is_empty());

    fixture.redo();
    assert_eq!(fixture.elements(), vec![placed]);
}

/// An Edit Element that changes a Wall's position moves every point and control point by the
/// same amount.
#[test]
fn moving_the_wall_moves_every_point() {
    let mut fixture = Fixture::new();
    let id = fixture.wall(&[Vec2::ZERO, Vec2::new(4.0, 0.0), Vec2::new(4.0, 2.0)]);
    fixture.edit(
        id,
        ElementChange::Control {
            segment: 0,
            position: Some(Vec2::new(2.0, -2.0)),
        },
    );
    let before = fixture.wall_of(id);
    let centre = fixture.element(id).element.position;
    let offset = Vec2::new(-7.5, 3.25);

    fixture.edit(id, ElementChange::Position(centre + offset));

    let after = fixture.wall_of(id);
    let moved: Vec<Vec2> = before.points.iter().map(|point| *point + offset).collect();
    assert_eq!(after.points, moved);
    assert_eq!(
        after.segments[0].control,
        Some(Vec2::new(2.0, -2.0) + offset)
    );
    assert_eq!(after.segments[1].control, None);
    assert_eq!(fixture.element(id).element.position, centre + offset);
}

/// Moving a point changes that point and nothing else; every other point and every control
/// point stays where it was, so the two segments meeting at the point keep their curves.
#[test]
fn a_point_moves_alone() {
    let mut fixture = Fixture::new();
    let id = fixture.wall(&[
        Vec2::ZERO,
        Vec2::new(4.0, 0.0),
        Vec2::new(8.0, 0.0),
        Vec2::new(8.0, 4.0),
    ]);
    for (segment, control) in [(0, Vec2::new(2.0, 2.0)), (1, Vec2::new(6.0, -2.0))] {
        fixture.edit(
            id,
            ElementChange::Control {
                segment,
                position: Some(control),
            },
        );
    }
    let before = fixture.wall_of(id);

    fixture.edit(
        id,
        ElementChange::Point {
            index: 1,
            position: Vec2::new(4.5, 3.0),
        },
    );

    let mut expected = before;
    expected.points[1] = Vec2::new(4.5, 3.0);
    assert_eq!(fixture.wall_of(id), expected);
    let shape = fixture.element(id).shape().clone();
    let meeting = shape
        .line
        .iter()
        .find(|point| point.segment == 1 && point.t == 0.0)
        .expect("the moved point is on the line");
    assert_eq!(meeting.position, Vec2::new(4.5, 3.0));
    assert!(
        shape.line.iter().filter(|point| point.segment < 2).count() > 4,
        "both segments meeting at the point are still curved"
    );
}

/// Dragging a point or a control point records a single undo step however long the drag, and
/// undo returns it to where the drag began.
#[test]
fn a_handle_drag_is_one_step() {
    let mut fixture = Fixture::new();
    let id = fixture.wall(&[Vec2::ZERO, Vec2::new(4.0, 0.0)]);
    let start = fixture.wall_of(id);
    let depth = fixture.history().undo_depth();

    for (position, gesture) in [
        (Vec2::new(4.5, 0.5), Gesture::Begin),
        (Vec2::new(5.0, 1.0), Gesture::Continue),
        (Vec2::new(6.0, 1.5), Gesture::Continue),
        (Vec2::new(7.0, 2.0), Gesture::End),
    ] {
        fixture.gesture(id, ElementChange::Point { index: 1, position }, gesture);
    }
    assert_eq!(fixture.wall_of(id).points[1], Vec2::new(7.0, 2.0));
    assert_eq!(fixture.history().undo_depth(), depth + 1);

    for (position, gesture) in [
        (Some(Vec2::new(2.0, 1.0)), Gesture::Begin),
        (Some(Vec2::new(2.0, 2.0)), Gesture::Continue),
        (Some(Vec2::new(3.0, 3.0)), Gesture::End),
    ] {
        fixture.gesture(
            id,
            ElementChange::Control {
                segment: 0,
                position,
            },
            gesture,
        );
    }
    assert_eq!(
        fixture.wall_of(id).segments[0].control,
        Some(Vec2::new(3.0, 3.0))
    );
    assert_eq!(fixture.history().undo_depth(), depth + 2);

    fixture.undo();
    assert_eq!(fixture.wall_of(id).segments[0].control, None);
    assert_eq!(fixture.wall_of(id).points[1], Vec2::new(7.0, 2.0));
    fixture.undo();
    assert_eq!(fixture.wall_of(id), start);
    fixture.redo();
    fixture.redo();
    assert_eq!(fixture.wall_of(id).points[1], Vec2::new(7.0, 2.0));
    assert_eq!(
        fixture.wall_of(id).segments[0].control,
        Some(Vec2::new(3.0, 3.0))
    );
}

/// Setting a segment's control point changes that control point only, and unsetting it makes
/// the segment straight; no point moves.
#[test]
fn bending_keeps_the_points() {
    let mut fixture = Fixture::new();
    let points = vec![Vec2::ZERO, Vec2::new(4.0, 0.0), Vec2::new(4.0, 4.0)];
    let id = fixture.wall(&points);

    fixture.edit(
        id,
        ElementChange::Control {
            segment: 1,
            position: Some(Vec2::new(6.0, 2.0)),
        },
    );
    let bent = fixture.wall_of(id);
    assert_eq!(bent.points, points);
    assert_eq!(
        bent.segments,
        vec![
            Segment { control: None },
            Segment {
                control: Some(Vec2::new(6.0, 2.0))
            }
        ]
    );

    fixture.edit(
        id,
        ElementChange::Control {
            segment: 1,
            position: None,
        },
    );
    assert_eq!(
        fixture.wall_of(id),
        Wall::straight(points.clone(), 0.125, GREY)
    );
    let line: Vec<Vec2> = fixture
        .element(id)
        .shape()
        .line
        .iter()
        .map(|point| point.position)
        .collect();
    assert_eq!(line, points);
}

/// A point added on a segment splits it into two segments whose joined curve is the one the
/// segment had, straight on a straight segment and curved on a curved one; the segments after it
/// are numbered one higher.
#[test]
fn adding_a_point_keeps_the_shape() {
    let mut fixture = Fixture::new();
    let id = fixture.wall(&[
        Vec2::ZERO,
        Vec2::new(4.0, 0.0),
        Vec2::new(8.0, 0.0),
        Vec2::new(8.0, 4.0),
    ]);
    let (control, after) = (Vec2::new(6.0, 3.0), Vec2::new(10.0, 2.0));
    fixture.edit(
        id,
        ElementChange::Control {
            segment: 1,
            position: Some(control),
        },
    );
    fixture.edit(
        id,
        ElementChange::Control {
            segment: 2,
            position: Some(after),
        },
    );
    let before = fixture.element(id);
    let depth = fixture.history().undo_depth();

    fixture.edit(id, ElementChange::AddPoint { segment: 1, t: 0.4 });

    let split = fixture.element(id);
    let wall = split.wall();
    assert_eq!(wall.points.len(), 5);
    assert!(
        wall.points[2].distance(quadratic(
            Vec2::new(4.0, 0.0),
            control,
            Vec2::new(8.0, 0.0),
            0.4
        )) < 1e-5,
        "the new point is on the curve"
    );
    assert_eq!(wall.segments[3].control, Some(after), "numbered one higher");
    assert!(wall.segments[1].control.is_some() && wall.segments[2].control.is_some());
    for point in &split.shape().line {
        assert!(
            to_line(point.position, before.shape()) <= 2.0 * TOLERANCE,
            "{} strays from the curve the Wall had",
            point.position
        );
    }
    for point in &before.shape().line {
        assert!(to_line(point.position, split.shape()) <= 2.0 * TOLERANCE);
    }
    assert_eq!(fixture.history().undo_depth(), depth + 1);

    fixture.edit(
        id,
        ElementChange::AddPoint {
            segment: 0,
            t: 0.25,
        },
    );
    let wall = fixture.wall_of(id);
    assert_eq!(wall.points[1], Vec2::new(1.0, 0.0));
    assert_eq!(wall.segments[0].control, None);
    assert_eq!(wall.segments[1].control, None);
    assert_eq!(wall.segments[4].control, Some(after));

    fixture.undo();
    fixture.undo();
    assert_eq!(fixture.element(id), before);
}

/// Removing a point joins the two segments at it into one straight segment; removing an end
/// point removes the first or the last segment; the segments after a removed point are numbered
/// one lower.
#[test]
fn removing_a_point_joins_straight() {
    let mut fixture = Fixture::new();
    let points = [
        Vec2::ZERO,
        Vec2::new(4.0, 0.0),
        Vec2::new(8.0, 0.0),
        Vec2::new(8.0, 4.0),
        Vec2::new(4.0, 4.0),
    ];
    let id = fixture.wall(&points);
    let controls = [
        Vec2::new(2.0, -1.0),
        Vec2::new(6.0, 1.0),
        Vec2::new(9.0, 2.0),
        Vec2::new(6.0, 5.0),
    ];
    for (segment, control) in controls.iter().enumerate() {
        fixture.edit(
            id,
            ElementChange::Control {
                segment,
                position: Some(*control),
            },
        );
    }
    let before = fixture.element(id);

    fixture.edit(id, ElementChange::RemovePoint { index: 2 });
    let joined = fixture.wall_of(id);
    assert_eq!(
        joined.points,
        vec![points[0], points[1], points[3], points[4]]
    );
    assert_eq!(
        joined.segments,
        vec![
            Segment {
                control: Some(controls[0])
            },
            Segment { control: None },
            Segment {
                control: Some(controls[3])
            },
        ]
    );

    fixture.edit(id, ElementChange::RemovePoint { index: 0 });
    let shortened = fixture.wall_of(id);
    assert_eq!(shortened.points, vec![points[1], points[3], points[4]]);
    assert_eq!(
        shortened.segments,
        vec![
            Segment { control: None },
            Segment {
                control: Some(controls[3])
            }
        ]
    );

    fixture.edit(id, ElementChange::RemovePoint { index: 2 });
    let shortened = fixture.wall_of(id);
    assert_eq!(shortened.points, vec![points[1], points[3]]);
    assert_eq!(shortened.segments, vec![Segment { control: None }]);

    fixture.undo();
    fixture.undo();
    fixture.undo();
    assert_eq!(fixture.element(id), before);
}

/// Removing a point from a Wall of two points removes the Wall, as one history step that undoes
/// to the Wall with both points.
#[test]
fn two_points_or_none() {
    let mut fixture = Fixture::new();
    let below = fixture.prop(Vec2::ZERO);
    let id = fixture.wall(&[Vec2::ZERO, Vec2::new(4.0, 1.0)]);
    let above = fixture.prop(Vec2::ONE);
    let before = fixture.elements();
    let depth = fixture.history().undo_depth();

    fixture.edit(id, ElementChange::RemovePoint { index: 1 });

    let order: Vec<ElementId> = fixture.elements().iter().map(|placed| placed.id).collect();
    assert_eq!(order, vec![below, above]);
    assert_eq!(fixture.history().undo_depth(), depth + 1);

    fixture.undo();
    assert_eq!(fixture.elements(), before);
    fixture.redo();
    let order: Vec<ElementId> = fixture.elements().iter().map(|placed| placed.id).collect();
    assert_eq!(order, vec![below, above]);
}

/// A Wall's thickness and colour are each changed through Edit Element, every change a step of
/// its own.
#[test]
fn properties_stay_editable() {
    let mut fixture = Fixture::new();
    let id = fixture.wall(&[Vec2::ZERO, Vec2::new(4.0, 0.0)]);
    let depth = fixture.history().undo_depth();

    fixture.edit(id, ElementChange::Thickness(0.5));
    fixture.edit(id, ElementChange::Colour(RED));
    let wall = fixture.wall_of(id);
    assert_eq!((wall.thickness, wall.colour), (0.5, RED));
    assert_eq!(fixture.element(id).element.size, Vec2::new(4.5, 0.5));
    assert_eq!(fixture.history().undo_depth(), depth + 2);

    fixture.undo();
    let wall = fixture.wall_of(id);
    assert_eq!((wall.thickness, wall.colour), (0.5, GREY));
    fixture.undo();
    let wall = fixture.wall_of(id);
    assert_eq!((wall.thickness, wall.colour), (0.125, GREY));
}

/// A Place Element of a Wall with fewer than two points, a point that is not finite, or a
/// thickness not above zero or not finite, and an Edit Element naming a point or segment the Wall
/// does not have, adding a point not strictly inside its segment, putting the Wall or a point
/// where it is not finite, or setting such a thickness, are answered with the reason, change
/// nothing, and record no history step.
#[test]
fn malformed_walls_are_refused() {
    let mut fixture = Fixture::new();
    let id = fixture.wall(&[Vec2::ZERO, Vec2::new(4.0, 0.0), Vec2::new(4.0, 4.0)]);
    let prop = fixture.prop(Vec2::ONE);
    let before = fixture.elements();
    let depth = fixture.history().undo_depth();
    let edit = |change| {
        Apply::EditElement(EditElement {
            element: id,
            change,
            gesture: Gesture::Single,
        })
    };

    let commands = [
        (
            fixture.wall_placement(&[Vec2::ONE], 0.125, GREY),
            "two or more points",
        ),
        (
            fixture.wall_placement(&[], 0.125, GREY),
            "two or more points",
        ),
        (
            fixture.wall_placement(&[Vec2::ZERO, Vec2::ONE], 0.0, GREY),
            "above zero",
        ),
        (
            fixture.wall_placement(&[Vec2::ZERO, Vec2::ONE], -1.0, GREY),
            "above zero",
        ),
        (
            fixture.wall_placement(&[Vec2::ZERO, Vec2::NAN], 0.125, GREY),
            "finite",
        ),
        (
            fixture.wall_placement(&[Vec2::ZERO, Vec2::ONE], f32::INFINITY, GREY),
            "above zero",
        ),
        (
            edit(ElementChange::Point {
                index: 3,
                position: Vec2::ONE,
            }),
            "no point 3",
        ),
        (
            edit(ElementChange::Control {
                segment: 2,
                position: Some(Vec2::ONE),
            }),
            "no segment 2",
        ),
        (
            edit(ElementChange::AddPoint { segment: 5, t: 0.5 }),
            "no segment 5",
        ),
        (edit(ElementChange::RemovePoint { index: 7 }), "no point 7"),
        (
            edit(ElementChange::AddPoint { segment: 0, t: 1.0 }),
            "strictly between",
        ),
        (
            edit(ElementChange::Position(Vec2::splat(f32::NAN))),
            "finite",
        ),
        (
            edit(ElementChange::Point {
                index: 1,
                position: Vec2::splat(f32::INFINITY),
            }),
            "finite",
        ),
        (
            edit(ElementChange::Control {
                segment: 0,
                position: Some(Vec2::NAN),
            }),
            "finite",
        ),
        (edit(ElementChange::Thickness(0.0)), "above zero"),
        (edit(ElementChange::Thickness(-0.5)), "above zero"),
        (
            Apply::EditElement(EditElement {
                element: prop,
                change: ElementChange::Thickness(1.0),
                gesture: Gesture::Single,
            }),
            "not a Wall",
        ),
    ];
    for (command, reason) in commands {
        let failed = fixture.try_apply(command.clone());

        assert_eq!(failed.len(), 1, "{command:?}");
        assert!(failed[0].contains(reason), "{} says {reason}", failed[0]);
        assert_eq!(fixture.elements(), before, "{command:?}");
        assert_eq!(fixture.history().undo_depth(), depth, "{command:?}");
    }
}

/// An Edit Element adding a point at a parameter not strictly between 0 and 1 is refused with
/// the reason, changes nothing, and records no history step: such a point would make a segment
/// of no length and leave no parameter to carry a Portal to.
#[test]
fn a_point_at_a_segment_end_is_refused() {
    let mut fixture = Fixture::new();
    let id = fixture.wall(&[Vec2::ZERO, Vec2::new(4.0, 0.0), Vec2::new(4.0, 4.0)]);
    let before = fixture.elements();
    let depth = fixture.history().undo_depth();

    for t in [0.0, 1.0, -0.25, 1.5, f32::NAN] {
        for segment in [0, 1] {
            let failed = fixture.try_apply(Apply::EditElement(EditElement {
                element: id,
                change: ElementChange::AddPoint { segment, t },
                gesture: Gesture::Single,
            }));

            assert_eq!(failed.len(), 1, "a point at {t} on segment {segment}");
            assert!(failed[0].contains("strictly between"), "{}", failed[0]);
            assert_eq!(fixture.elements(), before);
            assert_eq!(fixture.history().undo_depth(), depth);
        }
    }
}

/// A new Element is placed above every Element already on its Layer, a Wall as a Prop.
#[test]
fn walls_are_placed_on_top() {
    let mut fixture = Fixture::new();

    let order = vec![
        fixture.prop(Vec2::ZERO),
        fixture.wall(&[Vec2::ZERO, Vec2::ONE]),
        fixture.prop(Vec2::ONE),
        fixture.wall(&[Vec2::ONE, Vec2::new(2.0, 0.0)]),
    ];

    let placed: Vec<ElementId> = fixture.elements().iter().map(|placed| placed.id).collect();
    assert_eq!(placed, order);
}

/// An Element may lie outside the Bounds, a Wall with points there as a Prop placed there.
#[test]
fn walls_lie_anywhere_on_the_level() {
    let mut fixture = Fixture::new();
    let bounds = {
        let world = fixture.app.world_mut();
        *world
            .query::<&Bounds>()
            .single(world)
            .expect("exactly one Project")
    };
    let outside = [Vec2::new(-120.0, 450.5), Vec2::new(-80.0, 460.0)];
    assert!(
        outside
            .iter()
            .all(|point| point.x < bounds.origin.as_vec2().x)
    );

    let id = fixture.wall(&outside);
    fixture.edit(
        id,
        ElementChange::Point {
            index: 1,
            position: Vec2::new(2.0, 1000.0),
        },
    );

    assert_eq!(
        fixture.wall_of(id).points,
        vec![outside[0], Vec2::new(2.0, 1000.0)]
    );
}

/// Moving an Element by dragging records a single undo step however long the drag, and undo
/// returns the Element to where the drag began, a Wall moved whole by its line as a Prop.
#[test]
fn a_wall_drag_is_one_step() {
    let mut fixture = Fixture::new();
    let id = fixture.wall(&[Vec2::ZERO, Vec2::new(4.0, 2.0)]);
    let start = fixture.element(id);
    let depth = fixture.history().undo_depth();

    for (position, gesture) in [
        (Vec2::new(2.5, 1.5), Gesture::Begin),
        (Vec2::new(3.0, 2.0), Gesture::Continue),
        (Vec2::new(5.0, 3.0), Gesture::Continue),
        (Vec2::new(6.0, 4.0), Gesture::End),
    ] {
        fixture.gesture(id, ElementChange::Position(position), gesture);
    }

    assert_eq!(fixture.element(id).element.position, Vec2::new(6.0, 4.0));
    assert_eq!(
        fixture.wall_of(id).points,
        vec![Vec2::new(4.0, 3.0), Vec2::new(8.0, 5.0)]
    );
    assert_eq!(fixture.history().undo_depth(), depth + 1);
    fixture.undo();
    assert_eq!(fixture.element(id), start);
    fixture.redo();
    assert_eq!(fixture.element(id).element.position, Vec2::new(6.0, 4.0));
}

/// Undoing a Remove Element restores the Element with every property, its `ElementId`, and its
/// place in the stacking order, a Wall as a Prop.
#[test]
fn wall_removal_is_reversible_in_place() {
    let mut fixture = Fixture::new();
    fixture.prop(Vec2::ZERO);
    let id = fixture.wall(&[Vec2::ZERO, Vec2::new(4.0, 0.0), Vec2::new(4.0, 3.0)]);
    fixture.edit(
        id,
        ElementChange::Control {
            segment: 1,
            position: Some(Vec2::new(6.0, 1.5)),
        },
    );
    fixture.edit(id, ElementChange::Colour(RED));
    fixture.prop(Vec2::ONE);
    fixture.wall(&[Vec2::ONE, Vec2::new(2.0, 2.0)]);
    let before = fixture.elements();

    fixture.apply(Apply::RemoveElement(RemoveElement { element: id }));
    assert_eq!(fixture.elements().len(), 3);

    fixture.undo();
    assert_eq!(fixture.elements(), before);
}
