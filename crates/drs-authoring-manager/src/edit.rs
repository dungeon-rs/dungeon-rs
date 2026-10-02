//! Edit Element: a property change, grouped so that a gesture is one step.

use crate::AuthoringError;
use crate::wall::{add_point, remove_point, translated, wall_of};
use bevy_ecs::world::World;
use drs_history::{SetField, Target};
use drs_model::{EditElement, Element, ElementChange, ElementId, Gesture, Wall};

/// Edit Element: sets the changed property through the generic field command, or adds or
/// removes a point of a Wall as a step of its own.
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
/// [`AuthoringError::Shape`] for a point or segment the Wall does not have,
/// [`AuthoringError::MalformedWall`] for a thickness not above zero or a point that is not
/// finite, or [`AuthoringError::History`] when the change could not be recorded.
pub(crate) fn edit_element(world: &mut World, command: &EditElement) -> Result<(), AuthoringError> {
    let id = command.element;
    let entity = id
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(id))?;
    let history = |error: drs_history::HistoryError| AuthoringError::History(error.to_string());
    let change = match &command.change {
        ElementChange::Position(position) => match world.get::<Wall>(entity) {
            Some(wall) => {
                let moved = translated(wall, *position);
                if let Some(reason) = moved.malformation() {
                    return Err(AuthoringError::MalformedWall(reason));
                }
                SetField::<ElementId>::new::<Wall>(id, "", moved)
            }
            None => SetField::<ElementId>::new::<Element>(id, "position", *position),
        },
        ElementChange::Point { index, position } => {
            let points = wall_of(world, id)?.points.len();
            if *index >= points {
                return Err(AuthoringError::NoPoint {
                    index: *index,
                    points,
                });
            }
            finite(*position)?;
            SetField::<ElementId>::new::<Wall>(id, &format!("points[{index}]"), *position)
        }
        ElementChange::Control { segment, position } => {
            let segments = wall_of(world, id)?.segments.len();
            if *segment >= segments {
                return Err(drs_shape_engine::ShapeError::NoSegment {
                    segment: *segment,
                    segments,
                }
                .into());
            }
            if let Some(position) = position {
                finite(*position)?;
            }
            SetField::<ElementId>::new::<Wall>(
                id,
                &format!("segments[{segment}].control"),
                *position,
            )
        }
        ElementChange::Thickness(thickness) => {
            wall_of(world, id)?;
            if !(*thickness > 0.0 && thickness.is_finite()) {
                return Err(AuthoringError::MalformedWall(format!(
                    "a Wall's thickness must be above zero, not {thickness}"
                )));
            }
            SetField::<ElementId>::new::<Wall>(id, "thickness", *thickness)
        }
        ElementChange::Colour(colour) => {
            wall_of(world, id)?;
            SetField::<ElementId>::new::<Wall>(id, "colour", *colour)
        }
        ElementChange::AddPoint { segment, t } => return add_point(world, id, *segment, *t),
        ElementChange::RemovePoint { index } => return remove_point(world, id, *index),
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
    outcome
}

/// Refuses a point that is not finite.
///
/// # Errors
///
/// [`AuthoringError::MalformedWall`] when either coordinate is not finite.
fn finite(position: bevy_math::Vec2) -> Result<(), AuthoringError> {
    if position.is_finite() {
        Ok(())
    } else {
        Err(AuthoringError::MalformedWall(
            "a Wall's points must be finite".to_owned(),
        ))
    }
}
