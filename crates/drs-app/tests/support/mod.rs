//! What the seam tests of the Host share: the headless editor of every Manager with no window
//! and no render Engine, fixture images, adding an Asset Folder, and sending Commands, Undo, and
//! Redo to the editor and reading back what they did.
#![allow(
    dead_code,
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    reason = "each test file uses the part of the fixture it needs, and a fixture stops at the \
              first thing that is not as expected"
)]

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::message::Messages;
use bevy::math::UVec2;
use drs_authoring_manager::AuthoringManagerPlugin;
use drs_history::{History, HistoryPlugin};
use drs_library_access::LibraryAccessPlugin;
use drs_library_manager::LibraryManagerPlugin;
use drs_model::{
    AddFolder, Apply, CanonicalName, CommandFailed, EditElement, EditorDirectories, ElementChange,
    ElementId, FolderAdded, FolderRefused, Gesture, ModelPlugin, Project, Redo, Undo,
};
use drs_project_manager::ProjectManagerPlugin;
use std::fs;
use std::path::Path;

/// Writes an opaque PNG of one colour and `size` pixels at `place` under `folder`.
pub fn png(folder: &Path, place: &str, size: UVec2, colour: [u8; 4]) {
    let path = folder.join(place);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture folder");
    }
    image::RgbaImage::from_pixel(size.x, size.y, image::Rgba(colour))
        .save(&path)
        .expect("fixture image");
}

/// A headless editor whose configuration and cache directories live under `root`, started once:
/// the real plugins of the model, the history, `LibraryAccess`, `LibraryManager`,
/// `ProjectManager`, and `AuthoringManager`.
pub fn editor(root: &Path) -> App {
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

/// Sends Add Asset Folder and returns what came back, failing the test if it was refused.
pub fn add_folder(app: &mut App, path: &Path, name: &str) -> FolderAdded {
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

/// The first Layer of the first Level of the one Project.
pub fn first_layer(app: &mut App) -> Entity {
    let world = app.world_mut();
    let project = world
        .query::<(Entity, &Project)>()
        .single(world)
        .expect("one Project")
        .0;
    let first = |entity: Entity| {
        world
            .get::<Children>(entity)
            .and_then(|children| children.iter().next().copied())
    };
    first(project)
        .and_then(first)
        .expect("the first Level has a Layer")
}

/// Sends a Command and runs one update, returning the reasons of any failure.
pub fn try_apply(app: &mut App, command: Apply) -> Vec<String> {
    app.world_mut().write_message(command);
    app.update();
    app.world_mut()
        .resource_mut::<Messages<CommandFailed>>()
        .drain()
        .map(|failed| failed.reason)
        .collect()
}

/// Sends a Command and runs one update, failing the test if the Command was refused.
pub fn apply(app: &mut App, command: Apply) {
    let failed = try_apply(app, command);
    assert!(failed.is_empty(), "the Command failed: {failed:?}");
}

/// An Edit Element of `change` to `element` at `gesture`.
pub fn edit(element: ElementId, change: ElementChange, gesture: Gesture) -> Apply {
    Apply::EditElement(EditElement {
        element,
        change,
        gesture,
    })
}

/// Sends Undo and runs one update.
pub fn undo(app: &mut App) {
    app.world_mut().write_message(Undo);
    app.update();
}

/// Sends Redo and runs one update.
pub fn redo(app: &mut App) {
    app.world_mut().write_message(Redo);
    app.update();
}

/// The history.
pub fn history(app: &App) -> &History {
    app.world().resource::<History>()
}

/// The identities on `layer`, in stacking order.
pub fn order(app: &mut App, layer: Entity) -> Vec<ElementId> {
    let world = app.world_mut();
    let children: Vec<Entity> = world
        .get::<Children>(layer)
        .map(|children| children.iter().copied().collect())
        .unwrap_or_default();
    children
        .into_iter()
        .filter_map(|entity| world.get::<ElementId>(entity).copied())
        .collect()
}

/// The identity of the last Element on `layer`.
pub fn last_on(app: &mut App, layer: Entity) -> ElementId {
    order(app, layer)
        .last()
        .copied()
        .expect("the Layer has Elements")
}

/// The entity of an Element, if one carries the identity.
pub fn entity(app: &mut App, id: ElementId) -> Option<Entity> {
    let world = app.world_mut();
    world
        .query::<(Entity, &ElementId)>()
        .iter(world)
        .find(|(_, found)| **found == id)
        .map(|(entity, _)| entity)
}
