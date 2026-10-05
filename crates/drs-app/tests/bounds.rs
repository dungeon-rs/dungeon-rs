//! Resizing the Bounds through the headless editor: the real plugins of the model, the history,
//! `LibraryAccess`, `LibraryManager`, `ProjectManager`, and `AuthoringManager`, with no window and
//! no render Engine, over one Asset Folder holding a table, a door, and a texture of known pixel
//! size, driven by Apply, Undo, and Redo messages and asserted on the Project's Bounds, every
//! Element's components, derived shape, and coverage, the answers, and the history.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

mod support;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::math::{IVec2, UVec2, Vec2};
use drs_model::{
    Anchoring, Apply, AssetAddress, Bounds, BrushSettings, Colour, Element, ElementId, FolderKey,
    Gesture, Paint, PlaceElement, Placement, Portal, PortalAnchor, Prop, Room, RoomShape, Side,
    Stroke, Terrain, TerrainCoverage, Wall, WallShape,
};
use support::{
    add_folder, bounds_of, editor, first_layer, history, png, redo, resize_bounds, resize_to,
    try_apply, undo,
};
use tempfile::TempDir;

/// A table one cell a side at the Grid's 256 pixels per cell, for Props.
const TABLE: &str = "table.png";
/// A door two cells wide and half a cell tall.
const DOOR: &str = "door.png";
/// A flagstone texture two cells a side, for painting.
const FLAGSTONES: &str = "flagstones.png";
/// A dark grey for the Walls.
const GREY: Colour = Colour::rgb(60, 60, 60);
/// A light grey for the floor.
const LIGHT: Colour = Colour::rgb(200, 200, 200);

/// The headless editor with the fixture folder added.
struct Fixture {
    /// Keeps the temporary directory alive for the test.
    _root: TempDir,
    /// The folder's key once added.
    key: FolderKey,
    /// The headless editor.
    app: App,
    /// How deep the history is once the folder is added.
    start: usize,
}

/// Everything an Element is, as the World holds it: its identity, its common component, its own
/// kind's component, and what is derived from it.
#[derive(Debug, Clone, PartialEq)]
struct Everything {
    /// Its identity.
    id: ElementId,
    /// Its common component.
    element: Element,
    /// Its Prop, when it is one.
    prop: Option<Prop>,
    /// Its Wall and the Wall's derived shape, when it is one.
    wall: (Option<Wall>, Option<WallShape>),
    /// Its Portal and how it is anchored, when it is one.
    portal: (Option<Portal>, Option<Anchoring>),
    /// Its Room and the Room's derived shape, when it is one.
    room: (Option<Room>, Option<RoomShape>),
    /// Its Terrain and the Terrain's coverage, when it is one.
    terrain: (Option<Terrain>, Option<TerrainCoverage>),
}

/// The Bounds with their lower-left corner at `(left, bottom)` and `width` by `height` cells.
fn bounds(left: i32, bottom: i32, width: u32, height: u32) -> Bounds {
    Bounds {
        origin: IVec2::new(left, bottom),
        size: UVec2::new(width, height),
    }
}

impl Fixture {
    /// Creates the fixture folder, starts the editor, and adds the folder.
    fn new() -> Self {
        let root = TempDir::new().expect("temporary root");
        let folder = root.path().join("fixtures");
        png(&folder, TABLE, UVec2::splat(256), [120, 80, 40, 255]);
        png(&folder, DOOR, UVec2::new(512, 128), [0, 200, 200, 255]);
        png(&folder, FLAGSTONES, UVec2::splat(512), [90, 90, 100, 255]);
        let mut app = editor(root.path());
        let key = add_folder(&mut app, &folder, "Fixtures").key;
        let start = history(&app).undo_depth();
        Self {
            _root: root,
            key,
            app,
            start,
        }
    }

    /// The Asset at `place` in the fixture folder.
    fn asset(&self, place: &str) -> AssetAddress {
        AssetAddress {
            folder: self.key.clone(),
            place: place.to_owned(),
        }
    }

    /// The one Layer of the Project.
    fn layer(&mut self) -> Entity {
        first_layer(&mut self.app)
    }

    /// Sends a Command and runs one update, returning the reasons of any failure.
    fn try_apply(&mut self, command: Apply) -> Vec<String> {
        try_apply(&mut self.app, command)
    }

    /// Sends a Command and runs one update, failing the test if the Command was refused.
    fn apply(&mut self, command: Apply) {
        let failed = self.try_apply(command);
        assert!(failed.is_empty(), "the Command failed: {failed:?}");
    }

    /// Resizes the Bounds to `bounds` on their own, failing the test on a refusal.
    fn resize(&mut self, bounds: Bounds) {
        self.apply(resize_to(bounds));
    }

    /// Resizes the Bounds to `bounds` on their own and returns the refusals.
    fn try_resize(&mut self, bounds: Bounds) -> Vec<String> {
        self.try_apply(resize_to(bounds))
    }

    /// Places `placement` on the Layer, returning the identity of the Element placed.
    fn place(&mut self, placement: Placement) -> ElementId {
        let layer = self.layer();
        self.apply(Apply::PlaceElement(PlaceElement { layer, placement }));
        self.everything()
            .last()
            .map(|placed| placed.id)
            .expect("the Element is on top")
    }

    /// Places a table centred on `position`.
    fn prop(&mut self, position: Vec2) -> ElementId {
        let asset = self.asset(TABLE);
        self.place(Placement::Prop { position, asset })
    }

    /// The Project's Bounds.
    fn bounds(&mut self) -> Bounds {
        bounds_of(&mut self.app)
    }

    /// Sends Undo and runs one update.
    fn undo(&mut self) {
        undo(&mut self.app);
    }

    /// Sends Redo and runs one update.
    fn redo(&mut self) {
        redo(&mut self.app);
    }

    /// How many steps the history holds since the folder was added.
    fn steps(&self) -> usize {
        history(&self.app).undo_depth() - self.start
    }

    /// Whether there is a step to redo.
    fn can_redo(&self) -> bool {
        history(&self.app).can_redo()
    }

    /// Every Element on the Layer in stacking order, bottom first, with everything it is.
    fn everything(&mut self) -> Vec<Everything> {
        let layer = self.layer();
        let world = self.app.world_mut();
        let children: Vec<Entity> = world
            .get::<Children>(layer)
            .map(|children| children.iter().copied().collect())
            .unwrap_or_default();
        children
            .into_iter()
            .map(|child| {
                let entity = world.entity(child);
                Everything {
                    id: *entity.get::<ElementId>().expect("an identity"),
                    element: entity.get::<Element>().expect("an Element").clone(),
                    prop: entity.get::<Prop>().cloned(),
                    wall: (
                        entity.get::<Wall>().cloned(),
                        entity.get::<WallShape>().cloned(),
                    ),
                    portal: (
                        entity.get::<Portal>().cloned(),
                        entity.get::<Anchoring>().copied(),
                    ),
                    room: (
                        entity.get::<Room>().cloned(),
                        entity.get::<RoomShape>().cloned(),
                    ),
                    terrain: (
                        entity.get::<Terrain>().cloned(),
                        entity.get::<TerrainCoverage>().cloned(),
                    ),
                }
            })
            .collect()
    }
}

/// A Resize Bounds sets the Project's Bounds to the lower-left corner, width, and height it
/// carries, in whole cells, as one history step that undo returns to the Bounds before it and
/// redo sets again.
#[test]
fn resized_to_the_bounds_given() {
    let mut fixture = Fixture::new();
    let start = fixture.bounds();
    assert_eq!(start, bounds(0, 0, 30, 30));
    let resized = [
        // Grown on the left.
        bounds(-5, 0, 35, 30),
        // Shrunk at the top.
        bounds(-5, 0, 35, 20),
        // Moved whole.
        bounds(10, 7, 35, 20),
        // One cell by one.
        bounds(10, 7, 1, 1),
    ];

    for (step, resized) in resized.iter().enumerate() {
        fixture.resize(*resized);
        assert_eq!(fixture.bounds(), *resized);
        assert_eq!(fixture.steps(), step + 1, "one step per resize");
    }

    let before: Vec<Bounds> = std::iter::once(start)
        .chain(resized.iter().copied())
        .collect();
    for index in (0..resized.len()).rev() {
        fixture.undo();
        assert_eq!(
            fixture.bounds(),
            before[index],
            "undone to the Bounds before"
        );
    }
    for resized in resized {
        fixture.redo();
        assert_eq!(fixture.bounds(), resized, "redone to the Bounds given");
    }
    assert_eq!(fixture.steps(), resized.len());
}

/// Resize Bounds sent as a gesture, from its beginning through every continuation to its end,
/// is one history step, and undo returns the Bounds to where the gesture began.
#[test]
fn a_resize_gesture_is_one_step() {
    let mut fixture = Fixture::new();
    let start = fixture.bounds();

    fixture.apply(resize_bounds(bounds(0, 0, 31, 30), Gesture::Begin));
    fixture.apply(resize_bounds(bounds(0, 0, 32, 30), Gesture::Continue));
    fixture.apply(resize_bounds(bounds(-1, 0, 33, 31), Gesture::Continue));
    fixture.apply(resize_bounds(bounds(-2, 0, 34, 31), Gesture::Continue));
    fixture.apply(resize_bounds(bounds(-2, 0, 34, 31), Gesture::End));

    assert_eq!(fixture.bounds(), bounds(-2, 0, 34, 31));
    assert_eq!(fixture.steps(), 1, "the whole gesture is one step");
    fixture.undo();
    assert_eq!(fixture.bounds(), start, "back to where the gesture began");
    assert_eq!(fixture.steps(), 0);
    fixture.redo();
    assert_eq!(fixture.bounds(), bounds(-2, 0, 34, 31));
}

/// A gesture that ends with the Bounds as they were when it began records nothing and leaves what
/// could be redone redoable, however far it went in between.
#[test]
fn a_drag_back_to_its_start_records_nothing() {
    let mut fixture = Fixture::new();
    fixture.resize(bounds(2, 3, 20, 10));
    fixture.resize(bounds(2, 3, 25, 10));
    fixture.undo();
    let start = fixture.bounds();
    let steps = fixture.steps();

    for (at, gesture) in [
        (bounds(2, 3, 22, 10), Gesture::Begin),
        (bounds(1, 3, 24, 12), Gesture::Continue),
        (start, Gesture::Continue),
        (start, Gesture::End),
    ] {
        fixture.apply(resize_bounds(at, gesture));
    }
    assert_eq!(fixture.bounds(), start);
    assert_eq!(
        fixture.steps(),
        steps,
        "a drag back to where it began is no step"
    );
    assert!(fixture.can_redo(), "what could be redone still can be");
    fixture.redo();
    assert_eq!(fixture.bounds(), bounds(2, 3, 25, 10));
}

/// The end of a gesture closes its group even when that last Resize Bounds is refused, so the
/// resizes before it remain one step; a single Resize Bounds sent in the middle of a gesture
/// closes it and is a step of its own.
#[test]
fn a_gesture_closes_however_it_ends() {
    let mut fixture = Fixture::new();
    let start = fixture.bounds();

    fixture.apply(resize_bounds(bounds(0, 0, 31, 30), Gesture::Begin));
    fixture.apply(resize_bounds(bounds(0, 0, 32, 30), Gesture::Continue));
    let refused = fixture.try_apply(resize_bounds(bounds(0, 0, 0, 30), Gesture::End));
    assert_eq!(refused.len(), 1, "the last Resize Bounds is refused");
    assert_eq!(fixture.bounds(), bounds(0, 0, 32, 30));
    assert_eq!(fixture.steps(), 1, "the resizes before it are one step");
    fixture.undo();
    assert_eq!(fixture.bounds(), start);

    fixture.apply(resize_bounds(bounds(0, 0, 31, 30), Gesture::Begin));
    fixture.apply(resize_bounds(bounds(0, 0, 32, 30), Gesture::Continue));
    fixture.apply(resize_bounds(bounds(0, 0, 40, 30), Gesture::Single));
    fixture.apply(resize_bounds(bounds(0, 0, 40, 30), Gesture::End));
    assert_eq!(fixture.bounds(), bounds(0, 0, 40, 30));
    assert_eq!(fixture.steps(), 2, "the gesture, then the single resize");
    fixture.undo();
    assert_eq!(fixture.bounds(), bounds(0, 0, 32, 30));
    fixture.undo();
    assert_eq!(fixture.bounds(), start);
}

/// A Resize Bounds to the Bounds as they are changes nothing and records no history step, and a
/// gesture none of whose Resize Bounds changes the Bounds records none.
#[test]
fn the_same_bounds_record_nothing() {
    let mut fixture = Fixture::new();
    fixture.resize(bounds(2, 3, 20, 10));
    fixture.resize(bounds(2, 3, 25, 10));
    fixture.undo();
    assert_eq!(fixture.steps(), 1);
    assert!(fixture.can_redo());

    fixture.resize(bounds(2, 3, 20, 10));
    assert_eq!(fixture.bounds(), bounds(2, 3, 20, 10));
    assert_eq!(fixture.steps(), 1, "a resize to the same Bounds is no step");
    assert!(
        fixture.can_redo(),
        "nothing recorded, so what could be redone still can be"
    );

    for gesture in [
        Gesture::Begin,
        Gesture::Continue,
        Gesture::Continue,
        Gesture::End,
    ] {
        fixture.apply(resize_bounds(bounds(2, 3, 20, 10), gesture));
    }
    assert_eq!(fixture.bounds(), bounds(2, 3, 20, 10));
    assert_eq!(
        fixture.steps(),
        1,
        "a gesture that never changes the Bounds is no step"
    );
    assert!(fixture.can_redo());

    fixture.redo();
    assert_eq!(fixture.bounds(), bounds(2, 3, 25, 10));
}

/// A Resize Bounds, its undo, and its redo change no Element on any Layer, inside the Bounds,
/// across their edge, or outside them: each keeps its `ElementId`, its place in the stacking order,
/// every property, and its derived shape and coverage.
#[test]
fn elements_stay() {
    let mut fixture = Fixture::new();
    fixture.prop(Vec2::new(2.5, 2.5));
    let wall = fixture.place(Placement::Wall {
        points: vec![Vec2::new(7.0, 5.0), Vec2::new(14.0, 5.0)],
        thickness: 0.25,
        colour: GREY,
    });
    let door = fixture.asset(DOOR);
    fixture.place(Placement::Portal {
        position: Vec2::ZERO,
        asset: door,
        anchor: Some(PortalAnchor {
            host: wall,
            index: 0,
            t: 0.5,
            side: Side::Left,
        }),
    });
    fixture.place(Placement::Room {
        points: vec![
            Vec2::new(20.0, 20.0),
            Vec2::new(26.0, 20.0),
            Vec2::new(26.0, 25.0),
            Vec2::new(20.0, 25.0),
        ],
        thickness: 0.125,
        wall_colour: GREY,
        floor_colour: LIGHT,
        cuts: false,
    });
    let layer = fixture.layer();
    let brush = BrushSettings {
        size: 2.0,
        hardness: 0.5,
        strength: 1.0,
    };
    let flagstones = fixture.asset(FLAGSTONES);
    for points in [
        vec![Vec2::new(15.0, 2.0), Vec2::new(22.0, 2.0)],
        vec![Vec2::new(18.0, -3.0), Vec2::new(18.0, 6.0)],
        vec![Vec2::new(12.0, 14.0), Vec2::new(16.0, 18.0)],
    ] {
        fixture.apply(Apply::Paint(Paint {
            layer,
            stroke: Stroke {
                points,
                brush,
                erase: false,
            },
            asset: Some(flagstones.clone()),
        }));
    }
    let before = fixture.everything();
    assert_eq!(
        before.len(),
        5,
        "a Prop, a Wall, a Portal, a Room, and a Terrain"
    );
    assert!(
        before
            .iter()
            .all(|placed| placed.wall.0.is_none() || placed.wall.1.is_some()),
        "the Wall has its derived shape"
    );
    assert!(
        before.iter().any(|placed| placed
            .terrain
            .1
            .as_ref()
            .is_some_and(|coverage| !coverage.tiles.is_empty())),
        "the Terrain has its coverage"
    );

    fixture.resize(bounds(0, 0, 10, 10));
    assert_eq!(fixture.everything(), before, "after the resize");
    fixture.undo();
    assert_eq!(fixture.bounds(), bounds(0, 0, 30, 30));
    assert_eq!(fixture.everything(), before, "after its undo");
    fixture.redo();
    assert_eq!(fixture.bounds(), bounds(0, 0, 10, 10));
    assert_eq!(fixture.everything(), before, "after its redo");
}

/// A Resize Bounds whose width or height is below 1 or above 1,000 cells is refused with the
/// limits named, changes nothing, and records no history step.
#[test]
fn at_least_a_cell_at_most_a_thousand() {
    let mut fixture = Fixture::new();
    let start = fixture.bounds();

    for refused in [
        bounds(0, 0, 0, 30),
        bounds(0, 0, 30, 0),
        bounds(0, 0, 1_001, 30),
        bounds(0, 0, 30, 1_001),
    ] {
        let reasons = fixture.try_resize(refused);
        assert_eq!(reasons.len(), 1, "{refused:?} is refused");
        assert!(
            reasons[0].contains("at least 1 and at most 1,000 cells"),
            "the limits are named: {}",
            reasons[0]
        );
        assert_eq!(fixture.bounds(), start, "nothing changes");
        assert_eq!(fixture.steps(), 0, "nothing is recorded");
    }

    fixture.resize(bounds(0, 0, 1, 1));
    assert_eq!(fixture.bounds(), bounds(0, 0, 1, 1));
    fixture.resize(bounds(-500, -500, 1_000, 1_000));
    assert_eq!(fixture.bounds(), bounds(-500, -500, 1_000, 1_000));
    assert_eq!(fixture.steps(), 2);
}

/// A Resize Bounds any of whose edges lies more than 10,000 cells from the Level's origin, to the
/// left, the right, below, or above, is refused with the limit named, changes nothing, and
/// records no history step.
#[test]
fn within_reach_of_the_origin() {
    let mut fixture = Fixture::new();

    for accepted in [
        bounds(-10_000, 0, 30, 30),
        bounds(9_970, 0, 30, 30),
        bounds(0, -10_000, 30, 30),
        bounds(0, 9_970, 30, 30),
    ] {
        fixture.resize(accepted);
        assert_eq!(
            fixture.bounds(),
            accepted,
            "an edge at 10,000 cells is accepted"
        );
    }
    let steps = fixture.steps();
    let last = fixture.bounds();

    for refused in [
        bounds(-10_001, 0, 30, 30),
        bounds(9_971, 0, 30, 30),
        bounds(0, -10_001, 30, 30),
        bounds(0, 9_971, 30, 30),
        // Near the largest whole number, where adding the size to the corner would overflow.
        bounds(i32::MAX - 5, i32::MAX - 5, 30, 30),
        bounds(i32::MIN, i32::MIN, 1_000, 1_000),
    ] {
        let reasons = fixture.try_resize(refused);
        assert_eq!(reasons.len(), 1, "{refused:?} is refused");
        assert!(
            reasons[0].contains("10,000 cells"),
            "the limit is named: {}",
            reasons[0]
        );
        assert_eq!(fixture.bounds(), last, "nothing changes");
        assert_eq!(fixture.steps(), steps, "nothing is recorded");
    }
}

/// Resize Bounds is one undo step in the one history, undone and redone in the order of every
/// other Command around it.
#[test]
fn resizing_shares_the_history() {
    let mut fixture = Fixture::new();
    let first = fixture.prop(Vec2::new(1.5, 1.5));
    fixture.resize(bounds(-4, -4, 12, 12));
    let second = fixture.prop(Vec2::new(5.5, 5.5));
    let ids = |fixture: &mut Fixture| -> Vec<ElementId> {
        fixture
            .everything()
            .iter()
            .map(|placed| placed.id)
            .collect()
    };
    assert_eq!(fixture.steps(), 3);

    fixture.undo();
    assert_eq!(ids(&mut fixture), vec![first]);
    assert_eq!(fixture.bounds(), bounds(-4, -4, 12, 12));
    fixture.undo();
    assert_eq!(ids(&mut fixture), vec![first]);
    assert_eq!(fixture.bounds(), bounds(0, 0, 30, 30));
    fixture.undo();
    assert!(ids(&mut fixture).is_empty());

    fixture.redo();
    assert_eq!(ids(&mut fixture), vec![first]);
    assert_eq!(fixture.bounds(), bounds(0, 0, 30, 30));
    fixture.redo();
    assert_eq!(fixture.bounds(), bounds(-4, -4, 12, 12));
    assert_eq!(ids(&mut fixture), vec![first]);
    fixture.redo();
    assert_eq!(ids(&mut fixture), vec![first, second]);
}
