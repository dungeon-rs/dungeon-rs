//! Composing on a fixture Asset through the headless editor: the real plugins of the model, the
//! history, `LibraryAccess`, `LibraryManager`, `ProjectManager`, and `AuthoringManager` over one Asset
//! Folder of images with known pixel sizes, driven by messages and asserted on the World.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::message::Messages;
use bevy::math::{UVec2, Vec2};
use drs_authoring_manager::AuthoringManagerPlugin;
use drs_history::{History, HistoryPlugin};
use drs_library_access::LibraryAccessPlugin;
use drs_library_manager::LibraryManagerPlugin;
use drs_model::{
    AddFolder, Apply, AssetFolder, AssetKind, AssetReferences, CanonicalName, ChosenAsset,
    CommandFailed, EditElement, EditorDirectories, Element, ElementChange, ElementId, Fingerprint,
    FolderAdded, FolderKey, FolderRefused, Gesture, Layer, ModelPlugin, PROP, PlaceElement, Prop,
    Redo, RemoveElement, Undo,
};
use drs_project_manager::ProjectManagerPlugin;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// The Asset Folder every test places from: a table of 512 by 256 pixels and a barrel of 128 by
/// 128 pixels in a subfolder.
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
/// The pixel size of the table image.
const TABLE_PIXELS: UVec2 = UVec2::new(512, 256);
/// The pixel size of the barrel image.
const BARREL_PIXELS: UVec2 = UVec2::new(128, 128);

/// Writes an opaque PNG of `size` pixels at `place` under `folder`.
fn png(folder: &Path, place: &str, size: UVec2) {
    let path = folder.join(place);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture folder");
    }
    image::RgbaImage::from_pixel(size.x, size.y, image::Rgba([120, 80, 40, 255]))
        .save(&path)
        .expect("fixture image");
}

/// A headless editor whose configuration and cache directories live under `root`, started once.
fn editor(root: &Path) -> App {
    let mut app = App::new();
    app.insert_resource(EditorDirectories::under(root));
    app.add_plugins((
        ModelPlugin,
        HistoryPlugin,
        LibraryAccessPlugin,
        LibraryManagerPlugin,
        ProjectManagerPlugin,
        AuthoringManagerPlugin,
    ));
    app.update();
    app
}

/// Sends Add Asset Folder and returns what came back.
fn add_folder(app: &mut App, path: &Path, name: &str) -> FolderAdded {
    app.world_mut().write_message(AddFolder {
        path: path.to_path_buf(),
        name: CanonicalName(name.to_owned()),
    });
    app.update();
    let world = app.world_mut();
    let refused: Vec<FolderRefused> = world
        .resource_mut::<Messages<FolderRefused>>()
        .drain()
        .collect();
    assert!(
        refused.is_empty(),
        "the fixture folder was refused: {refused:?}"
    );
    world
        .resource_mut::<Messages<FolderAdded>>()
        .drain()
        .next()
        .expect("the fixture folder is added")
}

impl Fixture {
    /// Creates the fixture folder, starts the editor, and adds the folder as `Fixtures`.
    fn new() -> Self {
        let root = TempDir::new().expect("temporary root");
        let folder = root.path().join("fixtures");
        png(&folder, TABLE, TABLE_PIXELS);
        png(&folder, BARREL, BARREL_PIXELS);
        let mut app = editor(root.path());
        let added = add_folder(&mut app, &folder, "Fixtures");
        Self {
            _root: root,
            folder,
            key: added.key,
            app,
        }
    }

    /// The one Layer of the new Project.
    fn layer(&mut self) -> Entity {
        let world = self.app.world_mut();
        world
            .query::<(Entity, &Layer)>()
            .single(world)
            .expect("exactly one Layer")
            .0
    }

    /// Sends a Command and runs one update, failing the test if the Command was refused.
    fn apply(&mut self, command: Apply) {
        self.app.world_mut().write_message(command);
        self.app.update();
        let failed: Vec<CommandFailed> = self
            .app
            .world_mut()
            .resource_mut::<Messages<CommandFailed>>()
            .drain()
            .collect();
        assert!(failed.is_empty(), "the Command failed: {failed:?}");
    }

    /// Places a Prop of the Asset at `place` centred on `position`, returning its identity.
    fn place(&mut self, place: &str, position: Vec2) -> ElementId {
        let layer = self.layer();
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            position,
            asset: ChosenAsset {
                folder: self.key.clone(),
                place: place.to_owned(),
            },
        }));
        self.props()
            .last()
            .map(|prop| prop.id)
            .expect("the placed Prop is the last child of the Layer")
    }

    /// Moves an Element as part of a gesture.
    fn edit(&mut self, element: ElementId, position: Vec2, gesture: Gesture) {
        self.apply(Apply::EditElement(EditElement {
            element,
            change: ElementChange::Position(position),
            gesture,
        }));
    }

    /// Removes an Element.
    fn remove(&mut self, element: ElementId) {
        self.apply(Apply::RemoveElement(RemoveElement { element }));
    }

    /// Sends Undo and runs one update.
    fn undo(&mut self) {
        self.app.world_mut().write_message(Undo);
        self.app.update();
    }

    /// Sends Redo and runs one update.
    fn redo(&mut self) {
        self.app.world_mut().write_message(Redo);
        self.app.update();
    }

    /// The history.
    fn history(&self) -> &History {
        self.app.world().resource::<History>()
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
/// folder, its place, its byte size, its pixel size, and its content fingerprint; a second Prop
/// of the same Asset adds no second Asset Reference.
#[test]
fn placement_records_a_reference() {
    let mut fixture = Fixture::new();
    let bytes = fs::read(fixture.folder.join(TABLE)).expect("the fixture image");

    fixture.place(TABLE, Vec2::ZERO);
    let references = fixture.references();
    assert_eq!(references.assets.len(), 1);
    let reference = &references.assets[0];
    assert_eq!(reference.folder, CanonicalName("Fixtures".to_owned()));
    assert_eq!(reference.place, TABLE);
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
/// in the stacking order.
#[test]
fn removal_is_reversible_in_place() {
    let mut fixture = Fixture::new();
    fixture.place(TABLE, Vec2::ZERO);
    let middle = fixture.place(BARREL, Vec2::new(4.0, 4.0));
    fixture.place(TABLE, Vec2::ONE);
    let before = fixture.props();

    fixture.remove(middle);
    let remaining: Vec<ElementId> = fixture.props().iter().map(|prop| prop.id).collect();
    assert_eq!(remaining, vec![before[0].id, before[2].id]);

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

/// Add Asset Folder and Place Element are each one undo step, and undo walks back through them
/// in the order they were applied whichever Manager handled them.
#[test]
fn one_history() {
    let mut fixture = Fixture::new();
    fixture.place(TABLE, Vec2::ZERO);
    assert_eq!(fixture.history().undo_depth(), 2);

    fixture.undo();
    assert!(fixture.props().is_empty());
    assert_eq!(fixture.folders().len(), 1, "the folder outlives the Prop");

    fixture.undo();
    assert!(fixture.folders().is_empty());
    assert!(!fixture.history().can_undo());
}
