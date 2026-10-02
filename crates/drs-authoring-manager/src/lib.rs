#![doc = include_str!("../README.md")]

mod edit;
mod place;
mod portal;
mod remove;
mod room;
mod terrain;
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
    Apply, CanonicalName, CommandFailed, ElementId, FolderKey, HistoryFailed, ManagerSystems,
    PortalsRemoved, Redo, Undo,
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
    /// The change is one only a Wall or a Room has, and the Element is neither.
    #[error("the Element {0:?} is not a Wall or a Room")]
    NotAWall(ElementId),
    /// The change is one only a Room has, and the Element is no Room.
    #[error("the Element {0:?} is not a Room")]
    NotARoom(ElementId),
    /// A Portal is to be set into an Element that is neither a Wall nor a Room.
    #[error("the Element {0:?} is neither a Wall nor a Room to set a Portal into")]
    NotAHost(ElementId),
    /// The Wall or the Room has no point of that number.
    #[error("the {outline} has no point {index}; it has {points}")]
    NoPoint {
        /// What has no such point: a Wall or a Room.
        outline: &'static str,
        /// The point named.
        index: usize,
        /// How many points it has.
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
    /// The Room has no edge of that number.
    #[error("the Room has no edge {edge}; it has {edges}")]
    NoEdge {
        /// The edge named.
        edge: usize,
        /// How many edges the Room has.
        edges: usize,
    },
    /// The Wall would not be one: too few points, a thickness not above zero, or a coordinate
    /// that is not finite.
    #[error("{0}")]
    MalformedWall(String),
    /// The Room would not be one: too few points, a thickness not above zero, or a coordinate
    /// that is not finite.
    #[error("{0}")]
    MalformedRoom(String),
    /// The change or Command is one only a Portal has, and the Element is no Portal.
    #[error("the Element {0:?} is not a Portal")]
    NotAPortal(ElementId),
    /// The Portal would not be one, for the reason its own check gives (a width not above zero, a
    /// rotation that is not finite, a parameter along its segment outside zero to one), or it
    /// would stand at a position that is not finite.
    #[error("{0}")]
    MalformedPortal(String),
    /// A Portal is to be set into a Wall or a Room on another Level than its own.
    #[error("the Wall or Room {0:?} is on another Level than the Portal")]
    OnAnotherLevel(ElementId),
    /// The position, rotation, or mirroring of a Portal set into a Wall is to change, which
    /// follow its Wall.
    #[error("the Portal is set into a Wall, which it follows; free it first")]
    FollowsItsWall,
    /// The side or the place along a Wall of a freestanding Portal is to change.
    #[error("the Portal is freestanding; set it into a Wall first")]
    Freestanding,
    /// The stroke would not be one: no point, a point that is not finite, or Brush settings
    /// that are not a Brush's.
    #[error("{0}")]
    MalformedStroke(String),
    /// A Paint named no Asset on a Layer that has no Terrain to paint more onto.
    #[error("choose an Asset to paint with: the Layer has no Terrain yet")]
    NothingToPaintWith,
    /// A Paint named an Asset other than the one the Layer's Terrain shows.
    #[error(
        "the Layer's Terrain shows {shown}, not {painted}; change the Terrain's image to paint \
         with {painted}"
    )]
    AnotherImage {
        /// The name of the Asset painted with.
        painted: String,
        /// The name of the image the Terrain shows.
        shown: String,
    },
    /// The change is one only a Terrain has, and the Element is no Terrain.
    #[error("the Element {0:?} is not a Terrain")]
    NotATerrain(ElementId),
    /// The change is not one a Terrain has: only its image can be changed.
    #[error("only the image a Terrain shows can be changed, not its strokes or its place")]
    TerrainChangesOnlyItsMaterial(ElementId),
    /// The shape Engine could not reshape the Wall or the Room.
    #[error(transparent)]
    Shape(#[from] ShapeError),
    /// The Asset's file could not be read or is not an image.
    #[error(transparent)]
    Library(#[from] LibraryError),
    /// The history could not record, undo, or redo the step.
    #[error("{0}")]
    History(String),
}

/// Handles the [`Apply`], [`Undo`], and [`Redo`] messages, answering a Command that removed
/// Portals with [`PortalsRemoved`] and what fails with [`CommandFailed`] or [`HistoryFailed`].
pub struct AuthoringManagerPlugin;

impl Plugin for AuthoringManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_apply.in_set(ManagerSystems::Commands),
                handle_undo.in_set(ManagerSystems::Undo),
                handle_redo.in_set(ManagerSystems::Redo),
                // Every Manager has handled its Commands, Undo, and Redo by then, so a Wall, a
                // Room, or a Portal placed, edited, undone, redone, or opened has its shape and
                // its place before anything draws or picks it.
                wall::derive_shapes.after(ManagerSystems::Redo),
                // Likewise a Terrain painted, undone, redone, or opened has its coverage before
                // anything draws it.
                terrain::derive_coverage.after(ManagerSystems::Redo),
            ),
        );
    }
}

/// Apply: carries an authoring Command out and records it in the history. Returns the answer
/// naming the Portals the Command removed with the part of a Wall or a Room they were set into,
/// when it removed any.
///
/// # Errors
///
/// The [`AuthoringError`] that applies, in which case nothing is recorded.
pub(crate) fn apply(
    world: &mut World,
    command: &Apply,
) -> Result<Option<PortalsRemoved>, AuthoringError> {
    match command {
        Apply::PlaceElement(place) => place::place_element(world, place).map(|()| None),
        Apply::EditElement(edit) => edit::edit_element(world, edit),
        Apply::RemoveElement(remove) => remove::remove_element(world, remove),
        Apply::SetPortalIntoWall(set) => portal::set_portal_into_wall(world, set).map(|()| None),
        Apply::FreePortal(free) => portal::free_portal(world, free).map(|()| None),
        Apply::Paint(paint) => terrain::paint(world, paint).map(|()| None),
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

/// Closes the group a Command opened for its commands: on success they become one step, and on
/// failure the part already applied is taken back and nothing is recorded, so a Command is never
/// left half done. Returns `outcome`.
///
/// # Errors
///
/// `outcome`'s error, or [`AuthoringError::History`] when the history is gone or the applied
/// part could not be taken back.
pub(crate) fn close_group(
    world: &mut World,
    outcome: Result<(), AuthoringError>,
) -> Result<(), AuthoringError> {
    match outcome {
        Ok(()) => {
            history(world)?.end_group();
            Ok(())
        }
        Err(error) => {
            drs_history::abandon_group(world)
                .map_err(|undone| AuthoringError::History(undone.to_string()))?;
            Err(error)
        }
    }
}

/// Carries out every [`Apply`] request, answering one that removed Portals with
/// [`PortalsRemoved`] and one that fails with [`CommandFailed`].
fn handle_apply(world: &mut World, requests: &mut SystemState<MessageReader<Apply>>) {
    let requests: Vec<Apply> = match requests.get_mut(world) {
        Ok(mut reader) => reader.read().cloned().collect(),
        Err(_) => return,
    };
    for command in requests {
        match apply(world, &command) {
            Ok(Some(removed)) => {
                world.write_message(removed);
            }
            Ok(None) => {}
            Err(error) => {
                world.write_message(CommandFailed {
                    command,
                    reason: error.to_string(),
                });
            }
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
