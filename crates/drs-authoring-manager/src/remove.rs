//! Remove Element: a reflection snapshot that remembers the Element's place in the stacking order.

use crate::AuthoringError;
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;
use drs_history::{HistoryError, ReversibleCommand, Snapshot, Target};
use drs_model::{ElementId, PortalsRemoved, RemoveElement, Room, Wall};

/// The recorded step: the Element's reflected components, its Layer, and its index among the
/// Layer's children, so that undo puts it back exactly where it was.
pub(crate) struct Remove {
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
            // The whole order is rebuilt rather than inserted into, so the Props above the
            // restored one keep their places whatever the children collection does on insert.
            let mut order: Vec<Entity> = world
                .get::<Children>(layer)
                .map(|children| {
                    children
                        .iter()
                        .copied()
                        .filter(|child| *child != restored)
                        .collect()
                })
                .unwrap_or_default();
            order.insert(index.min(order.len()), restored);
            world
                .get_entity_mut(layer)
                .map_err(|_| AuthoringError::NotALayer)?
                .replace_children(&order);
        }
        Ok(())
    }
}

/// Remove Element: takes the Element off its Layer as one history step; a Wall or a Room takes
/// the Portals set into it with it, in the same step, and the answer naming them is returned.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] when no Element carries the identity, or
/// [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn remove_element(
    world: &mut World,
    command: &RemoveElement,
) -> Result<Option<PortalsRemoved>, AuthoringError> {
    let entity = command
        .element
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(command.element))?;
    if world.get::<Wall>(entity).is_some() || world.get::<Room>(entity).is_some() {
        return crate::wall::remove_with_portals(world, command.element);
    }
    crate::record_step(world, Remove::of(command.element)).map(|()| None)
}

impl Remove {
    /// The step that removes `element`.
    pub(crate) fn of(element: ElementId) -> Self {
        Self {
            element,
            snapshot: Snapshot::new(element),
            layer: None,
            index: None,
        }
    }
}
