//! What the seam tests of the Host share: the headless editor of every Manager with no window
//! and no render Engine, the one with offscreen rendering, fixture images, adding an Asset
//! Folder, sending Commands, Undo, and Redo to the editor and reading back what they did, and
//! comparing the geometry it derives.
#![allow(
    dead_code,
    reason = "each test file uses the part of the fixture it needs, so which part is unused \
              depends on the file"
)]
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    reason = "a fixture stops at the first thing that is not as expected"
)]

pub mod offscreen;
pub mod terrain;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::message::Messages;
use bevy::math::{Rect, UVec2, Vec2};
use drs_authoring_manager::AuthoringManagerPlugin;
use drs_history::{History, HistoryPlugin};
use drs_library_access::LibraryAccessPlugin;
use drs_library_manager::LibraryManagerPlugin;
use drs_model::{
    AddFolder, Apply, CanonicalName, CommandFailed, EditElement, EditorDirectories, ElementChange,
    ElementId, FolderAdded, FolderRefused, Gesture, ModelPlugin, Project, Redo, Undo, Viewport,
};
use drs_paint_engine::PaintEnginePlugin;
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
/// `ProjectManager`, `AuthoringManager`, and the paint Engine's, which has no renderer to
/// rasterize on here.
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
        PaintEnginePlugin,
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

/// Writes the Viewport as the Editor does, showing an area of `area` screen pixels around
/// `centre` at `zoom`, and runs one update.
pub fn look(app: &mut App, centre: Vec2, zoom: f32, area: Vec2) {
    let mut viewport = app
        .world_mut()
        .get_resource_mut::<Viewport>()
        .expect("the Viewport");
    viewport.centre = centre;
    viewport.zoom = zoom;
    viewport.area = Rect::from_corners(Vec2::ZERO, area);
    app.update();
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

/// How far apart two points computed along different paths may lie, in cells.
pub const CLOSE: f32 = 1e-4;

/// The point of the quadratic curve from `start` through `control` to `end` at `t`.
pub fn quadratic(start: Vec2, control: Vec2, end: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    start * (u * u) + control * (2.0 * u * t) + end * (t * t)
}

/// Asserts that two points lie within [`CLOSE`] of each other.
pub fn assert_near(found: Vec2, expected: Vec2, what: &str) {
    assert!(
        found.distance(expected) < CLOSE,
        "{what}: {found} against {expected}"
    );
}

/// Asserts that two numbers lie within [`CLOSE`] of each other.
pub fn assert_close(found: f32, expected: f32, what: &str) {
    assert!(
        (found - expected).abs() < CLOSE,
        "{what}: {found} against {expected}"
    );
}
