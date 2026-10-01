//! Undoing an Asset Folder through the headless editor: the real plugins of the model, the
//! history, `LibraryAccess`, `LibraryManager`, `ProjectManager`, and `AuthoringManager`, driven by
//! the `Undo` and `Redo` messages over a fixture folder in a temporary directory.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

use bevy::app::App;
use bevy::ecs::message::Messages;
use drs_authoring_manager::AuthoringManagerPlugin;
use drs_history::{History, HistoryPlugin};
use drs_library_access::LibraryAccessPlugin;
use drs_library_manager::LibraryManagerPlugin;
use drs_model::{
    AddFolder, AssetFolder, CanonicalName, EditorDirectories, FolderAdded, FolderRefused,
    ModelPlugin, Redo, Undo,
};
use drs_project_manager::ProjectManagerPlugin;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

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
fn add(app: &mut App, path: &Path, name: &str) -> FolderAdded {
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

/// Every Asset Folder in the World.
fn folders(app: &mut App) -> Vec<AssetFolder> {
    let world = app.world_mut();
    world.query::<&AssetFolder>().iter(world).cloned().collect()
}

/// The Manifest files in the configuration directory under `root`.
fn manifests(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(root.join("configuration")) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect()
}

/// Undoing makes the folder's Assets unavailable and removes its Manifest; redoing restores it
/// with the same Canonical Name without asking.
#[test]
fn add_asset_folder_is_undoable() {
    let root = TempDir::new().expect("temporary root");
    let maps = root.path().join("maps");
    fs::create_dir_all(&maps).expect("fixture folder");
    fs::write(maps.join("table.png"), b"fixture").expect("fixture file");
    let mut app = editor(root.path());
    let added = add(&mut app, &maps, "Maps");
    assert!(app.world().resource::<History>().can_undo());

    app.world_mut().write_message(Undo);
    app.update();
    assert!(folders(&mut app).is_empty());
    assert!(manifests(root.path()).is_empty());
    assert!(app.world().resource::<History>().can_redo());

    app.world_mut().write_message(Redo);
    app.update();
    let folders = folders(&mut app);
    assert_eq!(folders.len(), 1);
    assert_eq!(folders[0].name, CanonicalName("Maps".to_owned()));
    assert_eq!(folders[0].key, added.key);
    assert_eq!(folders[0].assets[0].place, "table.png");
    assert_eq!(manifests(root.path()).len(), 1);
    assert!(
        app.world_mut()
            .resource_mut::<Messages<FolderRefused>>()
            .drain()
            .next()
            .is_none(),
        "redo asks nothing and refuses nothing"
    );
}
