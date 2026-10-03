//! Composing on a fixture Asset through the headless editor: the real plugins of the model, the
//! history, `LibraryAccess`, `LibraryManager`, `ProjectManager`, and `AuthoringManager` over one Asset
//! Folder of images with known pixel sizes, driven by messages and asserted on the World.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

mod support;

use bevy::app::App;
use bevy::asset::io::AssetSourceId;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::{ChildOf, Children};
use bevy::ecs::message::Messages;
use bevy::math::{UVec2, Vec2};
use drs_history::History;
use drs_library_access::{LIBRARY_SOURCE, asset_path};
use drs_model::{
    Apply, AssetAddress, AssetFolder, AssetFolderReference, AssetKind, AssetReferences,
    BrushSettings, CanonicalName, Colour, CommandFailed, EditElement, Element, ElementChange,
    ElementId, Fingerprint, FolderKey, Gesture, PROP, Paint, PlaceElement, Placement, Portal,
    PortalAnchor, Prop, RemoveElement, Resolution, ResolutionTable, Side, Stroke, Viewport,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// The Asset Folder every test places from: a table of 512 by 256 pixels, a barrel of 128 by
/// 128 pixels in a subfolder, and a crate of 128 by 128 pixels at a place full of symbols.
struct Fixture {
    /// Keeps the temporary directory alive for the test.
    _root: TempDir,
    /// The folder's path.
    folder: PathBuf,
    /// The folder's key once added.
    key: FolderKey,
    /// The headless editor with the folder added.
    app: App,
}

/// The place of the table image in the fixture folder.
const TABLE: &str = "table.png";
/// The place of the barrel image in the fixture folder.
const BARREL: &str = "props/barrel.png";
/// The place of the crate image: spaces, quotes, non-ASCII letters, and symbols, among them the
/// `#` and `?` an asset path would otherwise read as a label and a query.
#[cfg(not(windows))]
const CRATE: &str = "odd 'things' & more/caf\u{e9} #1? [v2].png";
/// The place of the crate image; Windows forbids `?` in file names.
#[cfg(windows)]
const CRATE: &str = "odd 'things' & more/caf\u{e9} #1 [v2].png";
/// The pixel size of the table image.
const TABLE_PIXELS: UVec2 = UVec2::new(512, 256);
/// The pixel size of the barrel image.
const BARREL_PIXELS: UVec2 = UVec2::new(128, 128);
/// The pixel size of the crate image.
const CRATE_PIXELS: UVec2 = UVec2::new(128, 128);

/// Writes an opaque PNG of `size` pixels at `place` under `folder`.
fn png(folder: &Path, place: &str, size: UVec2) {
    support::png(folder, place, size, [120, 80, 40, 255]);
}

impl Fixture {
    /// Creates the fixture folder, starts the editor, and adds the folder as `Fixtures`.
    fn new() -> Self {
        let root = TempDir::new().expect("temporary root");
        let folder = root.path().join("fixtures");
        png(&folder, TABLE, TABLE_PIXELS);
        png(&folder, BARREL, BARREL_PIXELS);
        png(&folder, CRATE, CRATE_PIXELS);
        let mut app = support::editor(root.path());
        let added = support::add_folder(&mut app, &folder, "Fixtures");
        Self {
            _root: root,
            folder,
            key: added.key,
            app,
        }
    }

    /// The one Layer of the new Project.
    fn layer(&mut self) -> Entity {
        support::first_layer(&mut self.app)
    }

    /// Sends a Command and runs one update, failing the test if the Command was refused.
    fn apply(&mut self, command: Apply) {
        support::apply(&mut self.app, command);
    }

    /// Places a Prop of the Asset at `place` centred on `position`, returning its identity.
    fn place(&mut self, place: &str, position: Vec2) -> ElementId {
        let layer = self.layer();
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Prop {
                position,
                asset: AssetAddress {
                    folder: self.key.clone(),
                    place: place.to_owned(),
                },
            },
        }));
        self.props()
            .last()
            .map(|prop| prop.id)
            .expect("the placed Prop is the last child of the Layer")
    }

    /// Moves an Element as part of a gesture.
    fn edit(&mut self, element: ElementId, position: Vec2, gesture: Gesture) {
        self.apply(support::edit(
            element,
            ElementChange::Position(position),
            gesture,
        ));
    }

    /// Removes an Element.
    fn remove(&mut self, element: ElementId) {
        self.apply(Apply::RemoveElement(RemoveElement { element }));
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

    /// The Props on the Layer in stacking order, bottom first.
    fn props(&mut self) -> Vec<PlacedProp> {
        let layer = self.layer();
        let world = self.app.world_mut();
        let children: Vec<Entity> = world
            .get::<Children>(layer)
            .map(|children| children.iter().copied().collect())
            .unwrap_or_default();
        children
            .into_iter()
            .map(|entity| PlacedProp {
                id: *world
                    .get::<ElementId>(entity)
                    .expect("an Element has an identity"),
                element: world
                    .get::<Element>(entity)
                    .expect("a child is an Element")
                    .clone(),
                prop: world
                    .get::<Prop>(entity)
                    .expect("the Element is a Prop")
                    .clone(),
            })
            .collect()
    }

    /// The Project's Asset Reference table.
    fn references(&mut self) -> AssetReferences {
        let world = self.app.world_mut();
        world
            .query::<&AssetReferences>()
            .single(world)
            .expect("exactly one Project")
            .clone()
    }

    /// Every Asset Folder in the World.
    fn folders(&mut self) -> Vec<AssetFolder> {
        let world = self.app.world_mut();
        world.query::<&AssetFolder>().iter(world).cloned().collect()
    }

    /// The Project's resolution table.
    fn resolutions(&mut self) -> Vec<Resolution> {
        let world = self.app.world_mut();
        world
            .query::<&ResolutionTable>()
            .single(world)
            .expect("exactly one Project")
            .rows
            .clone()
    }
}

/// What the World holds about one placed Prop.
#[derive(Debug, Clone, PartialEq)]
struct PlacedProp {
    /// Its identity.
    id: ElementId,
    /// Its kind, position, and size.
    element: Element,
    /// The Asset it shows.
    prop: Prop,
}

/// With an Asset chosen, a click on the Level places a Prop of that Asset on the current Layer,
/// centred on the clicked point.
#[test]
fn placed_where_clicked() {
    let mut fixture = Fixture::new();

    fixture.place(TABLE, Vec2::new(3.5, -2.25));

    let props = fixture.props();
    assert_eq!(props.len(), 1);
    assert_eq!(props[0].element.kind, PROP);
    assert_eq!(props[0].element.position, Vec2::new(3.5, -2.25));
    assert_eq!(fixture.history().undo_depth(), 2, "the folder and the Prop");
}

/// A Prop's size in Grid cells is its image's pixel size divided by 256 pixels per cell.
#[test]
fn natural_size() {
    let mut fixture = Fixture::new();

    fixture.place(TABLE, Vec2::ZERO);
    fixture.place(BARREL, Vec2::ZERO);

    let props = fixture.props();
    assert_eq!(props[0].element.size, Vec2::new(2.0, 1.0));
    assert_eq!(props[1].element.size, Vec2::new(0.5, 0.5));
}

/// A new Prop is placed above every Element already on its Layer.
#[test]
fn placed_on_top() {
    let mut fixture = Fixture::new();

    let first = fixture.place(TABLE, Vec2::ZERO);
    let second = fixture.place(BARREL, Vec2::ONE);
    let third = fixture.place(TABLE, Vec2::ONE);

    let order: Vec<ElementId> = fixture.props().iter().map(|prop| prop.id).collect();
    assert_eq!(order, vec![first, second, third]);
}

/// Placing a Prop records an Asset Reference holding the Asset's name, the Canonical Name of its
/// folder, the place it sits at, its byte size, its pixel size, and its content fingerprint; a
/// second Prop of the same Asset adds no second Asset Reference.
#[test]
fn placement_records_a_reference() {
    let mut fixture = Fixture::new();
    let bytes = fs::read(fixture.folder.join(TABLE)).expect("the fixture image");

    fixture.place(TABLE, Vec2::ZERO);
    let references = fixture.references();
    assert_eq!(references.assets.len(), 1);
    let reference = &references.assets[0];
    assert_eq!(reference.folder, CanonicalName("Fixtures".to_owned()));
    assert_eq!(reference.places, vec![TABLE]);
    assert_eq!(reference.name, "table");
    assert_eq!(reference.kind, AssetKind::IMAGE);
    assert_eq!(reference.byte_size, bytes.len() as u64);
    assert_eq!(reference.pixel_size, Some(TABLE_PIXELS));
    assert_eq!(
        reference.fingerprint,
        Fingerprint::blake3(&blake3::hash(&bytes).to_hex())
    );
    let row = fixture.props()[0].prop.asset;
    assert_eq!(references.get(row), Some(reference));

    fixture.place(TABLE, Vec2::ONE);
    assert_eq!(fixture.references().assets.len(), 1);
}

/// Placing a Prop records its Asset Folder's Canonical Name and version in the Project once.
#[test]
fn placement_records_the_folder() {
    let mut fixture = Fixture::new();
    let folder = fixture.folders().remove(0);
    assert!(!folder.version.is_empty(), "the folder has a version");

    fixture.place(TABLE, Vec2::ZERO);
    assert_eq!(
        fixture.references().folders,
        vec![AssetFolderReference {
            name: CanonicalName("Fixtures".to_owned()),
            version: folder.version,
        }]
    );

    fixture.place(BARREL, Vec2::ONE);
    assert_eq!(fixture.references().folders.len(), 1);
}

/// A Prop may be placed outside the Bounds.
#[test]
fn anywhere_on_the_level() {
    let mut fixture = Fixture::new();
    let far_outside = Vec2::new(-120.0, 450.5);

    fixture.place(TABLE, far_outside);

    let props = fixture.props();
    assert_eq!(props.len(), 1);
    assert_eq!(props[0].element.position, far_outside);
}

/// An Asset whose place holds spaces, quotes, non-ASCII letters, or symbols is placed like any
/// other: its own pixels are read for its size, its place is recorded and resolved as spelled,
/// and the asset path it is drawn from names the file and nothing else.
#[test]
fn any_path_works() {
    let mut fixture = Fixture::new();

    fixture.place(CRATE, Vec2::ONE);

    let props = fixture.props();
    assert_eq!(props.len(), 1);
    assert_eq!(
        props[0].element.size,
        Vec2::new(0.5, 0.5),
        "the size comes from the image's own pixels"
    );
    assert_eq!(fixture.references().assets[0].places, vec![CRATE]);
    assert_eq!(
        fixture.resolutions(),
        vec![Resolution::Resolved {
            folder: fixture.key.clone(),
            place: CRATE.to_owned()
        }]
    );
    let path = asset_path(&fixture.key, CRATE);
    assert_eq!(path.path(), Path::new(fixture.key.as_str()).join(CRATE));
    assert_eq!(path.label(), None, "a `#` in the name is not a label");
    assert_eq!(*path.source(), AssetSourceId::from(LIBRARY_SOURCE));
}

/// Several Props placed from the same Asset are independent Elements, each with its own
/// `ElementId`, sharing one Asset Reference.
#[test]
fn many_of_the_same() {
    let mut fixture = Fixture::new();

    let ids = [
        fixture.place(BARREL, Vec2::ZERO),
        fixture.place(BARREL, Vec2::X),
        fixture.place(BARREL, Vec2::Y),
    ];

    let props = fixture.props();
    assert_eq!(props.len(), 3);
    assert_ne!(ids[0], ids[1]);
    assert_ne!(ids[1], ids[2]);
    assert_ne!(ids[0], ids[2]);
    assert!(
        props
            .iter()
            .all(|prop| prop.prop.asset == props[0].prop.asset)
    );
    assert_eq!(fixture.references().assets.len(), 1);
}

/// Moving a Prop by dragging records a single undo step however long the drag, and undo returns
/// the Prop to where the drag began.
#[test]
fn a_drag_is_one_step() {
    let mut fixture = Fixture::new();
    let start = Vec2::new(1.0, 1.0);
    let end = Vec2::new(6.0, 2.5);
    let id = fixture.place(TABLE, start);
    let before = fixture.history().undo_depth();

    fixture.edit(id, Vec2::new(2.0, 1.5), Gesture::Begin);
    fixture.edit(id, Vec2::new(3.0, 2.0), Gesture::Continue);
    fixture.edit(id, Vec2::new(5.0, 2.5), Gesture::Continue);
    fixture.edit(id, end, Gesture::End);

    assert_eq!(fixture.props()[0].element.position, end);
    assert_eq!(fixture.history().undo_depth(), before + 1);
    fixture.undo();
    assert_eq!(fixture.props()[0].element.position, start);
    fixture.redo();
    assert_eq!(fixture.props()[0].element.position, end);
}

/// Undoing a Remove Element restores the Prop with every property, its `ElementId`, and its place
/// in the stacking order, however many Props sit above it.
#[test]
fn removal_is_reversible_in_place() {
    let mut fixture = Fixture::new();
    fixture.place(TABLE, Vec2::ZERO);
    let second = fixture.place(BARREL, Vec2::new(4.0, 4.0));
    fixture.place(TABLE, Vec2::ONE);
    fixture.place(BARREL, Vec2::new(2.0, 2.0));
    let before = fixture.props();

    fixture.remove(second);
    let remaining: Vec<ElementId> = fixture.props().iter().map(|prop| prop.id).collect();
    assert_eq!(remaining, vec![before[0].id, before[2].id, before[3].id]);

    fixture.undo();
    assert_eq!(fixture.props(), before);
}

/// An Element removed by undoing a Place Element and brought back by redo has the `ElementId` it
/// had before.
#[test]
fn identity_survives_undo() {
    let mut fixture = Fixture::new();
    let id = fixture.place(TABLE, Vec2::ZERO);

    fixture.undo();
    assert!(fixture.props().is_empty());

    fixture.redo();
    let props = fixture.props();
    assert_eq!(props.len(), 1);
    assert_eq!(props[0].id, id);
}

/// Redoing a Place Element, Edit Element, or Remove Element leaves the Level as it was before the
/// undo.
#[test]
fn redo_repeats_exactly() {
    let mut fixture = Fixture::new();
    fixture.place(TABLE, Vec2::ZERO);
    let moved = fixture.place(BARREL, Vec2::ONE);
    let after_place = fixture.props();
    fixture.undo();
    fixture.redo();
    assert_eq!(fixture.props(), after_place, "redo of Place Element");

    fixture.edit(moved, Vec2::new(7.0, -3.0), Gesture::Single);
    let after_edit = fixture.props();
    fixture.undo();
    fixture.redo();
    assert_eq!(fixture.props(), after_edit, "redo of Edit Element");

    fixture.remove(after_place[0].id);
    let after_remove = fixture.props();
    fixture.undo();
    fixture.redo();
    assert_eq!(fixture.props(), after_remove, "redo of Remove Element");
}

/// A Command applied after an undo discards the undone steps.
#[test]
fn a_new_step_clears_redo() {
    let mut fixture = Fixture::new();
    let first = fixture.place(TABLE, Vec2::ZERO);
    fixture.place(BARREL, Vec2::ONE);
    fixture.undo();
    assert!(fixture.history().can_redo());

    let third = fixture.place(TABLE, Vec2::new(2.0, 2.0));

    assert!(!fixture.history().can_redo());
    fixture.redo();
    let order: Vec<ElementId> = fixture.props().iter().map(|prop| prop.id).collect();
    assert_eq!(order, vec![first, third]);
}

/// A Command that cannot be carried out is answered with the reason; nothing is placed, changed,
/// or removed, and nothing is recorded.
#[test]
fn a_failed_command_is_reported() {
    let mut fixture = Fixture::new();
    let layer = fixture.layer();
    let placed = fixture.place(TABLE, Vec2::ZERO);
    let before = fixture.props();
    let depth = fixture.history().undo_depth();
    let unknown = ElementId::new();

    let commands = [
        Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Prop {
                position: Vec2::ZERO,
                asset: AssetAddress {
                    folder: fixture.key.clone(),
                    place: "nowhere.png".to_owned(),
                },
            },
        }),
        Apply::EditElement(EditElement {
            element: unknown,
            change: ElementChange::Position(Vec2::ONE),
            gesture: Gesture::Single,
        }),
        Apply::RemoveElement(RemoveElement { element: unknown }),
        Apply::Paint(Paint {
            layer,
            stroke: Stroke {
                points: vec![Vec2::ZERO],
                brush: BrushSettings {
                    size: 2.0,
                    hardness: 0.5,
                    strength: 1.0,
                },
                erase: false,
            },
            asset: Some(AssetAddress {
                folder: fixture.key.clone(),
                place: "nowhere.png".to_owned(),
            }),
        }),
    ];
    for command in commands {
        fixture.app.world_mut().write_message(command.clone());
        fixture.app.update();

        let failed: Vec<CommandFailed> = fixture
            .app
            .world_mut()
            .resource_mut::<Messages<CommandFailed>>()
            .drain()
            .collect();
        assert_eq!(failed.len(), 1, "{command:?}");
        assert_eq!(failed[0].command, command);
        let named = match &command {
            Apply::PlaceElement(_) | Apply::Paint(_) => "nowhere.png".to_owned(),
            Apply::EditElement(_)
            | Apply::RemoveElement(_)
            | Apply::SetPortalIntoWall(_)
            | Apply::FreePortal(_) => format!("{unknown:?}"),
        };
        assert!(
            failed[0].reason.contains(&named),
            "{} names {named}",
            failed[0].reason
        );
        assert_eq!(fixture.props(), before, "{command:?}");
        assert_eq!(fixture.history().undo_depth(), depth, "{command:?}");
        assert!(!fixture.history().can_redo());
    }
    assert_eq!(fixture.props()[0].id, placed);
}

/// A Command recorded as several steps that fails partway is taken back whole: nothing of it
/// stays applied, it records nothing, and what could be redone still can be.
#[test]
fn a_command_failing_halfway_is_taken_back() {
    let mut fixture = Fixture::new();
    let layer = fixture.layer();
    fixture.apply(Apply::PlaceElement(PlaceElement {
        layer,
        placement: Placement::Wall {
            points: vec![Vec2::ZERO, Vec2::new(4.0, 0.0)],
            thickness: 0.25,
            colour: Colour::rgb(60, 60, 60),
        },
    }));
    let wall = support::last_on(&mut fixture.app, layer);
    let anchor = PortalAnchor {
        host: wall,
        index: 0,
        t: 0.5,
        side: Side::Left,
    };
    fixture.apply(Apply::PlaceElement(PlaceElement {
        layer,
        placement: Placement::Portal {
            position: Vec2::ZERO,
            asset: AssetAddress {
                folder: fixture.key.clone(),
                place: TABLE.to_owned(),
            },
            anchor: Some(anchor),
        },
    }));
    let portal = support::last_on(&mut fixture.app, layer);
    fixture.apply(Apply::PlaceElement(PlaceElement {
        layer,
        placement: Placement::Prop {
            position: Vec2::ONE,
            asset: AssetAddress {
                folder: fixture.key.clone(),
                place: BARREL.to_owned(),
            },
        },
    }));
    let barrel = support::last_on(&mut fixture.app, layer);
    fixture.undo();
    // The Wall leaves its Layer and its Portal goes with it, so removing the Wall removes the
    // Portal first and then fails, as the Wall sits on no Layer.
    let (wall_entity, portal_entity) = (
        support::entity(&mut fixture.app, wall).expect("the Wall"),
        support::entity(&mut fixture.app, portal).expect("the Portal"),
    );
    let world = fixture.app.world_mut();
    let stray = world.spawn_empty().id();
    world.entity_mut(wall_entity).remove::<ChildOf>();
    world.entity_mut(portal_entity).insert(ChildOf(stray));
    fixture.app.update();
    let depth = fixture.history().undo_depth();

    let failed = support::try_apply(
        &mut fixture.app,
        Apply::RemoveElement(RemoveElement { element: wall }),
    );

    assert_eq!(failed.len(), 1, "the removal is refused with a reason");
    assert!(failed[0].contains("sits on no Layer"), "{}", failed[0]);
    let restored = support::entity(&mut fixture.app, portal).expect("the Portal is back");
    let world = fixture.app.world();
    assert_eq!(
        world
            .get::<Portal>(restored)
            .and_then(|portal| portal.anchor),
        Some(anchor),
        "the Portal is set into its Wall again"
    );
    assert_eq!(
        world.get::<ChildOf>(restored).map(ChildOf::parent),
        Some(stray),
        "the Portal is back where it was"
    );
    assert!(
        support::entity(&mut fixture.app, wall).is_some(),
        "the Wall stays"
    );
    assert_eq!(fixture.history().undo_depth(), depth, "nothing is recorded");
    assert!(
        fixture.history().can_redo(),
        "what was undone can still be redone"
    );
    fixture.redo();
    assert_eq!(support::last_on(&mut fixture.app, layer), barrel);
}

/// Add Asset Folder, Place Element, Edit Element, and Remove Element are each one undo step, and
/// undo walks back through them in the order they were applied whichever Manager handled them.
#[test]
fn one_history() {
    let mut fixture = Fixture::new();
    let id = fixture.place(TABLE, Vec2::ZERO);
    fixture.edit(id, Vec2::new(3.0, 4.0), Gesture::Single);
    fixture.remove(id);
    assert_eq!(fixture.history().undo_depth(), 4);
    assert!(fixture.props().is_empty());

    fixture.undo();
    assert_eq!(fixture.props()[0].element.position, Vec2::new(3.0, 4.0));

    fixture.undo();
    assert_eq!(fixture.props()[0].element.position, Vec2::ZERO);

    fixture.undo();
    assert!(fixture.props().is_empty());
    assert_eq!(fixture.folders().len(), 1, "the folder outlives the Prop");

    fixture.undo();
    assert!(fixture.folders().is_empty());
    assert!(!fixture.history().can_undo());
}

/// Panning and zooming change the view, never the Level, and record no history step.
#[test]
fn view_is_not_a_step() {
    let mut fixture = Fixture::new();
    fixture.place(TABLE, Vec2::ZERO);
    fixture.undo();
    let props = fixture.props();
    let depth = fixture.history().undo_depth();
    let position = fixture.history().position();

    {
        let mut viewport = fixture.app.world_mut().resource_mut::<Viewport>();
        viewport.centre = Vec2::new(7.0, -3.0);
        viewport.zoom *= 2.0;
    }
    fixture.app.update();

    assert_eq!(fixture.history().undo_depth(), depth);
    assert_eq!(fixture.history().position(), position);
    assert!(
        fixture.history().can_redo(),
        "the view took nothing away from redo"
    );
    assert_eq!(fixture.props(), props);
    assert_eq!(
        fixture.app.world().resource::<Viewport>().centre,
        Vec2::new(7.0, -3.0)
    );
}
