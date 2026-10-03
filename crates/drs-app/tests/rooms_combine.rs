//! Rooms that combine and cut, through the headless editor: the real plugins of the model, the
//! history, `LibraryAccess`, `LibraryManager`, `ProjectManager`, and `AuthoringManager`, with no
//! window and no render Engine, over one Asset Folder holding a door image of known pixel size,
//! driven by Apply, Undo, and Redo messages and asserted on the Room, Portal, and Element
//! components, each Room's derived shape, the Layer's children, the answers, and the history.
//! Every Room's points are given exactly, so edges that are meant to coincide do.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

mod support;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::message::Messages;
use bevy::math::{UVec2, Vec2};
use drs_history::History;
use drs_model::{
    Anchoring, Apply, AssetAddress, Colour, ElementChange, ElementId, FolderKey, Gesture,
    LinePlace, LinePoint, OpenProject, PlaceElement, Placement, Portal, PortalAnchor,
    PortalsRemoved, ProjectOpened, ProjectRefused, RemoveElement, Room, RoomShape, SaveProject,
    SetPortalIntoWall, Side,
};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use support::edit;
use tempfile::TempDir;

/// The place of the door image: 512 by 128 pixels, two cells wide and half a cell tall.
const DOOR: &str = "door.png";
/// A dark grey for the Walls.
const GREY: Colour = Colour::rgb(60, 60, 60);
/// A light grey for the floor.
const LIGHT: Colour = Colour::rgb(200, 200, 200);
/// How thick the Walls of most Rooms here are: a quarter of a cell.
const THICK: f32 = 0.25;

/// The rectangle from `low` to `high`, counter-clockwise from its lower-left corner.
fn rect(low: [f32; 2], high: [f32; 2]) -> Vec<Vec2> {
    vec![
        Vec2::new(low[0], low[1]),
        Vec2::new(high[0], low[1]),
        Vec2::new(high[0], high[1]),
        Vec2::new(low[0], high[1]),
    ]
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

/// Whether the Walls drawn in a shape's look cover `p`.
fn on_walls(shape: &RoomShape, p: Vec2) -> bool {
    in_triangles(p, &shape.mesh.vertices, &shape.mesh.indices)
}

/// Whether a closed line winds around `p`, by the non-zero rule.
fn winds_around(line: &[LinePoint], p: Vec2) -> bool {
    let mut winding = 0_i32;
    for pair in line.windows(2) {
        let (a, b) = (pair[0].position, pair[1].position);
        let left = (b - a).perp_dot(p - a);
        if a.y <= p.y {
            if b.y > p.y && left > 0.0 {
                winding += 1;
            }
        } else if b.y <= p.y && left < 0.0 {
            winding -= 1;
        }
    }
    winding != 0
}

/// Whether a Wall drawn in a shape's look runs along the segment from `a` to `b`: one of its
/// chords lies on that line, overlapping it by more than a hundredth of a cell.
fn runs_along(shape: &RoomShape, from: Vec2, to: Vec2) -> bool {
    let (a, along) = (from, to - from);
    let length = along.length();
    shape
        .walls
        .iter()
        .flat_map(|line| line.windows(2))
        .any(|pair| {
            let (p, q) = (pair[0].position, pair[1].position);
            let off = |x: Vec2| (along.perp_dot(x - a) / length).abs();
            if off(p) > 1e-4 || off(q) > 1e-4 {
                return false;
            }
            let portion = |x: Vec2| along.dot(x - a) / (length * length);
            let (low, high) = (
                portion(p).min(portion(q)).max(0.0),
                portion(p).max(portion(q)).min(1.0),
            );
            (high - low) * length > 0.01
        })
}

/// The headless editor with the fixture folder added.
struct Fixture {
    /// Keeps the temporary directory alive for the test.
    root: TempDir,
    /// The folder's key once added.
    key: FolderKey,
    /// The headless editor.
    app: App,
}

impl Fixture {
    /// Creates the fixture folder, starts the editor, and adds the folder.
    fn new() -> Self {
        let root = TempDir::new().expect("temporary root");
        let folder = root.path().join("fixtures");
        support::png(&folder, DOOR, UVec2::new(512, 128), [150, 90, 40, 255]);
        let mut app = support::editor(root.path());
        let key = support::add_folder(&mut app, &folder, "Fixtures").key;
        Self { root, key, app }
    }

    /// The first Layer of the first Level of the Project.
    fn layer(&mut self) -> Entity {
        support::first_layer(&mut self.app)
    }

    /// Sends a Command and runs one update, failing the test if it was refused.
    fn apply(&mut self, command: Apply) {
        support::apply(&mut self.app, command);
    }

    /// Sends a Command and runs one update, returning the reasons of any failure.
    fn try_apply(&mut self, command: Apply) -> Vec<String> {
        support::try_apply(&mut self.app, command)
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
    fn placement(layer: Entity, points: &[Vec2], thickness: f32, cuts: bool) -> Apply {
        Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Room {
                points: points.to_vec(),
                thickness,
                wall_colour: GREY,
                floor_colour: LIGHT,
                cuts,
            },
        })
    }

    /// Places a Room through `points` on the first Layer, `thickness` thick, cutting or not.
    fn room_of_thickness(&mut self, points: &[Vec2], thickness: f32, cuts: bool) -> ElementId {
        let layer = self.layer();
        self.apply(Self::placement(layer, points, thickness, cuts));
        support::last_on(&mut self.app, layer)
    }

    /// Places a Room that adds floor through `points` on the first Layer.
    fn room(&mut self, points: &[Vec2]) -> ElementId {
        self.room_of_thickness(points, THICK, false)
    }

    /// Places a Room that cuts through `points` on the first Layer.
    fn cut(&mut self, points: &[Vec2]) -> ElementId {
        self.room_of_thickness(points, THICK, true)
    }

    /// The door as a Place Element names it.
    fn door(&self) -> AssetAddress {
        AssetAddress {
            folder: self.key.clone(),
            place: DOOR.to_owned(),
        }
    }

    /// The Place Element Command for a door set into `host` at `edge` and `t`.
    fn door_placement(&mut self, host: ElementId, edge: usize, t: f32) -> Apply {
        Apply::PlaceElement(PlaceElement {
            layer: self.layer(),
            placement: Placement::Portal {
                position: Vec2::ZERO,
                asset: self.door(),
                anchor: Some(PortalAnchor {
                    host,
                    index: edge,
                    t,
                    side: Side::Left,
                }),
            },
        })
    }

    /// Places a door, two cells wide, set into `host` at `edge` and `t`.
    fn set_door(&mut self, host: ElementId, edge: usize, t: f32) -> ElementId {
        let command = self.door_placement(host, edge, t);
        self.apply(command);
        let layer = self.layer();
        support::last_on(&mut self.app, layer)
    }

    /// Places a freestanding door centred on `position`.
    fn free_door(&mut self, position: Vec2) -> ElementId {
        let layer = self.layer();
        let asset = self.door();
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Portal {
                position,
                asset,
                anchor: None,
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

    /// The entity of an Element, which must exist.
    fn entity(&mut self, id: ElementId) -> Entity {
        support::entity(&mut self.app, id).expect("the Element exists")
    }

    /// Whether an Element with the identity exists.
    fn exists(&mut self, id: ElementId) -> bool {
        support::entity(&mut self.app, id).is_some()
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

    /// The Portal with an identity, with its box and whether it follows its host.
    fn portal(&mut self, id: ElementId) -> (drs_model::Element, Portal, Option<Anchoring>) {
        let entity = self.entity(id);
        let world = self.app.world();
        (
            world
                .get::<drs_model::Element>(entity)
                .expect("an Element")
                .clone(),
            world.get::<Portal>(entity).expect("a Portal").clone(),
            world.get::<Anchoring>(entity).copied(),
        )
    }

    /// Every Element with its Portal and Room, by identity, for comparing whole states.
    fn state(&mut self) -> Vec<(ElementId, drs_model::Element, Option<Portal>, Option<Room>)> {
        let world = self.app.world_mut();
        let mut state: Vec<_> = world
            .query::<(
                &ElementId,
                &drs_model::Element,
                Option<&Portal>,
                Option<&Room>,
            )>()
            .iter(world)
            .map(|(id, element, portal, room)| {
                (*id, element.clone(), portal.cloned(), room.cloned())
            })
            .collect();
        state.sort_by_key(|(id, ..)| *id);
        state
    }

    /// The Rooms on `layer` in stacking order, with their derived shapes.
    fn rooms_on(&mut self, layer: Entity) -> Vec<(ElementId, Room, RoomShape)> {
        let world = self.app.world_mut();
        let children: Vec<Entity> = world
            .get::<Children>(layer)
            .map(|children| children.iter().copied().collect())
            .unwrap_or_default();
        children
            .into_iter()
            .filter_map(|entity| {
                Some((
                    *world.get::<ElementId>(entity)?,
                    world.get::<Room>(entity)?.clone(),
                    world.get::<RoomShape>(entity)?.clone(),
                ))
            })
            .collect()
    }

    /// The derived shapes of the first Layer's Rooms in stacking order, each with the Room its
    /// Walls are drawn at named by its place among them rather than by its identity, so the shapes
    /// of the same Rooms placed by another editor compare equal.
    fn comparable(&mut self) -> Vec<(Room, RoomShape, Option<usize>)> {
        let layer = self.layer();
        let rooms = self.rooms_on(layer);
        let ids: Vec<ElementId> = rooms.iter().map(|(id, ..)| *id).collect();
        rooms
            .into_iter()
            .map(|(_, room, shape)| {
                let at = ids.iter().position(|id| *id == shape.drawn_at);
                let shape = RoomShape {
                    drawn_at: ElementId::from_raw(0),
                    ..shape
                };
                (room, shape, at)
            })
            .collect()
    }

    /// Asserts that every Room of the first Layer has the shape the same Rooms placed afresh, in
    /// the same order, would have.
    fn as_if_placed_afresh(&mut self, what: &str) {
        let now = self.comparable();
        let mut afresh = Fixture::new();
        for (room, ..) in &now {
            let id = afresh.room_of_thickness(&room.points, room.thickness, room.cuts);
            for (edge, control) in room.edges.iter().enumerate() {
                if let Some(control) = control.control {
                    afresh.edit(
                        id,
                        ElementChange::Control {
                            segment: edge,
                            position: Some(control),
                        },
                    );
                }
            }
        }
        let fresh = afresh.comparable();
        assert_eq!(now.len(), fresh.len(), "{what}: as many Rooms");
        for (index, (current, placed)) in now.iter().zip(&fresh).enumerate() {
            assert_eq!(current.0, placed.0, "{what}: Room {index}");
            assert_eq!(current.2, placed.2, "{what}: Room {index} drawn at");
            assert!(current.1 == placed.1, "{what}: the shape of Room {index}");
        }
    }

    /// Saves the Project to `name` under the temporary root, returning the file written.
    fn save(&mut self, name: &str) -> PathBuf {
        let path = self.root.path().join(name);
        self.app.world_mut().write_message(SaveProject {
            path: Some(path.clone()),
        });
        self.app.update();
        let world = self.app.world_mut();
        let refused: Vec<ProjectRefused> = world
            .resource_mut::<Messages<ProjectRefused>>()
            .drain()
            .collect();
        assert!(refused.is_empty(), "the save was refused: {refused:?}");
        path
    }

    /// Opens the Project file at `path`, failing the test if it is refused.
    fn open(&mut self, path: &Path) {
        self.app.world_mut().write_message(OpenProject {
            path: path.to_path_buf(),
        });
        self.app.update();
        let world = self.app.world_mut();
        let refused: Vec<ProjectRefused> = world
            .resource_mut::<Messages<ProjectRefused>>()
            .drain()
            .collect();
        assert!(refused.is_empty(), "the file was refused: {refused:?}");
        world
            .resource_mut::<Messages<ProjectOpened>>()
            .drain()
            .next()
            .expect("the Project is opened");
        // The opened Elements are derived on the frame after they are spawned.
        self.app.update();
    }

    /// Every Layer of the first Level, in order.
    fn layers(&mut self) -> Vec<Entity> {
        let first = self.layer();
        let world = self.app.world();
        let level = world
            .get::<bevy::ecs::hierarchy::ChildOf>(first)
            .expect("the Layer lies on a Level")
            .parent();
        world
            .get::<Children>(level)
            .map(|children| children.iter().copied().collect())
            .unwrap_or_default()
    }
}

/// Every Room on a Layer that does not cut adds the place its outline winds around to the Layer's
/// combined floor: two overlapping Rooms, a chain of three, and a Room inside another are each
/// walled only round the outside of the floor they cover together.
#[test]
fn rooms_of_a_layer_combine() {
    let mut fixture = Fixture::new();
    let first = fixture.room(&rect([0.0, 0.0], [6.0, 6.0]));
    let second = fixture.room(&rect([4.0, 2.0], [10.0, 8.0]));
    let (a, b) = (fixture.shape(first), fixture.shape(second));
    for outside in [
        Vec2::new(0.0, 3.0),
        Vec2::new(3.0, 0.0),
        Vec2::new(10.0, 5.0),
        Vec2::new(8.0, 2.0),
        Vec2::new(5.0, 8.0),
    ] {
        assert!(
            on_walls(&a, outside) || on_walls(&b, outside),
            "walled at {outside}"
        );
    }
    for inside in [
        Vec2::new(6.0, 4.0),
        Vec2::new(4.0, 4.0),
        Vec2::new(5.0, 2.0),
    ] {
        assert!(
            !on_walls(&a, inside) && !on_walls(&b, inside),
            "no Wall across the floor at {inside}"
        );
    }
    assert!(on_floor(&a, Vec2::new(5.0, 4.0)) && on_floor(&b, Vec2::new(5.0, 4.0)));

    let chain = [
        fixture.room(&rect([20.0, 0.0], [24.0, 4.0])),
        fixture.room(&rect([22.0, 2.0], [26.0, 6.0])),
        fixture.room(&rect([25.0, 5.0], [29.0, 9.0])),
    ];
    let shapes: Vec<RoomShape> = chain.iter().map(|id| fixture.shape(*id)).collect();
    for inside in [
        Vec2::new(24.0, 3.0),
        Vec2::new(22.0, 3.0),
        Vec2::new(25.5, 6.0),
    ] {
        assert!(
            shapes.iter().all(|shape| !on_walls(shape, inside)),
            "no Wall inside the chain at {inside}"
        );
    }
    for outside in [
        Vec2::new(20.0, 2.0),
        Vec2::new(29.0, 7.0),
        Vec2::new(26.0, 4.0),
    ] {
        assert!(
            shapes.iter().any(|shape| on_walls(shape, outside)),
            "walled round the chain at {outside}"
        );
    }

    let outer = fixture.room(&rect([40.0, 0.0], [50.0, 10.0]));
    let inner = fixture.room(&rect([42.0, 2.0], [44.0, 4.0]));
    let patch = fixture.shape(inner);
    assert!(
        patch.mesh.indices.is_empty(),
        "a Room inside another has no Walls"
    );
    assert!(on_floor(&patch, Vec2::new(43.0, 3.0)), "but its own floor");
    let hall = fixture.shape(outer);
    assert!(on_floor(&hall, Vec2::new(43.0, 3.0)));
    assert!(on_walls(&hall, Vec2::new(40.0, 5.0)));
}

/// A Room that cuts takes the place its outline winds around away from the Rooms before it on its
/// Layer, with Walls round the hole: a hole inside a Room, a notch across its outline, a cut over
/// no Room that takes nothing, and a cut across an edge two Rooms share.
#[test]
fn a_cut_takes_floor_away() {
    let mut fixture = Fixture::new();
    let room = fixture.room(&rect([0.0, 0.0], [10.0, 10.0]));
    let hole = fixture.cut(&rect([3.0, 3.0], [6.0, 6.0]));
    let notch = fixture.cut(&rect([8.0, -2.0], [12.0, 4.0]));
    let nowhere = fixture.cut(&rect([30.0, 30.0], [32.0, 32.0]));

    let floor = fixture.shape(room);
    assert!(on_floor(&floor, Vec2::new(1.0, 1.0)));
    assert!(!on_floor(&floor, Vec2::new(4.5, 4.5)), "the hole");
    assert!(!on_floor(&floor, Vec2::new(9.0, 1.0)), "the notch");
    assert!(on_walls(&floor, Vec2::new(0.0, 5.0)));
    assert!(
        !on_walls(&floor, Vec2::new(9.0, 0.0)),
        "no Wall where the notch begins"
    );

    let pit = fixture.shape(hole);
    assert!(pit.floor.indices.is_empty(), "a cut has no floor");
    assert!(on_walls(&pit, Vec2::new(3.0, 4.5)) && on_walls(&pit, Vec2::new(4.5, 6.0)));
    let recess = fixture.shape(notch);
    assert!(on_walls(&recess, Vec2::new(8.0, 2.0)), "the notch's side");
    assert!(
        on_walls(&recess, Vec2::new(9.0, 4.0)),
        "the notch's top inside the Room"
    );
    assert!(
        !on_walls(&recess, Vec2::new(11.0, 4.0)),
        "nothing outside the Room"
    );
    let none = fixture.shape(nowhere);
    assert!(none.mesh.indices.is_empty() && none.floor.indices.is_empty());
    assert_eq!(none.drawn_at, nowhere, "a combination of its own");

    let left = fixture.room(&rect([50.0, 0.0], [54.0, 4.0]));
    let right = fixture.room(&rect([54.0, 0.0], [58.0, 4.0]));
    let across = fixture.cut(&rect([53.0, 1.0], [55.0, 2.0]));
    let shapes = [left, right, across].map(|id| fixture.shape(id));
    assert!(
        shapes
            .iter()
            .any(|shape| on_walls(shape, Vec2::new(54.0, 3.0))),
        "the shared Wall above the cut"
    );
    assert!(
        !shapes[..2]
            .iter()
            .any(|shape| on_walls(shape, Vec2::new(54.0, 1.5))),
        "the shared Wall is gone where the cut lies"
    );
    assert!(on_walls(&shapes[2], Vec2::new(53.0, 1.5)));
}

/// A Room after a cut adds floor as any other: drawn over the hole, it fills it back in.
#[test]
fn a_room_after_a_cut_fills_it() {
    let mut fixture = Fixture::new();
    let room = fixture.room(&rect([0.0, 0.0], [10.0, 10.0]));
    let hole = fixture.cut(&rect([3.0, 3.0], [7.0, 7.0]));
    let fill = fixture.room(&rect([4.0, 4.0], [6.0, 6.0]));

    assert!(!on_floor(&fixture.shape(room), Vec2::new(5.0, 5.0)));
    let filled = fixture.shape(fill);
    assert!(
        on_floor(&filled, Vec2::new(5.0, 5.0)),
        "the hole is filled back in"
    );
    assert!(
        on_walls(&filled, Vec2::new(4.0, 5.0)),
        "walled where it meets the hole"
    );
    assert!(on_walls(&fixture.shape(hole), Vec2::new(3.0, 5.0)));
    assert_eq!(fixture.shape(room).drawn_at, fill);
}

/// Walls run along every stretch of a Room's edge with the combined floor on one side and not on
/// the other: the walled pieces of two overlapping Rooms, one of them curved, end where the
/// other's outline crosses them, and none lies inside the other's floor.
#[test]
fn walls_run_where_the_floor_ends() {
    let mut fixture = Fixture::new();
    let curved = fixture.room(&rect([0.0, 0.0], [8.0, 6.0]));
    fixture.edit(
        curved,
        ElementChange::Control {
            segment: 2,
            position: Some(Vec2::new(4.0, 10.0)),
        },
    );
    let square = fixture.room(&rect([5.0, 4.0], [12.0, 12.0]));
    let (a, b) = (fixture.shape(curved), fixture.shape(square));

    let inside_square =
        |p: Vec2| p.x > 5.0 + 1e-3 && p.x < 12.0 - 1e-3 && p.y > 4.0 + 1e-3 && p.y < 12.0 - 1e-3;
    for pair in a.walls.iter().flat_map(|line| line.windows(2)) {
        let middle = pair[0].position.midpoint(pair[1].position);
        assert!(!inside_square(middle), "the curved Room's Wall at {middle}");
    }
    for pair in b.walls.iter().flat_map(|line| line.windows(2)) {
        let middle = pair[0].position.midpoint(pair[1].position);
        assert!(
            !winds_around(&a.outline, middle),
            "the square's Wall at {middle}"
        );
    }
    let curve_end = a
        .walls
        .iter()
        .flatten()
        .filter(|point| point.segment == 2)
        .map(|point| point.position.x)
        .fold(f32::MIN, f32::max);
    assert!(
        (curve_end - 5.0).abs() < 2e-3,
        "the curve's Wall ends where the square begins: {curve_end}"
    );
}

/// Where an edge of one Room lies exactly on an edge of another, from outside, the stretch they
/// share carries a Wall wherever the combined floor lies on both sides, whatever lies over it: a
/// whole shared edge, a partly shared one, and one with a Room over it; edges a hair apart are
/// walled each on their own, and Rooms overlapping by a hair have no Wall between them.
#[test]
fn shared_edges_keep_their_wall() {
    let mut fixture = Fixture::new();
    let walled = |fixture: &mut Fixture, ids: &[ElementId], p: Vec2| {
        ids.iter().any(|id| on_walls(&fixture.shape(*id), p))
    };

    let pair = [
        fixture.room(&rect([0.0, 0.0], [4.0, 4.0])),
        fixture.room(&rect([4.0, 0.0], [8.0, 4.0])),
    ];
    assert!(
        walled(&mut fixture, &pair, Vec2::new(4.0, 2.0)),
        "a whole shared edge"
    );

    let hall = [
        fixture.room(&rect([10.0, 0.0], [20.0, 4.0])),
        fixture.room(&rect([12.0, 4.0], [14.0, 6.0])),
    ];
    for p in [
        Vec2::new(13.0, 4.0),
        Vec2::new(11.0, 4.0),
        Vec2::new(16.0, 4.0),
    ] {
        assert!(
            walled(&mut fixture, &hall, p),
            "a partly shared edge at {p}"
        );
    }

    let apart = [
        fixture.room(&rect([30.0, 0.0], [34.0, 4.0])),
        fixture.room(&rect([34.001, 0.0], [38.0, 4.0])),
    ];
    for id in apart {
        let shape = fixture.shape(id);
        assert_eq!(shape.walls.len(), 1, "walled on its own");
        let line = &shape.walls[0];
        assert_eq!(
            line.first().map(|p| p.position),
            line.last().map(|p| p.position)
        );
    }

    let overlapping = [
        fixture.room(&rect([40.0, 0.0], [44.0, 4.0])),
        fixture.room(&rect([43.999, 0.0], [48.0, 4.0])),
    ];
    assert!(
        !walled(&mut fixture, &overlapping, Vec2::new(43.9995, 2.0)),
        "no Wall between Rooms overlapping by a hair"
    );

    let covered = [
        fixture.room(&rect([50.0, 0.0], [54.0, 4.0])),
        fixture.room(&rect([54.0, 0.0], [58.0, 4.0])),
        fixture.room(&rect([53.0, 1.0], [55.0, 3.0])),
    ];
    assert!(
        walled(&mut fixture, &covered, Vec2::new(54.0, 2.0)),
        "the shared Wall stays under a Room lying over it"
    );
}

/// A Room inside another with one of its edges on the other's edge, beside a third Room sharing
/// that edge from the outside, changes nothing about the Wall between the outer Rooms.
#[test]
fn a_nested_room_on_a_shared_edge() {
    let mut fixture = Fixture::new();
    let rooms = [
        fixture.room(&rect([0.0, 0.0], [4.0, 4.0])),
        fixture.room(&rect([4.0, 0.0], [8.0, 4.0])),
        fixture.room(&rect([4.0, 1.0], [6.0, 3.0])),
    ];
    let shapes = rooms.map(|id| fixture.shape(id));
    for y in [0.5, 2.0, 3.5] {
        assert!(
            shapes
                .iter()
                .any(|shape| on_walls(shape, Vec2::new(4.0, y))),
            "the Wall between the outer Rooms at height {y}"
        );
    }
    for p in [Vec2::new(5.0, 3.0), Vec2::new(6.0, 2.0)] {
        assert!(
            shapes.iter().all(|shape| !on_walls(shape, p)),
            "no Wall round the nested Room at {p}"
        );
    }
}

/// A stretch where edges of two or more Rooms lie on one another and a Wall runs is walled once,
/// by the last of those Rooms: a shared edge and two outline edges running together.
#[test]
fn walled_once() {
    let mut fixture = Fixture::new();
    let first = fixture.room(&rect([0.0, 0.0], [4.0, 4.0]));
    let second = fixture.room(&rect([4.0, 0.0], [8.0, 4.0]));
    let shared = (Vec2::new(4.0, 0.0), Vec2::new(4.0, 4.0));
    assert!(!runs_along(&fixture.shape(first), shared.0, shared.1));
    assert!(runs_along(&fixture.shape(second), shared.0, shared.1));

    let wide = fixture.room(&rect([10.0, 0.0], [14.0, 4.0]));
    let low = fixture.room(&rect([12.0, 0.0], [18.0, 2.0]));
    let together = (Vec2::new(12.0, 0.0), Vec2::new(14.0, 0.0));
    assert!(!runs_along(&fixture.shape(wide), together.0, together.1));
    assert!(runs_along(&fixture.shape(low), together.0, together.1));
    assert!(runs_along(
        &fixture.shape(wide),
        Vec2::new(10.0, 0.0),
        Vec2::new(12.0, 0.0)
    ));
}

/// Every stretch of Wall is drawn as thick as the Room whose edge it runs along.
#[test]
fn each_wall_in_its_rooms_look() {
    let mut fixture = Fixture::new();
    let thick = fixture.room_of_thickness(&rect([0.0, 0.0], [4.0, 4.0]), 0.5, false);
    let thin = fixture.room_of_thickness(&rect([3.0, 1.0], [7.0, 3.0]), 0.125, false);
    let (thick, thin) = (fixture.shape(thick), fixture.shape(thin));
    assert!(on_walls(&thick, Vec2::new(-0.2, 2.0)) && !on_walls(&thick, Vec2::new(-0.3, 2.0)));
    assert!(on_walls(&thin, Vec2::new(7.05, 2.0)) && !on_walls(&thin, Vec2::new(7.1, 2.0)));
}

/// Every Room that does not cut keeps its own floor over what its outline winds around, less what
/// a later cut takes away, and a Room that cuts has none.
#[test]
fn each_room_keeps_its_floor() {
    let mut fixture = Fixture::new();
    let first = fixture.room(&rect([0.0, 0.0], [6.0, 6.0]));
    let second = fixture.room(&rect([4.0, 2.0], [10.0, 8.0]));
    let cut = fixture.cut(&rect([1.0, 1.0], [2.0, 2.0]));
    let (a, b, c) = (
        fixture.shape(first),
        fixture.shape(second),
        fixture.shape(cut),
    );
    assert!(on_floor(&a, Vec2::new(5.0, 4.0)) && on_floor(&a, Vec2::new(3.0, 3.0)));
    assert!(!on_floor(&a, Vec2::new(9.0, 7.0)));
    assert!(on_floor(&b, Vec2::new(5.0, 4.0)) && on_floor(&b, Vec2::new(9.0, 7.0)));
    assert!(!on_floor(&a, Vec2::new(1.5, 1.5)), "the later cut's hole");
    assert!(c.floor.indices.is_empty());
}

/// Rooms whose outlines overlap or touch, directly or through others, cuts included, form one
/// combination whose Walls are drawn at its last Room's place; a Room apart is its own.
#[test]
fn walls_over_the_combination() {
    let mut fixture = Fixture::new();
    let chain = [
        fixture.room(&rect([0.0, 0.0], [4.0, 4.0])),
        fixture.cut(&rect([3.0, 3.0], [5.0, 5.0])),
        fixture.room(&rect([4.5, 4.5], [8.0, 8.0])),
    ];
    let apart = fixture.room(&rect([20.0, 20.0], [22.0, 22.0]));
    for id in chain {
        assert_eq!(fixture.shape(id).drawn_at, chain[2], "through the cut");
    }
    assert_eq!(fixture.shape(apart).drawn_at, apart);
}

/// A Room combines only with the Rooms of its own Layer, and a cut takes floor away only from
/// them: a Room on a second Layer over overlapping Rooms and a cut is floored and walled whole.
#[test]
fn layers_keep_their_rooms_apart() {
    let mut fixture = Fixture::new();
    let first = fixture.room(&rect([0.0, 0.0], [6.0, 6.0]));
    let second = fixture.room(&rect([4.0, 0.0], [10.0, 6.0]));
    let cut = fixture.cut(&rect([2.0, 2.0], [3.0, 3.0]));
    let above = fixture.room(&rect([2.0, 1.0], [8.0, 5.0]));
    let file = fixture.save("one-layer.dungeon");
    let mut written: Value =
        serde_json::from_slice(&fs::read(&file).expect("the file")).expect("JSON");
    let layer = written["levels"][0]["layers"][0].clone();
    let raw = json!(above.as_raw().to_string());
    let elements = written["levels"][0]["layers"][0]["elements"]
        .as_array_mut()
        .expect("the Layer's Elements");
    elements.retain(|element| *element != raw);
    let mut upper = layer;
    upper["elements"] = json!([raw]);
    written["levels"][0]["layers"]
        .as_array_mut()
        .expect("the Layers")
        .push(upper);
    let two = fixture.root.path().join("two-layers.dungeon");
    fs::write(&two, serde_json::to_vec(&written).expect("JSON")).expect("written");
    fixture.open(&two);
    assert_eq!(fixture.layers().len(), 2);

    let whole = fixture.shape(above);
    assert_eq!(whole.walls.len(), 1, "walled whole, round its outline");
    assert_eq!(whole.drawn_at, above);
    assert!(
        on_floor(&whole, Vec2::new(2.5, 2.5)),
        "no cut of another Layer"
    );
    assert!(on_walls(&whole, Vec2::new(5.0, 1.0)) && on_walls(&whole, Vec2::new(2.0, 3.0)));
    let below = fixture.shape(first);
    assert!(
        !on_floor(&below, Vec2::new(2.5, 2.5)),
        "the cut of its own Layer"
    );
    assert!(
        !on_walls(&below, Vec2::new(6.0, 3.0)),
        "combined with its own Layer"
    );
    assert_eq!(below.drawn_at, cut);
    assert_eq!(fixture.shape(second).drawn_at, cut);
}

/// After every step that changes a Room, and every undo and redo of one, every Room of the Layer
/// has the floor and Walls the same Rooms placed afresh would have.
#[test]
fn combined_after_every_step() {
    let mut fixture = Fixture::new();
    let first = fixture.room(&rect([0.0, 0.0], [6.0, 6.0]));
    let second = fixture.room(&rect([4.0, 2.0], [10.0, 8.0]));
    let cut = fixture.cut(&rect([1.0, 1.0], [3.0, 3.0]));
    fixture.as_if_placed_afresh("placed");

    let layer = fixture.layer();
    let steps: Vec<(&str, Apply)> = vec![
        (
            "a Room placed",
            Fixture::placement(layer, &rect([8.0, 6.0], [12.0, 10.0]), THICK, false),
        ),
        (
            "a Room moved",
            edit(
                second,
                ElementChange::MoveBy(Vec2::new(1.0, 0.0)),
                Gesture::Single,
            ),
        ),
        (
            "a point moved",
            edit(
                first,
                ElementChange::Point {
                    index: 2,
                    position: Vec2::new(7.0, 7.0),
                },
                Gesture::Single,
            ),
        ),
        (
            "a point added",
            edit(
                first,
                ElementChange::AddPoint { segment: 0, t: 0.5 },
                Gesture::Single,
            ),
        ),
        (
            "a point removed",
            edit(
                first,
                ElementChange::RemovePoint { index: 1 },
                Gesture::Single,
            ),
        ),
        (
            "a bend",
            edit(
                second,
                ElementChange::Control {
                    segment: 2,
                    position: Some(Vec2::new(7.0, 11.0)),
                },
                Gesture::Single,
            ),
        ),
        (
            "a thickness",
            edit(second, ElementChange::Thickness(0.5), Gesture::Single),
        ),
        (
            "the cut switch",
            edit(cut, ElementChange::Cuts(false), Gesture::Single),
        ),
        (
            "a removal",
            Apply::RemoveElement(RemoveElement { element: first }),
        ),
    ];
    for (what, step) in steps {
        fixture.apply(step);
        fixture.as_if_placed_afresh(what);
        fixture.undo();
        fixture.as_if_placed_afresh(&format!("{what}, undone"));
        fixture.redo();
        fixture.as_if_placed_afresh(&format!("{what}, redone"));
    }
}

/// While a point is dragged, every Room of its Layer has the floor and Walls of that moment after
/// each change of the drag, and the drag stays one history step.
#[test]
fn followed_through_a_drag() {
    let mut fixture = Fixture::new();
    let _hall = fixture.room(&rect([0.0, 0.0], [6.0, 6.0]));
    let dragged = fixture.room(&rect([8.0, 0.0], [12.0, 4.0]));
    let depth = fixture.history().undo_depth();
    for (at, gesture) in [
        (Vec2::new(7.0, 1.0), Gesture::Begin),
        (Vec2::new(5.0, 2.0), Gesture::Continue),
        (Vec2::new(3.0, 3.0), Gesture::Continue),
        (Vec2::new(3.0, 3.0), Gesture::End),
    ] {
        fixture.apply(edit(
            dragged,
            ElementChange::Point {
                index: 0,
                position: at,
            },
            gesture,
        ));
        fixture.as_if_placed_afresh(&format!("dragged to {at}"));
    }
    assert_eq!(fixture.history().undo_depth(), depth + 1);
}

/// Whether a Room cuts is given when it is placed and changed by an Edit Element of its own,
/// which undo returns, the Room's points, edges, thickness, and colours kept either way.
#[test]
fn whether_a_room_cuts_stays_editable() {
    let mut fixture = Fixture::new();
    let id = fixture.cut(&rect([0.0, 0.0], [4.0, 4.0]));
    let placed = fixture.room_of(id);
    assert!(placed.cuts, "placed cutting");
    let depth = fixture.history().undo_depth();

    fixture.edit(id, ElementChange::Cuts(false));
    let switched = fixture.room_of(id);
    assert!(!switched.cuts);
    assert_eq!(
        Room {
            cuts: true,
            ..switched.clone()
        },
        placed,
        "nothing else changes"
    );
    assert_eq!(fixture.history().undo_depth(), depth + 1, "one step");
    assert!(
        !fixture.shape(id).floor.indices.is_empty(),
        "now it has a floor"
    );
    fixture.undo();
    assert_eq!(fixture.room_of(id), placed);
    assert!(fixture.shape(id).floor.indices.is_empty());
    fixture.redo();
    assert_eq!(fixture.room_of(id), switched);
}

/// A Portal is set into a Room only where a Wall runs: placing, setting, and sliding onto an edge
/// hidden inside another Room's floor are refused with the reason; onto the Wall between two Rooms
/// that share an edge and onto a hole's Wall they are accepted.
#[test]
fn portals_go_where_a_wall_runs() {
    let mut fixture = Fixture::new();
    let hall = fixture.room(&rect([0.0, 0.0], [6.0, 6.0]));
    let _over = fixture.room(&rect([4.0, 2.0], [10.0, 8.0]));
    let hidden = fixture.door_placement(hall, 1, 0.5);
    let reason = fixture.refused(hidden);
    assert!(reason.contains("Wall"), "{reason}");

    let door = fixture.set_door(hall, 1, 0.1);
    let slide = edit(
        door,
        ElementChange::Along { segment: 1, t: 0.5 },
        Gesture::Single,
    );
    let reason = fixture.refused(slide);
    assert!(reason.contains("Wall"), "{reason}");

    let free = fixture.free_door(Vec2::new(30.0, 30.0));
    let set = |host, index, t| {
        Apply::SetPortalIntoWall(SetPortalIntoWall {
            portal: free,
            anchor: PortalAnchor {
                host,
                index,
                t,
                side: Side::Left,
            },
        })
    };
    let reason = fixture.refused(set(hall, 1, 0.5));
    assert!(reason.contains("Wall"), "{reason}");

    let left = fixture.room(&rect([20.0, 0.0], [24.0, 4.0]));
    let _right = fixture.room(&rect([24.0, 0.0], [28.0, 4.0]));
    fixture.apply(set(left, 1, 0.5));
    assert_eq!(
        fixture.portal(free).2,
        Some(Anchoring::Set),
        "on the shared Wall"
    );

    let room = fixture.room(&rect([40.0, 0.0], [50.0, 10.0]));
    let pit = fixture.cut(&rect([43.0, 3.0], [47.0, 7.0]));
    let _ = room;
    fixture.apply(set(pit, 0, 0.5));
    assert_eq!(
        fixture.portal(free).2,
        Some(Anchoring::Set),
        "on the hole's Wall"
    );
}

/// Placing, editing, or removing a Room other than the one a Portal is set into changes none of
/// the Portal's anchor, centre, rotation, or mirroring.
#[test]
fn other_rooms_never_move_a_portal() {
    let mut fixture = Fixture::new();
    let hall = fixture.room(&rect([0.0, 0.0], [6.0, 6.0]));
    let door = fixture.set_door(hall, 0, 0.5);
    let before = fixture.portal(door);

    let far = fixture.room(&rect([10.0, 10.0], [14.0, 14.0]));
    fixture.edit(far, ElementChange::MoveBy(Vec2::new(1.0, 1.0)));
    let near = fixture.room(&rect([4.0, 4.0], [8.0, 8.0]));
    fixture.edit(
        near,
        ElementChange::Point {
            index: 0,
            position: Vec2::new(3.0, 3.0),
        },
    );
    fixture.edit(near, ElementChange::Cuts(true));
    fixture.apply(Apply::RemoveElement(RemoveElement { element: near }));
    assert_eq!(fixture.portal(door), before);
}

/// A Portal stays set as long as a Wall runs at its centre, however much of its width lies where
/// the Wall is gone: a cut straddling it beside its centre leaves it, its stretch following the
/// Wall round the new corner, and a cut over its centre removes it.
#[test]
fn the_centre_decides() {
    let mut fixture = Fixture::new();
    let hall = fixture.room(&rect([0.0, 0.0], [8.0, 8.0]));
    let door = fixture.set_door(hall, 0, 0.5);
    let anchor = fixture.portal(door).1.anchor;

    let right = fixture.cut(&rect([4.5, -1.0], [7.0, 1.0]));
    assert_eq!(
        fixture.portal(door).1.anchor,
        anchor,
        "kept beside its centre"
    );
    assert!(
        !fixture.shape(right).stretches.is_empty(),
        "its stretch follows the Wall round the corner onto the cut's"
    );
    let left = fixture.cut(&rect([1.0, -1.0], [3.5, 1.0]));
    assert_eq!(fixture.portal(door).1.anchor, anchor, "kept on either side");
    let _ = left;

    fixture.cut(&rect([3.5, -1.0], [4.25, 1.0]));
    assert!(!fixture.exists(door), "a cut over its centre removes it");
}

/// Placing a Room over a door removes the door with its Wall in the same step, answered with the
/// door and its Room, and undo restores it set where it was.
#[test]
fn gone_under_a_placed_room() {
    let mut fixture = Fixture::new();
    let hall = fixture.room(&rect([0.0, 0.0], [6.0, 6.0]));
    let door = fixture.set_door(hall, 0, 0.5);
    let set = fixture.portal(door);
    let depth = fixture.history().undo_depth();
    fixture.removed();

    let layer = fixture.layer();
    fixture.apply(Fixture::placement(
        layer,
        &rect([2.0, -2.0], [4.0, 2.0]),
        THICK,
        false,
    ));
    assert!(!fixture.exists(door));
    assert_eq!(
        fixture.removed(),
        vec![PortalsRemoved {
            host: hall,
            portals: vec![door]
        }]
    );
    assert_eq!(fixture.history().undo_depth(), depth + 1, "one step");
    fixture.undo();
    assert_eq!(fixture.portal(door), set, "restored set where it was");
    fixture.redo();
    assert!(!fixture.exists(door));
}

/// A cut over a door removes it in the same step, and undo restores it.
#[test]
fn gone_under_a_cut() {
    let mut fixture = Fixture::new();
    let hall = fixture.room(&rect([0.0, 0.0], [6.0, 6.0]));
    let door = fixture.set_door(hall, 0, 0.5);
    let set = fixture.portal(door);
    fixture.removed();

    fixture.cut(&rect([2.0, -1.0], [4.0, 1.0]));
    assert!(!fixture.exists(door));
    assert_eq!(
        fixture.removed(),
        vec![PortalsRemoved {
            host: hall,
            portals: vec![door]
        }]
    );
    fixture.undo();
    assert_eq!(fixture.portal(door), set);
}

/// Switching Cut on for a Room takes away its Walls outside the Rooms before it, and the door set
/// into one goes in the same step; undo restores it.
#[test]
fn gone_when_the_cut_switch_changes() {
    let mut fixture = Fixture::new();
    let _hall = fixture.room(&rect([0.0, 0.0], [6.0, 6.0]));
    let wing = fixture.room(&rect([4.0, 2.0], [10.0, 8.0]));
    let door = fixture.set_door(wing, 1, 0.5);
    let set = fixture.portal(door);
    fixture.removed();

    fixture.edit(wing, ElementChange::Cuts(true));
    assert!(!fixture.exists(door));
    assert_eq!(
        fixture.removed(),
        vec![PortalsRemoved {
            host: wing,
            portals: vec![door]
        }]
    );
    fixture.undo();
    assert!(!fixture.room_of(wing).cuts);
    assert_eq!(fixture.portal(door), set);
}

/// Moving another Room over a door removes the door in the same step, and undo restores it.
#[test]
fn gone_when_another_room_moves() {
    let mut fixture = Fixture::new();
    let hall = fixture.room(&rect([0.0, 0.0], [6.0, 6.0]));
    let door = fixture.set_door(hall, 0, 0.5);
    let set = fixture.portal(door);
    let block = fixture.room(&rect([10.0, -1.0], [12.0, 1.0]));
    fixture.removed();

    fixture.edit(block, ElementChange::MoveBy(Vec2::new(-8.0, 0.0)));
    assert!(!fixture.exists(door));
    assert_eq!(
        fixture.removed(),
        vec![PortalsRemoved {
            host: hall,
            portals: vec![door]
        }]
    );
    fixture.undo();
    assert_eq!(fixture.portal(door), set);
}

/// While a drag is under way a door left with no Wall at its centre stands where it stood and is
/// set again when the Wall comes back; a drag over a door and back out keeps it, and one that
/// ends over it removes it in the drag's one step, which undo restores.
#[test]
fn a_drag_decides_at_its_end() {
    let mut fixture = Fixture::new();
    let hall = fixture.room(&rect([0.0, 0.0], [6.0, 6.0]));
    let door = fixture.set_door(hall, 0, 0.5);
    let set = fixture.portal(door);
    let block = fixture.room(&rect([10.0, -1.0], [12.0, 1.0]));
    let depth = fixture.history().undo_depth();

    fixture.apply(edit(
        block,
        ElementChange::MoveBy(Vec2::new(-3.0, 0.0)),
        Gesture::Begin,
    ));
    fixture.apply(edit(
        block,
        ElementChange::MoveBy(Vec2::new(-8.0, 0.0)),
        Gesture::Continue,
    ));
    let over = fixture.portal(door);
    assert_eq!(
        over.2,
        Some(Anchoring::Lost),
        "standing with no Wall at its centre"
    );
    assert_eq!(
        (over.0, over.1),
        (set.0.clone(), set.1.clone()),
        "where it stood"
    );
    assert!(
        fixture.shape(hall).stretches.is_empty(),
        "no Wall gives way for it"
    );
    fixture.apply(edit(
        block,
        ElementChange::MoveBy(Vec2::new(-3.0, 0.0)),
        Gesture::Continue,
    ));
    fixture.apply(edit(
        block,
        ElementChange::MoveBy(Vec2::new(-3.0, 0.0)),
        Gesture::End,
    ));
    assert_eq!(fixture.portal(door), set, "passing over it costs nothing");
    assert_eq!(fixture.history().undo_depth(), depth + 1);
    fixture.removed();

    let before = fixture.room_of(block);
    for (travel, gesture) in [
        (Vec2::new(-1.0, 0.0), Gesture::Begin),
        (Vec2::new(-5.0, 0.0), Gesture::Continue),
        (Vec2::new(-5.0, 0.0), Gesture::End),
    ] {
        fixture.apply(edit(block, ElementChange::MoveBy(travel), gesture));
    }
    assert!(!fixture.exists(door), "ending over it removes it");
    assert_eq!(
        fixture.removed(),
        vec![PortalsRemoved {
            host: hall,
            portals: vec![door]
        }]
    );
    assert_eq!(
        fixture.history().undo_depth(),
        depth + 2,
        "in the drag's one step"
    );
    fixture.undo();
    assert_eq!(fixture.room_of(block), before);
    assert_eq!(fixture.portal(door), set);
}

/// A Portal's stretch follows the Walls it stands in: round the edge of the combined floor across
/// the point where one Room's Wall meets another's and onto the other Room's Wall, and along the
/// Wall of a shared edge, stopping at its ends.
#[test]
fn a_stretch_follows_the_walls() {
    let mut fixture = Fixture::new();
    let first = fixture.room(&rect([0.0, 0.0], [4.0, 4.0]));
    let second = fixture.room(&rect([3.0, 1.0], [7.0, 3.0]));
    let _door = fixture.set_door(first, 1, 0.875);
    let own = fixture.shape(first).stretches;
    let other = fixture.shape(second).stretches;
    assert_eq!(own.len(), 1, "{own:?}");
    assert_eq!(other.len(), 1, "{other:?}");
    assert!(
        other[0].covers(LinePlace { segment: 2, t: 0.7 }),
        "{other:?}"
    );
    assert!(
        !other[0].covers(LinePlace { segment: 2, t: 0.5 }),
        "{other:?}"
    );
    assert!(
        own[0].covers(LinePlace {
            segment: 2,
            t: 0.05
        }),
        "{own:?}"
    );

    let left = fixture.room(&rect([10.0, 0.0], [14.0, 4.0]));
    let right = fixture.room(&rect([14.0, 1.0], [18.0, 3.0]));
    let wide = fixture.set_door(left, 1, 0.5);
    fixture.edit(wide, ElementChange::Width(3.0));
    let shared = fixture.shape(right).stretches;
    assert_eq!(shared.len(), 1, "{shared:?}");
    assert!(shared[0].covers(LinePlace { segment: 3, t: 0.0 }));
    assert!(shared[0].covers(LinePlace { segment: 3, t: 1.0 }));
    assert!(
        fixture.shape(left).stretches.is_empty(),
        "the stretch stops at the shared Wall's ends"
    );
}
