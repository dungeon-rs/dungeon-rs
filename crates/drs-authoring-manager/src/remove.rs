//! Remove Element: a reflection snapshot that remembers the Element's place in the stacking order.

use crate::AuthoringError;
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;
use drs_history::{HistoryError, ReversibleCommand, Snapshot, Target};
use drs_model::{ElementId, RemoveElement};

/// The recorded step: the Element's reflected components, its Layer, and its index among the
/// Layer's children, so that undo puts it back exactly where it was.
struct Remove {
    /// The identity of the Element.
    element: ElementId,
    /// The Element's components while it is removed.
    snapshot: Snapshot<ElementId>,
    /// The Layer the Element sat on.
    layer: Option<Entity>,
    /// The Element's index among the Layer's children.
    index: Option<usize>,
}

impl ReversibleCommand for Remove {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        let entity = self.element.entity(world)?;
        let layer = world
            .get::<ChildOf>(entity)
            .map(ChildOf::parent)
            .ok_or(AuthoringError::NotOnALayer(self.element))?;
        self.index = world
            .get::<Children>(layer)
            .and_then(|children| children.iter().position(|child| *child == entity));
        self.layer = Some(layer);
        self.snapshot.apply(world)
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        self.snapshot.revert(world)?;
        let restored = self
            .snapshot
            .restored()
            .ok_or(HistoryError::MissingTarget)?;
        if let (Some(layer), Some(index)) = (self.layer, self.index) {
            world
                .get_entity_mut(layer)
                .map_err(|_| AuthoringError::NotALayer)?
                .insert_child(index, restored);
        }
        Ok(())
    }
}

/// Remove Element: takes the Element off its Layer as one history step.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] when no Element carries the identity, or
/// [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn remove_element(
    world: &mut World,
    command: &RemoveElement,
) -> Result<(), AuthoringError> {
    command
        .element
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(command.element))?;
    crate::history(world)?.end_group();
    crate::record(
        world,
        Remove {
            element: command.element,
            snapshot: Snapshot::new(command.element),
            layer: None,
            index: None,
        },
    )
}
