//! Snapping through the headless editor: the real plugins of the model, the history,
//! `LibraryAccess`, `LibraryManager`, `ProjectManager`, and `AuthoringManager`, with no window and
//! no render Engine, over one Asset Folder holding an image of known pixel size. A test writes the
//! Pointer as the Editor would, runs one update, and asserts the snapped point; it places and
//! edits Walls and Rooms with Apply messages carrying the points the snapped point gave, and
//! asserts the components bit for bit, the answers, and the history.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    clippy::panic,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

mod support;

use bevy::app::App;
use bevy::ecs::change_detection::Tick;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::{ChildOf, Children};
use bevy::ecs::message::Messages;
use bevy::math::{UVec2, Vec2};
use drs_model::{
    Apply, AssetAddress, Colour, Element, ElementChange, ElementId, FolderKey, Gesture, Layer,
    Level, OpenProject, PlaceElement, Placement, PointOf, Pointer, Project, ProjectOpened,
    ProjectRefused, ProjectRequest, ProjectSaved, RemoveElement, Room, SaveProject, SavedMark,
    Snapped, SnappedPoint, Snapping, Wall,
};
use std::path::{Path, PathBuf};
use support::edit;
use tempfile::TempDir;

/// The place of a door image: 512 by 128 pixels, two cells wide and half a cell tall.
const DOOR: &str = "door.png";
/// The place of a square image of one cell, for Props.
const TABLE: &str = "table.png";
/// A dark grey for the Walls.
const GREY: Colour = Colour::rgb(60, 60, 60);
/// A light grey for the floor.
const LIGHT: Colour = Colour::rgb(200, 200, 200);
/// The reach the Editor writes at its starting zoom: eight pixels at 64 pixels per cell.
const REACH: f32 = 8.0 / 64.0;
/// A Room drawn freely, off the Grid: four cells wide and three high, its corners a tenth of a
/// cell right of and three tenths of a cell above Grid corners.
const FREE: [Vec2; 4] = [
    Vec2::new(0.1, 0.3),
    Vec2::new(4.1, 0.3),
    Vec2::new(4.1, 3.3),
    Vec2::new(0.1, 3.3),
];

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
        support::png(&folder, TABLE, UVec2::splat(256), [120, 80, 40, 255]);
        let mut app = support::editor(root.path());
        let key = support::add_folder(&mut app, &folder, "Fixtures").key;
        Self { root, key, app }
    }

    /// The Levels of the Project, in order, each with its first Layer.
    fn levels(&mut self) -> Vec<(Entity, Entity)> {
        let world = self.app.world_mut();
        let project = world
            .query::<(Entity, &Project)>()
            .single(world)
            .expect("one Project")
            .0;
        let children = |entity: Entity| -> Vec<Entity> {
            world
                .get::<Children>(entity)
                .map(|children| children.iter().copied().collect())
                .unwrap_or_default()
        };
        children(project)
            .into_iter()
            .filter(|level| world.get::<Level>(*level).is_some())
            .map(|level| {
                let layer = children(level)
                    .into_iter()
                    .find(|layer| world.get::<Layer>(*layer).is_some())
                    .expect("every Level has a Layer");
                (level, layer)
            })
            .collect()
    }

    /// The first Level and its first Layer.
    fn level(&mut self) -> (Entity, Entity) {
        self.levels()[0]
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

    /// A second Layer on the first Level, above its first, returning it.
    fn second_layer(&mut self) -> Entity {
        let (level, _) = self.level();
        self.app
            .world_mut()
            .spawn((
                Layer {
                    name: "Upper".to_owned(),
                },
                ChildOf(level),
            ))
            .id()
    }

    /// Sends a Command and runs one update, failing the test if it was refused.
    fn apply(&mut self, command: Apply) {
        support::apply(&mut self.app, command);
    }

    /// Places a Room through `points` on `layer`, returning its identity.
    fn room_on(&mut self, layer: Entity, points: &[Vec2]) -> ElementId {
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Room {
                points: points.to_vec(),
                thickness: 0.125,
                wall_colour: GREY,
                floor_colour: LIGHT,
                cuts: false,
            },
        }));
        support::last_on(&mut self.app, layer)
    }

    /// Places a Room through `points` on the first Level, returning its identity.
    fn room(&mut self, points: &[Vec2]) -> ElementId {
        let (_, layer) = self.level();
        self.room_on(layer, points)
    }

    /// Places a Wall through `points` on the first Level, returning its identity.
    fn wall(&mut self, points: &[Vec2]) -> ElementId {
        let (_, layer) = self.level();
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Wall {
                points: points.to_vec(),
                thickness: 0.125,
                colour: GREY,
            },
        }));
        support::last_on(&mut self.app, layer)
    }

    /// The fixture Asset at `place`.
    fn asset(&self, place: &str) -> AssetAddress {
        AssetAddress {
            folder: self.key.clone(),
            place: place.to_owned(),
        }
    }

    /// Places a Prop of the table centred on `position` on the first Level.
    fn prop(&mut self, position: Vec2) {
        let (_, layer) = self.level();
        let asset = self.asset(TABLE);
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Prop { position, asset },
        }));
    }

    /// Places a freestanding door centred on `position` on the first Level.
    fn door(&mut self, position: Vec2) {
        let (_, layer) = self.level();
        let asset = self.asset(DOOR);
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Portal {
                position,
                asset,
                anchor: None,
            },
        }));
    }

    /// Changes an Element on its own, or as a step of a gesture.
    fn edit(&mut self, element: ElementId, change: ElementChange, gesture: Gesture) {
        self.apply(edit(element, change, gesture));
    }

    /// Writes the Pointer as the Editor would, runs one update, and returns the snapped point,
    /// failing the test if it does not answer that Pointer.
    fn snap(&mut self, pointer: Pointer) -> Option<Snapped> {
        *self.app.world_mut().resource_mut::<Pointer>() = pointer;
        self.app.update();
        let snapped = *self.app.world().resource::<SnappedPoint>();
        assert!(
            snapped.pointer.same_but_for_position(&pointer),
            "the snapped point answers {:?}, not {pointer:?}",
            snapped.pointer
        );
        snapped.snapped
    }

    /// What the snapped point holds now, without writing the Pointer.
    fn answer(&self) -> Option<Snapped> {
        self.app.world().resource::<SnappedPoint>().snapped
    }

    /// When the snapped point was last written.
    fn written(&self) -> Tick {
        self.app
            .world()
            .get_resource_change_ticks::<SnappedPoint>()
            .expect("the snapped point is a resource")
            .changed
    }

    /// The point a pointer at `cells` on the first Level is put at, leaving out `left_out`.
    fn point_leaving_out(
        &mut self,
        cells: Vec2,
        left_out: Option<PointOf>,
    ) -> (Vec2, Option<ElementId>) {
        let (level, _) = self.level();
        match self.snap(Pointer {
            level: Some(level),
            cells,
            reach: REACH,
            snapping: Snapping::Point { left_out },
        }) {
            Some(Snapped::Point { position, on }) => (position, on),
            other => panic!("a point is snapped to a point, not {other:?}"),
        }
    }

    /// The point a pointer at `cells` on the first Level is put at.
    fn point(&mut self, cells: Vec2) -> (Vec2, Option<ElementId>) {
        self.point_leaving_out(cells, None)
    }

    /// The travel of a whole drag that began at `from` with the pointer now at `cells`.
    fn travel(&mut self, from: Vec2, cells: Vec2) -> Vec2 {
        let (level, _) = self.level();
        match self.snap(Pointer {
            level: Some(level),
            cells,
            reach: REACH,
            snapping: Snapping::Move { from },
        }) {
            Some(Snapped::Move { travel }) => travel,
            other => panic!("a move is snapped to a travel, not {other:?}"),
        }
    }

    /// The Room with an identity.
    fn room_of(&mut self, id: ElementId) -> Room {
        let entity = support::entity(&mut self.app, id).expect("the Room exists");
        self.app
            .world()
            .get::<Room>(entity)
            .expect("a Room")
            .clone()
    }

    /// The Wall with an identity.
    fn wall_of(&mut self, id: ElementId) -> Wall {
        let entity = support::entity(&mut self.app, id).expect("the Wall exists");
        self.app
            .world()
            .get::<Wall>(entity)
            .expect("a Wall")
            .clone()
    }

    /// The Element with an identity.
    fn element(&mut self, id: ElementId) -> Element {
        let entity = support::entity(&mut self.app, id).expect("the Element exists");
        self.app
            .world()
            .get::<Element>(entity)
            .expect("an Element")
            .clone()
    }

    /// Every Room, by identity.
    fn rooms(&mut self) -> Vec<(ElementId, Room)> {
        let world = self.app.world_mut();
        let mut rooms: Vec<(ElementId, Room)> = world
            .query::<(&ElementId, &Room)>()
            .iter(world)
            .map(|(id, room)| (*id, room.clone()))
            .collect();
        rooms.sort_by_key(|(id, _)| *id);
        rooms
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
        world
            .resource_mut::<Messages<ProjectSaved>>()
            .drain()
            .next()
            .expect("the Project is saved")
            .path
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
        assert!(
            refused.iter().all(|refused| refused.request
                != ProjectRequest::Open {
                    path: path.to_path_buf()
                }),
            "the file was refused: {refused:?}"
        );
        world
            .resource_mut::<Messages<ProjectOpened>>()
            .drain()
            .next()
            .expect("the Project is opened");
        // The opened Elements are derived on the frame after they are spawned.
        self.app.update();
    }
}

/// Whether both coordinates are whole numbers of cells.
fn whole(point: Vec2) -> bool {
    point.x.fract() == 0.0 && point.y.fract() == 0.0
}

/// The bits of every coordinate of `points`, to compare them exactly.
fn bits(points: &[Vec2]) -> Vec<[u32; 2]> {
    points
        .iter()
        .map(|point| [point.x.to_bits(), point.y.to_bits()])
        .collect()
}

/// With snapping, a point is put at the corner of a Grid cell nearest the pointer, each
/// coordinate rounded to the nearest whole number of cells and a coordinate exactly halfway
/// rounded away from zero, unless a point of a Wall or Room lies within reach.
#[test]
fn snapping_to_the_grid() {
    let mut fixture = Fixture::new();
    fixture.room(&FREE);

    for (cells, corner) in [
        (Vec2::new(2.3, 4.6), Vec2::new(2.0, 5.0)),
        (Vec2::new(7.2, 1.9), Vec2::new(7.0, 2.0)),
        (Vec2::new(-2.3, -4.6), Vec2::new(-2.0, -5.0)),
        (Vec2::new(-0.4, 0.4), Vec2::new(0.0, 0.0)),
        (Vec2::new(2.5, -0.5), Vec2::new(3.0, -1.0)),
        (Vec2::new(-2.5, 6.5), Vec2::new(-3.0, 7.0)),
    ] {
        let (position, on) = fixture.point(cells);
        assert_eq!(position, corner, "the pointer at {cells}");
        assert_eq!(
            bits(&[position]),
            bits(&[corner]),
            "the pointer at {cells}: a zero is never negative"
        );
        assert!(whole(position));
        assert_eq!(on, None, "no point of the Room is within reach of {cells}");
    }
}

/// With snapping, when points of Walls or Rooms lie within reach of the pointer, eight logical
/// pixels of the view, the point is put at the nearest of them, the topmost Element's on a tie,
/// even where a Grid corner lies nearer.
#[test]
fn points_win_within_reach() {
    let mut fixture = Fixture::new();
    let corner = Vec2::new(2.1, 0.05);
    let below = fixture.room(&[
        corner,
        Vec2::new(6.1, 0.05),
        Vec2::new(6.1, 3.05),
        Vec2::new(2.1, 3.05),
    ]);

    assert_eq!(
        fixture.point(Vec2::new(2.03, 0.0)),
        (corner, Some(below)),
        "the Room's corner, though the Grid corner at 2, 0 is nearer"
    );
    assert_eq!(
        fixture.point(Vec2::new(1.97, 0.05)),
        (Vec2::new(2.0, 0.0), None),
        "the Room's corner just beyond reach loses to the Grid"
    );

    let nearer = Vec2::new(2.2, 0.05);
    let beside = fixture.wall(&[nearer, Vec2::new(2.2, 2.0)]);
    assert_eq!(
        fixture.point(Vec2::new(2.17, 0.05)),
        (nearer, Some(beside)),
        "of two points within reach, the nearer"
    );

    let above = fixture.room(&[corner, Vec2::new(-2.0, 0.0), Vec2::new(-2.0, 2.0)]);
    assert_eq!(
        fixture.point(Vec2::new(2.08, 0.05)),
        (corner, Some(above)),
        "of two points as near, the one of the Room on top"
    );
}

/// The points within reach are the points of every Wall and Room on the Level the Author is
/// working on, on any of its Layers; control points, Props, Portals, and Elements on other
/// Levels are never within reach.
#[test]
fn what_a_point_snaps_to() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&[Vec2::new(10.1, 0.1), Vec2::new(14.1, 0.1)]);
    let room = fixture.room(&FREE);
    let control = Vec2::new(2.1, -1.7);
    fixture.edit(
        room,
        ElementChange::Control {
            segment: 0,
            position: Some(control),
        },
        Gesture::Single,
    );
    fixture.prop(Vec2::new(20.1, 20.1));
    fixture.door(Vec2::new(30.1, 30.1));
    let upper = fixture.second_layer();
    let above = fixture.room_on(
        upper,
        &[
            Vec2::new(50.1, 0.1),
            Vec2::new(54.1, 0.1),
            Vec2::new(54.1, 4.1),
        ],
    );
    let upstairs = fixture.second_level();
    fixture.room_on(
        upstairs,
        &[
            Vec2::new(40.1, 40.1),
            Vec2::new(44.1, 40.1),
            Vec2::new(44.1, 44.1),
        ],
    );
    let file = fixture.save("two-levels.dungeon");
    fixture.open(&file);
    assert_eq!(
        fixture.levels().len(),
        2,
        "the opened Project has two Levels"
    );

    assert_eq!(
        fixture.point(Vec2::new(14.05, 0.15)),
        (Vec2::new(14.1, 0.1), Some(wall))
    );
    assert_eq!(fixture.point(Vec2::new(4.15, 3.25)), (FREE[2], Some(room)));
    assert_eq!(
        fixture.point(Vec2::new(54.05, 4.05)),
        (Vec2::new(54.1, 4.1), Some(above)),
        "a Room on another Layer of the Level"
    );
    for (cells, what) in [
        (control, "a control point"),
        (Vec2::new(20.1, 20.1), "a Prop's centre"),
        (Vec2::new(30.1, 30.1), "a Portal's centre"),
        (Vec2::new(40.1, 40.1), "a Room on another Level"),
    ] {
        assert_eq!(fixture.point(cells), (cells.round(), None), "{what}");
    }
}

/// The points within reach are those of the Walls and Rooms as they stand: with the pointer
/// still, a Room placed within reach, moved away, or removed changes where the point is put at
/// once, and the snapped point is written only when that answer differs.
#[test]
fn points_within_reach_follow_the_rooms() {
    let mut fixture = Fixture::new();
    let (level, layer) = fixture.level();
    let grid = Some(Snapped::Point {
        position: Vec2::new(2.0, 1.0),
        on: None,
    });
    let still = Pointer {
        level: Some(level),
        cells: Vec2::new(2.13, 1.04),
        reach: REACH,
        snapping: Snapping::Point { left_out: None },
    };
    assert_eq!(fixture.snap(still), grid);
    let written = fixture.written();
    fixture.app.update();
    assert_eq!(
        fixture.written(),
        written,
        "nothing changed, nothing written"
    );

    let corner = Vec2::new(2.1, 1.1);
    let room = fixture.room_on(layer, &[corner, Vec2::new(5.1, 1.1), Vec2::new(5.1, 4.1)]);
    let on_corner = Some(Snapped::Point {
        position: corner,
        on: Some(room),
    });
    assert_eq!(fixture.answer(), on_corner, "a Room placed within reach");

    let written = fixture.written();
    fixture.edit(room, ElementChange::Colour(LIGHT), Gesture::Single);
    assert_eq!(fixture.answer(), on_corner);
    assert_eq!(
        fixture.written(),
        written,
        "an edit that leaves the answer as it was writes nothing"
    );

    fixture.edit(
        room,
        ElementChange::MoveBy(Vec2::new(1.0, 0.0)),
        Gesture::Single,
    );
    assert_eq!(fixture.answer(), grid, "a Room moved out of reach");
    fixture.edit(
        room,
        ElementChange::MoveBy(Vec2::new(-1.0, 0.0)),
        Gesture::Single,
    );
    assert_eq!(fixture.answer(), on_corner, "and back");

    fixture.apply(Apply::RemoveElement(RemoveElement { element: room }));
    assert_eq!(fixture.answer(), grid, "a Room removed");
    assert!(
        fixture
            .app
            .world()
            .resource::<SnappedPoint>()
            .pointer
            .same_but_for_position(&still),
        "every answer is to the Pointer as it was left"
    );
}

/// While a point is dragged, that point is never within reach of itself; every other point of
/// its Wall or Room is.
#[test]
fn the_dragged_point_is_left_out() {
    let mut fixture = Fixture::new();
    let room = fixture.room(&[
        Vec2::new(0.1, 0.3),
        Vec2::new(0.2, 0.35),
        Vec2::new(0.1, 2.3),
    ]);
    let dragged = Some(PointOf {
        element: room,
        index: 1,
    });

    assert_eq!(
        fixture.point_leaving_out(Vec2::new(0.2, 0.35), dragged),
        (Vec2::new(0.1, 0.3), Some(room)),
        "from on top of the dragged point, its neighbour"
    );
    assert_eq!(
        fixture.point_leaving_out(Vec2::new(0.25, 0.4), dragged),
        (Vec2::new(0.0, 0.0), None),
        "with only the dragged point within reach, the Grid"
    );
}

/// A point snapped to a Grid corner has whole numbers of cells as its coordinates, and one
/// snapped to another point has exactly that point's coordinates, so the two are equal.
#[test]
fn snapped_exactly() {
    let mut fixture = Fixture::new();
    let first = fixture.room(&FREE);
    let corners: Vec<Vec2> = [
        Vec2::new(4.15, 0.28),
        Vec2::new(8.2, 0.1),
        Vec2::new(7.9, 3.1),
        Vec2::new(4.05, 3.35),
    ]
    .into_iter()
    .map(|cells| fixture.point(cells).0)
    .collect();
    let second = fixture.room(&corners);

    let check = |first: &Room, second: &Room| {
        assert_eq!(bits(&second.points[..1]), bits(&first.points[1..2]));
        assert_eq!(bits(&second.points[3..]), bits(&first.points[2..3]));
        assert!(whole(second.points[1]) && whole(second.points[2]));
    };
    check(&fixture.room_of(first), &fixture.room_of(second));
    let before = fixture.rooms();

    let file = fixture.save("shared.dungeon");
    fixture.open(&file);

    let after = fixture.rooms();
    assert_eq!(after.len(), 2);
    for ((_, before), (_, after)) in before.iter().zip(&after) {
        assert_eq!(bits(&after.points), bits(&before.points));
    }
    check(&fixture.room_of(first), &fixture.room_of(second));
}

/// Snapping records nothing in the history and changes nothing in the Project; a Place Element
/// or Edit Element applies the points it carries as they are, whether they were snapped or not.
#[test]
fn snapping_is_not_a_step() {
    let mut fixture = Fixture::new();
    let room = fixture.room(&FREE);
    fixture.save("still.dungeon");
    let mark = fixture.app.world().resource::<SavedMark>().clone();
    let depth = support::history(&fixture.app).undo_depth();
    let before = fixture.rooms();
    let (level, _) = fixture.level();

    for (step, cells) in (0_u8..12).map(|step| (step, Vec2::new(f32::from(step) * 0.37, 1.3))) {
        let snapping = match step % 3 {
            0 => Snapping::Point { left_out: None },
            1 => Snapping::Move {
                from: Vec2::new(0.5, 0.5),
            },
            _ => Snapping::Nothing,
        };
        fixture.snap(Pointer {
            level: Some(level),
            cells,
            reach: REACH,
            snapping,
        });
    }
    assert_eq!(support::history(&fixture.app).undo_depth(), depth);
    assert_eq!(
        fixture.rooms(),
        before,
        "writing the Pointer changes no Room"
    );
    let world = fixture.app.world();
    assert_eq!(*world.resource::<SavedMark>(), mark);
    assert!(
        !mark.unsaved(support::history(&fixture.app)),
        "the Project is still saved"
    );

    let dragged = Some(PointOf {
        element: room,
        index: 2,
    });
    for (cells, gesture) in [
        (Vec2::new(4.6, 3.2), Gesture::Begin),
        (Vec2::new(5.2, 3.7), Gesture::Continue),
        (Vec2::new(6.4, 4.1), Gesture::Continue),
        (Vec2::new(6.4, 4.1), Gesture::End),
    ] {
        let (position, _) = fixture.point_leaving_out(cells, dragged);
        fixture.edit(room, ElementChange::Point { index: 2, position }, gesture);
    }
    assert_eq!(fixture.room_of(room).points[2], Vec2::new(6.0, 4.0));
    assert_eq!(support::history(&fixture.app).undo_depth(), depth + 1);
    support::undo(&mut fixture.app);
    assert_eq!(
        bits(&fixture.room_of(room).points),
        bits(&FREE),
        "undo returns the point exactly to where the drag began"
    );
}

/// With snapping, dragging a whole Wall or Room moves it by the pointer's travel since the press,
/// each coordinate rounded to the nearest whole number of cells, a coordinate exactly halfway
/// rounded away from zero.
#[test]
fn moving_by_whole_cells() {
    let mut fixture = Fixture::new();
    let from = Vec2::new(0.25, 0.75);

    for (cells, travel) in [
        (Vec2::new(2.5, 1.25), Vec2::new(2.0, 1.0)),
        (Vec2::new(-1.0, 0.25), Vec2::new(-1.0, -1.0)),
        (Vec2::new(3.875, -2.0), Vec2::new(4.0, -3.0)),
        (Vec2::new(0.6, 0.9), Vec2::new(0.0, 0.0)),
    ] {
        assert_eq!(
            fixture.travel(from, cells),
            travel,
            "the pointer at {cells}"
        );
    }
}

/// Moving a Wall or a Room by a whole number of cells in each direction leaves every point that
/// lay on a Grid corner exactly on a Grid corner, however the drag travelled before, through
/// amounts that are not whole while snapping was off included; an Edit Element that moves a Room
/// by an amount moves every point and control point by the same amount, and a drag of several
/// such moves is one history step.
#[test]
fn whole_cells_stay_whole() {
    let mut fixture = Fixture::new();
    let points = [
        Vec2::new(0.0, 0.0),
        Vec2::new(5.0, 0.0),
        Vec2::new(5.0, 4.0),
        Vec2::new(0.0, 4.0),
    ];
    let room = fixture.room(&points);
    let control = Vec2::new(2.3, -1.7);
    fixture.edit(
        room,
        ElementChange::Control {
            segment: 0,
            position: Some(control),
        },
        Gesture::Single,
    );
    let start = fixture.room_of(room);
    let depth = support::history(&fixture.app).undo_depth();

    // Each step carries the travel since the press; the second and third are a stretch of the
    // drag with snapping off, which moved it by amounts no Grid corner is at.
    let steps = [
        (Vec2::new(1.0, 0.0), Gesture::Begin),
        (Vec2::new(2.37, -0.61), Gesture::Continue),
        (Vec2::new(-5.13, 1.71), Gesture::Continue),
        (Vec2::new(3.0, -1.0), Gesture::Continue),
        (Vec2::new(-4.0, 2.0), Gesture::Continue),
        (Vec2::new(-4.0, 2.0), Gesture::End),
    ];
    for (travel, gesture) in steps {
        fixture.edit(room, ElementChange::MoveBy(travel), gesture);
    }
    let moved_control = control + Vec2::new(-4.0, 2.0);

    let moved = fixture.room_of(room);
    let expected: Vec<Vec2> = points
        .iter()
        .map(|point| *point + Vec2::new(-4.0, 2.0))
        .collect();
    assert_eq!(bits(&moved.points), bits(&expected));
    assert!(moved.points.iter().all(|point| whole(*point)));
    assert_eq!(moved.edges[0].control, Some(moved_control));
    assert_eq!(support::history(&fixture.app).undo_depth(), depth + 1);
    support::undo(&mut fixture.app);
    assert_eq!(
        fixture.room_of(room),
        start,
        "undo returns it to where the drag began"
    );
}

/// An Edit Element that moves a Wall by an amount moves every point and control point by the
/// same amount, and its box with them; a drag of several such moves, each the travel since the
/// press, is one history step, which undo returns to where the drag began.
#[test]
fn a_wall_moves_by_an_amount() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&[
        Vec2::new(0.1, 0.2),
        Vec2::new(3.0, 1.0),
        Vec2::new(6.0, 0.0),
    ]);
    let control = Vec2::new(4.5, 2.25);
    fixture.edit(
        wall,
        ElementChange::Control {
            segment: 1,
            position: Some(control),
        },
        Gesture::Single,
    );
    let before = (fixture.wall_of(wall), fixture.element(wall));
    let depth = support::history(&fixture.app).undo_depth();
    let amount = Vec2::new(-2.5, 1.25);

    for (travel, gesture) in [
        (Vec2::new(-1.0, 0.0), Gesture::Begin),
        (amount, Gesture::Continue),
        (amount, Gesture::End),
    ] {
        fixture.edit(wall, ElementChange::MoveBy(travel), gesture);
    }

    let moved = fixture.wall_of(wall);
    let expected: Vec<Vec2> = before
        .0
        .points
        .iter()
        .map(|point| *point + amount)
        .collect();
    assert_eq!(moved.points, expected);
    assert_eq!(moved.segments[0].control, None);
    assert_eq!(moved.segments[1].control, Some(control + amount));
    support::assert_near(
        fixture.element(wall).position,
        before.1.position + amount,
        "the box moves with it",
    );
    assert_eq!(support::history(&fixture.app).undo_depth(), depth + 1);
    support::undo(&mut fixture.app);
    assert_eq!(
        fixture.wall_of(wall),
        before.0,
        "undo returns it to where the drag began"
    );
}
