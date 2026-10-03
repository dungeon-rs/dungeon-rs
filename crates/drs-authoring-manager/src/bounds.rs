//! Resize Bounds: the Project's Bounds set whole, grouped so that a gesture is one step.

use crate::AuthoringError;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::world::World;
use drs_history::SetField;
use drs_model::{Bounds, Gesture, Project, ResizeBounds};

/// Resize Bounds: swaps the whole Bounds of the one Project for those the Command carries,
/// through the generic field command on the Project entity.
///
/// A gesture ([`Gesture::Begin`] through [`Gesture::End`]) is recorded as one history group, so
/// undoing it returns the Bounds to where the gesture began; a [`Gesture::Single`] resize is a
/// step on its own and closes any gesture left open. Bounds equal to the current ones record
/// nothing, so a gesture none of whose resizes changes them leaves no step. The Project entity is
/// addressed by its entity: Open replaces the Project and empties the history, so no step
/// outlives the entity it names.
///
/// # Errors
///
/// [`AuthoringError::BoundsSides`] for a side below one or above a thousand cells,
/// [`AuthoringError::BoundsReach`] for an edge more than ten thousand cells from the Level's
/// origin, [`AuthoringError::NoProjectToResize`] when the World holds no Project, or
/// [`AuthoringError::History`] when the step could not be recorded. A refused end of a gesture
/// still closes the gesture's group, so the resizes before it are one step.
pub(crate) fn resize_bounds(
    world: &mut World,
    command: &ResizeBounds,
) -> Result<(), AuthoringError> {
    let outcome = checked(world, command.bounds);
    let (project, current) = match outcome {
        Ok(found) => found,
        Err(error) => {
            if command.gesture == Gesture::End {
                crate::history(world)?.end_group();
            }
            return Err(error);
        }
    };
    match command.gesture {
        Gesture::Begin => crate::history(world)?.begin_group(),
        Gesture::Single if current != command.bounds => crate::history(world)?.end_group(),
        Gesture::Single | Gesture::Continue | Gesture::End => {}
    }
    let outcome = if current == command.bounds {
        Ok(())
    } else {
        SetField::<Entity>::new::<Bounds>(project, "", command.bounds)
            .map_err(|error| AuthoringError::History(error.to_string()))
            .and_then(|field| crate::record(world, field))
    };
    if command.gesture == Gesture::End {
        crate::history(world)?.end_group();
    }
    outcome
}

/// The Project entity and its Bounds, once `bounds` are found within the limits.
///
/// # Errors
///
/// As [`resize_bounds`], but for the history.
fn checked(world: &mut World, bounds: Bounds) -> Result<(Entity, Bounds), AuthoringError> {
    if !bounds.sides_within_limits() {
        return Err(AuthoringError::BoundsSides {
            width: bounds.size.x,
            height: bounds.size.y,
        });
    }
    if !bounds.within_reach() {
        return Err(AuthoringError::BoundsReach);
    }
    world
        .query_filtered::<(Entity, &Bounds), With<Project>>()
        .iter(world)
        .next()
        .map(|(project, current)| (project, *current))
        .ok_or(AuthoringError::NoProjectToResize)
}
