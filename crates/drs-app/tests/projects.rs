//! Saving and reopening Projects through the headless editor: the real plugins of the model, the
//! history, `LibraryAccess`, `LibraryManager`, `ProjectManager`, and `AuthoringManager` over fixture
//! Asset Folders and Project files in temporary directories, driven by messages and asserted on
//! the World, the files written, and the answers. A second editor over other directories plays
//! the other device.
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
use bevy::math::{IVec2, UVec2, Vec2};
use drs_authoring_manager::AuthoringManagerPlugin;
use drs_history::{History, HistoryPlugin};
use drs_library_access::{
    LibraryAccessPlugin, LibraryDirectories, Manifest, asset_path, write_manifest,
};
use drs_library_manager::LibraryManagerPlugin;
use drs_model::{
    AddFolder, Apply, AssetAddress, AssetReferences, Bounds, CanonicalName, Colour, CommandFailed,
    EditElement, EditorDirectories, Element, ElementChange, ElementId, ElementKindName,
    ElementKindRegistry, FolderAdded, FolderKey, FolderRefused, Gesture, Grid, Layer, Level,
    MissingAsset, MissingReason, ModelPlugin, OpenProject, PlaceElement, Placement, Project,
    ProjectOpened, ProjectRefused, ProjectRequest, ProjectSaved, Prop, Redo, Resolution,
    ResolutionTable, SaveProject, SavedMark, Serialisable, SerialisationRegistry, Undo,
    UnknownComponents, UnknownKind, Viewport, WALL, Wall, WallShape,
};
use drs_project_manager::ProjectManagerPlugin;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// The place of the table image in the fixture folder.
const TABLE: &str = "table.png";
/// The place of the barrel image in the fixture folder.
const BARREL: &str = "props/barrel.png";
/// The place of a crate image: spaces, quotes, non-ASCII letters, and symbols, among them the `#`
/// and `?` an asset path would otherwise read as a label and a query.
#[cfg(not(windows))]
const CRATE: &str = "odd 'things' & more/caf\u{e9} #1? [v2].png";
/// The place of a crate image; Windows forbids `?` in file names.
#[cfg(windows)]
const CRATE: &str = "odd 'things' & more/caf\u{e9} #1 [v2].png";
/// The pixel size of the table image.
const TABLE_PIXELS: UVec2 = UVec2::new(512, 256);
/// The pixel size of the barrel image.
const BARREL_PIXELS: UVec2 = UVec2::new(128, 128);
/// The Canonical Name the fixture folder is added under.
const FIXTURES: &str = "Fixtures";

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

/// A folder of images at `root/<name>`, each at its place with its pixel size.
fn library(root: &Path, name: &str, files: &[(&str, UVec2)]) -> PathBuf {
    let folder = root.join(name);
    fs::create_dir_all(&folder).expect("library folder");
    for (place, size) in files {
        png(&folder, place, *size);
    }
    folder
}

/// The fixture folder as it is on the device that saves: a table and a barrel.
fn fixtures(root: &Path) -> PathBuf {
    library(
        root,
        "library",
        &[(TABLE, TABLE_PIXELS), (BARREL, BARREL_PIXELS)],
    )
}

/// The file at `path` as JSON.
fn json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("the Project file is readable"))
        .expect("the Project file is JSON")
}

/// Writes `value` pretty-printed to `path`.
fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_string_pretty(value).expect("JSON")).expect("written");
}

/// What the World holds about one Element on the Layer.
#[derive(Debug, Clone, PartialEq)]
struct Placed {
    /// Its identity.
    id: ElementId,
    /// Its kind, position, and size.
    element: Element,
    /// The Asset it shows, when it is a Prop.
    prop: Option<Prop>,
    /// The envelopes this editor does not know, when it carries any.
    unknown: Option<UnknownComponents>,
}

/// One device: a headless editor whose own files live under a temporary root of its own.
struct Device {
    /// Keeps the root alive for the device.
    root: TempDir,
    /// The editor.
    app: App,
}

impl Device {
    /// A device with no Asset Folders, started once.
    fn new() -> Self {
        let root = TempDir::new().expect("temporary root");
        let app = editor(root.path());
        Self { root, app }
    }

    /// A device that remembers the folder at `path` under `name` at `version` from before the
    /// editor started, so the version is not today's.
    fn remembering(path: &Path, name: &str, version: &str) -> Self {
        let root = TempDir::new().expect("temporary root");
        let directories = LibraryDirectories::resolve(&EditorDirectories::under(root.path()))
            .expect("the directories under the root");
        let mut manifest = Manifest::new(path.to_path_buf(), CanonicalName(name.to_owned()));
        version.clone_into(&mut manifest.version);
        write_manifest(&directories, &manifest).expect("the Manifest is written");
        let app = editor(root.path());
        Self { root, app }
    }

    /// The device's root.
    fn root(&self) -> &Path {
        self.root.path()
    }

    /// Sends Add Asset Folder and returns the folder's key.
    fn add_folder(&mut self, path: &Path, name: &str) -> FolderKey {
        self.app.world_mut().write_message(AddFolder {
            path: path.to_path_buf(),
            name: CanonicalName(name.to_owned()),
        });
        self.app.update();
        let world = self.app.world_mut();
        let refused: Vec<FolderRefused> = world
            .resource_mut::<Messages<FolderRefused>>()
            .drain()
            .collect();
        assert!(refused.is_empty(), "the folder was refused: {refused:?}");
        world
            .resource_mut::<Messages<FolderAdded>>()
            .drain()
            .next()
            .expect("the folder is added")
            .key
    }

    /// The one Layer of the Project.
    fn layer(&mut self) -> Entity {
        let world = self.app.world_mut();
        world
            .query::<(Entity, &Layer)>()
            .single(world)
            .expect("exactly one Layer")
            .0
    }

    /// Places a Prop of the Asset at `place` in the folder with `key`, centred on `position`.
    fn place(&mut self, key: &FolderKey, place: &str, position: Vec2) -> ElementId {
        let layer = self.layer();
        self.app
            .world_mut()
            .write_message(Apply::PlaceElement(PlaceElement {
                layer,
                placement: Placement::Prop {
                    position,
                    asset: AssetAddress {
                        folder: key.clone(),
                        place: place.to_owned(),
                    },
                },
            }));
        self.app.update();
        let failed: Vec<CommandFailed> = self
            .app
            .world_mut()
            .resource_mut::<Messages<CommandFailed>>()
            .drain()
            .collect();
        assert!(failed.is_empty(), "the placement failed: {failed:?}");
        self.elements()
            .last()
            .map(|placed| placed.id)
            .expect("the placed Prop is the last child of the Layer")
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

    /// Places a Wall through `points` and bends each segment that has a control.
    fn wall(
        &mut self,
        points: &[Vec2],
        controls: &[Option<Vec2>],
        thickness: f32,
        colour: Colour,
    ) -> ElementId {
        let layer = self.layer();
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Wall {
                points: points.to_vec(),
                thickness,
                colour,
            },
        }));
        let id = self
            .elements()
            .last()
            .map(|placed| placed.id)
            .expect("the placed Wall is the last child of the Layer");
        for (segment, control) in controls.iter().enumerate() {
            if control.is_some() {
                self.apply(Apply::EditElement(EditElement {
                    element: id,
                    change: ElementChange::Control {
                        segment,
                        position: *control,
                    },
                    gesture: Gesture::Single,
                }));
            }
        }
        id
    }

    /// Every Wall on the Layer in stacking order, with its Element and derived shape.
    fn walls(&mut self) -> Vec<(ElementId, Element, Wall, Option<WallShape>)> {
        let layer = self.layer();
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
                    world.get::<Element>(entity)?.clone(),
                    world.get::<Wall>(entity)?.clone(),
                    world.get::<WallShape>(entity).cloned(),
                ))
            })
            .collect()
    }

    /// Makes the device an editor that does not know the Wall kind, as an older one would be.
    fn forget_walls(&mut self) {
        let world = self.app.world_mut();
        world.resource_mut::<ElementKindRegistry>().remove(&WALL);
        assert!(
            world
                .resource_mut::<SerialisationRegistry>()
                .remove(<Wall as Serialisable>::NAME),
            "the Wall was known"
        );
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

    /// Sends Save, to `path` or to the Project's file, and returns what came back.
    ///
    /// # Errors
    ///
    /// The refusal, when the save was refused.
    fn save(&mut self, path: Option<&Path>) -> Result<ProjectSaved, ProjectRefused> {
        self.app.world_mut().write_message(SaveProject {
            path: path.map(Path::to_path_buf),
        });
        self.app.update();
        let world = self.app.world_mut();
        if let Some(refused) = world
            .resource_mut::<Messages<ProjectRefused>>()
            .drain()
            .next()
        {
            return Err(refused);
        }
        Ok(world
            .resource_mut::<Messages<ProjectSaved>>()
            .drain()
            .next()
            .expect("a Save is either saved or refused"))
    }

    /// Saves to `path`, failing the test on a refusal, and returns the file written.
    fn save_as(&mut self, path: &Path) -> PathBuf {
        self.save(Some(path))
            .map_err(|refused| refused.reason)
            .expect("the save was accepted")
            .path
    }

    /// Sends Open and returns what came back.
    ///
    /// # Errors
    ///
    /// The refusal, when the file was refused.
    fn open(&mut self, path: &Path) -> Result<ProjectOpened, ProjectRefused> {
        self.app.world_mut().write_message(OpenProject {
            path: path.to_path_buf(),
        });
        self.app.update();
        let world = self.app.world_mut();
        if let Some(refused) = world
            .resource_mut::<Messages<ProjectRefused>>()
            .drain()
            .next()
        {
            return Err(refused);
        }
        Ok(world
            .resource_mut::<Messages<ProjectOpened>>()
            .drain()
            .next()
            .expect("an Open is either opened or refused"))
    }

    /// Opens `path`, failing the test on a refusal.
    fn opens(&mut self, path: &Path) -> ProjectOpened {
        self.open(path)
            .map_err(|refused| refused.reason)
            .expect("the file was opened")
    }

    /// The history.
    fn history(&self) -> &History {
        self.app.world().resource::<History>()
    }

    /// The Project's file and the history's position at the last save or open.
    fn mark(&self) -> SavedMark {
        self.app.world().resource::<SavedMark>().clone()
    }

    /// Whether the Project has unsaved changes: a step was recorded or undone since the mark.
    fn has_unsaved_changes(&self) -> bool {
        self.history().position() != self.mark().position
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
                prop: world.get::<Prop>(entity).cloned(),
                unknown: world.get::<UnknownComponents>(entity).cloned(),
            })
            .collect()
    }

    /// The identities on the Layer in stacking order.
    fn order(&mut self) -> Vec<ElementId> {
        self.elements()
            .into_iter()
            .map(|placed| placed.id)
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

    /// The Project, its Grid, and its Bounds.
    fn project(&mut self) -> (Project, Grid, Bounds) {
        let world = self.app.world_mut();
        let (project, grid, bounds) = world
            .query::<(&Project, &Grid, &Bounds)>()
            .single(world)
            .expect("exactly one Project");
        (project.clone(), *grid, *bounds)
    }

    /// How many Projects, Levels, and Layers the World holds.
    fn counts(&mut self) -> (usize, usize, usize) {
        let world = self.app.world_mut();
        (
            world.query::<&Project>().iter(world).count(),
            world.query::<&Level>().iter(world).count(),
            world.query::<&Layer>().iter(world).count(),
        )
    }
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

/// The device that saves: the fixture folder added under `Fixtures`, a table placed at
/// `(3.5, -2.25)`, a barrel at `(1, 1)`, and a second table at the origin, saved to `map.dungeon`.
struct Saved {
    /// The device.
    device: Device,
    /// The fixture folder's key on the device.
    key: FolderKey,
    /// The identities placed, in stacking order.
    ids: Vec<ElementId>,
    /// The file saved.
    file: PathBuf,
}

impl Saved {
    /// Places the three Props and saves.
    fn new() -> Self {
        let mut device = Device::new();
        let folder = fixtures(device.root());
        let key = device.add_folder(&folder, FIXTURES);
        let ids = vec![
            device.place(&key, TABLE, Vec2::new(3.5, -2.25)),
            device.place(&key, BARREL, Vec2::ONE),
            device.place(&key, TABLE, Vec2::ZERO),
        ];
        let file = device.save_as(&device.root().join("map.dungeon"));
        Self {
            device,
            key,
            ids,
            file,
        }
    }

    /// The version the Project recorded for the fixture folder.
    fn recorded_version(&mut self) -> String {
        self.device.references().folders[0].version.clone()
    }
}

/// Save writes the whole Project into a single file with the `.dungeon` extension, which is
/// added when the chosen name lacks it, as readable as any file made in the same place.
#[test]
fn saved_as_one_file() {
    let mut device = Device::new();
    let out = device.root().join("out");
    fs::create_dir_all(&out).expect("output folder");

    let written = device.save_as(&out.join("map"));

    assert_eq!(written, out.join("map.dungeon"));
    assert_eq!(device.mark().file, Some(written.clone()));
    let files: Vec<PathBuf> = fs::read_dir(&out)
        .expect("output folder")
        .map(|entry| entry.expect("entry").path())
        .collect();
    assert_eq!(files, vec![written.clone()]);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let plain = device.root().join("plain");
        fs::write(&plain, b"").expect("a plain file");
        assert_eq!(
            fs::metadata(&written)
                .expect("the file")
                .permissions()
                .mode(),
            fs::metadata(&plain)
                .expect("the plain file")
                .permissions()
                .mode(),
            "the saved file is as readable as any file"
        );
    }
    assert_eq!(
        device.save_as(&out.join("other.dungeon")),
        out.join("other.dungeon")
    );
}

/// Once a Project has been saved to or opened from a file, Save writes to that file without
/// asking.
#[test]
fn save_remembers_its_file() {
    let mut saved = Saved::new();
    let before = fs::read(&saved.file).expect("the file");
    saved.device.place(&saved.key, BARREL, Vec2::new(2.0, 2.0));

    let again = saved
        .device
        .save(None)
        .expect("saved to the remembered file");

    assert_eq!(again.path, saved.file);
    let after = json(&saved.file);
    assert_ne!(fs::read(&saved.file).expect("the file"), before);
    assert_eq!(
        after["levels"][0]["layers"][0]["elements"]
            .as_array()
            .map(Vec::len),
        Some(4)
    );

    let mut other = Device::new();
    other.opens(&saved.file);
    let key = other.add_folder(&fixtures(other.root()), FIXTURES);
    other.place(&key, TABLE, Vec2::ZERO);
    assert_eq!(other.elements().len(), 5);
}

/// Save on a Project that has no file behaves as Save As: without a path it is refused, so the
/// Editor asks where.
#[test]
fn save_without_a_file_asks() {
    let mut device = Device::new();
    let folder = fixtures(device.root());
    let key = device.add_folder(&folder, FIXTURES);
    device.place(&key, TABLE, Vec2::ZERO);

    let refused = device
        .save(None)
        .expect_err("a Project without a file is refused");

    assert_eq!(refused.request, ProjectRequest::Save { path: None });
    assert!(!refused.reason.is_empty());
    assert_eq!(device.mark().file, None);
    assert!(!fs::read_dir(device.root()).expect("root").any(|entry| {
        entry
            .expect("entry")
            .path()
            .extension()
            .is_some_and(|e| e == "dungeon")
    }));
}

/// After Save As, the Project's file is the new one, and the previous file is left as it was.
#[test]
fn save_as_moves_the_project() {
    let mut saved = Saved::new();
    let first = saved.file.clone();
    let first_bytes = fs::read(&first).expect("the first file");

    let second = saved
        .device
        .save_as(&saved.device.root().join("copy.dungeon"));
    saved.device.place(&saved.key, BARREL, Vec2::new(5.0, 5.0));
    let again = saved.device.save(None).expect("saved to the new file");

    assert_eq!(again.path, second);
    assert_eq!(saved.device.mark().file, Some(second.clone()));
    assert_eq!(fs::read(&first).expect("the first file"), first_bytes);
    assert_eq!(
        json(&second)["levels"][0]["layers"][0]["elements"]
            .as_array()
            .map(Vec::len),
        Some(4)
    );
}

/// The file holds the Grid, the Bounds, every Level with its Layers, every Element with its
/// `ElementId`, kind, position, size, and the properties of its kind, and the Asset Reference
/// table with the Canonical Name and version of every Asset Folder recorded in it.
#[test]
fn everything_the_project_is() {
    let mut saved = Saved::new();
    let version = saved.recorded_version();
    let references = saved.device.references();
    let elements = saved.device.elements();

    let file = json(&saved.file);

    assert_eq!(file["format"], json!(1));
    assert_eq!(
        file["project"]["grid"]["data"]["pixels_per_cell"],
        json!(256)
    );
    assert_eq!(file["project"]["bounds"]["data"]["size"], json!([30, 30]));
    assert_eq!(file["project"]["bounds"]["data"]["origin"], json!([0, 0]));
    let table = &file["project"]["asset_references"]["data"];
    assert_eq!(
        table["folders"],
        json!([{ "name": FIXTURES, "version": version }])
    );
    assert_eq!(table["assets"].as_array().map(Vec::len), Some(2));
    assert_eq!(table["assets"][0]["folder"], json!(FIXTURES));
    assert_eq!(table["assets"][0]["places"], json!([TABLE]));
    assert_eq!(table["assets"][0]["name"], json!("table"));
    assert_eq!(table["assets"][0]["pixel_size"], json!([512, 256]));
    assert_eq!(table["assets"][1]["places"], json!([BARREL]));
    assert_eq!(
        table["assets"][1]["fingerprint"],
        json!(references.assets[1].fingerprint.as_str())
    );
    let level = &file["levels"][0];
    assert_eq!(
        level["components"]["level"]["data"]["name"],
        json!("Level 1")
    );
    let layer = &level["layers"][0];
    assert_eq!(
        layer["components"]["layer"]["data"]["name"],
        json!("Layer 1")
    );
    let ids: Vec<Value> = saved
        .ids
        .iter()
        .map(|id| json!(id.as_raw().to_string()))
        .collect();
    assert_eq!(layer["elements"], Value::Array(ids.clone()));
    for (id, placed) in ids.iter().zip(&elements) {
        let element = &file["elements"][id.as_str().expect("an id")];
        assert_eq!(element["element"]["version"], json!(1));
        assert_eq!(element["element"]["data"]["kind"], json!("prop"));
        assert_eq!(
            element["element"]["data"]["position"],
            json!([placed.element.position.x, placed.element.position.y])
        );
        assert_eq!(
            element["element"]["data"]["size"],
            json!([placed.element.size.x, placed.element.size.y])
        );
        assert_eq!(
            element["prop"]["data"]["asset"],
            json!(placed.prop.as_ref().expect("a Prop").asset.0)
        );
    }
}

/// The file holds no path, folder key, or other value specific to this device.
#[test]
fn nothing_of_this_device() {
    let saved = Saved::new();

    let text = fs::read_to_string(&saved.file).expect("the file");

    assert!(
        !text.contains(saved.key.as_str()),
        "the folder key is in the file"
    );
    let root = saved.device.root().to_string_lossy().into_owned();
    assert!(!text.contains(&root), "the device's paths are in the file");
    assert!(
        !text.contains("library"),
        "the folder's path is in the file"
    );
    assert!(
        !text.contains("lib://"),
        "a device-local asset path is in the file"
    );
}

/// The file holds nothing of the history, and an opened Project has an empty history.
#[test]
fn history_is_not_saved() {
    let mut saved = Saved::new();
    saved.device.undo();
    assert!(saved.device.history().can_undo());
    assert!(saved.device.history().can_redo());

    let text = fs::read_to_string(&saved.file).expect("the file");
    assert!(!text.contains("history"));

    saved.device.opens(&saved.file);
    assert!(!saved.device.history().can_undo());
    assert!(!saved.device.history().can_redo());

    let mut other = Device::new();
    other.add_folder(&fixtures(other.root()), FIXTURES);
    other.opens(&saved.file);
    assert!(!other.history().can_undo());
    assert!(!other.history().can_redo());
}

/// Save writes every Asset Reference and every Element, including Elements whose Asset is
/// Missing and Elements of an unknown kind.
#[test]
fn saving_keeps_every_reference() {
    let saved = Saved::new();
    let original = fs::read(&saved.file).expect("the file");
    let mut other = Device::new();
    other.opens(&saved.file);
    assert!(
        other
            .resolutions()
            .iter()
            .all(|row| matches!(row, Resolution::Missing(_)))
    );

    let copy = other.save_as(&other.root().join("map.dungeon"));

    assert_eq!(fs::read(&copy).expect("the copy"), original);
}

/// Saving a Project, opening the file, and saving it again produces a byte-identical file.
#[test]
fn saving_is_a_fixed_point() {
    let mut saved = Saved::new();
    let first = fs::read(&saved.file).expect("the file");
    let elsewhere = saved.device.root().join("elsewhere");
    fs::create_dir_all(&elsewhere).expect("folder");

    saved.device.opens(&saved.file);
    let second = saved.device.save_as(&elsewhere.join("map.dungeon"));
    assert_eq!(fs::read(&second).expect("the second file"), first);

    saved.device.opens(&second);
    let third = saved.device.save(None).expect("saved").path;
    assert_eq!(third, second);
    assert_eq!(fs::read(&third).expect("the third file"), first);
}

/// When a save cannot complete, the file that was at the chosen path before is unchanged, the
/// Author is told the reason, and the Project keeps its unsaved changes and its remembered file.
#[test]
fn a_failed_save_leaves_the_old_file() {
    let mut saved = Saved::new();
    let before = fs::read(&saved.file).expect("the file");
    saved.device.place(&saved.key, BARREL, Vec2::new(2.0, 2.0));
    assert!(saved.device.has_unsaved_changes());
    let root = saved.device.root().to_path_buf();

    let vanished = root.join("nowhere").join("map.dungeon");
    let refused = saved
        .device
        .save(Some(&vanished))
        .expect_err("a vanished location refuses the save");
    assert_eq!(
        refused.request,
        ProjectRequest::Save {
            path: Some(vanished.clone())
        }
    );
    assert!(!refused.reason.is_empty());
    assert!(!vanished.exists());
    assert_eq!(saved.device.mark().file, Some(saved.file.clone()));
    assert_eq!(fs::read(&saved.file).expect("the file"), before);
    assert!(saved.device.has_unsaved_changes());
    assert_eq!(saved.device.elements().len(), 4);

    // A folder the process may not write into exists only where modes are honoured.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&root, fs::Permissions::from_mode(0o555)).expect("read-only root");
        if fs::write(root.join("probe"), b"").is_ok() {
            fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).expect("writable root");
            eprintln!(
                "skipped: this process may write into a read-only folder, so the check would \
                 prove nothing"
            );
            return;
        }
        let refused = saved
            .device
            .save(None)
            .expect_err("a read-only location refuses the save");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).expect("writable root");

        assert_eq!(refused.request, ProjectRequest::Save { path: None });
        assert!(!refused.reason.is_empty());
        assert_eq!(fs::read(&saved.file).expect("the file"), before);
        assert_eq!(saved.device.mark().file, Some(saved.file.clone()));
        assert!(saved.device.has_unsaved_changes());
        assert_eq!(saved.device.elements().len(), 4);
    }
}

/// A Project file whose path holds spaces, quotes, non-ASCII letters, or symbols is saved and
/// opened like any other.
#[test]
fn any_path_works_for_projects() {
    let mut saved = Saved::new();
    let odd = saved
        .device
        .root()
        .join("my \"maps\" & 'more' — café 地図 #1 (v2) [final]");
    fs::create_dir_all(&odd).expect("odd folder");

    let file = saved.device.save_as(&odd.join("Straße's map №7"));
    assert_eq!(file, odd.join("Straße's map №7.dungeon"));

    let mut other = Device::new();
    other.add_folder(&fixtures(other.root()), FIXTURES);
    other.opens(&file);
    assert_eq!(other.order(), saved.ids);
    assert_eq!(other.project().0.name, "Straße's map №7");
}

/// An Asset whose place holds spaces, quotes, non-ASCII letters, or symbols is recorded, saved,
/// and resolved on another device like any other, and the asset path it is drawn from names the
/// file and nothing else.
#[test]
fn any_path_works_for_assets() {
    let mut device = Device::new();
    let folder = library(
        device.root(),
        "library",
        &[(TABLE, TABLE_PIXELS), (CRATE, BARREL_PIXELS)],
    );
    let key = device.add_folder(&folder, FIXTURES);
    device.place(&key, CRATE, Vec2::ONE);
    let file = device.save_as(&device.root().join("map.dungeon"));
    assert_eq!(device.references().assets[0].places, vec![CRATE]);

    let mut other = Device::new();
    let copy = library(
        other.root(),
        "library",
        &[(TABLE, TABLE_PIXELS), (CRATE, BARREL_PIXELS)],
    );
    let other_key = other.add_folder(&copy, FIXTURES);
    let opened = other.opens(&file);

    assert!(opened.report.missing_assets.is_empty());
    assert_eq!(
        other.resolutions(),
        vec![Resolution::Resolved {
            folder: other_key.clone(),
            place: CRATE.to_owned()
        }]
    );
    assert_eq!(other.elements()[0].element.size, Vec2::new(0.5, 0.5));
    let path = asset_path(&other_key, CRATE);
    assert_eq!(path.path(), Path::new(other_key.as_str()).join(CRATE));
    assert_eq!(path.label(), None, "a `#` in the name is not a label");
}

/// Save adds nothing to the history and changes no Element, the selection, or the view.
#[test]
fn saving_is_not_a_step() {
    let mut device = Device::new();
    let folder = fixtures(device.root());
    let key = device.add_folder(&folder, FIXTURES);
    device.place(&key, TABLE, Vec2::ZERO);
    device.place(&key, BARREL, Vec2::ONE);
    device.undo();
    {
        let mut viewport = device.app.world_mut().resource_mut::<Viewport>();
        viewport.centre = Vec2::new(7.0, -3.0);
        viewport.zoom *= 2.0;
    }
    let elements = device.elements();
    let depth = device.history().undo_depth();
    let view = *device.app.world().resource::<Viewport>();

    device.save_as(&device.root().join("map.dungeon"));
    device.save(None).expect("saved again");

    assert_eq!(device.history().undo_depth(), depth);
    assert!(device.history().can_redo());
    assert_eq!(device.elements(), elements);
    assert_eq!(*device.app.world().resource::<Viewport>(), view);
}

/// A Project has unsaved changes exactly when the history's position differs from its position
/// at the last save or open; undoing back to that position counts as having none.
#[test]
fn unsaved_means_a_step_since_the_save() {
    let mut saved = Saved::new();
    assert!(!saved.device.has_unsaved_changes());

    saved.device.place(&saved.key, BARREL, Vec2::new(2.0, 2.0));
    assert!(saved.device.has_unsaved_changes());
    saved.device.undo();
    assert!(!saved.device.has_unsaved_changes());
    saved.device.redo();
    assert!(saved.device.has_unsaved_changes());
    saved.device.undo();
    saved.device.undo();
    assert!(
        saved.device.has_unsaved_changes(),
        "undoing past the save is a change too"
    );

    saved.device.opens(&saved.file);
    assert!(!saved.device.has_unsaved_changes());
    saved.device.place(&saved.key, BARREL, Vec2::new(2.0, 2.0));
    assert!(saved.device.has_unsaved_changes());
    saved.device.save(None).expect("saved");
    assert!(!saved.device.has_unsaved_changes());
}

/// Opening a saved file yields the same Grid, Bounds, Levels, Layers, and Elements, each Element
/// with the `ElementId`, kind, position, size, and properties it was saved with, in the same
/// stacking order, and the same Asset Reference table.
#[test]
fn opened_as_saved() {
    let mut saved = Saved::new();
    let elements = saved.device.elements();
    let references = saved.device.references();
    let (_, grid, bounds) = saved.device.project();
    let mut other = Device::new();
    let elsewhere = library(
        &other.root().join("somewhere").join("else"),
        "Fixtures copy",
        &[(TABLE, TABLE_PIXELS), (BARREL, BARREL_PIXELS)],
    );
    other.add_folder(&elsewhere, FIXTURES);

    other.opens(&saved.file);

    assert_eq!(other.elements(), elements);
    assert_eq!(other.references(), references);
    let (project, other_grid, other_bounds) = other.project();
    assert_eq!(project.name, "map");
    assert_eq!(other_grid, grid);
    assert_eq!(other_bounds, bounds);
    assert_eq!(
        other_bounds,
        Bounds {
            origin: IVec2::ZERO,
            size: UVec2::splat(30)
        }
    );
    assert_eq!(other.counts(), (1, 1, 1));
}

/// The opened Project replaces the current one, and the opened file becomes the Project's file.
#[test]
fn open_replaces_the_project() {
    let saved = Saved::new();
    let mut other = Device::new();
    let key = other.add_folder(&fixtures(other.root()), FIXTURES);
    other.place(&key, BARREL, Vec2::new(9.0, 9.0));
    other.save_as(&other.root().join("mine.dungeon"));

    let opened = other.opens(&saved.file);

    assert_eq!(opened.path, saved.file);
    assert_eq!(other.order(), saved.ids);
    assert_eq!(other.counts(), (1, 1, 1));
    assert_eq!(other.mark().file, Some(saved.file.clone()));
    assert_eq!(other.project().0.name, "map");
    let again = other.save(None).expect("saved");
    assert_eq!(again.path, saved.file);
}

/// A file that is not a Project, cannot be read, or holds malformed data is refused with the
/// reason, and the current Project and its history are untouched.
#[test]
fn a_bad_file_is_refused() {
    let mut saved = Saved::new();
    saved.device.undo();
    let elements = saved.device.elements();
    let depth = saved.device.history().undo_depth();
    let root = saved.device.root().to_path_buf();

    let prose = root.join("prose.dungeon");
    fs::write(&prose, "once upon a time").expect("written");
    let refused = saved
        .device
        .open(&prose)
        .expect_err("prose is not a Project");
    assert_eq!(refused.request, ProjectRequest::Open { path: prose });
    assert!(!refused.reason.is_empty());

    let missing = root.join("missing.dungeon");
    let refused = saved
        .device
        .open(&missing)
        .expect_err("a missing file cannot be read");
    assert_eq!(refused.request, ProjectRequest::Open { path: missing });

    let mut malformed = json(&saved.file);
    malformed["levels"][0]["layers"][0]["elements"]
        .as_array_mut()
        .expect("the order")
        .push(json!("42"));
    let listed = root.join("listed.dungeon");
    write_json(&listed, &malformed);
    let refused = saved
        .device
        .open(&listed)
        .expect_err("an Element listed but not held");
    assert!(refused.reason.contains("42"), "{}", refused.reason);

    let mut malformed = json(&saved.file);
    malformed["project"]["grid"] = json!({ "version": 1, "data": { "pixels_per_cell": "many" } });
    let grid = root.join("grid.dungeon");
    write_json(&grid, &malformed);
    let refused = saved
        .device
        .open(&grid)
        .expect_err("a Grid that is not one");
    assert!(refused.reason.contains("grid"), "{}", refused.reason);

    let mut malformed = json(&saved.file);
    let id = saved.ids[0].as_raw().to_string();
    let element = malformed["elements"][&id]
        .as_object_mut()
        .expect("the Element's envelopes");
    element.remove("prop");
    element["element"]["data"]["kind"] = json!("wall");
    element.insert(
        "wall".to_owned(),
        json!({
            "version": 1,
            "data": {
                "points": [[1.0, 1.0]],
                "segments": [],
                "thickness": 0.125,
                "colour": { "red": 64, "green": 64, "blue": 64 }
            }
        }),
    );
    let lone_point = root.join("lone-point.dungeon");
    write_json(&lone_point, &malformed);
    let refused = saved
        .device
        .open(&lone_point)
        .expect_err("a Wall of one point");
    assert!(
        refused.reason.contains("wall") && refused.reason.contains("two or more points"),
        "{}",
        refused.reason
    );

    malformed["elements"][&id]["wall"]["data"] = json!({
        "points": [[1.0, 1.0], [3.0, 1.0], [3.0, 3.0]],
        "segments": [{ "control": null }],
        "thickness": 0.125,
        "colour": { "red": 64, "green": 64, "blue": 64 }
    });
    let short = root.join("short.dungeon");
    write_json(&short, &malformed);
    let refused = saved
        .device
        .open(&short)
        .expect_err("a Wall of three points and one segment");
    assert!(
        refused.reason.contains("wall") && refused.reason.contains("needs 2 segments"),
        "{}",
        refused.reason
    );

    assert_eq!(saved.device.elements(), elements);
    assert_eq!(saved.device.history().undo_depth(), depth);
    assert!(saved.device.history().can_redo());
    assert_eq!(saved.device.counts(), (1, 1, 1));
    assert_eq!(saved.device.mark().file, Some(saved.file.clone()));
}

/// A file that holds a known component under another entity than its own, a Level's component
/// on the Project or a Prop's on a Layer, is refused naming the component, and the current
/// Project is untouched.
#[test]
fn a_misplaced_envelope_is_refused() {
    let mut saved = Saved::new();
    let elements = saved.device.elements();
    let root = saved.device.root().to_path_buf();
    let file = json(&saved.file);

    let mut misplaced = file.clone();
    misplaced["project"]["level"] = file["levels"][0]["components"]["level"].clone();
    let on_project = root.join("level-on-project.dungeon");
    write_json(&on_project, &misplaced);
    let refused = saved
        .device
        .open(&on_project)
        .expect_err("a Level's component on the Project");
    assert_eq!(refused.request, ProjectRequest::Open { path: on_project });
    assert!(refused.reason.contains("level"), "{}", refused.reason);

    let mut misplaced = file.clone();
    let id = saved.ids[1].as_raw().to_string();
    misplaced["levels"][0]["layers"][0]["components"]["prop"] =
        file["elements"][&id]["prop"].clone();
    let on_layer = root.join("prop-on-layer.dungeon");
    write_json(&on_layer, &misplaced);
    let refused = saved
        .device
        .open(&on_layer)
        .expect_err("a Prop's component on a Layer");
    assert!(refused.reason.contains("prop"), "{}", refused.reason);

    assert_eq!(saved.device.elements(), elements);
    assert_eq!(saved.device.counts(), (1, 1, 1));
    assert_eq!(saved.device.mark().file, Some(saved.file.clone()));
}

/// A file whose format version, or any component version in it, is newer than this editor knows
/// is refused, naming the version, and the current Project is untouched.
#[test]
fn a_newer_file_is_refused() {
    let mut saved = Saved::new();
    let elements = saved.device.elements();
    let root = saved.device.root().to_path_buf();

    let mut newer = json(&saved.file);
    newer["format"] = json!(2);
    let format = root.join("format.dungeon");
    write_json(&format, &newer);
    let refused = saved.device.open(&format).expect_err("a newer format");
    assert!(refused.reason.contains('2'), "{}", refused.reason);

    let mut newer = json(&saved.file);
    let id = saved.ids[1].as_raw().to_string();
    newer["elements"][&id]["prop"]["version"] = json!(3);
    let component = root.join("component.dungeon");
    write_json(&component, &newer);
    let refused = saved
        .device
        .open(&component)
        .expect_err("a newer component");
    assert!(refused.reason.contains("prop"), "{}", refused.reason);
    assert!(refused.reason.contains('3'), "{}", refused.reason);

    assert_eq!(saved.device.elements(), elements);
    assert_eq!(saved.device.counts(), (1, 1, 1));
}

/// Data in an Element under a component name this editor does not know is kept with the Element
/// and written back unchanged on save.
#[test]
fn unknown_components_round_trip() {
    let mut saved = Saved::new();
    let id = saved.ids[1].as_raw().to_string();
    let lighting =
        json!({ "version": 3, "data": { "zeta": 1.5, "alpha": [1, 2, 3], "name": "glow" } });
    let mut file = json(&saved.file);
    file["elements"][&id]["lighting"] = lighting.clone();
    let with_lighting = saved.device.root().join("lighting.dungeon");
    write_json(&with_lighting, &file);

    saved.device.opens(&with_lighting);
    let elements = saved.device.elements();
    assert_eq!(elements[1].id, saved.ids[1]);
    let kept = elements[1]
        .unknown
        .as_ref()
        .expect("the unknown envelope is kept");
    assert_eq!(kept.envelopes.keys().collect::<Vec<_>>(), vec!["lighting"]);
    assert!(elements[0].unknown.is_none());
    saved.device.place(&saved.key, BARREL, Vec2::new(4.0, 4.0));

    let again = saved.device.save(None).expect("saved").path;
    let written = json(&again);
    assert_eq!(written["elements"][&id]["lighting"], lighting);
    assert_eq!(written["elements"][&id]["prop"]["version"], json!(1));

    let bytes = fs::read(&again).expect("the file");
    saved.device.opens(&again);
    saved.device.save(None).expect("saved again");
    assert_eq!(fs::read(&again).expect("the file"), bytes);
}

/// An Element whose kind is not registered on this editor stays on its Layer at its position in
/// the stacking order, is counted in the report shown after opening, and is written back
/// unchanged on save.
#[test]
fn unknown_kinds_are_kept() {
    let mut saved = Saved::new();
    let door = ElementId::new();
    let door_id = door.as_raw().to_string();
    let envelopes = json!({
        "element": { "version": 1, "data": { "kind": "plugin/door", "position": [2.0, 3.0], "size": [1.0, 2.5] } },
        "door": { "version": 1, "data": { "open": true, "hinge": "left" } }
    });
    let mut file = json(&saved.file);
    file["elements"][&door_id] = envelopes.clone();
    file["levels"][0]["layers"][0]["elements"]
        .as_array_mut()
        .expect("the order")
        .insert(1, json!(door_id));
    let with_door = saved.device.root().join("door.dungeon");
    write_json(&with_door, &file);

    let opened = saved.device.opens(&with_door);

    assert_eq!(
        saved.device.order(),
        vec![saved.ids[0], door, saved.ids[1], saved.ids[2]]
    );
    let placed = saved.device.elements().remove(1);
    assert_eq!(placed.element.kind, ElementKindName::new("plugin/door"));
    assert_eq!(placed.element.position, Vec2::new(2.0, 3.0));
    assert_eq!(placed.element.size, Vec2::new(1.0, 2.5));
    assert_eq!(placed.prop, None);
    assert_eq!(
        placed
            .unknown
            .expect("the kind's component is kept")
            .envelopes
            .keys()
            .collect::<Vec<_>>(),
        vec!["door"]
    );
    assert_eq!(
        opened.report.unknown_kinds,
        vec![UnknownKind {
            kind: ElementKindName::new("plugin/door"),
            elements: 1
        }]
    );
    assert!(opened.report.missing_assets.is_empty());

    let again = saved.device.save(None).expect("saved").path;
    let written = json(&again);
    assert_eq!(written["elements"][&door_id], envelopes);
    assert_eq!(
        written["levels"][0]["layers"][0]["elements"][1],
        json!(door_id)
    );
}

/// An Asset Reference resolves to the file at its recorded place inside the Asset Folder on this
/// device whose Canonical Name equals the recorded one, compared ignoring letter case and Unicode
/// normalisation, wherever that folder sits on this device.
#[test]
fn resolved_by_canonical_name_and_place() {
    let saved = Saved::new();
    let mut other = Device::new();
    let elsewhere = library(
        &other.root().join("Volumes").join("Art"),
        "dungeon-things",
        &[(TABLE, TABLE_PIXELS), (BARREL, BARREL_PIXELS)],
    );
    let key = other.add_folder(&elsewhere, "FIXTURES");

    let opened = other.opens(&saved.file);

    assert_eq!(
        other.resolutions(),
        vec![
            Resolution::Resolved {
                folder: key.clone(),
                place: TABLE.to_owned()
            },
            Resolution::Resolved {
                folder: key,
                place: BARREL.to_owned()
            },
        ]
    );
    assert!(opened.report.missing_assets.is_empty());
    assert_eq!(
        other.references().folders[0].name,
        CanonicalName(FIXTURES.to_owned())
    );
}

/// When no file sits at the exact recorded place, a file whose path differs from it only in
/// letter case or Unicode normalisation resolves; when two or more such files exist, the Asset
/// Reference is Missing.
#[test]
fn spelling_differences_resolve() {
    // The saving device spells the café with a precomposed `é` and the fire with the `ﬁ`
    // ligature, which case-folds to `fi` while every file system keeps it apart from `f` `i`.
    let mut device = Device::new();
    let folder = library(
        device.root(),
        "library",
        &[
            (TABLE, TABLE_PIXELS),
            (BARREL, BARREL_PIXELS),
            ("café.png", BARREL_PIXELS),
            ("\u{fb01}re.png", BARREL_PIXELS),
        ],
    );
    let key = device.add_folder(&folder, FIXTURES);
    device.place(&key, TABLE, Vec2::ZERO);
    device.place(&key, BARREL, Vec2::ONE);
    device.place(&key, "café.png", Vec2::X);
    device.place(&key, "\u{fb01}re.png", Vec2::Y);
    let file = device.save_as(&device.root().join("map.dungeon"));

    let mut other = Device::new();
    let respelled = library(
        other.root(),
        "library",
        &[
            ("Table.PNG", TABLE_PIXELS),
            ("Props/Barrel.png", BARREL_PIXELS),
            ("cafe\u{301}.png", BARREL_PIXELS),
            ("fire.png", BARREL_PIXELS),
            ("\u{fb01}re.PNG", BARREL_PIXELS),
        ],
    );
    // A case-insensitive file system keeps one file for the two spellings of the fire, so the
    // ambiguity is exercised where the file system keeps both, and the one kept resolves where
    // it does not.
    let keeps_both = fs::read_dir(&respelled).expect("the folder").count() == 5;
    let other_key = other.add_folder(&respelled, FIXTURES);
    let opened = other.opens(&file);

    let rows = other.resolutions();
    let resolved = |place: &str| Resolution::Resolved {
        folder: other_key.clone(),
        place: place.to_owned(),
    };
    assert_eq!(rows[0], resolved("Table.PNG"));
    assert_eq!(rows[1], resolved("Props/Barrel.png"));
    assert_eq!(rows[2], resolved("cafe\u{301}.png"));
    if keeps_both {
        let Resolution::Missing(MissingReason::Ambiguous { candidates, .. }) = &rows[3] else {
            panic!(
                "two files differing only in letter case are never chosen between: {:?}",
                rows[3]
            );
        };
        assert_eq!(candidates.len(), 2);
        assert_eq!(opened.report.missing_assets.len(), 1);
        assert_eq!(opened.report.missing_assets[0].name, "\u{fb01}re");
        assert_eq!(opened.report.missing_assets[0].elements, 1);
    } else {
        assert_eq!(rows[3], resolved("fire.png"));
        assert!(opened.report.missing_assets.is_empty());
    }
}

/// A file at the recorded place resolves even when its byte size or content fingerprint differs
/// from the recorded one.
#[test]
fn same_place_changed_content() {
    let saved = Saved::new();
    let mut other = Device::new();
    let retouched = library(
        other.root(),
        "library",
        &[(TABLE, UVec2::new(64, 64)), (BARREL, UVec2::new(300, 200))],
    );
    let key = other.add_folder(&retouched, FIXTURES);

    let opened = other.opens(&saved.file);

    assert_eq!(
        other.resolutions(),
        vec![
            Resolution::Resolved {
                folder: key.clone(),
                place: TABLE.to_owned()
            },
            Resolution::Resolved {
                folder: key,
                place: BARREL.to_owned()
            },
        ]
    );
    assert!(opened.report.missing_assets.is_empty());
    let sizes: Vec<Vec2> = other
        .elements()
        .iter()
        .map(|placed| placed.element.size)
        .collect();
    assert_eq!(
        sizes,
        vec![
            Vec2::new(2.0, 1.0),
            Vec2::new(0.5, 0.5),
            Vec2::new(2.0, 1.0)
        ]
    );
}

/// An Asset Folder whose version differs from the version recorded in the Project resolves by
/// place like any other, and nothing is said about the difference for Assets that resolve.
#[test]
fn versions_do_not_block() {
    let mut saved = Saved::new();
    let recorded = saved.recorded_version();
    assert_ne!(recorded, "2024-01-01");
    let root = TempDir::new().expect("the other device's library");
    let older = fixtures(root.path());
    let mut other = Device::remembering(&older, FIXTURES, "2024-01-01");

    let opened = other.opens(&saved.file);

    assert!(
        other
            .resolutions()
            .iter()
            .all(|row| matches!(row, Resolution::Resolved { .. })),
        "{:?}",
        other.resolutions()
    );
    assert!(opened.report.missing_assets.is_empty());
    assert!(opened.report.unknown_kinds.is_empty());
}

/// An Asset Reference that does not resolve is a Missing Asset; every Element that uses it stays
/// on its Layer at its position in the stacking order.
#[test]
fn missing_assets_stay() {
    let saved = Saved::new();
    let mut other = Device::new();
    let without_barrel = library(other.root(), "library", &[(TABLE, TABLE_PIXELS)]);
    let key = other.add_folder(&without_barrel, FIXTURES);

    other.opens(&saved.file);

    let rows = other.resolutions();
    assert_eq!(
        rows[0],
        Resolution::Resolved {
            folder: key,
            place: TABLE.to_owned()
        }
    );
    assert!(matches!(
        rows[1],
        Resolution::Missing(MissingReason::AssetAbsent { .. })
    ));
    assert_eq!(other.order(), saved.ids);
    let barrel = other.elements().remove(1);
    assert_eq!(barrel.element.position, Vec2::ONE);
    assert_eq!(barrel.element.size, Vec2::new(0.5, 0.5));
    assert_eq!(barrel.prop.map(|prop| prop.asset.0), Some(1));
}

/// After opening, the Author is shown, for each Missing Asset, its name, the Canonical Name of its
/// Asset Folder, the version recorded in the Project, how many Elements use it, and whether that
/// folder is absent on this device or present without the Asset; when present, the folder's
/// version on this device is named beside the recorded one.
#[test]
fn missing_assets_are_explained() {
    let mut saved = Saved::new();
    let recorded = saved.recorded_version();
    let fixtures_name = CanonicalName(FIXTURES.to_owned());

    let mut bare = Device::new();
    let opened = bare.opens(&saved.file);
    assert_eq!(
        opened.report.missing_assets,
        vec![
            MissingAsset {
                name: "table".to_owned(),
                folder: fixtures_name.clone(),
                recorded_version: recorded.clone(),
                elements: 2,
                reason: MissingReason::FolderAbsent,
            },
            MissingAsset {
                name: "barrel".to_owned(),
                folder: fixtures_name.clone(),
                recorded_version: recorded.clone(),
                elements: 1,
                reason: MissingReason::FolderAbsent,
            },
        ]
    );

    let root = TempDir::new().expect("the other device's library");
    let without_barrel = library(root.path(), "library", &[(TABLE, TABLE_PIXELS)]);
    let mut other = Device::remembering(&without_barrel, FIXTURES, "2024-01-01");
    let opened = other.opens(&saved.file);
    assert_eq!(
        opened.report.missing_assets,
        vec![MissingAsset {
            name: "barrel".to_owned(),
            folder: fixtures_name,
            recorded_version: recorded,
            elements: 1,
            reason: MissingReason::AssetAbsent {
                version: "2024-01-01".to_owned()
            },
        }]
    );
}

/// Adding an Asset Folder, or redoing its addition, re-resolves the Asset References recorded
/// against its Canonical Name; undoing its addition makes them Missing again.
#[test]
fn resolved_when_a_folder_arrives() {
    let saved = Saved::new();
    let mut other = Device::new();
    other.opens(&saved.file);
    let absent = vec![
        Resolution::Missing(MissingReason::FolderAbsent),
        Resolution::Missing(MissingReason::FolderAbsent),
    ];
    assert_eq!(other.resolutions(), absent);

    let folder = fixtures(other.root());
    let key = other.add_folder(&folder, "fixtures");
    let resolved = vec![
        Resolution::Resolved {
            folder: key.clone(),
            place: TABLE.to_owned(),
        },
        Resolution::Resolved {
            folder: key,
            place: BARREL.to_owned(),
        },
    ];
    assert_eq!(other.resolutions(), resolved);

    other.undo();
    assert_eq!(other.resolutions(), absent);

    other.redo();
    assert_eq!(other.resolutions(), resolved);
}

/// Opening and saving never change the Asset Folder versions recorded in the Project.
#[test]
fn recorded_versions_stay() {
    let mut saved = Saved::new();
    let recorded = saved.recorded_version();
    let root = TempDir::new().expect("the other device's library");
    let folder = fixtures(root.path());
    let mut other = Device::remembering(&folder, FIXTURES, "2024-01-01");

    other.opens(&saved.file);
    assert_eq!(other.references().folders[0].version, recorded);
    let copy = other.save_as(&other.root().join("copy.dungeon"));

    assert_eq!(
        json(&copy)["project"]["asset_references"]["data"]["folders"][0]["version"],
        json!(recorded)
    );
    let mut bare = Device::new();
    bare.opens(&copy);
    assert_eq!(bare.references().folders[0].version, recorded);
}

/// A Project opens on a device with no Asset Folders, with every Asset Reference Missing.
#[test]
fn opened_without_folders() {
    let saved = Saved::new();
    let mut bare = Device::new();

    let opened = bare.opens(&saved.file);

    assert_eq!(
        bare.resolutions(),
        vec![
            Resolution::Missing(MissingReason::FolderAbsent),
            Resolution::Missing(MissingReason::FolderAbsent),
        ]
    );
    assert_eq!(bare.order(), saved.ids);
    assert_eq!(opened.report.missing_assets.len(), 2);
}

/// An Element has the same `ElementId` after saving and reopening as before.
#[test]
fn identity_survives_the_file() {
    let mut saved = Saved::new();

    saved.device.opens(&saved.file);
    assert_eq!(saved.device.order(), saved.ids);

    let mut other = Device::new();
    other.opens(&saved.file);
    assert_eq!(other.order(), saved.ids);
    let copy = other.save_as(&other.root().join("copy.dungeon"));
    let mut third = Device::new();
    third.opens(&copy);
    assert_eq!(third.order(), saved.ids);
}

/// A device that saved a straight and a curved Wall between the fixture's Props.
struct SavedWalls {
    /// The device.
    device: Device,
    /// The straight Wall and the curved one.
    walls: [ElementId; 2],
    /// The file saved.
    file: PathBuf,
}

impl SavedWalls {
    /// Places the Props of [`Saved`] and two Walls among them, and saves.
    fn new() -> Self {
        let mut saved = Saved::new();
        let straight = saved.device.wall(
            &[
                Vec2::new(1.0, 1.0),
                Vec2::new(6.0, 1.0),
                Vec2::new(6.0, 4.0),
            ],
            &[None, None],
            0.125,
            Colour::rgb(60, 60, 60),
        );
        let curved = saved.device.wall(
            &[
                Vec2::new(-2.0, 0.5),
                Vec2::new(3.0, -1.25),
                Vec2::new(8.0, 2.0),
            ],
            &[Some(Vec2::new(0.5, 4.0)), None],
            0.5,
            Colour::rgb(200, 30, 30),
        );
        let file = saved
            .device
            .save_as(&saved.device.root().join("walls.dungeon"));
        Self {
            device: saved.device,
            walls: [straight, curved],
            file,
        }
    }
}

/// A saved Wall holds its points, which segments are curved and their control points, its
/// thickness, and its colour, and reopens the same.
#[test]
fn walls_are_saved_as_their_points() {
    let mut saved = SavedWalls::new();
    let walls = saved.device.walls();
    let order = saved.device.order();

    let written = json(&saved.file);
    let curved = &written["elements"][saved.walls[1].as_raw().to_string()];
    assert_eq!(curved["element"]["data"]["kind"], json!("wall"));
    assert_eq!(
        curved["wall"],
        json!({
            "version": 1,
            "data": {
                "points": [[-2.0, 0.5], [3.0, -1.25], [8.0, 2.0]],
                "segments": [{ "control": [0.5, 4.0] }, { "control": null }],
                "thickness": 0.5,
                "colour": { "red": 200, "green": 30, "blue": 30 }
            }
        })
    );

    let mut other = Device::new();
    other.opens(&saved.file);
    assert_eq!(other.order(), order);
    let reopened = other.walls();
    assert_eq!(reopened, walls);
    assert!(reopened.iter().all(|(_, _, _, shape)| shape.is_some()));

    let again = other.save_as(&other.root().join("again.dungeon"));
    assert_eq!(
        fs::read(&again).expect("the file saved again"),
        fs::read(&saved.file).expect("the file")
    );
}

/// An editor that does not know the Wall kind keeps a saved Wall as a placeholder of its size
/// and writes it back unchanged.
#[test]
fn unknown_walls_round_trip() {
    let mut saved = SavedWalls::new();
    let walls = saved.device.walls();
    let order = saved.device.order();
    let mut unaware = Device::new();
    unaware.forget_walls();

    let opened = unaware.opens(&saved.file);

    assert_eq!(unaware.order(), order);
    assert!(unaware.walls().is_empty(), "no Wall is known");
    for (id, element, _, _) in &walls {
        let placed = unaware
            .elements()
            .into_iter()
            .find(|placed| placed.id == *id)
            .expect("the Wall stays on its Layer");
        assert_eq!(&placed.element, element, "a placeholder of the Wall's size");
        assert_eq!(
            placed
                .unknown
                .expect("the Wall is kept")
                .envelopes
                .keys()
                .collect::<Vec<_>>(),
            vec!["wall"]
        );
    }
    assert_eq!(
        opened.report.unknown_kinds,
        vec![UnknownKind {
            kind: WALL,
            elements: 2
        }]
    );

    let copy = unaware.save_as(&unaware.root().join("copy.dungeon"));
    assert_eq!(
        fs::read(&copy).expect("the copy"),
        fs::read(&saved.file).expect("the file")
    );
    saved.device.opens(&copy);
    assert_eq!(saved.device.walls(), walls);
}
