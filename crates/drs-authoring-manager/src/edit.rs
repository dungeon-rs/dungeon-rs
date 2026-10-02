//! Edit Element: a property change, grouped so that a gesture is one step.

use crate::AuthoringError;
use crate::portal::portal_change;
use crate::wall::{add_point, remove_point, translated, wall_of, well_formed};
use bevy_ecs::world::World;
use drs_history::{SetField, Target};
use drs_model::{
    EditElement, Element, ElementChange, ElementId, Gesture, Portal, PortalsRemoved, Terrain, Wall,
};

/// Edit Element: sets the changed property through the generic field command, or adds or
/// removes a point of a Wall as a step of its own, carrying the Portals set into it.
///
/// A gesture ([`Gesture::Begin`] through [`Gesture::End`]) is recorded as one history group, so
/// undoing it returns the Element to where the gesture began; a [`Gesture::Single`] change is a
/// step on its own and closes any gesture left open. Adding and removing a point are always a
/// step of their own.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] when no Element carries the identity,
/// [`AuthoringError::NotAWall`] for a change only a Wall has, [`AuthoringError::NoPoint`] or
/// [`AuthoringError::NoSegment`] for a point or segment the Wall does not have,
/// [`AuthoringError::Shape`] for a point added where the Wall cannot be split,
/// [`AuthoringError::MalformedWall`] for a thickness not above zero or a point that is not
/// finite, [`AuthoringError::FollowsItsWall`] for the position of a Portal set into a Wall, what
/// a Portal's own changes report, or [`AuthoringError::History`] when the change could not be
/// recorded.
pub(crate) fn edit_element(
    world: &mut World,
    command: &EditElement,
) -> Result<Option<PortalsRemoved>, AuthoringError> {
    let id = command.element;
    let entity = id
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(id))?;
    // A Terrain is its strokes, which no Edit Element moves or reshapes: only its image changes.
    if world.get::<Terrain>(entity).is_some()
        && !matches!(command.change, ElementChange::Material(_))
    {
        return Err(AuthoringError::TerrainChangesOnlyItsMaterial(id));
    }
    let history = |error: drs_history::HistoryError| AuthoringError::History(error.to_string());
    let change = match &command.change {
        ElementChange::Position(position) => {
            if let Some(wall) = world.get::<Wall>(entity) {
                let moved = translated(wall, *position);
                well_formed(&moved)?;
                SetField::<ElementId>::new::<Wall>(id, "", moved)
            } else if world
                .get::<Portal>(entity)
                .is_some_and(|portal| portal.anchor.is_some())
            {
                return Err(AuthoringError::FollowsItsWall);
            } else {
                SetField::<ElementId>::new::<Element>(id, "position", *position)
            }
        }
        ElementChange::Point { index, position } => {
            let mut wall = wall_of(world, id)?;
            let points = wall.points.len();
            *wall.points.get_mut(*index).ok_or(AuthoringError::NoPoint {
                index: *index,
                points,
            })? = *position;
            well_formed(&wall)?;
            SetField::<ElementId>::new::<Wall>(id, &format!("points[{index}]"), *position)
        }
        ElementChange::Control { segment, position } => {
            let mut wall = wall_of(world, id)?;
            let segments = wall.segments.len();
            wall.segments
                .get_mut(*segment)
                .ok_or(AuthoringError::NoSegment {
                    segment: *segment,
                    segments,
                })?
                .control = *position;
            well_formed(&wall)?;
            SetField::<ElementId>::new::<Wall>(
                id,
                &format!("segments[{segment}].control"),
                *position,
            )
        }
        ElementChange::Thickness(thickness) => {
            let mut wall = wall_of(world, id)?;
            wall.thickness = *thickness;
            well_formed(&wall)?;
            SetField::<ElementId>::new::<Wall>(id, "thickness", *thickness)
        }
        ElementChange::Colour(colour) => {
            wall_of(world, id)?;
            SetField::<ElementId>::new::<Wall>(id, "colour", *colour)
        }
        ElementChange::AddPoint { segment, t } => {
            return add_point(world, id, *segment, *t).map(|()| None);
        }
        ElementChange::RemovePoint { index } => return remove_point(world, id, *index),
        ElementChange::Width(_)
        | ElementChange::Rotation(_)
        | ElementChange::Mirrored(_)
        | ElementChange::Side(_)
        | ElementChange::Along { .. } => match portal_change(world, id, &command.change)? {
            Some(field) => Ok(field),
            None => return Err(AuthoringError::NotAPortal(id)),
        },
        ElementChange::Material(asset) => {
            return crate::terrain::set_material(world, id, asset).map(|()| None);
        }
    }
    .map_err(history)?;

    match command.gesture {
        Gesture::Begin => crate::history(world)?.begin_group(),
        Gesture::Single => crate::history(world)?.end_group(),
        Gesture::Continue | Gesture::End => {}
    }
    let outcome = crate::record(world, change);
    if matches!(command.gesture, Gesture::End) {
        crate::history(world)?.end_group();
    }
    outcome.map(|()| None)
}
