#![doc = include_str!("../README.md")]

mod edit;
mod place;
mod portal;
mod remove;
mod wall;

use bevy_app::{App, Plugin, Update};
use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageReader;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::system::SystemState;
use bevy_ecs::world::{Mut, World};
use drs_history::History;
use drs_library_access::LibraryError;
use drs_model::{
    Apply, CanonicalName, CommandFailed, ElementId, FolderKey, HistoryFailed, ManagerSystems, Redo,
    Undo,
};
use drs_shape_engine::ShapeError;

/// Why an authoring Command could not be carried out.
#[derive(Debug, thiserror::Error)]
pub enum AuthoringError {
    /// No added Asset Folder has the key the chosen Asset names.
    #[error("no Asset Folder has the key {0}")]
    UnknownFolder(FolderKey),
    /// The Asset Folder holds no Asset at the chosen place.
    #[error("the Asset Folder {folder} holds no Asset at `{place}`")]
    UnknownAsset {
        /// The folder's Canonical Name.
        folder: CanonicalName,
        /// The place that was chosen.
        place: String,
    },
    /// The entity to place on is not a Layer.
    #[error("the entity to place on is not a Layer")]
    NotALayer,
    /// The Layer belongs to no Project, so there is no table to record the Asset Reference in.
    #[error("the Layer belongs to no Project")]
    NoProject,
    /// No Element carries the identity.
    #[error("no Element has the identity {0:?}")]
    UnknownElement(ElementId),
    /// The Element sits on no Layer.
    #[error("the Element {0:?} sits on no Layer")]
    NotOnALayer(ElementId),
    /// The change is one only a Wall has, and the Element is no Wall.
    #[error("the Element {0:?} is not a Wall")]
    NotAWall(ElementId),
    /// The Wall has no point of that number.
    #[error("the Wall has no point {index}; it has {points}")]
    NoPoint {
        /// The point named.
        index: usize,
        /// How many points the Wall has.
        points: usize,
    },
    /// The Wall has no segment of that number.
    #[error("the Wall has no segment {segment}; it has {segments}")]
    NoSegment {
        /// The segment named.
        segment: usize,
        /// How many segments the Wall has.
        segments: usize,
    },
    /// The Wall would not be one: too few points, a thickness not above zero, or a coordinate
    /// that is not finite.
    #[error("{0}")]
    MalformedWall(String),
    /// The change or Command is one only a Portal has, and the Element is no Portal.
    #[error("the Element {0:?} is not a Portal")]
    NotAPortal(ElementId),
    /// The Portal would not be one, for the reason its own check gives (a width not above zero, a
    /// rotation that is not finite, a parameter along its segment outside zero to one), or it
    /// would stand at a position that is not finite.
    #[error("{0}")]
    MalformedPortal(String),
    /// A Portal is to be set into a Wall on another Level than its own.
    #[error("the Wall {0:?} is on another Level than the Portal")]
    OnAnotherLevel(ElementId),
    /// The position, rotation, or mirroring of a Portal set into a Wall is to change, which
    /// follow its Wall.
    #[error("the Portal is set into a Wall, which it follows; free it first")]
    FollowsItsWall,
    /// The side or the place along a Wall of a freestanding Portal is to change.
    #[error("the Portal is freestanding; set it into a Wall first")]
    Freestanding,
    /// The shape Engine could not reshape the Wall.
    #[error(transparent)]
    Shape(#[from] ShapeError),
    /// The Asset's file could not be read or is not an image.
    #[error(transparent)]
    Library(#[from] LibraryError),
    /// The history could not record, undo, or redo the step.
    #[error("{0}")]
    History(String),
}

/// Handles the [`Apply`], [`Undo`], and [`Redo`] messages, answering what fails with
/// [`CommandFailed`] or [`HistoryFailed`].
pub struct AuthoringManagerPlugin;

impl Plugin for AuthoringManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_apply.in_set(ManagerSystems::Commands),
                handle_undo.in_set(ManagerSystems::Undo),
                handle_redo.in_set(ManagerSystems::Redo),
                // Every Manager has handled its Commands, Undo, and Redo by then, so a Wall or a
                // Portal placed, edited, undone, redone, or opened has its shape and its place
                // before anything draws or picks it.
                wall::derive_shapes.after(ManagerSystems::Redo),
            ),
        );
    }
}

/// Apply: carries an authoring Command out and records it in the history.
///
/// # Errors
///
/// The [`AuthoringError`] that applies, in which case nothing is recorded.
pub(crate) fn apply(world: &mut World, command: &Apply) -> Result<(), AuthoringError> {
    match command {
        Apply::PlaceElement(place) => place::place_element(world, place),
        Apply::EditElement(edit) => edit::edit_element(world, edit),
        Apply::RemoveElement(remove) => remove::remove_element(world, remove),
        Apply::SetPortalIntoWall(set) => portal::set_portal_into_wall(world, set),
        Apply::FreePortal(free) => portal::free_portal(world, free),
    }
}

/// Undo: takes the most recent step back, whichever Manager recorded it. Returns `false` when
/// there was nothing to undo.
///
/// # Errors
///
/// [`AuthoringError::History`] when the step could not be reverted; it then stays where it was.
pub(crate) fn undo(world: &mut World) -> Result<bool, AuthoringError> {
    drs_history::undo(world).map_err(|error| AuthoringError::History(error.to_string()))
}

/// Redo: carries the most recently undone step out again. Returns `false` when there was nothing
/// to redo.
///
/// # Errors
///
/// [`AuthoringError::History`] when the step could not be applied again; it then stays where it was.
pub(crate) fn redo(world: &mut World) -> Result<bool, AuthoringError> {
    drs_history::redo(world).map_err(|error| AuthoringError::History(error.to_string()))
}

/// The nearest ancestor of `entity` that `found` accepts, going up through `parent_of`.
pub(crate) fn ancestor(
    entity: Entity,
    parent_of: impl Fn(Entity) -> Option<Entity>,
    found: impl Fn(Entity) -> bool,
) -> Option<Entity> {
    let mut current = entity;
    while let Some(parent) = parent_of(current) {
        if found(parent) {
            return Some(parent);
        }
        current = parent;
    }
    None
}

/// The history, which every step is recorded in.
///
/// # Errors
///
/// [`AuthoringError::History`] when the `World` has no history.
pub(crate) fn history(world: &mut World) -> Result<Mut<'_, History>, AuthoringError> {
    world
        .get_resource_mut::<History>()
        .ok_or_else(|| AuthoringError::History(drs_history::HistoryError::NoHistory.to_string()))
}

/// Records a command in the history, joining the gesture group that is open, if any.
///
/// # Errors
///
/// [`AuthoringError::History`] with the command's reason when it could not be applied.
pub(crate) fn record(
    world: &mut World,
    command: impl drs_history::ReversibleCommand,
) -> Result<(), AuthoringError> {
    drs_history::apply(world, command).map_err(|error| AuthoringError::History(error.to_string()))
}

/// Records a command as a step of its own, closing any gesture group left open.
///
/// # Errors
///
/// [`AuthoringError::History`] with the command's reason when it could not be applied.
pub(crate) fn record_step(
    world: &mut World,
    command: impl drs_history::ReversibleCommand,
) -> Result<(), AuthoringError> {
    drs_history::apply_step(world, command)
        .map_err(|error| AuthoringError::History(error.to_string()))
}

/// Carries out every [`Apply`] request, answering one that fails with [`CommandFailed`].
fn handle_apply(world: &mut World, requests: &mut SystemState<MessageReader<Apply>>) {
    let requests: Vec<Apply> = match requests.get_mut(world) {
        Ok(mut reader) => reader.read().cloned().collect(),
        Err(_) => return,
    };
    for command in requests {
        if let Err(error) = apply(world, &command) {
            world.write_message(CommandFailed {
                command,
                reason: error.to_string(),
            });
        }
    }
}

/// Takes one step back for every [`Undo`] request, answering one that fails with
/// [`HistoryFailed`].
fn handle_undo(world: &mut World, requests: &mut SystemState<MessageReader<Undo>>) {
    let count = match requests.get_mut(world) {
        Ok(mut reader) => reader.read().count(),
        Err(_) => return,
    };
    for _ in 0..count {
        if let Err(error) = undo(world) {
            world.write_message(HistoryFailed {
                reason: format!("The step could not be undone: {error}"),
            });
        }
    }
}

/// Carries one undone step out again for every [`Redo`] request, answering one that fails with
/// [`HistoryFailed`].
fn handle_redo(world: &mut World, requests: &mut SystemState<MessageReader<Redo>>) {
    let count = match requests.get_mut(world) {
        Ok(mut reader) => reader.read().count(),
        Err(_) => return,
    };
    for _ in 0..count {
        if let Err(error) = redo(world) {
            world.write_message(HistoryFailed {
                reason: format!("The step could not be redone: {error}"),
            });
        }
    }
}
