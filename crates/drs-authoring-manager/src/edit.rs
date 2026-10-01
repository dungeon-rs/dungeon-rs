//! Edit Element: a property change, grouped so that a gesture is one step.

use crate::AuthoringError;
use bevy_ecs::world::World;
use drs_history::{SetField, Target};
use drs_model::{EditElement, Element, ElementChange, ElementId, Gesture};

/// Edit Element: sets the changed property through the generic field command.
///
/// A gesture ([`Gesture::Begin`] through [`Gesture::End`]) is recorded as one history group, so
/// undoing it returns the Element to where the gesture began; a [`Gesture::Single`] change is a
/// step on its own and closes any gesture left open.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] when no Element carries the identity, or
/// [`AuthoringError::History`] when the change could not be recorded.
pub(crate) fn edit_element(world: &mut World, command: &EditElement) -> Result<(), AuthoringError> {
    command
        .element
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(command.element))?;
    let change = match &command.change {
        ElementChange::Position(position) => {
            SetField::<ElementId>::new::<Element>(command.element, "position", *position)
        }
    }
    .map_err(|error| AuthoringError::History(error.to_string()))?;

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
