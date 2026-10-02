//! Setting Portals into Walls through the headless editor: the real plugins of the model, the
//! history, `LibraryAccess`, `LibraryManager`, `ProjectManager`, and `AuthoringManager`, with no
//! window and no render Engine, over one Asset Folder holding a door image of known pixel size,
//! driven by Apply, Undo, and Redo messages and asserted on the Portal and Element components,
//! the Wall's derived shape and its stretches, the Layer's children, the answers, and the
//! history.
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
    Apply, AssetAddress, AssetReferences, CanonicalName, Colour, Element, ElementChange, ElementId,
    FolderKey, FreePortal, Gesture, Layer, Level, LinePlace, PORTAL, PlaceElement, Placement,
    Portal, PortalAnchor, PortalsRemoved, Project, RemoveElement, SetPortalIntoWall, Side, Wall,
    WallShape,
};
use std::f32::consts::FRAC_PI_2;
use support::{assert_close, assert_near, edit, quadratic};
use tempfile::TempDir;

/// The place of the door image: 512 by 128 pixels, two cells wide and half a cell tall.
const DOOR: &str = "door.png";
/// The door's pixel size.
const DOOR_PIXELS: UVec2 = UVec2::new(512, 128);
/// The place of a square image of one cell, for Props.
const TABLE: &str = "table.png";
/// A dark grey.
const GREY: Colour = Colour::rgb(60, 60, 60);

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
    /// Its identity.
    id: ElementId,
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
        support::png(&folder, DOOR, DOOR_PIXELS, [150, 90, 40, 255]);
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

    /// The door as a Place Element names it.
    fn door(&self) -> AssetAddress {
        AssetAddress {
            folder: self.key.clone(),
            place: DOOR.to_owned(),
        }
    }

    /// Sends a Command and runs one update, returning the reasons of any failure.
    fn try_apply(&mut self, command: Apply) -> Vec<String> {
        support::try_apply(&mut self.app, command)
    }

    /// Sends a Command and runs one update, failing the test if the Command was refused.
    fn apply(&mut self, command: Apply) {
        support::apply(&mut self.app, command);
    }

    /// Sends a Command, failing the test unless it is refused with nothing recorded, nothing
    /// changed on the Layer, and no Asset Reference added to the Project.
    fn refused(&mut self, command: Apply) {
        let depth = self.history().undo_depth();
        let before = self.state();
        let references = self.references();
        let failed = self.try_apply(command.clone());
        assert_eq!(failed.len(), 1, "{command:?} is refused with a reason");
        assert_eq!(
            self.history().undo_depth(),
            depth,
            "{command:?} records nothing"
        );
        assert_eq!(self.state(), before, "{command:?} changes nothing");
        assert_eq!(
            self.references(),
            references,
            "{command:?} records no Asset Reference"
        );
    }

    /// Places a grey Wall an eighth of a cell thick through `points` on `layer`.
    fn wall_on(&mut self, layer: Entity, points: &[Vec2]) -> ElementId {
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Wall {
                points: points.to_vec(),
                thickness: 0.125,
                colour: GREY,
            },
        }));
        self.last_on(layer)
    }

    /// Places a grey Wall an eighth of a cell thick through `points` on the first Layer.
    fn wall(&mut self, points: &[Vec2]) -> ElementId {
        let layer = self.layer();
        self.wall_on(layer, points)
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

    /// Places a door set into `host` at `segment` and `t`, facing `side`.
    fn set_door(&mut self, host: ElementId, segment: usize, t: f32, side: Side) -> ElementId {
        let command = self.door_placement(
            Vec2::ZERO,
            Some(PortalAnchor {
                host,
                index: segment,
                t,
                side,
            }),
        );
        self.apply(command);
        let layer = self.layer();
        self.last_on(layer)
    }

    /// Places a freestanding door centred on `position`.
    fn free_door(&mut self, position: Vec2) -> ElementId {
        let command = self.door_placement(position, None);
        self.apply(command);
        let layer = self.layer();
        self.last_on(layer)
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
        self.last_on(layer)
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

    /// The identity of the last Element on `layer`.
    fn last_on(&mut self, layer: Entity) -> ElementId {
        support::last_on(&mut self.app, layer)
    }

    /// The identities on the first Layer, in stacking order.
    fn order(&mut self) -> Vec<ElementId> {
        let layer = self.layer();
        support::order(&mut self.app, layer)
    }

    /// The entity of an Element, if one carries the identity.
    fn entity(&mut self, id: ElementId) -> Option<Entity> {
        support::entity(&mut self.app, id)
    }

    /// The Portal with an identity.
    fn portal(&mut self, id: ElementId) -> Placed {
        let entity = self.entity(id).expect("the Portal exists");
        let world = self.app.world();
        Placed {
            id,
            element: world.get::<Element>(entity).expect("an Element").clone(),
            portal: world.get::<Portal>(entity).expect("a Portal").clone(),
        }
    }

    /// The derived shape of the Wall with an identity.
    fn shape(&mut self, id: ElementId) -> WallShape {
        let entity = self.entity(id).expect("the Wall exists");
        self.app
            .world()
            .get::<WallShape>(entity)
            .expect("the Wall has its shape")
            .clone()
    }

    /// The anchor of a Portal, which it must have.
    fn anchor(&mut self, id: ElementId) -> PortalAnchor {
        self.portal(id).portal.anchor.expect("the Portal is set")
    }

    /// Every Element of every Level with its Portal and Wall, by identity, for comparing whole
    /// states.
    fn state(&mut self) -> Vec<(ElementId, Element, Option<Portal>, Option<Wall>)> {
        let world = self.app.world_mut();
        let mut state: Vec<_> = world
            .query::<(&ElementId, &Element, Option<&Portal>, Option<&Wall>)>()
            .iter(world)
            .map(|(id, element, portal, wall)| {
                (*id, element.clone(), portal.cloned(), wall.cloned())
            })
            .collect();
        state.sort_by_key(|(id, ..)| *id);
        state
    }

    /// The Project's Asset Reference table.
    fn references(&mut self) -> AssetReferences {
        let world = self.app.world_mut();
        world
            .query::<&AssetReferences>()
            .single(world)
            .expect("one Project")
            .clone()
    }
}

/// The direction of that curve at `t`, as an angle.
fn quadratic_angle(start: Vec2, control: Vec2, end: Vec2, t: f32) -> f32 {
    let along = (control - start) * (2.0 * (1.0 - t)) + (end - control) * (2.0 * t);
    ops::atan2(along.y, along.x)
}

/// An L-shaped Wall: four cells along the x axis, then four up.
const CORNER: [Vec2; 3] = [Vec2::ZERO, Vec2::new(4.0, 0.0), Vec2::new(4.0, 4.0)];

/// A Portal shows an image Asset at a width in cells, with a height that keeps the image's
/// proportions, and is either freestanding, with a position, a rotation, and whether it is
/// mirrored, or set into a Wall, anchored by that Wall's `ElementId`, one of its segments, a
/// parameter along that segment, and a side.
#[test]
fn a_portal_is_an_image_with_a_width() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let free = fixture.free_door(Vec2::new(10.0, 10.0));
    let set = fixture.set_door(wall, 1, 0.5, Side::Right);

    let row = fixture
        .references()
        .row_of(&CanonicalName("Fixtures".to_owned()), DOOR)
        .expect("the door is recorded");
    let free = fixture.portal(free);
    assert_eq!(free.element.kind, PORTAL);
    assert_eq!(free.element.position, Vec2::new(10.0, 10.0));
    assert_eq!(
        free.portal,
        Portal {
            asset: row,
            width: 2.0,
            rotation: 0.0,
            mirrored: false,
            anchor: None,
        }
    );
    let set = fixture.portal(set);
    assert_eq!(set.element.kind, PORTAL);
    assert_eq!(set.portal.asset, row);
    assert_eq!(set.portal.width, 2.0);
    assert_eq!(
        set.portal.anchor,
        Some(PortalAnchor {
            host: wall,
            index: 1,
            t: 0.5,
            side: Side::Right,
        })
    );

    fixture.edit(free.id, ElementChange::Width(3.0));
    assert_eq!(
        fixture.portal(free.id).element.size,
        Vec2::new(3.0, 0.75),
        "the height keeps the image's proportions"
    );
}

/// A placed Portal's width and height are its image's pixel width and height divided by the Grid's
/// 256 pixels per cell.
#[test]
fn natural_width() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let free = fixture.free_door(Vec2::new(10.0, 10.0));
    let set = fixture.set_door(wall, 0, 0.5, Side::Left);

    for id in [free, set] {
        let placed = fixture.portal(id);
        assert_eq!(placed.portal.width, 2.0);
        assert_eq!(placed.element.size, Vec2::new(2.0, 0.5));
    }
}

/// A Portal set into a Wall is centred on the point of its segment at its parameter, with its width
/// along the segment's direction there and its image's top facing its side: drawn as it is when it
/// faces the left, mirrored across the Wall's line when it faces the right.
#[test]
fn set_portals_stand_on_the_line() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let control = Vec2::new(8.0, 2.0);
    let along = fixture.set_door(wall, 0, 0.25, Side::Left);
    let up = fixture.set_door(wall, 1, 0.5, Side::Right);

    let placed = fixture.portal(along);
    assert_near(
        placed.element.position,
        Vec2::new(1.0, 0.0),
        "on the x axis",
    );
    assert_close(placed.portal.rotation, 0.0, "along the x axis");
    assert!(!placed.portal.mirrored, "facing the left is drawn as it is");
    let placed = fixture.portal(up);
    assert_near(
        placed.element.position,
        Vec2::new(4.0, 2.0),
        "up the second",
    );
    assert_close(placed.portal.rotation, FRAC_PI_2, "turned to point up");
    assert!(placed.portal.mirrored, "facing the right is mirrored");

    fixture.edit(
        wall,
        ElementChange::Control {
            segment: 1,
            position: Some(control),
        },
    );
    let placed = fixture.portal(up);
    let (start, end) = (CORNER[1], CORNER[2]);
    assert_near(
        placed.element.position,
        quadratic(start, control, end, 0.5),
        "on the curve",
    );
    assert_close(
        placed.portal.rotation,
        quadratic_angle(start, control, end, 0.5),
        "turned to the curve",
    );
    fixture.edit(up, ElementChange::Side(Side::Left));
    assert!(!fixture.portal(up).portal.mirrored, "flipped to the left");
}

/// A Portal set into a Wall covers the stretch of the Wall's line that reaches half its width
/// either way from its centre, measured along the line and across the Wall's points, and stopping
/// at the Wall's ends.
#[test]
fn a_portal_covers_its_width() {
    let mut fixture = Fixture::new();
    let inside = fixture.wall(&[Vec2::ZERO, Vec2::new(8.0, 0.0)]);
    fixture.set_door(inside, 0, 0.5, Side::Left);
    let shape = fixture.shape(inside);
    assert_eq!(shape.stretches.len(), 1);
    let stretch = shape.stretches[0];
    assert_eq!(stretch.start.segment, 0);
    assert_close(stretch.start.t, 0.375, "a cell before the centre");
    assert_close(stretch.end.t, 0.625, "a cell after the centre");

    let corner = fixture.wall(&CORNER);
    fixture.set_door(corner, 0, 0.875, Side::Left);
    let stretch = fixture.shape(corner).stretches[0];
    assert_eq!(stretch.start.segment, 0);
    assert_close(stretch.start.t, 0.625, "a cell before the centre");
    assert_eq!(stretch.end.segment, 1, "across the point");
    assert_close(stretch.end.t, 0.125, "half a cell past the point");

    let end = fixture.wall(&[Vec2::ZERO, Vec2::new(8.0, 0.0)]);
    fixture.set_door(end, 0, 0.0625, Side::Left);
    let stretch = fixture.shape(end).stretches[0];
    assert_eq!(
        stretch.start,
        LinePlace { segment: 0, t: 0.0 },
        "stopped at the end"
    );
    assert_close(stretch.end.t, 0.1875, "a cell after the centre");
}

/// A Place Element of a Portal with an anchor places the Portal already set into the Wall, as one
/// history step that undo takes away whole and redo brings back set into the same place.
#[test]
fn placed_into_a_wall_at_once() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let depth = fixture.history().undo_depth();
    let door = fixture.set_door(wall, 0, 0.5, Side::Left);
    let placed = fixture.portal(door);
    assert_eq!(fixture.history().undo_depth(), depth + 1);
    assert_eq!(fixture.shape(wall).stretches.len(), 1);

    fixture.undo();
    assert_eq!(fixture.entity(door), None, "the Portal is taken away");
    assert!(
        fixture.shape(wall).stretches.is_empty(),
        "the Wall unbroken"
    );

    fixture.redo();
    assert_eq!(fixture.portal(door), placed, "set into the same place");
    assert_eq!(fixture.shape(wall).stretches.len(), 1);
}

/// A Set Portal into Wall anchors a Portal, freestanding or set into any Wall, to the given Wall,
/// segment, parameter, and side, as one history step that undo returns to the anchor, position,
/// rotation, and mirroring it had.
#[test]
fn set_into_a_wall() {
    let mut fixture = Fixture::new();
    let first = fixture.wall(&CORNER);
    let second = fixture.wall(&[Vec2::new(0.0, 6.0), Vec2::new(8.0, 6.0)]);
    let door = fixture.free_door(Vec2::new(2.0, 2.0));
    fixture.edit(door, ElementChange::Rotation(0.3));
    fixture.edit(door, ElementChange::Mirrored(true));
    let freestanding = fixture.portal(door);
    let depth = fixture.history().undo_depth();

    let anchor = PortalAnchor {
        host: first,
        index: 1,
        t: 0.25,
        side: Side::Left,
    };
    fixture.apply(Apply::SetPortalIntoWall(SetPortalIntoWall {
        portal: door,
        anchor,
    }));
    assert_eq!(fixture.history().undo_depth(), depth + 1);
    let set = fixture.portal(door);
    assert_eq!(set.portal.anchor, Some(anchor));
    assert_near(set.element.position, Vec2::new(4.0, 1.0), "on the Wall");
    assert_close(set.portal.rotation, FRAC_PI_2, "turned to the Wall");
    assert!(!set.portal.mirrored);
    assert_eq!(fixture.shape(first).stretches.len(), 1);

    let into_the_second = PortalAnchor {
        host: second,
        index: 0,
        t: 0.5,
        side: Side::Right,
    };
    fixture.apply(Apply::SetPortalIntoWall(SetPortalIntoWall {
        portal: door,
        anchor: into_the_second,
    }));
    assert_near(
        fixture.portal(door).element.position,
        Vec2::new(4.0, 6.0),
        "moved",
    );
    assert!(fixture.shape(first).stretches.is_empty());
    assert_eq!(fixture.shape(second).stretches.len(), 1);

    fixture.undo();
    assert_eq!(fixture.portal(door), set);
    fixture.undo();
    assert_eq!(fixture.portal(door), freestanding, "as it stood before");
    assert!(fixture.shape(first).stretches.is_empty());
}

/// A Free Portal makes a Portal set into a Wall freestanding with the position, rotation, and
/// mirroring it had, so nothing moves on the Level and its Wall is drawn whole again, as one
/// history step that undo returns to the same anchor.
#[test]
fn freed_where_it_stands() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let door = fixture.set_door(wall, 1, 0.5, Side::Right);
    let set = fixture.portal(door);
    let depth = fixture.history().undo_depth();

    fixture.apply(Apply::FreePortal(FreePortal { portal: door }));
    assert_eq!(fixture.history().undo_depth(), depth + 1);
    let free = fixture.portal(door);
    assert_eq!(free.portal.anchor, None);
    assert_eq!(free.element, set.element, "nothing moves");
    assert_eq!(free.portal.rotation, set.portal.rotation);
    assert_eq!(free.portal.mirrored, set.portal.mirrored);
    assert!(
        fixture.shape(wall).stretches.is_empty(),
        "the Wall whole again"
    );

    fixture.edit(wall, ElementChange::Position(Vec2::new(10.0, 10.0)));
    assert_eq!(fixture.portal(door), free, "no longer following its Wall");
    fixture.undo();

    fixture.undo();
    assert_eq!(fixture.portal(door), set, "the same anchor");
    assert_eq!(fixture.shape(wall).stretches.len(), 1);
}

/// A Set Portal into Wall or a Place Element of a Portal whose anchor names an Element that is not
/// a Wall, a Wall on another Level, a segment the Wall does not have, or a parameter outside 0 to
/// 1, a Free Portal of a freestanding Portal, and a Set Portal into Wall or Free Portal of an
/// Element that is not a Portal are answered with the reason, change nothing, record no history
/// step, and add no Asset Reference to the Project.
#[test]
fn refused_anchors() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let table = fixture.prop(Vec2::new(8.0, 8.0));
    let upstairs = fixture.second_level();
    let elsewhere = fixture.wall_on(upstairs, &CORNER);
    let anchor = |host, segment, t| PortalAnchor {
        host,
        index: segment,
        t,
        side: Side::Left,
    };
    let bad = [
        anchor(table, 0, 0.5),
        anchor(elsewhere, 0, 0.5),
        anchor(wall, 2, 0.5),
        anchor(wall, 0, 1.5),
        anchor(wall, 0, -0.25),
    ];
    // Placements are tried before any door stands, so a refused one would be the first to
    // record the door's Asset Reference.
    for anchor in bad {
        let placement = fixture.door_placement(Vec2::ZERO, Some(anchor));
        fixture.refused(placement);
    }
    let door = fixture.free_door(Vec2::new(2.0, 2.0));
    for anchor in bad {
        fixture.refused(Apply::SetPortalIntoWall(SetPortalIntoWall {
            portal: door,
            anchor,
        }));
    }
    fixture.refused(Apply::FreePortal(FreePortal { portal: door }));
    fixture.refused(Apply::FreePortal(FreePortal { portal: wall }));
    fixture.refused(Apply::SetPortalIntoWall(SetPortalIntoWall {
        portal: table,
        anchor: anchor(wall, 0, 0.5),
    }));
}

/// An Edit Element changing a set Portal's segment and parameter moves it along its Wall with its
/// side kept, and a drag of it records a single history step however long, which undo returns to
/// where the drag began.
#[test]
fn sliding_along_the_wall() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let door = fixture.set_door(wall, 0, 0.25, Side::Right);
    let start = fixture.portal(door);
    let depth = fixture.history().undo_depth();

    let slides = [
        (0, 0.5, Gesture::Begin),
        (0, 0.75, Gesture::Continue),
        (1, 0.25, Gesture::Continue),
        (1, 0.5, Gesture::End),
    ];
    for (segment, t, gesture) in slides {
        fixture.apply(edit(door, ElementChange::Along { segment, t }, gesture));
        let anchor = fixture.anchor(door);
        assert_eq!((anchor.index, anchor.t), (segment, t));
        assert_eq!(anchor.side, Side::Right, "the side is kept");
    }
    assert_near(
        fixture.portal(door).element.position,
        Vec2::new(4.0, 2.0),
        "slid",
    );
    assert_eq!(fixture.history().undo_depth(), depth + 1, "one step");

    fixture.undo();
    assert_eq!(fixture.portal(door), start, "where the drag began");
}

/// A Portal's width, a set Portal's side, and a freestanding Portal's position, rotation, and
/// mirroring are each changed through Edit Element, every change a step of its own outside a drag;
/// a width not above zero, a segment or parameter its Wall does not have, a position, rotation, or
/// mirroring of a set Portal, and a side or a place along a Wall of a freestanding Portal are
/// refused with the reason, change nothing, and record no history step.
#[test]
fn portals_stay_editable() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let set = fixture.set_door(wall, 0, 0.5, Side::Left);
    let free = fixture.free_door(Vec2::new(8.0, 8.0));

    let changes = [
        (set, ElementChange::Width(1.0)),
        (set, ElementChange::Side(Side::Right)),
        (free, ElementChange::Width(4.0)),
        (free, ElementChange::Position(Vec2::new(9.0, 9.0))),
        (free, ElementChange::Rotation(1.25)),
        (free, ElementChange::Mirrored(true)),
    ];
    for (id, change) in changes {
        let depth = fixture.history().undo_depth();
        fixture.edit(id, change.clone());
        assert_eq!(fixture.history().undo_depth(), depth + 1, "{change:?}");
    }
    let set_portal = fixture.portal(set);
    assert_eq!(set_portal.portal.width, 1.0);
    assert_eq!(set_portal.element.size, Vec2::new(1.0, 0.25));
    assert!(set_portal.portal.mirrored, "facing the right");
    let free_portal = fixture.portal(free);
    assert_eq!(free_portal.element.position, Vec2::new(9.0, 9.0));
    assert_eq!(free_portal.element.size, Vec2::new(4.0, 1.0));
    assert_eq!(free_portal.portal.rotation, 1.25);
    assert!(free_portal.portal.mirrored);

    let refusals = [
        (set, ElementChange::Width(0.0)),
        (free, ElementChange::Width(-1.0)),
        (set, ElementChange::Along { segment: 2, t: 0.5 }),
        (set, ElementChange::Along { segment: 0, t: 1.5 }),
        (set, ElementChange::Position(Vec2::new(1.0, 1.0))),
        (set, ElementChange::Rotation(0.5)),
        (set, ElementChange::Mirrored(false)),
        (free, ElementChange::Side(Side::Left)),
        (free, ElementChange::Along { segment: 0, t: 0.5 }),
    ];
    for (id, change) in refusals {
        fixture.refused(edit(id, change, Gesture::Single));
    }
}

/// Undoing the removal of a Portal set into a Wall restores it with its `ElementId`, its anchor,
/// and its place in the stacking order, and its Wall gives way again.
#[test]
fn removed_portals_return_set() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let door = fixture.set_door(wall, 1, 0.5, Side::Right);
    fixture.prop(Vec2::new(8.0, 8.0));
    let placed = fixture.portal(door);
    let order = fixture.order();

    fixture.apply(Apply::RemoveElement(RemoveElement { element: door }));
    assert_eq!(fixture.entity(door), None);
    assert!(fixture.shape(wall).stretches.is_empty(), "the gap closes");

    fixture.undo();
    assert_eq!(fixture.portal(door), placed);
    assert_eq!(fixture.order(), order, "its place in the stacking order");
    assert_eq!(
        fixture.shape(wall).stretches.len(),
        1,
        "the Wall gives way again"
    );
}

/// Moving a Wall, moving a point, setting or unsetting a control point, and changing the thickness
/// or the colour change no Portal's segment, parameter, or side; each Portal set into the Wall
/// stands at its parameter on its segment as the segment now is.
#[test]
fn moves_with_its_wall() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&[
        Vec2::ZERO,
        Vec2::new(4.0, 0.0),
        Vec2::new(8.0, 0.0),
        Vec2::new(8.0, 4.0),
    ]);
    let door = fixture.set_door(wall, 1, 0.5, Side::Left);
    let anchor = fixture.anchor(door);

    fixture.edit(
        wall,
        ElementChange::Point {
            index: 1,
            position: Vec2::new(4.0, 2.0),
        },
    );
    let placed = fixture.portal(door);
    assert_near(
        placed.element.position,
        Vec2::new(6.0, 1.0),
        "on its segment",
    );
    assert_close(
        placed.portal.rotation,
        ops::atan2(-2.0, 4.0),
        "turned with it",
    );

    fixture.edit(
        wall,
        ElementChange::Point {
            index: 3,
            position: Vec2::new(12.0, 9.0),
        },
    );
    assert_eq!(
        fixture.portal(door),
        placed,
        "two segments away, not at all"
    );

    let control = Vec2::new(6.0, 5.0);
    fixture.edit(
        wall,
        ElementChange::Control {
            segment: 1,
            position: Some(control),
        },
    );
    let (start, end) = (Vec2::new(4.0, 2.0), Vec2::new(8.0, 0.0));
    let bent = fixture.portal(door);
    assert_near(
        bent.element.position,
        quadratic(start, control, end, 0.5),
        "bent",
    );
    assert_close(
        bent.portal.rotation,
        quadratic_angle(start, control, end, 0.5),
        "turned to the curve",
    );

    fixture.edit(
        wall,
        ElementChange::Control {
            segment: 1,
            position: None,
        },
    );
    let straight = fixture.portal(door);
    assert_near(
        straight.element.position,
        placed.element.position,
        "back on the straight segment",
    );
    assert_close(
        straight.portal.rotation,
        placed.portal.rotation,
        "turned to it again",
    );

    let before = fixture.portal(door).element.position;
    let wall_box = fixture.state();
    let centre = wall_box
        .iter()
        .find(|(id, ..)| *id == wall)
        .map(|(_, element, ..)| element.position)
        .expect("the Wall");
    fixture.edit(wall, ElementChange::Position(centre + Vec2::new(3.0, -1.0)));
    assert_near(
        fixture.portal(door).element.position,
        before + Vec2::new(3.0, -1.0),
        "moved with the Wall",
    );

    let stretches = fixture.shape(wall).stretches;
    fixture.edit(wall, ElementChange::Thickness(0.75));
    fixture.edit(wall, ElementChange::Colour(Colour::rgb(200, 30, 30)));
    assert_eq!(
        fixture.shape(wall).stretches,
        stretches,
        "the gap unchanged"
    );
    assert_eq!(fixture.anchor(door), anchor, "the anchor never changed");
}

/// Adding a point on segment k at parameter s moves a Portal on segment k at a parameter t below s
/// to t / s on segment k, one at or above s to (t − s) / (1 − s) on segment k + 1, and every Portal
/// on a later segment one segment on, in the same history step, so no Portal moves on the Level.
#[test]
fn adding_a_point_keeps_portals_in_place() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&[Vec2::ZERO, Vec2::new(8.0, 0.0), Vec2::new(16.0, 0.0)]);
    fixture.edit(
        wall,
        ElementChange::Control {
            segment: 1,
            position: Some(Vec2::new(12.0, 6.0)),
        },
    );
    let before = fixture.set_door(wall, 0, 0.25, Side::Left);
    let after = fixture.set_door(wall, 0, 0.75, Side::Right);
    let curved = fixture.set_door(wall, 1, 0.4, Side::Left);
    let doors = [before, after, curved];
    let centres: Vec<Vec2> = doors
        .iter()
        .map(|door| fixture.portal(*door).element.position)
        .collect();
    let depth = fixture.history().undo_depth();

    fixture.edit(wall, ElementChange::AddPoint { segment: 0, t: 0.5 });
    assert_eq!(fixture.history().undo_depth(), depth + 1, "one step");
    let places: Vec<(usize, f32)> = doors
        .iter()
        .map(|door| {
            let anchor = fixture.anchor(*door);
            (anchor.index, anchor.t)
        })
        .collect();
    assert_eq!(places, vec![(0, 0.5), (1, 0.5), (2, 0.4)]);
    assert_eq!(fixture.anchor(after).side, Side::Right);

    fixture.edit(
        wall,
        ElementChange::AddPoint {
            segment: 2,
            t: 0.25,
        },
    );
    assert_eq!(fixture.anchor(curved).index, 3);
    assert_close(fixture.anchor(curved).t, 0.2, "past the new point");
    for (door, centre) in doors.iter().zip(&centres) {
        assert_near(fixture.portal(*door).element.position, *centre, "in place");
    }

    fixture.undo();
    fixture.undo();
    assert_eq!(
        fixture.anchor(after),
        PortalAnchor {
            host: wall,
            index: 0,
            t: 0.75,
            side: Side::Right
        }
    );
}

/// Removing an inner point moves each Portal on the two segments it joins whose stretch does not
/// cover it onto the joined segment, at the share of the two segments' combined length that lay
/// before the Portal's centre, and every Portal on a later segment one segment back; removing the
/// first point moves every Portal on a remaining segment one segment back; all in the same history
/// step.
#[test]
fn removing_a_point_carries_the_portals_beside_it() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&[
        Vec2::ZERO,
        Vec2::new(4.0, 0.0),
        Vec2::new(12.0, 0.0),
        Vec2::new(12.0, 4.0),
    ]);
    let first = fixture.set_door(wall, 0, 0.25, Side::Left);
    let second = fixture.set_door(wall, 1, 0.75, Side::Right);
    let later = fixture.set_door(wall, 2, 0.5, Side::Left);
    let depth = fixture.history().undo_depth();

    fixture.edit(wall, ElementChange::RemovePoint { index: 1 });
    assert_eq!(fixture.history().undo_depth(), depth + 1, "one step");
    assert_eq!(fixture.anchor(first).index, 0);
    assert_close(fixture.anchor(first).t, 1.0 / 12.0, "a cell of twelve");
    assert_eq!(fixture.anchor(second).index, 0);
    assert_close(fixture.anchor(second).t, 10.0 / 12.0, "ten cells of twelve");
    assert_eq!(fixture.anchor(second).side, Side::Right);
    assert_eq!(
        (fixture.anchor(later).index, fixture.anchor(later).t),
        (1, 0.5)
    );
    assert_near(
        fixture.portal(first).element.position,
        Vec2::new(1.0, 0.0),
        "stays",
    );
    assert_near(
        fixture.portal(second).element.position,
        Vec2::new(10.0, 0.0),
        "stays",
    );
    assert!(fixture.removed().is_empty(), "nothing removed");

    fixture.edit(wall, ElementChange::RemovePoint { index: 0 });
    assert_eq!(fixture.entity(first), None, "gone with the first segment");
    assert_eq!(
        (fixture.anchor(later).index, fixture.anchor(later).t),
        (0, 0.5),
        "one segment back"
    );
}

/// A Portal whose stretch covers an inner point being removed is removed in the same history step,
/// which undo restores whole; the Author is told how many Portals were removed.
#[test]
fn gone_with_a_covered_point() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let covering = fixture.set_door(wall, 0, 0.875, Side::Left);
    let beside = fixture.set_door(wall, 1, 0.75, Side::Left);
    let before = fixture.state();
    let depth = fixture.history().undo_depth();

    fixture.edit(wall, ElementChange::RemovePoint { index: 1 });
    assert_eq!(fixture.history().undo_depth(), depth + 1, "one step");
    assert_eq!(
        fixture.entity(covering),
        None,
        "the covering Portal is gone"
    );
    assert!(fixture.entity(beside).is_some(), "the other stays");
    assert_eq!(
        fixture.removed(),
        vec![PortalsRemoved {
            host: wall,
            portals: vec![covering]
        }]
    );

    fixture.undo();
    assert_eq!(fixture.state(), before, "restored whole");
}

/// A Portal on the segment that removing an end point takes away is removed in the same history
/// step, which undo restores whole; the Author is told.
#[test]
fn gone_with_an_end_segment() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let first = fixture.set_door(wall, 0, 0.5, Side::Left);
    let last = fixture.set_door(wall, 1, 0.5, Side::Left);
    let before = fixture.state();

    fixture.edit(wall, ElementChange::RemovePoint { index: 2 });
    assert_eq!(fixture.entity(last), None);
    assert_eq!(fixture.anchor(first).index, 0);
    assert_eq!(
        fixture.removed(),
        vec![PortalsRemoved {
            host: wall,
            portals: vec![last]
        }]
    );

    fixture.undo();
    assert_eq!(fixture.state(), before);
}

/// Every Portal set into a Wall removed by Remove Element is removed in the same history step,
/// which undo restores whole; the Author is told how many.
#[test]
fn gone_with_the_wall() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let doors = [
        fixture.set_door(wall, 0, 0.5, Side::Left),
        fixture.set_door(wall, 1, 0.5, Side::Right),
    ];
    let before = fixture.state();
    let order = fixture.order();
    let depth = fixture.history().undo_depth();

    fixture.apply(Apply::RemoveElement(RemoveElement { element: wall }));
    assert_eq!(fixture.history().undo_depth(), depth + 1, "one step");
    assert!(
        fixture.order().is_empty(),
        "the Wall and its Portals are gone"
    );
    let mut removed = fixture.removed();
    assert_eq!(removed.len(), 1);
    let mut named = removed.remove(0);
    named.portals.sort();
    let mut expected = doors.to_vec();
    expected.sort();
    assert_eq!(
        named,
        PortalsRemoved {
            host: wall,
            portals: expected
        }
    );

    fixture.undo();
    assert_eq!(fixture.state(), before);
    assert_eq!(fixture.order(), order);
    assert_eq!(fixture.shape(wall).stretches.len(), 2);
}

/// Removing a point of a two-point Wall removes the Wall and every Portal set into it in the same
/// history step, which undo restores whole; the Author is told.
#[test]
fn gone_with_a_two_point_wall() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&[Vec2::ZERO, Vec2::new(6.0, 0.0)]);
    let door = fixture.set_door(wall, 0, 0.5, Side::Left);
    let before = fixture.state();
    let depth = fixture.history().undo_depth();

    fixture.edit(wall, ElementChange::RemovePoint { index: 0 });
    assert_eq!(fixture.history().undo_depth(), depth + 1);
    assert!(fixture.order().is_empty());
    assert_eq!(
        fixture.removed(),
        vec![PortalsRemoved {
            host: wall,
            portals: vec![door]
        }]
    );

    fixture.undo();
    assert_eq!(fixture.state(), before);
    assert_eq!(fixture.shape(wall).stretches.len(), 1);
}

/// No edit of any Wall moves, turns, or removes a freestanding Portal.
#[test]
fn freestanding_portals_stay_put() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let door = fixture.free_door(Vec2::new(2.0, 0.0));
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
        fixture.edit(wall, change.clone());
        assert_eq!(fixture.portal(door), free, "{change:?}");
    }
    fixture.apply(Apply::RemoveElement(RemoveElement { element: wall }));
    assert_eq!(fixture.portal(door), free, "the Wall removed");
    assert!(fixture.removed().is_empty());
}

/// A Portal set into a Wall at a place where its segment has no direction, a segment of no length
/// or a curve whose control point lies on its end, keeps the rotation it had.
#[test]
fn no_direction_keeps_the_rotation() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let door = fixture.set_door(wall, 1, 0.5, Side::Left);
    assert_close(fixture.portal(door).portal.rotation, FRAC_PI_2, "up");

    fixture.edit(
        wall,
        ElementChange::Point {
            index: 2,
            position: CORNER[1],
        },
    );
    let collapsed = fixture.portal(door);
    assert_near(collapsed.element.position, CORNER[1], "on the point");
    assert_close(collapsed.portal.rotation, FRAC_PI_2, "the angle it had");

    fixture.undo();
    fixture.edit(
        wall,
        ElementChange::Control {
            segment: 1,
            position: Some(CORNER[2]),
        },
    );
    fixture.edit(
        wall,
        ElementChange::Point {
            index: 1,
            position: Vec2::new(4.0, -2.0),
        },
    );
    fixture.apply(edit(
        door,
        ElementChange::Along { segment: 1, t: 1.0 },
        Gesture::Single,
    ));
    let pinched = fixture.portal(door);
    assert_near(pinched.element.position, CORNER[2], "at the end");
    assert_close(pinched.portal.rotation, FRAC_PI_2, "the angle it had");
}

/// Placing a Prop or a Portal records in the Project an Asset Reference; placing a second Element
/// of the same Asset adds no second Asset Reference.
#[test]
fn portals_record_a_reference() {
    let mut fixture = Fixture::new();
    fixture.free_door(Vec2::new(1.0, 1.0));
    let references = fixture.references();
    assert_eq!(references.assets.len(), 1);
    let reference = &references.assets[0];
    assert_eq!(reference.places, vec![DOOR.to_owned()]);
    assert_eq!(reference.pixel_size, Some(DOOR_PIXELS));

    let wall = fixture.wall(&CORNER);
    fixture.set_door(wall, 0, 0.5, Side::Left);
    assert_eq!(fixture.references().assets.len(), 1, "no second reference");
}

/// The first Prop or Portal placed from an Asset Folder records that folder's Canonical Name and
/// version in the Project; later Elements from the same folder add no second record.
#[test]
fn portals_record_the_folder() {
    let mut fixture = Fixture::new();
    fixture.free_door(Vec2::new(1.0, 1.0));
    let folders = fixture.references().folders;
    assert_eq!(folders.len(), 1);
    assert_eq!(folders[0].name, CanonicalName("Fixtures".to_owned()));

    fixture.prop(Vec2::new(3.0, 3.0));
    assert_eq!(fixture.references().folders, folders);
}

/// Place Element, Edit Element, Remove Element, Set Portal into Wall, and Free Portal are each one
/// undo step, and undo walks back through them in the order they were applied.
#[test]
fn portal_commands_share_the_history() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let mut states = vec![fixture.state()];
    let door = fixture.free_door(Vec2::new(2.0, 2.0));
    states.push(fixture.state());
    let table = fixture.prop(Vec2::new(8.0, 8.0));
    states.push(fixture.state());
    let steps = [
        Apply::SetPortalIntoWall(SetPortalIntoWall {
            portal: door,
            anchor: PortalAnchor {
                host: wall,
                index: 0,
                t: 0.5,
                side: Side::Left,
            },
        }),
        edit(door, ElementChange::Side(Side::Right), Gesture::Single),
        edit(
            wall,
            ElementChange::AddPoint {
                segment: 0,
                t: 0.25,
            },
            Gesture::Single,
        ),
        edit(
            table,
            ElementChange::Position(Vec2::new(9.0, 9.0)),
            Gesture::Single,
        ),
        Apply::FreePortal(FreePortal { portal: door }),
        Apply::RemoveElement(RemoveElement { element: door }),
    ];
    for step in steps {
        fixture.apply(step);
        states.push(fixture.state());
    }

    states.pop();
    while let Some(expected) = states.pop() {
        fixture.undo();
        assert_eq!(fixture.state(), expected);
        if states.len() == 1 {
            break;
        }
    }
}

/// Redoing a Place Element, Edit Element, Remove Element, Set Portal into Wall, or Free Portal
/// leaves the Level as it was before the undo.
#[test]
fn portal_commands_redo_exactly() {
    let mut fixture = Fixture::new();
    let wall = fixture.wall(&CORNER);
    let door = fixture.set_door(wall, 1, 0.5, Side::Left);
    let free = fixture.free_door(Vec2::new(8.0, 8.0));
    let steps = [
        fixture.door_placement(Vec2::new(5.0, 5.0), None),
        edit(
            door,
            ElementChange::Along {
                segment: 0,
                t: 0.25,
            },
            Gesture::Single,
        ),
        edit(free, ElementChange::Rotation(0.5), Gesture::Single),
        edit(
            wall,
            ElementChange::RemovePoint { index: 1 },
            Gesture::Single,
        ),
        Apply::SetPortalIntoWall(SetPortalIntoWall {
            portal: free,
            anchor: PortalAnchor {
                host: wall,
                index: 0,
                t: 0.75,
                side: Side::Right,
            },
        }),
        Apply::FreePortal(FreePortal { portal: door }),
        Apply::RemoveElement(RemoveElement { element: wall }),
    ];
    for step in steps {
        fixture.apply(step.clone());
        let after = fixture.state();
        fixture.undo();
        fixture.redo();
        assert_eq!(fixture.state(), after, "{step:?}");
    }
}

/// A Portal placed, set into a Wall or freestanding, lands on top of the Elements already on the
/// Layer, the Wall it is set into among them.
#[test]
fn portals_are_placed_on_top() {
    let mut fixture = Fixture::new();
    let table = fixture.prop(Vec2::new(1.0, 1.0));
    let wall = fixture.wall(&CORNER);
    let set = fixture.set_door(wall, 0, 0.5, Side::Left);
    let free = fixture.free_door(Vec2::new(1.0, 1.0));

    assert_eq!(fixture.order(), vec![table, wall, set, free]);
}

/// A drag of a freestanding Portal, however many moves it takes, is one undo step, and undo
/// returns it to where the drag began.
#[test]
fn a_freestanding_portal_drag_is_one_step() {
    let mut fixture = Fixture::new();
    let door = fixture.free_door(Vec2::new(1.0, 1.0));
    let depth = fixture.history().undo_depth();

    let path = [
        (Vec2::new(2.0, 1.0), Gesture::Begin),
        (Vec2::new(3.0, 2.0), Gesture::Continue),
        (Vec2::new(4.0, 3.0), Gesture::Continue),
        (Vec2::new(4.0, 3.0), Gesture::End),
    ];
    for (position, gesture) in path {
        fixture.apply(edit(door, ElementChange::Position(position), gesture));
    }
    assert_eq!(fixture.portal(door).element.position, Vec2::new(4.0, 3.0));
    assert_eq!(fixture.history().undo_depth(), depth + 1);

    fixture.undo();
    assert_eq!(fixture.portal(door).element.position, Vec2::new(1.0, 1.0));
    fixture.redo();
    assert_eq!(fixture.portal(door).element.position, Vec2::new(4.0, 3.0));
}
