//! Edit Element: a property change, grouped so that a gesture is one step.

use crate::AuthoringError;
use crate::combined::{Walled, take_walls_away};
use crate::outline::{OutlineEdit, OutlineHost, outline_edit, walls_before};
use crate::portal::{follows_host, portal_change};
use bevy_ecs::entity::Entity;
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;
use drs_history::{SetField, Target};
use drs_model::{
    EditElement, Element, ElementChange, ElementId, Gesture, Portal, PortalsRemoved, Room, Wall,
};
use std::any::Any;

/// Edit Element: sets the changed property through the generic field command, or adds or
/// removes a point of a Wall or a Room as a step of its own, carrying the Portals set into it, or
/// removes a stroke of a Terrain as a step of its own.
///
/// A gesture ([`Gesture::Begin`] through [`Gesture::End`]) is recorded as one history group, so
/// undoing it returns the Element to where the gesture began; a [`Gesture::Single`] change is a
/// step on its own and closes any gesture left open. Adding and removing a point are always a
/// step of their own. A gesture that leaves a Wall or a Room exactly as it began records
/// nothing.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] when no Element carries the identity, what an edit of a
/// Wall or a Room reports, [`AuthoringError::NotAnOutline`] for a change only a Wall or a Room
/// has, [`AuthoringError::NotARoom`] for a floor colour,
/// [`AuthoringError::FollowsItsHost`] for the position of a Portal that follows its host, what a
/// Portal's and a Terrain's own changes report,
/// [`AuthoringError::TerrainChangesOnlyItsMaterialAndStrokes`] for any other change of a
/// Terrain, or [`AuthoringError::History`] when the change could not be recorded.
pub(crate) fn edit_element(
    world: &mut World,
    command: &EditElement,
) -> Result<Vec<PortalsRemoved>, AuthoringError> {
    let id = command.element;
    let entity = id
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(id))?;
    crate::terrain::only_material_and_strokes(world, entity, id, &command.change)?;
    if world.get::<Wall>(entity).is_some() {
        return edit_outline::<Wall>(world, entity, command);
    }
    if world.get::<Room>(entity).is_some() {
        return edit_outline::<Room>(world, entity, command);
    }
    let change = match element_change(world, id, entity, &command.change)? {
        OutlineEdit::Field(field) => field,
        OutlineEdit::Recorded(removed) => return Ok(removed),
    };
    let outcome = record_in_gesture(world, command.gesture, change);
    if matches!(command.gesture, Gesture::End) {
        crate::history(world)?.end_group();
    }
    outcome.map(|()| Vec::new())
}

/// Whether a change of a Wall or a Room can move where Walls run: a move, a point or a control
/// point moved, or whether it cuts.
fn takes_walls(change: &ElementChange) -> bool {
    matches!(
        change,
        ElementChange::Position(_)
            | ElementChange::MoveBy(_)
            | ElementChange::Point { .. }
            | ElementChange::Control { .. }
            | ElementChange::Cuts(_)
    )
}

/// An Edit Element of the Wall or the Room on `entity`: its first step remembers the outline as
/// the gesture began, a move by an amount counts from it, and the last step records nothing when
/// the outline is back where it began. A change that can move where Walls run removes, in its
/// own step or the last step of its gesture, every Portal set into another of its kind on its
/// Layer that had a Wall at its centre before the change, or the gesture, and has none after.
///
/// # Errors
///
/// What [`outline_edit`] reports, or [`AuthoringError::History`] when the change could not be
/// recorded.
fn edit_outline<H: OutlineHost>(
    world: &mut World,
    entity: Entity,
    command: &EditElement,
) -> Result<Vec<PortalsRemoved>, AuthoringError> {
    let id = command.element;
    let walls_move = takes_walls(&command.change);
    if matches!(command.gesture, Gesture::Begin) {
        let began = world.get::<H>(entity).cloned();
        let walls = walls_move.then(|| walls_before::<H>(world, id)).flatten();
        world.get_resource_or_insert_with(GestureStart::default).0 = began.map(|outline| Started {
            element: id,
            outline: Box::new(outline),
            walls,
        });
    }
    let began = match command.gesture {
        Gesture::Single => None,
        Gesture::Begin | Gesture::Continue | Gesture::End => started::<H>(world, id),
    };
    let walls = match command.gesture {
        Gesture::Single => walls_move.then(|| walls_before::<H>(world, id)).flatten(),
        Gesture::Begin | Gesture::Continue => None,
        Gesture::End => world
            .get_resource_mut::<GestureStart>()
            .and_then(|mut start| start.0.take())
            .and_then(|start| start.walls),
    };
    let change = match outline_edit::<H>(world, id, &command.change, began.as_ref())? {
        OutlineEdit::Field(field) => field,
        OutlineEdit::Recorded(removed) => return Ok(removed),
    };
    let walls = walls.filter(|(_, before)| !before.is_empty());
    if matches!(command.gesture, Gesture::Single) {
        let Some((layer, before)) = walls else {
            return crate::record_step(world, change).map(|()| Vec::new());
        };
        crate::history(world)?.begin_group();
        let mut taken = Vec::new();
        let outcome = crate::record(world, change).and_then(|()| {
            take_walls_away::<H>(world, layer, &before).map(|removed| taken = removed)
        });
        crate::close_group(world, outcome)?;
        return Ok(taken);
    }
    let mut outcome = record_in_gesture(world, command.gesture, change);
    if !matches!(command.gesture, Gesture::End) {
        return outcome.map(|()| Vec::new());
    }
    if outcome.is_ok() && began.is_some() && began.as_ref() == world.get::<H>(entity) {
        drs_history::abandon_group(world)
            .map_err(|error| AuthoringError::History(error.to_string()))?;
        return Ok(Vec::new());
    }
    let mut taken = Vec::new();
    if let Some((layer, before)) = walls {
        outcome = outcome.and_then(|()| {
            take_walls_away::<H>(world, layer, &before).map(|removed| taken = removed)
        });
    }
    crate::history(world)?.end_group();
    outcome.map(|()| taken)
}

/// Records a field command as the step of `gesture`: a change on its own closes any gesture left
/// open and is a step of its own, and the first change of a gesture opens its group.
///
/// # Errors
///
/// [`AuthoringError::History`] when the change could not be recorded.
fn record_in_gesture(
    world: &mut World,
    gesture: Gesture,
    change: SetField<ElementId>,
) -> Result<(), AuthoringError> {
    match gesture {
        Gesture::Begin => crate::history(world)?.begin_group(),
        Gesture::Single => crate::history(world)?.end_group(),
        Gesture::Continue | Gesture::End => {}
    }
    crate::record(world, change)
}

/// The Wall or the Room a gesture changes as it stood when the gesture began, kept from its first
/// step to its last, with the Portals set into the others of its kind on its Layer that had a
/// Wall at their centre then, when the gesture can move where Walls run.
#[derive(Resource, Default)]
pub(crate) struct GestureStart(Option<Started>);

/// What a gesture under way began from.
pub(crate) struct Started {
    /// The Wall or the Room it changes.
    element: ElementId,
    /// Its outline as the gesture began.
    outline: Box<dyn Any + Send + Sync>,
    /// Its Layer and the Portals with a Wall at their centre then.
    walls: Option<(Entity, Walled)>,
}

/// The outline `id` had when the gesture under way began, if the gesture changes it.
fn started<H: OutlineHost>(world: &World, id: ElementId) -> Option<H> {
    let start = world.get_resource::<GestureStart>()?.0.as_ref()?;
    (start.element == id)
        .then(|| start.outline.downcast_ref::<H>().cloned())
        .flatten()
}

/// The edit of an Element that is neither a Wall nor a Room: the field command of its position,
/// unless it is a Portal that follows its host, of a change only a Portal has, or of a stroke of
/// a Terrain; or a Terrain's Material or the removal of one of its strokes, each recorded as a
/// step of its own.
///
/// # Errors
///
/// [`AuthoringError::FollowsItsHost`] for the position of a Portal that follows its host,
/// [`AuthoringError::NotAnOutline`] or [`AuthoringError::NotARoom`] for a change only a Wall or
/// a Room has, what a Portal's own changes report, what a Terrain's own changes report, or
/// [`AuthoringError::History`] when the field cannot be addressed.
fn element_change(
    world: &mut World,
    id: ElementId,
    entity: Entity,
    change: &ElementChange,
) -> Result<OutlineEdit, AuthoringError> {
    let field = match change {
        ElementChange::Position(position) => {
            if world.get::<Portal>(entity).is_some() && follows_host(world, id)? {
                return Err(AuthoringError::FollowsItsHost);
            }
            SetField::<ElementId>::new::<Element>(id, "position", *position)
                .map_err(|error| AuthoringError::History(error.to_string()))
        }
        ElementChange::MoveBy(_)
        | ElementChange::Point { .. }
        | ElementChange::Control { .. }
        | ElementChange::AddPoint { .. }
        | ElementChange::RemovePoint { .. }
        | ElementChange::Thickness(_)
        | ElementChange::Colour(_) => Err(AuthoringError::NotAnOutline(id)),
        ElementChange::FloorColour(_) | ElementChange::Cuts(_) => Err(AuthoringError::NotARoom(id)),
        ElementChange::Width(_)
        | ElementChange::Rotation(_)
        | ElementChange::Mirrored(_)
        | ElementChange::Side(_)
        | ElementChange::Along { .. } => {
            portal_change(world, id, change)?.ok_or(AuthoringError::NotAPortal(id))
        }
        ElementChange::Material(asset) => {
            return crate::terrain::set_material(world, id, asset)
                .map(|()| OutlineEdit::Recorded(Vec::new()));
        }
        ElementChange::Stroke { stroke, change } => {
            match crate::terrain::stroke_change(world, id, *stroke, change)? {
                Some(field) => Ok(field),
                None => return Ok(OutlineEdit::Recorded(Vec::new())),
            }
        }
    };
    field.map(OutlineEdit::Field)
}
