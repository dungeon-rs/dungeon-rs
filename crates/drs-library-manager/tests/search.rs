//! Searching the library through the headless editor: the real plugins of the model, the
//! history, `LibraryAccess`, and `LibraryManager` over fixture folders in temporary directories,
//! asked with Browse and answered in the matches resource.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

use bevy_app::App;
use bevy_ecs::message::Messages;
use drs_history::HistoryPlugin;
use drs_library_access::LibraryAccessPlugin;
use drs_library_manager::LibraryManagerPlugin;
use drs_model::{
    AddFolder, AssetFolder, Browse, CanonicalName, EditorDirectories, FolderAdded, FolderRefused,
    ModelPlugin, SearchMatches,
};
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

/// Sends Browse with the search text `text` and nothing wanted, without running a frame.
fn type_in(app: &mut App, text: &str) {
    app.world_mut().write_message(Browse {
        search: text.to_owned(),
        wanted: Vec::new(),
    });
}

/// What the matches resource answers: the text, the total, and each match as its folder's
/// Canonical Name and its place, in order.
fn matches(app: &mut App) -> (String, usize, Vec<(String, String)>) {
    let world = app.world_mut();
    let answer = world.resource::<SearchMatches>().clone();
    let assets = answer
        .assets
        .iter()
        .map(|found| {
            let folder = world
                .get::<AssetFolder>(found.folder)
                .expect("a match names an added folder");
            (
                folder.name.0.clone(),
                folder.assets[found.position as usize].place.clone(),
            )
        })
        .collect();
    (answer.text, answer.total, assets)
}

/// The places of the matches, in order.
fn places(app: &mut App) -> Vec<String> {
    matches(app).2.into_iter().map(|(_, place)| place).collect()
}

/// Writes a small file at `place` under `folder`, creating folders on the way. A scan reads
/// metadata only, so the bytes need not be an image.
fn file(folder: &Path, place: &str) {
    let path = folder.join(place);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture folder");
    }
    fs::write(&path, b"fixture").expect("fixture file");
}

/// A fixture Asset Folder at `root/<name>` holding a file at each of `places`.
fn folder(root: &Path, name: &str, places: &[&str]) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(&path).expect("fixture folder");
    for place in places {
        file(&path, place);
    }
    path
}

/// Each change of the typed text is searched without the Author confirming it, and the matches
/// for it are shown in the frame after the change.
#[test]
fn every_keystroke_searches() {
    let root = TempDir::new().expect("temporary root");
    let props = folder(
        root.path(),
        "props",
        &["Barrel.png", "Barn_Door.png", "Table.png"],
    );
    let mut app = editor(root.path());
    add(&mut app, &props, "Props");

    type_in(&mut app, "bar");
    app.update();
    assert_eq!(
        matches(&mut app),
        (
            "bar".to_owned(),
            2,
            vec![
                ("Props".to_owned(), "Barn_Door.png".to_owned()),
                ("Props".to_owned(), "Barrel.png".to_owned()),
            ]
        )
    );

    type_in(&mut app, "barr");
    app.update();
    assert_eq!(places(&mut app), vec!["Barrel.png"]);

    type_in(&mut app, "");
    app.update();
    let (text, total, _) = matches(&mut app);
    assert_eq!((text.as_str(), total), ("", 3));
}

/// The matches for the typed text include a folder's matching Assets from the frame Add Asset
/// Folder is applied, without the text being typed again.
#[test]
fn matches_follow_an_added_folder() {
    let root = TempDir::new().expect("temporary root");
    let props = folder(root.path(), "props", &["Barrel.png", "Table.png"]);
    let vendor = folder(
        root.path(),
        "vendor",
        &["barrels/Old_Barrel.png", "Crate.png"],
    );
    let mut app = editor(root.path());
    add(&mut app, &props, "Props");
    type_in(&mut app, "barrel");
    app.update();
    assert_eq!(places(&mut app), vec!["Barrel.png"]);

    add(&mut app, &vendor, "Vendor");

    assert_eq!(
        matches(&mut app),
        (
            "barrel".to_owned(),
            2,
            vec![
                ("Props".to_owned(), "Barrel.png".to_owned()),
                ("Vendor".to_owned(), "barrels/Old_Barrel.png".to_owned()),
            ]
        )
    );
}

/// The matches reflect at start the Assets added to or removed from a remembered folder while
/// the editor was closed.
#[test]
fn matches_are_current_at_start() {
    let root = TempDir::new().expect("temporary root");
    let props = folder(root.path(), "props", &["Barrel.png", "Table.png"]);
    {
        let mut app = editor(root.path());
        add(&mut app, &props, "Props");
    }
    fs::remove_file(props.join("Barrel.png")).expect("the barrel is removed");
    file(&props, "Barrel_Large.png");
    file(&props, "Chair.png");

    let mut app = editor(root.path());
    app.update();
    assert_eq!(matches(&mut app).1, 3);
    type_in(&mut app, "barrel");
    app.update();

    assert_eq!(places(&mut app), vec!["Barrel_Large.png"]);
}

/// A remembered folder that cannot be scanned contributes no matches.
#[test]
fn only_available_assets_match() {
    let root = TempDir::new().expect("temporary root");
    let props = folder(root.path(), "props", &["Barrel.png"]);
    let unplugged = folder(root.path(), "unplugged", &["Barrel_Old.png", "Table.png"]);
    {
        let mut app = editor(root.path());
        add(&mut app, &props, "Props");
        add(&mut app, &unplugged, "Unplugged");
    }
    fs::remove_dir_all(&unplugged).expect("the disk is unplugged");

    let mut app = editor(root.path());
    type_in(&mut app, "barrel");
    app.update();

    assert_eq!(
        matches(&mut app),
        (
            "barrel".to_owned(),
            1,
            vec![("Props".to_owned(), "Barrel.png".to_owned())]
        )
    );
    let counts = app.world().resource::<SearchMatches>().counts.len();
    assert_eq!(
        counts, 2,
        "the unplugged folder is still listed, with no match"
    );
}
