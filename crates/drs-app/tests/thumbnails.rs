//! Thumbnails through undo and redo in the headless editor: the real plugins of the model, the
//! history, `LibraryAccess`, `LibraryManager`, `ProjectManager`, and `AuthoringManager`, driven by
//! the `Undo` and `Redo` messages over a fixture folder of images in a temporary directory.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

use bevy::app::App;
use bevy::ecs::message::Messages;
use drs_authoring_manager::AuthoringManagerPlugin;
use drs_history::HistoryPlugin;
use drs_library_access::LibraryAccessPlugin;
use drs_library_manager::LibraryManagerPlugin;
use drs_model::{
    AddFolder, AssetFolder, CanonicalName, EditorDirectories, FolderAdded, FolderRefused,
    ModelPlugin, Redo, ThumbnailState, Thumbnails, Undo,
};
use drs_project_manager::ProjectManagerPlugin;
use image::{Rgb, RgbImage};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
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

/// The thumbnail state of every Asset of every folder.
fn states(app: &mut App) -> Vec<ThumbnailState> {
    let world = app.world_mut();
    world
        .query::<(&AssetFolder, Option<&Thumbnails>)>()
        .iter(world)
        .flat_map(|(folder, thumbnails)| {
            (0..folder.assets.len()).map(move |index| {
                thumbnails
                    .and_then(|thumbnails| thumbnails.states.get(index).copied())
                    .unwrap_or_default()
            })
        })
        .collect()
}

/// Whether every Asset is ready.
fn all_ready(states: &[ThumbnailState]) -> bool {
    states
        .iter()
        .all(|state| matches!(state, ThumbnailState::Ready(_)))
}

/// Undoing and redoing Add Asset Folder keeps the folder's thumbnails, which are served again on
/// redo without being generated again.
#[test]
fn kept_through_undo_and_redo() {
    let root = TempDir::new().expect("temporary root");
    let maps = root.path().join("maps");
    fs::create_dir_all(&maps).expect("fixture folder");
    for index in 0..3 {
        RgbImage::from_pixel(40, 40, Rgb([40, 90, 160]))
            .save(maps.join(format!("tile_{index}.png")))
            .expect("fixture image");
    }
    let mut app = editor(root.path());
    add(&mut app, &maps, "Maps");
    let start = Instant::now();
    while !all_ready(&states(&mut app)) {
        assert!(
            start.elapsed() < Duration::from_secs(120),
            "the thumbnails were not generated"
        );
        app.update();
        std::thread::sleep(Duration::from_millis(2));
    }
    let index = root.path().join("cache/thumbnails/thumbnails.index");
    let generated = fs::read(&index).expect("the thumbnail index");

    app.world_mut().write_message(Undo);
    app.update();
    assert!(states(&mut app).is_empty());
    app.world_mut().write_message(Redo);
    app.update();

    let states = states(&mut app);
    assert_eq!(states.len(), 3);
    assert!(all_ready(&states), "served again on redo: {states:?}");
    for _ in 0..20 {
        app.update();
    }
    assert_eq!(
        fs::read(&index).expect("the thumbnail index"),
        generated,
        "nothing generated again"
    );
}
