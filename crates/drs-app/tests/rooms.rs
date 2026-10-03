//! Drawing and editing Rooms through the headless editor: the real plugins of the model, the
//! history, `LibraryAccess`, `LibraryManager`, `ProjectManager`, and `AuthoringManager`, with no
//! window and no render Engine, over one Asset Folder holding a door image of known pixel size,
//! driven by Apply, Undo, and Redo messages and asserted on the Room, Portal, and Element
//! components, the Room's derived shape with its floor and stretches, the Layer's children, the
//! answers, and the history.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    clippy::float_cmp,
    reason = "a test and its fixtures stop at the first thing that is not as expected, and the \
              geometry asserted on is exact where it is compared exactly"
)]

mod support;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::message::Messages;
use bevy::math::{UVec2, Vec2, ops};
use drs_history::History;
use drs_model::{
    Apply, AssetAddress, Bounds, Colour, Edge, Element, ElementChange, ElementId, FolderKey,
    FreePortal, Gesture, Layer, Level, LinePlace, PlaceElement, Placement, Portal, PortalAnchor,
    PortalsRemoved, Project, ROOM, RemoveElement, Room, RoomShape, SetPortalIntoWall, Side,
};
use std::f32::consts::{FRAC_PI_2, PI};
use support::{CLOSE, assert_close, assert_near, edit, quadratic};
use tempfile::TempDir;

/// The place of the door image: 512 by 128 pixels, two cells wide and half a cell tall.
const DOOR: &str = "door.png";
/// The place of a square image of one cell, for Props.
const TABLE: &str = "table.png";
/// A dark grey for the Walls.
const GREY: Colour = Colour::rgb(60, 60, 60);
/// A light grey for the floor.
const LIGHT: Colour = Colour::rgb(200, 200, 200);
/// A red.
const RED: Colour = Colour::rgb(200, 30, 30);
/// A rectangle eight cells wide and six high, from its lower-left corner counter-clockwise: its
/// outline is 28 cells round.
const RECTANGLE: [Vec2; 4] = [
    Vec2::ZERO,
    Vec2::new(8.0, 0.0),
    Vec2::new(8.0, 6.0),
    Vec2::new(0.0, 6.0),
];
/// The rectangle with a fifth point in the middle of its bottom edge, after the first.
const NOTCHED: [Vec2; 5] = [
    Vec2::ZERO,
    Vec2::new(4.0, 0.0),
    Vec2::new(8.0, 0.0),
    Vec2::new(8.0, 6.0),
    Vec2::new(0.0, 6.0),
];

/// The headless editor with the fixture folder added.
struct Fixture {
    /// Keeps the temporary directory alive for the test.
    _root: TempDir,
    /// The folder's key once added.
    key: FolderKey,
    /// The headless editor.
    app: App,
}

/// What the World holds about one Portal.
#[derive(Debug, Clone, PartialEq)]
struct Placed {
    /// Its kind and box.
    element: Element,
    /// The Portal.
    portal: Portal,
}

impl Fixture {
    /// Creates the fixture folder, starts the editor, and adds the folder.
    fn new() -> Self {
        let root = TempDir::new().expect("temporary root");
        let folder = root.path().join("fixtures");
        support::png(&folder, DOOR, UVec2::new(512, 128), [150, 90, 40, 255]);
        support::png(&folder, TABLE, UVec2::splat(256), [120, 80, 40, 255]);
        let mut app = support::editor(root.path());
        let key = support::add_folder(&mut app, &folder, "Fixtures").key;
        Self {
            _root: root,
            key,
            app,
        }
    }

    /// The first Layer of the first Level of the Project.
    fn layer(&mut self) -> Entity {
        support::first_layer(&mut self.app)
    }

    /// A second Level with a Layer of its own on the Project, returning the Layer.
    fn second_level(&mut self) -> Entity {
        let world = self.app.world_mut();
        let project = world
            .query::<(Entity, &Project)>()
            .single(world)
            .expect("one Project")
            .0;
        let level = world
            .spawn((
                Level {
                    name: "Upstairs".to_owned(),
                },
                ChildOf(project),
            ))
            .id();
        world
            .spawn((
                Layer {
                    name: "Floor".to_owned(),
                },
                ChildOf(level),
            ))
            .id()
    }

    /// Sends a Command and runs one update, returning the reasons of any failure.
    fn try_apply(&mut self, command: Apply) -> Vec<String> {
        support::try_apply(&mut self.app, command)
    }

    /// Sends a Command and runs one update, failing the test if the Command was refused.
    fn apply(&mut self, command: Apply) {
        support::apply(&mut self.app, command);
    }

    /// Sends a Command, failing the test unless it is refused with nothing recorded and nothing
    /// changed, and returns the reason.
    fn refused(&mut self, command: Apply) -> String {
        let depth = self.history().undo_depth();
        let before = self.state();
        let failed = self.try_apply(command.clone());
        assert_eq!(failed.len(), 1, "{command:?} is refused with a reason");
        assert_eq!(
            self.history().undo_depth(),
            depth,
            "{command:?} records nothing"
        );
        assert_eq!(self.state(), before, "{command:?} changes nothing");
        failed.into_iter().next().unwrap_or_default()
    }

    /// The Place Element Command for a Room through `points` on `layer`.
    fn room_placement_on(layer: Entity, points: &[Vec2], thickness: f32, floor: Colour) -> Apply {
        Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Room {
                points: points.to_vec(),
                thickness,
                wall_colour: GREY,
                floor_colour: floor,
            },
        })
    }

    /// The Place Element Command for a Room through `points` on the first Layer.
    fn room_placement(&mut self, points: &[Vec2], thickness: f32) -> Apply {
        let layer = self.layer();
        Self::room_placement_on(layer, points, thickness, LIGHT)
    }

    /// Places a Room through `points` with Walls an eighth of a cell thick on `layer`.
    fn room_on(&mut self, layer: Entity, points: &[Vec2]) -> ElementId {
        let command = Self::room_placement_on(layer, points, 0.125, LIGHT);
        self.apply(command);
        support::last_on(&mut self.app, layer)
    }

    /// Places a Room through `points` with Walls an eighth of a cell thick on the first Layer.
    fn room(&mut self, points: &[Vec2]) -> ElementId {
        let layer = self.layer();
        self.room_on(layer, points)
    }

    /// The door as a Place Element names it.
    fn door(&self) -> AssetAddress {
        AssetAddress {
            folder: self.key.clone(),
            place: DOOR.to_owned(),
        }
    }

    /// The Place Element Command for a door, set at `anchor` or freestanding at `position`.
    fn door_placement(&mut self, position: Vec2, anchor: Option<PortalAnchor>) -> Apply {
        Apply::PlaceElement(PlaceElement {
            layer: self.layer(),
            placement: Placement::Portal {
                position,
                asset: self.door(),
                anchor,
            },
        })
    }

    /// Places a door set into `host` at `edge` and `t`, facing `side`.
    fn set_door(&mut self, host: ElementId, edge: usize, t: f32, side: Side) -> ElementId {
        let command = self.door_placement(
            Vec2::ZERO,
            Some(PortalAnchor {
                host,
                index: edge,
                t,
                side,
            }),
        );
        self.apply(command);
        let layer = self.layer();
        support::last_on(&mut self.app, layer)
    }

    /// Places a freestanding door centred on `position`.
    fn free_door(&mut self, position: Vec2) -> ElementId {
        let command = self.door_placement(position, None);
        self.apply(command);
        let layer = self.layer();
        support::last_on(&mut self.app, layer)
    }

    /// Places a Prop of the table centred on `position`.
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
        support::last_on(&mut self.app, layer)
    }

    /// Changes an Element on its own.
    fn edit(&mut self, element: ElementId, change: ElementChange) {
        self.apply(edit(element, change, Gesture::Single));
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

    /// The answers naming removed Portals since the last call.
    fn removed(&mut self) -> Vec<PortalsRemoved> {
        self.app
            .world_mut()
            .resource_mut::<Messages<PortalsRemoved>>()
            .drain()
            .collect()
    }

    /// The identities on the first Layer, in stacking order.
    fn order(&mut self) -> Vec<ElementId> {
        let layer = self.layer();
        support::order(&mut self.app, layer)
    }

    /// The entity of an Element, which must exist.
    fn entity(&mut self, id: ElementId) -> Entity {
        support::entity(&mut self.app, id).expect("the Element exists")
    }

    /// Whether an Element with the identity exists.
    fn exists(&mut self, id: ElementId) -> bool {
        support::entity(&mut self.app, id).is_some()
    }

    /// The common Element of an identity.
    fn element(&mut self, id: ElementId) -> Element {
        let entity = self.entity(id);
        self.app
            .world()
            .get::<Element>(entity)
            .expect("an Element")
            .clone()
    }

    /// The Room with an identity.
    fn room_of(&mut self, id: ElementId) -> Room {
        let entity = self.entity(id);
        self.app
            .world()
            .get::<Room>(entity)
            .expect("a Room")
            .clone()
    }

    /// The derived shape of the Room with an identity.
    fn shape(&mut self, id: ElementId) -> RoomShape {
        let entity = self.entity(id);
        self.app
            .world()
            .get::<RoomShape>(entity)
            .expect("the Room has its shape")
            .clone()
    }

    /// The Portal with an identity.
    fn portal(&mut self, id: ElementId) -> Placed {
        let entity = self.entity(id);
        let world = self.app.world();
        Placed {
            element: world.get::<Element>(entity).expect("an Element").clone(),
            portal: world.get::<Portal>(entity).expect("a Portal").clone(),
        }
    }

    /// The anchor of a Portal, which it must have.
    fn anchor(&mut self, id: ElementId) -> PortalAnchor {
        self.portal(id).portal.anchor.expect("the Portal is set")
    }

    /// Every Element of every Level with its Portal and Room, by identity, for comparing whole
    /// states.
    fn state(&mut self) -> Vec<(ElementId, Element, Option<Portal>, Option<Room>)> {
        let world = self.app.world_mut();
        let mut state: Vec<_> = world
            .query::<(&ElementId, &Element, Option<&Portal>, Option<&Room>)>()
            .iter(world)
            .map(|(id, element, portal, room)| {
                (*id, element.clone(), portal.cloned(), room.cloned())
            })
            .collect();
        state.sort_by_key(|(id, ..)| *id);
        state
    }
}

/// Whether `p` lies in a triangle of `vertices` and `indices`, edges included.
fn in_triangles(p: Vec2, vertices: &[Vec2], indices: &[u32]) -> bool {
    indices.chunks(3).any(|corners| {
        let [a, b, c] = [0, 1, 2].map(|i| vertices[corners[i] as usize]);
        let side = |u: Vec2, v: Vec2| (v - u).perp_dot(p - u);
        let (ab, bc, ca) = (side(a, b), side(b, c), side(c, a));
        (ab >= 0.0 && bc >= 0.0 && ca >= 0.0) || (ab <= 0.0 && bc <= 0.0 && ca <= 0.0)
    })
}

/// Whether the floor of a shape covers `p`.
fn on_floor(shape: &RoomShape, p: Vec2) -> bool {
    in_triangles(p, &shape.floor.vertices, &shape.floor.indices)
}

/// Whether the Walls of a shape cover `p`.
fn on_walls(shape: &RoomShape, p: Vec2) -> bool {
    in_triangles(p, &shape.walls.mesh.vertices, &shape.walls.mesh.indices)
}

/// The edge numbers the derived closed line runs through, in order, each once.
fn edges_of(shape: &RoomShape) -> Vec<usize> {
    let mut edges: Vec<usize> = shape.walls.line.iter().map(|point| point.segment).collect();
    edges.dedup();
    edges
}

/// The point of the derived line tagged with `edge` at `t`.
fn line_point(shape: &RoomShape, edge: usize, t: f32) -> Vec2 {
    shape
        .walls
        .line
        .iter()
        .find(|point| point.segment == edge && point.t == t)
        .map(|point| point.position)
        .expect("the line has the point")
}

/// A Room is an ordered list of three or more points in Grid cells with an edge from each point
/// to the next and from the last point back to the first, each edge either straight or curved by
/// one control point, a wall thickness in cells, an opaque wall colour, and an opaque floor
/// colour.
#[test]
fn a_room_is_its_outline() {
    let mut fixture = Fixture::new();
    let id = fixture.room(&RECTANGLE);
    let control = Vec2::new(4.0, 9.0);
    fixture.edit(
        id,
        ElementChange::Control {
            segment: 2,
            position: Some(control),
        },
    );

    assert_eq!(fixture.element(id).kind, ROOM);
    assert_eq!(
        fixture.room_of(id),
        Room {
            points: RECTANGLE.to_vec(),
            edges: vec![
                Edge::default(),
                Edge::default(),
                Edge {
                    control: Some(control)
                },
                Edge::default(),
            ],
            thickness: 0.125,
            wall_colour: GREY,
            floor_colour: LIGHT,
        }
    );
}

/// The edge from the first point to the second is the first, the edge from the last point back
/// to the first the last; moving the Room, a point, or a control point, bending or straightening
/// an edge, and changing the wall thickness or either colour change no edge's number.
#[test]
fn edges_are_numbered() {
    let mut fixture = Fixture::new();
    let id = fixture.room(&RECTANGLE);
    let shape = fixture.shape(id);
    assert_eq!(edges_of(&shape), vec![0, 1, 2, 3]);
    assert_eq!(line_point(&shape, 3, 0.0), RECTANGLE[3]);
    assert_eq!(
        line_point(&shape, 3, 1.0),
        RECTANGLE[0],
        "back to the first"
    );

    let changes = [
        ElementChange::Position(Vec2::new(10.0, 10.0)),
        ElementChange::Point {
            index: 1,
            position: Vec2::new(16.0, 6.0),
        },
        ElementChange::Control {
            segment: 1,
            position: Some(Vec2::new(18.0, 10.0)),
        },
        ElementChange::Control {
            segment: 1,
            position: None,
        },
        ElementChange::Thickness(0.5),
        ElementChange::Colour(RED),
        ElementChange::FloorColour(RED),
    ];
    for change in changes {
        fixture.edit(id, change.clone());
        let room = fixture.room_of(id);
        let shape = fixture.shape(id);
        assert_eq!(edges_of(&shape), vec![0, 1, 2, 3], "{change:?}");
        for edge in 0..4 {
            assert_eq!(
                line_point(&shape, edge, 0.0),
                room.points[edge],
                "edge {edge} starts at its point after {change:?}"
            );
        }
        assert_eq!(line_point(&shape, 3, 1.0), room.points[0], "{change:?}");
    }
}

/// A curved edge is the quadratic Bézier curve from its first point to its second with its
/// control point, the edge from the last point back to the first included; an edge without one
/// is the straight line between its points.
#[test]
fn an_edge_curves_by_one_control_point() {
    let mut fixture = Fixture::new();
    let id = fixture.room(&RECTANGLE);
    let control = Vec2::new(-3.0, 3.0);
    fixture.edit(
        id,
        ElementChange::Control {
            segment: 3,
            position: Some(control),
        },
    );

    let shape = fixture.shape(id);
    assert_near(
        line_point(&shape, 3, 0.5),
        quadratic(RECTANGLE[3], control, RECTANGLE[0], 0.5),
        "halfway along the closing edge",
    );
    for point in shape.walls.line.iter().filter(|point| point.segment == 3) {
        let on_curve = quadratic(RECTANGLE[3], control, RECTANGLE[0], point.t);
        assert!(point.position.distance(on_curve) < CLOSE, "{point:?}");
    }
    let straight: Vec<Vec2> = shape
        .walls
        .line
        .iter()
        .filter(|point| point.segment == 0)
        .map(|point| point.position)
        .collect();
    assert_eq!(straight, vec![RECTANGLE[0]], "a straight edge is one chord");
}

/// A Room's position is the centre of the smallest box around its points and control points,
/// and its size is that box grown by half the wall thickness on every side.
#[test]
fn the_box_follows_the_outline() {
    let mut fixture = Fixture::new();
    let id = fixture.room(&RECTANGLE);
    let element = fixture.element(id);
    assert_eq!(element.position, Vec2::new(4.0, 3.0));
    assert_eq!(element.size, Vec2::new(8.125, 6.125));

    fixture.edit(
        id,
        ElementChange::Control {
            segment: 0,
            position: Some(Vec2::new(4.0, -4.0)),
        },
    );
    fixture.edit(id, ElementChange::Thickness(1.0));
    let element = fixture.element(id);
    assert_eq!(element.position, Vec2::new(4.0, 1.0));
    assert_eq!(element.size, Vec2::new(9.0, 11.0));
}

/// A Place Element of a Room puts on the Layer a Room with the given points, thickness, and
/// colours, every edge straight, as one history step that undo takes away whole and redo brings
/// back whole.
#[test]
fn a_room_is_placed_as_one_step() {
    let mut fixture = Fixture::new();
    let depth = fixture.history().undo_depth();
    let command = fixture.room_placement(&RECTANGLE, 0.25);
    fixture.apply(command);
    let id = *fixture.order().last().expect("the Room is on the Layer");
    let placed = fixture.room_of(id);

    assert_eq!(
        placed,
        Room::straight(RECTANGLE.to_vec(), 0.25, GREY, LIGHT)
    );
    assert_eq!(fixture.history().undo_depth(), depth + 1);
    let shape = fixture.shape(id);
    assert!(!shape.floor.indices.is_empty() && !shape.walls.mesh.indices.is_empty());

    fixture.undo();
    assert!(fixture.order().is_empty(), "undo takes the whole Room away");
    fixture.redo();
    assert_eq!(fixture.order(), vec![id], "redo brings back the same Room");
    assert_eq!(fixture.room_of(id), placed);
    assert_eq!(fixture.shape(id), shape, "floor and Walls with it");
}

/// An Edit Element that changes a Room's position moves every point and control point by the
/// same amount.
#[test]
fn moving_the_room_moves_every_point() {
    let mut fixture = Fixture::new();
    let id = fixture.room(&RECTANGLE);
    let control = Vec2::new(4.0, 8.0);
    fixture.edit(
        id,
        ElementChange::Control {
            segment: 2,
            position: Some(control),
        },
    );
    let before = fixture.element(id).position;
    let offset = Vec2::new(-12.5, 3.0);

    fixture.edit(id, ElementChange::Position(before + offset));

    let room = fixture.room_of(id);
    let moved: Vec<Vec2> = RECTANGLE.iter().map(|point| *point + offset).collect();
    assert_eq!(room.points, moved);
    assert_eq!(room.edges[2].control, Some(control + offset));
    assert_eq!(fixture.element(id).position, before + offset);
}

/// Moving an Element by dragging records a single undo step however long the drag, and undo
/// returns the Element to where the drag began: a Room dragged whole by its floor or its Walls
/// moves by the travel since the press, every point and control point with it. A drag that
/// leaves the Room where it began records nothing.
#[test]
fn a_room_drag_is_one_step() {
    let mut fixture = Fixture::new();
    let id = fixture.room(&RECTANGLE);
    let control = Vec2::new(4.0, 8.0);
    fixture.edit(
        id,
        ElementChange::Control {
            segment: 2,
            position: Some(control),
        },
    );
    let start = (fixture.element(id), fixture.room_of(id));
    let depth = fixture.history().undo_depth();

    let offset = Vec2::new(4.0, -1.0);
    for (travel, gesture) in [
        (Vec2::new(0.5, 0.5), Gesture::Begin),
        (Vec2::new(1.0, 2.0), Gesture::Continue),
        (Vec2::new(3.0, 2.0), Gesture::Continue),
        (offset, Gesture::Continue),
        (offset, Gesture::End),
    ] {
        fixture.apply(edit(id, ElementChange::MoveBy(travel), gesture));
    }

    let moved: Vec<Vec2> = RECTANGLE.iter().map(|point| *point + offset).collect();
    assert_eq!(fixture.room_of(id).points, moved);
    assert_eq!(fixture.room_of(id).edges[2].control, Some(control + offset));
    assert_eq!(fixture.history().undo_depth(), depth + 1);
    fixture.undo();
    assert_eq!((fixture.element(id), fixture.room_of(id)), start);
    fixture.redo();
    assert_eq!(fixture.room_of(id).points, moved);

    let moved_state = fixture.room_of(id);
    for (travel, gesture) in [
        (Vec2::new(1.0, 0.0), Gesture::Begin),
        (Vec2::new(2.0, 3.0), Gesture::Continue),
        (Vec2::ZERO, Gesture::Continue),
        (Vec2::ZERO, Gesture::End),
    ] {
        fixture.apply(edit(id, ElementChange::MoveBy(travel), gesture));
    }
    assert_eq!(fixture.room_of(id), moved_state);
    assert_eq!(
        fixture.history().undo_depth(),
        depth + 1,
        "a drag back to where it began records nothing"
    );
    assert!(!fixture.history().can_redo());
}

/// Moving a point of a Room changes that point and nothing else: every other point and every
/// control point stays where it was, so the two edges meeting at the point keep their curves.
#[test]
fn a_rooms_point_moves_alone() {
    let mut fixture = Fixture::new();
    let id = fixture.room(&RECTANGLE);
    let controls = [
        Some(Vec2::new(4.0, -2.0)),
        None,
        None,
        Some(Vec2::new(-2.0, 3.0)),
    ];
    for (segment, control) in controls.iter().enumerate() {
        if control.is_some() {
            fixture.edit(
                id,
                ElementChange::Control {
                    segment,
                    position: *control,
                },
            );
        }
    }

    fixture.edit(
        id,
        ElementChange::Point {
            index: 0,
            position: Vec2::new(-1.0, -1.0),
        },
    );

    let room = fixture.room_of(id);
    let mut expected = RECTANGLE.to_vec();
    expected[0] = Vec2::new(-1.0, -1.0);
    assert_eq!(room.points, expected);
    let kept: Vec<Option<Vec2>> = room.edges.iter().map(|edge| edge.control).collect();
    assert_eq!(
        kept, controls,
        "both edges at the first point keep their curves"
    );
}

/// Dragging a point or a control point of a Room records a single undo step however long the
/// drag, and undo returns it to where the drag began; a drag that ends where it began records
/// nothing.
#[test]
fn a_rooms_handle_drag_is_one_step() {
    let mut fixture = Fixture::new();
    let id = fixture.room(&RECTANGLE);
    let depth = fixture.history().undo_depth();

    let drags = [
        |at: Vec2| ElementChange::Point {
            index: 2,
            position: at,
        },
        |at: Vec2| ElementChange::Control {
            segment: 3,
            position: Some(at),
        },
    ];
    for (step, drag) in drags.iter().enumerate() {
        let before = fixture.room_of(id);
        let path = [
            (Vec2::new(9.0, 7.0), Gesture::Begin),
            (Vec2::new(10.0, 8.0), Gesture::Continue),
            (Vec2::new(11.0, 9.0), Gesture::Continue),
            (Vec2::new(11.0, 9.0), Gesture::End),
        ];
        for (at, gesture) in path {
            fixture.apply(edit(id, drag(at), gesture));
        }
        assert_eq!(fixture.history().undo_depth(), depth + step + 1);
        fixture.undo();
        assert_eq!(fixture.room_of(id), before, "back to where the drag began");
        fixture.redo();
    }

    let before = fixture.room_of(id);
    let depth = fixture.history().undo_depth();
    for (at, gesture) in [
        (Vec2::new(5.0, 7.0), Gesture::Begin),
        (RECTANGLE[1], Gesture::Continue),
        (RECTANGLE[1], Gesture::End),
    ] {
        fixture.apply(edit(
            id,
            ElementChange::Point {
                index: 1,
                position: at,
            },
            gesture,
        ));
    }
    assert_eq!(fixture.room_of(id), before);
    assert_eq!(
        fixture.history().undo_depth(),
        depth,
        "a drag of a point back to where it began records nothing"
    );
}

/// Setting an edge's control point changes that control point only, and unsetting it makes the
/// edge straight; no point moves.
#[test]
fn bending_an_edge_keeps_the_points() {
    let mut fixture = Fixture::new();
    let id = fixture.room(&RECTANGLE);

    fixture.edit(
        id,
        ElementChange::Control {
            segment: 3,
            position: Some(Vec2::new(-4.0, 3.0)),
        },
    );
    let room = fixture.room_of(id);
    assert_eq!(room.points, RECTANGLE.to_vec());
    assert_eq!(room.edges[3].control, Some(Vec2::new(-4.0, 3.0)));
    assert!(room.edges[..3].iter().all(|edge| edge.control.is_none()));

    fixture.edit(
        id,
        ElementChange::Control {
            segment: 3,
            position: None,
        },
    );
    assert_eq!(
        fixture.room_of(id),
        Room::straight(RECTANGLE.to_vec(), 0.125, GREY, LIGHT)
    );
}

/// A point added on an edge strictly between its ends splits it into two edges whose joined
/// curve is the one it had, straight on a straight edge and curved on a curved one; the new point
/// comes after the edge's first point, and every later point and edge is numbered one higher, so
/// a point added on the last edge becomes the last point.
#[test]
fn adding_a_point_keeps_the_rooms_shape() {
    let mut fixture = Fixture::new();
    let id = fixture.room(&RECTANGLE);
    let control = Vec2::new(4.0, 10.0);
    fixture.edit(
        id,
        ElementChange::Control {
            segment: 2,
            position: Some(control),
        },
    );

    fixture.edit(
        id,
        ElementChange::AddPoint {
            segment: 0,
            t: 0.25,
        },
    );
    let room = fixture.room_of(id);
    assert_eq!(room.points[1], Vec2::new(2.0, 0.0), "straight");
    assert_eq!(room.points.len(), 5);
    assert_eq!(
        room.edges[3].control,
        Some(control),
        "the curve one edge on"
    );

    fixture.edit(id, ElementChange::AddPoint { segment: 3, t: 0.5 });
    let room = fixture.room_of(id);
    let halves = [3, 4].map(|edge| {
        (
            room.points[edge],
            room.edges[edge].control.expect("both halves curve"),
            room.points[edge + 1],
        )
    });
    for step in 0..=20 {
        #[expect(clippy::cast_precision_loss, reason = "small whole numbers")]
        let u = step as f32 / 20.0;
        let traced = if u <= 0.5 {
            quadratic(halves[0].0, halves[0].1, halves[0].2, u * 2.0)
        } else {
            quadratic(halves[1].0, halves[1].1, halves[1].2, u * 2.0 - 1.0)
        };
        let expected = quadratic(RECTANGLE[2], control, RECTANGLE[3], u);
        assert!(traced.distance(expected) < CLOSE, "curved at {u}");
    }

    fixture.edit(id, ElementChange::AddPoint { segment: 5, t: 0.5 });
    let room = fixture.room_of(id);
    assert_eq!(room.points.len(), 7);
    assert_eq!(
        room.points.last(),
        Some(&Vec2::new(0.0, 3.0)),
        "the last point"
    );
    assert_eq!(room.edges.len(), 7);
}

/// Removing a point joins the edge that ends at it and the edge that starts at it into one
/// straight edge from the point before it to the point after it, in the place of the edge that
/// ended at it, every later point and edge numbered one lower, so removing the first point makes
/// the joined edge the last.
#[test]
fn removing_a_point_joins_the_room_straight() {
    let mut fixture = Fixture::new();
    let id = fixture.room(&NOTCHED);
    for segment in [0, 1, 3] {
        fixture.edit(
            id,
            ElementChange::Control {
                segment,
                position: Some(Vec2::splat(-1.0)),
            },
        );
    }

    fixture.edit(id, ElementChange::RemovePoint { index: 1 });
    let room = fixture.room_of(id);
    assert_eq!(
        room.points,
        vec![NOTCHED[0], NOTCHED[2], NOTCHED[3], NOTCHED[4]]
    );
    let controls: Vec<Option<Vec2>> = room.edges.iter().map(|edge| edge.control).collect();
    assert_eq!(controls, vec![None, None, Some(Vec2::splat(-1.0)), None]);

    fixture.edit(id, ElementChange::RemovePoint { index: 0 });
    let room = fixture.room_of(id);
    assert_eq!(room.points, vec![NOTCHED[2], NOTCHED[3], NOTCHED[4]]);
    let controls: Vec<Option<Vec2>> = room.edges.iter().map(|edge| edge.control).collect();
    assert_eq!(
        controls,
        vec![None, Some(Vec2::splat(-1.0)), None],
        "the joined edge is the last, from the last point to the second"
    );
}

/// Removing a point from a Room of three points removes the Room and the Portals set into it as
/// one history step that undoes to the Room with its three points and its Portals.
#[test]
fn three_points_or_none() {
    let mut fixture = Fixture::new();
    let id = fixture.room(&RECTANGLE[..3]);
    let door = fixture.set_door(id, 1, 0.5, Side::Left);
    let before = fixture.state();
    let depth = fixture.history().undo_depth();

    fixture.edit(id, ElementChange::RemovePoint { index: 1 });
    assert!(!fixture.exists(id) && !fixture.exists(door));
    assert_eq!(fixture.history().undo_depth(), depth + 1);

    fixture.undo();
    assert_eq!(fixture.state(), before);
}

/// A Room's wall thickness, wall colour, and floor colour are each changed through Edit Element,
/// every change a step of its own.
#[test]
fn room_properties_stay_editable() {
    let mut fixture = Fixture::new();
    let id = fixture.room(&RECTANGLE);
    let depth = fixture.history().undo_depth();

    fixture.edit(id, ElementChange::Thickness(0.75));
    fixture.edit(id, ElementChange::Colour(RED));
    fixture.edit(id, ElementChange::FloorColour(GREY));

    let room = fixture.room_of(id);
    assert_eq!(room.thickness, 0.75);
    assert_eq!(room.wall_colour, RED);
    assert_eq!(room.floor_colour, GREY);
    assert_eq!(fixture.history().undo_depth(), depth + 3);
    fixture.undo();
    assert_eq!(fixture.room_of(id).floor_colour, LIGHT);
    assert_eq!(fixture.room_of(id).wall_colour, RED);
}

/// A Place Element of a Room with fewer than three points or a wall thickness not above zero,
/// and an Edit Element naming a point or edge the Room does not have, setting a wall thickness
/// not above zero, or adding a point at a parameter not strictly between 0 and 1, are answered
/// with the reason, change nothing, and record no history step.
#[test]
fn malformed_rooms_are_refused() {
    let mut fixture = Fixture::new();
    let id = fixture.room(&RECTANGLE);
    let commands = [
        (
            fixture.room_placement(&RECTANGLE[..2], 0.125),
            "three or more points",
        ),
        (fixture.room_placement(&[], 0.125), "three or more points"),
        (fixture.room_placement(&RECTANGLE, 0.0), "above zero"),
        (fixture.room_placement(&RECTANGLE, -1.0), "above zero"),
        (
            edit(
                id,
                ElementChange::Point {
                    index: 4,
                    position: Vec2::ONE,
                },
                Gesture::Single,
            ),
            "no point 4",
        ),
        (
            edit(
                id,
                ElementChange::Control {
                    segment: 4,
                    position: Some(Vec2::ONE),
                },
                Gesture::Single,
            ),
            "no edge 4",
        ),
        (
            edit(
                id,
                ElementChange::AddPoint { segment: 9, t: 0.5 },
                Gesture::Single,
            ),
            "no edge 9",
        ),
        (
            edit(id, ElementChange::RemovePoint { index: 6 }, Gesture::Single),
            "no point 6",
        ),
        (
            edit(id, ElementChange::Thickness(0.0), Gesture::Single),
            "above zero",
        ),
        (
            edit(id, ElementChange::Thickness(-2.0), Gesture::Single),
            "above zero",
        ),
    ];
    for (command, reason) in commands {
        let failed = fixture.refused(command);
        assert!(failed.contains(reason), "{failed} says {reason}");
    }
    for t in [0.0, 1.0, -0.5, 1.5, f32::NAN] {
        let failed = fixture.refused(edit(
            id,
            ElementChange::AddPoint { segment: 3, t },
            Gesture::Single,
        ));
        assert!(failed.contains("strictly between"), "{failed}");
    }
}

/// A Room's floor covers every place its outline winds around, including every part of an
/// outline whose edges cross, up to the outline's line.
#[test]
fn the_floor_fills_the_outline() {
    let mut fixture = Fixture::new();
    let rectangle = fixture.room(&RECTANGLE);
    let crossed = fixture.room(&[
        Vec2::new(20.0, 0.0),
        Vec2::new(26.0, 0.0),
        Vec2::new(20.0, 6.0),
        Vec2::new(26.0, 6.0),
    ]);

    let shape = fixture.shape(rectangle);
    assert!(on_floor(&shape, Vec2::new(4.0, 3.0)), "inside");
    assert!(on_floor(&shape, Vec2::new(7.99, 0.01)), "up to the line");
    assert!(!on_floor(&shape, Vec2::new(8.1, 3.0)), "outside");
    let shape = fixture.shape(crossed);
    assert!(on_floor(&shape, Vec2::new(23.0, 1.0)), "the lower lobe");
    assert!(on_floor(&shape, Vec2::new(23.0, 5.0)), "the upper lobe");
    assert!(
        !on_floor(&shape, Vec2::new(20.5, 3.0)),
        "beside the crossing"
    );
}

/// A Portal can be set into a Room's Walls, anchored by the Room's `ElementId`, an edge, a
/// parameter along it, and a side: it stands on the outline facing its side, and is placed, set,
/// freed, slid, flipped, and removed as a Portal set into a Wall, with the Room's edges in place of
/// the Wall's segments.
#[test]
fn portals_set_into_rooms() {
    let mut fixture = Fixture::new();
    let room = fixture.room(&RECTANGLE);

    let bottom = fixture.set_door(room, 0, 0.5, Side::Left);
    let placed = fixture.portal(bottom);
    assert_near(
        placed.element.position,
        Vec2::new(4.0, 0.0),
        "on the bottom edge",
    );
    assert_close(placed.portal.rotation, 0.0, "along the bottom edge");
    assert!(!placed.portal.mirrored, "facing the left");
    assert_eq!(
        placed.portal.anchor,
        Some(PortalAnchor {
            host: room,
            index: 0,
            t: 0.5,
            side: Side::Left,
        })
    );

    let closing = fixture.set_door(room, 3, 0.25, Side::Right);
    let placed = fixture.portal(closing);
    assert_near(
        placed.element.position,
        Vec2::new(0.0, 4.5),
        "on the closing edge",
    );
    assert_close(placed.portal.rotation, -FRAC_PI_2, "down the closing edge");
    assert!(placed.portal.mirrored, "facing the right");

    let free = fixture.free_door(Vec2::new(20.0, 20.0));
    fixture.apply(Apply::SetPortalIntoWall(SetPortalIntoWall {
        portal: free,
        anchor: PortalAnchor {
            host: room,
            index: 1,
            t: 0.5,
            side: Side::Left,
        },
    }));
    let placed = fixture.portal(free);
    assert_near(
        placed.element.position,
        Vec2::new(8.0, 3.0),
        "set into the right edge",
    );
    assert_close(placed.portal.rotation, FRAC_PI_2, "up the right edge");

    fixture.apply(edit(
        free,
        ElementChange::Along {
            segment: 2,
            t: 0.25,
        },
        Gesture::Single,
    ));
    let placed = fixture.portal(free);
    assert_near(
        placed.element.position,
        Vec2::new(6.0, 6.0),
        "slid onto the top edge",
    );
    assert_close(ops::sin(placed.portal.rotation), 0.0, "along the top edge");
    assert_close(ops::cos(placed.portal.rotation), -1.0, "leftwards");
    assert_close(placed.portal.rotation.abs(), PI, "half a turn");

    let slid = fixture.portal(free);
    fixture.edit(free, ElementChange::Side(Side::Right));
    let flipped = fixture.portal(free);
    assert_eq!(
        flipped.portal.anchor.map(|anchor| anchor.side),
        Some(Side::Right)
    );
    assert!(flipped.portal.mirrored, "flipped to face the right");
    assert_eq!(flipped.element, slid.element, "flipped where it stands");
    assert_eq!(
        flipped.portal.rotation, slid.portal.rotation,
        "still along the top edge"
    );

    let before = fixture.portal(bottom);
    fixture.apply(Apply::RemoveElement(RemoveElement { element: bottom }));
    assert!(!fixture.exists(bottom), "removed");
    fixture.undo();
    assert_eq!(fixture.portal(bottom), before, "back where it was set");
    fixture.redo();
    assert!(!fixture.exists(bottom), "removed again");

    let standing = fixture.portal(free).element;
    fixture.apply(Apply::FreePortal(FreePortal { portal: free }));
    let placed = fixture.portal(free);
    assert_eq!(placed.portal.anchor, None, "freed");
    assert_eq!(placed.element, standing, "where it stood");
}

/// A Portal set into a Room's Walls covers the stretch of the outline reaching half its width
/// either way from its centre, measured along the outline, across every point including the
/// first, and never stopped by an end; a Portal wider than the whole outline covers all of it.
#[test]
fn a_stretch_runs_round_the_room() {
    let mut fixture = Fixture::new();
    let room = fixture.room(&RECTANGLE);
    let door = fixture.set_door(room, 0, 0.0625, Side::Left);

    let shape = fixture.shape(room);
    assert_eq!(shape.walls.stretches.len(), 1, "one stretch");
    let stretch = shape.walls.stretches[0];
    assert_eq!(stretch.start.segment, 3, "starting on the closing edge");
    assert_close(
        stretch.start.t,
        5.5 / 6.0,
        "half a cell before the first point",
    );
    assert_eq!(stretch.end.segment, 0);
    assert_close(stretch.end.t, 1.5 / 8.0, "a cell and a half after it");
    for gap in [Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(0.0, 0.25)] {
        assert!(!on_walls(&shape, gap), "the Wall gives way at {gap}");
    }
    for wall in [Vec2::new(2.0, 0.0), Vec2::new(0.0, 1.0)] {
        assert!(on_walls(&shape, wall), "the Wall stands at {wall}");
    }
    assert!(
        on_floor(&shape, Vec2::new(0.5, 0.01)),
        "the floor reaches the outline"
    );

    fixture.edit(door, ElementChange::Width(30.0));
    let shape = fixture.shape(room);
    assert_eq!(shape.walls.stretches.len(), 1, "one stretch");
    let stretch = shape.walls.stretches[0];
    for segment in 0..4 {
        for t in [0.0, 0.5, 1.0] {
            assert!(stretch.covers(LinePlace { segment, t }), "{segment} at {t}");
        }
    }
    assert!(shape.walls.mesh.indices.is_empty(), "no Wall is left");
}

/// Moving a Room, moving a point, setting or unsetting a control point, and changing the wall
/// thickness or either colour change no Portal's edge, parameter, or side; each Portal stands at
/// its parameter on its edge as the edge now is.
#[test]
fn moves_with_its_room() {
    let mut fixture = Fixture::new();
    let room = fixture.room(&RECTANGLE);
    let door = fixture.set_door(room, 1, 0.5, Side::Right);
    let anchor = fixture.anchor(door);

    fixture.edit(room, ElementChange::Position(Vec2::new(14.0, 3.0)));
    assert_near(
        fixture.portal(door).element.position,
        Vec2::new(18.0, 3.0),
        "moved",
    );

    fixture.edit(
        room,
        ElementChange::Point {
            index: 2,
            position: Vec2::new(18.0, 10.0),
        },
    );
    assert_near(
        fixture.portal(door).element.position,
        Vec2::new(18.0, 5.0),
        "a point",
    );

    let control = Vec2::new(22.0, 5.0);
    fixture.edit(
        room,
        ElementChange::Control {
            segment: 1,
            position: Some(control),
        },
    );
    let expected = quadratic(Vec2::new(18.0, 0.0), control, Vec2::new(18.0, 10.0), 0.5);
    assert_near(fixture.portal(door).element.position, expected, "bent");

    fixture.edit(
        room,
        ElementChange::Control {
            segment: 1,
            position: None,
        },
    );
    for change in [
        ElementChange::Thickness(0.5),
        ElementChange::Colour(RED),
        ElementChange::FloorColour(RED),
    ] {
        fixture.edit(room, change);
    }
    assert_near(
        fixture.portal(door).element.position,
        Vec2::new(18.0, 5.0),
        "straight",
    );
    assert_eq!(fixture.anchor(door), anchor, "the anchor never moved");
    assert!(fixture.portal(door).portal.mirrored, "facing its side");
}

/// Adding a point on edge k at parameter s moves a Portal on edge k below s to t / s on edge k,
/// one at or above s to (t − s) / (1 − s) on edge k + 1, and every Portal on a later edge one
/// edge on, in the same history step, so no Portal moves on the Level.
#[test]
fn adding_a_point_keeps_a_rooms_portals_in_place() {
    let mut fixture = Fixture::new();
    let room = fixture.room(&RECTANGLE);
    fixture.edit(
        room,
        ElementChange::Control {
            segment: 3,
            position: Some(Vec2::new(-3.0, 3.0)),
        },
    );
    let doors = [
        fixture.set_door(room, 0, 0.25, Side::Left),
        fixture.set_door(room, 0, 0.75, Side::Left),
        fixture.set_door(room, 3, 0.25, Side::Left),
        fixture.set_door(room, 3, 0.75, Side::Right),
    ];
    let centres: Vec<Vec2> = doors
        .iter()
        .map(|door| fixture.portal(*door).element.position)
        .collect();
    let depth = fixture.history().undo_depth();

    fixture.edit(room, ElementChange::AddPoint { segment: 0, t: 0.5 });
    let places: Vec<(usize, f32)> = doors
        .iter()
        .map(|door| {
            let anchor = fixture.anchor(*door);
            (anchor.index, anchor.t)
        })
        .collect();
    assert_eq!(places, vec![(0, 0.5), (1, 0.5), (4, 0.25), (4, 0.75)]);

    fixture.edit(room, ElementChange::AddPoint { segment: 4, t: 0.5 });
    let places: Vec<(usize, f32)> = doors
        .iter()
        .map(|door| {
            let anchor = fixture.anchor(*door);
            (anchor.index, anchor.t)
        })
        .collect();
    assert_eq!(places, vec![(0, 0.5), (1, 0.5), (4, 0.5), (5, 0.5)]);
    for (door, centre) in doors.iter().zip(&centres) {
        assert_near(fixture.portal(*door).element.position, *centre, "in place");
    }
    assert_eq!(fixture.history().undo_depth(), depth + 2, "a step each");
}

/// Removing a point moves each Portal on the two edges it joins whose stretch does not cover it
/// onto the joined edge, at the share of the two edges' combined length that lay before its
/// centre, and every Portal on a later edge one edge back, in the same history step; the first
/// point's joined edge is the last.
#[test]
fn removing_a_point_carries_a_rooms_portals() {
    let mut fixture = Fixture::new();
    let inner = fixture.room(&NOTCHED);
    let doors = [
        fixture.set_door(inner, 0, 0.5, Side::Left),
        fixture.set_door(inner, 1, 0.5, Side::Left),
        fixture.set_door(inner, 2, 0.5, Side::Left),
    ];
    fixture.edit(inner, ElementChange::RemovePoint { index: 1 });
    let places: Vec<(usize, f32)> = doors
        .iter()
        .map(|door| {
            let anchor = fixture.anchor(*door);
            (anchor.index, anchor.t)
        })
        .collect();
    assert_eq!(places, vec![(0, 0.25), (0, 0.75), (1, 0.5)]);
    assert_near(
        fixture.portal(doors[0]).element.position,
        Vec2::new(2.0, 0.0),
        "kept",
    );
    assert_near(
        fixture.portal(doors[1]).element.position,
        Vec2::new(6.0, 0.0),
        "kept",
    );
    assert!(fixture.removed().is_empty());

    // Two edges of unequal lengths, two cells and six, share the joined edge by length.
    let unequal = fixture.room(&[
        Vec2::new(40.0, 0.0),
        Vec2::new(42.0, 0.0),
        Vec2::new(48.0, 0.0),
        Vec2::new(48.0, 6.0),
        Vec2::new(40.0, 6.0),
    ]);
    let doors = [
        fixture.set_door(unequal, 0, 0.25, Side::Left),
        fixture.set_door(unequal, 1, 0.75, Side::Left),
    ];
    fixture.edit(unequal, ElementChange::RemovePoint { index: 1 });
    let places: Vec<(usize, f32)> = doors
        .iter()
        .map(|door| {
            let anchor = fixture.anchor(*door);
            (anchor.index, anchor.t)
        })
        .collect();
    assert_eq!(places, vec![(0, 0.0625), (0, 0.8125)]);
    assert_near(
        fixture.portal(doors[0]).element.position,
        Vec2::new(40.5, 0.0),
        "kept on the short edge's share",
    );
    assert_near(
        fixture.portal(doors[1]).element.position,
        Vec2::new(46.5, 0.0),
        "kept on the long edge's share",
    );
    assert!(fixture.removed().is_empty());

    // The first point lies in the middle of the bottom edge, so joining round it keeps the line.
    let first = fixture.room(&[
        Vec2::new(24.0, 0.0),
        Vec2::new(28.0, 0.0),
        Vec2::new(28.0, 6.0),
        Vec2::new(20.0, 6.0),
        Vec2::new(20.0, 0.0),
    ]);
    let doors = [
        fixture.set_door(first, 4, 0.5, Side::Left),
        fixture.set_door(first, 0, 0.5, Side::Left),
        fixture.set_door(first, 1, 0.5, Side::Left),
    ];
    let depth = fixture.history().undo_depth();
    fixture.edit(first, ElementChange::RemovePoint { index: 0 });
    let places: Vec<(usize, f32)> = doors
        .iter()
        .map(|door| {
            let anchor = fixture.anchor(*door);
            (anchor.index, anchor.t)
        })
        .collect();
    assert_eq!(places, vec![(3, 0.25), (3, 0.75), (0, 0.5)]);
    assert_near(
        fixture.portal(doors[0]).element.position,
        Vec2::new(22.0, 0.0),
        "kept",
    );
    assert_near(
        fixture.portal(doors[1]).element.position,
        Vec2::new(26.0, 0.0),
        "kept",
    );
    assert_eq!(fixture.history().undo_depth(), depth + 1, "one step");
}

/// A Portal whose stretch covers a point being removed from its Room is removed in the same
/// history step, which undo restores whole, and the answer names it.
#[test]
fn gone_with_a_covered_point_of_the_room() {
    let mut fixture = Fixture::new();
    let room = fixture.room(&NOTCHED);
    let covering = fixture.set_door(room, 0, 0.875, Side::Left);
    let kept = fixture.set_door(room, 2, 0.5, Side::Left);
    let before = fixture.state();
    let depth = fixture.history().undo_depth();

    fixture.edit(room, ElementChange::RemovePoint { index: 1 });

    assert!(!fixture.exists(covering), "the Portal over the point goes");
    assert!(fixture.exists(kept));
    assert_eq!(
        fixture.removed(),
        vec![PortalsRemoved {
            host: room,
            portals: vec![covering],
        }]
    );
    assert_eq!(fixture.history().undo_depth(), depth + 1);
    fixture.undo();
    assert_eq!(fixture.state(), before, "restored whole");
}

/// Every Portal set into a Room being removed by Remove Element is removed in the same history
/// step, which undo restores whole, and the answer names them.
#[test]
fn gone_with_the_room() {
    let mut fixture = Fixture::new();
    let table = fixture.prop(Vec2::new(4.0, 3.0));
    let room = fixture.room(&RECTANGLE);
    let mut doors = vec![
        fixture.set_door(room, 0, 0.5, Side::Left),
        fixture.set_door(room, 3, 0.5, Side::Right),
    ];
    doors.sort();
    let order = fixture.order();
    let before = fixture.state();
    let depth = fixture.history().undo_depth();

    fixture.apply(Apply::RemoveElement(RemoveElement { element: room }));

    assert_eq!(fixture.order(), vec![table]);
    assert_eq!(
        fixture.removed(),
        vec![PortalsRemoved {
            host: room,
            portals: doors,
        }]
    );
    assert_eq!(fixture.history().undo_depth(), depth + 1);
    fixture.undo();
    assert_eq!(fixture.state(), before, "restored whole");
    assert_eq!(fixture.order(), order, "in its place in the stacking order");
}

/// Every Portal set into a Room of three points from which a point is removed goes with the
/// Room in the same history step, which undo restores whole, and the answer names them.
#[test]
fn gone_with_a_three_point_room() {
    let mut fixture = Fixture::new();
    let room = fixture.room(&RECTANGLE[..3]);
    let door = fixture.set_door(room, 0, 0.5, Side::Left);
    let before = fixture.state();

    fixture.edit(room, ElementChange::RemovePoint { index: 2 });

    assert!(!fixture.exists(room) && !fixture.exists(door));
    assert_eq!(
        fixture.removed(),
        vec![PortalsRemoved {
            host: room,
            portals: vec![door],
        }]
    );
    fixture.undo();
    assert_eq!(fixture.state(), before, "restored whole");
}

/// An Element may lie outside the Bounds, a Room with points there as a Prop placed there.
#[test]
fn rooms_lie_anywhere_on_the_level() {
    let mut fixture = Fixture::new();
    let bounds = {
        let world = fixture.app.world_mut();
        *world
            .query::<&Bounds>()
            .single(world)
            .expect("exactly one Project")
    };
    let outside = [
        Vec2::new(-120.0, 450.0),
        Vec2::new(-100.0, 450.0),
        Vec2::new(-100.0, 470.0),
    ];
    assert!(
        outside
            .iter()
            .all(|point| point.x < bounds.origin.as_vec2().x)
    );

    let id = fixture.room(&outside);
    fixture.edit(
        id,
        ElementChange::Point {
            index: 2,
            position: Vec2::new(-90.0, 2000.0),
        },
    );

    let room = fixture.room_of(id);
    assert_eq!(room.points[..2], outside[..2]);
    assert_eq!(room.points[2], Vec2::new(-90.0, 2000.0));
}

/// A Portal's anchor is refused when it names a Room on another Level, an edge the Room does
/// not have, or a parameter outside 0 to 1, as a Wall's would be.
#[test]
fn room_anchors_are_refused_as_wall_anchors() {
    let mut fixture = Fixture::new();
    let room = fixture.room(&RECTANGLE);
    let upstairs = fixture.second_level();
    let elsewhere = fixture.room_on(upstairs, &RECTANGLE);
    let anchor = |host, edge, t| PortalAnchor {
        host,
        index: edge,
        t,
        side: Side::Left,
    };
    let bad = [
        anchor(elsewhere, 0, 0.5),
        anchor(room, 4, 0.5),
        anchor(room, 0, 1.5),
        anchor(room, 0, -0.25),
    ];
    for anchor in bad {
        let placement = fixture.door_placement(Vec2::ZERO, Some(anchor));
        fixture.refused(placement);
    }
    let door = fixture.free_door(Vec2::new(20.0, 20.0));
    for anchor in bad {
        fixture.refused(Apply::SetPortalIntoWall(SetPortalIntoWall {
            portal: door,
            anchor,
        }));
    }
    let set = fixture.set_door(room, 0, 0.5, Side::Left);
    fixture.refused(edit(
        set,
        ElementChange::Along { segment: 4, t: 0.5 },
        Gesture::Single,
    ));
}

/// No edit of any Room moves, turns, or removes a freestanding Portal.
#[test]
fn freestanding_portals_ignore_rooms() {
    let mut fixture = Fixture::new();
    let room = fixture.room(&NOTCHED);
    let door = fixture.free_door(Vec2::new(4.0, 0.0));
    let free = fixture.portal(door);

    let changes = [
        ElementChange::Position(Vec2::new(5.0, 5.0)),
        ElementChange::Point {
            index: 0,
            position: Vec2::new(-1.0, 0.0),
        },
        ElementChange::Control {
            segment: 0,
            position: Some(Vec2::new(2.0, 3.0)),
        },
        ElementChange::AddPoint { segment: 0, t: 0.5 },
        ElementChange::RemovePoint { index: 1 },
        ElementChange::Thickness(1.0),
    ];
    for change in changes {
        fixture.edit(room, change.clone());
        assert_eq!(fixture.portal(door), free, "{change:?}");
    }
    fixture.apply(Apply::RemoveElement(RemoveElement { element: room }));
    assert_eq!(fixture.portal(door), free, "the Room removed");
    assert!(fixture.removed().is_empty());
}
