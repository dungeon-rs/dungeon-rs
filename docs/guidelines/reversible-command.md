# Reversible command

**Use when**: a Manager records a new undoable step. **Not when**: the Command sets one reflected field of one component, or a whole component at the reflect path `""` (`drs_history::SetField`, as `edit.rs` does for a moved point or a Wall moved whole), or the work is not undone at all (Export Level, restoring folders at start). A step that swaps a whole component is still its own struct when later changes must hook into that step alone: `Reshape` in `wall.rs` adds and removes a Wall's points, the only edits that renumber its segments.
**Exemplar**: `crates/drs-authoring-manager/src/remove.rs`

## Rules

- One struct per step, holding what `apply` needs to do the same thing again and what `revert` needs to put it back; what `apply` learns on the way (the Layer, the index, the spawned entity) goes into `Option` fields it fills.
- The struct is `pub(crate)` with a `pub(crate) fn of(..) -> Self` constructor only when another step records it too (removing the last two points of a Wall records `Remove::of`); otherwise it stays private.
- The `pub(crate) fn <verb>_<noun>(world, command) -> Result<_, ManagerError>` does every check and read that can fail (resolving the Asset, loading its file, checking the name) before it builds the struct, then records it with `drs_history::apply_step` (`crate::record_step` in the authoring Manager), mapping the history's error into the Manager's own. Nothing is recorded when the first `apply` fails.
- Inside `apply`, what can fail comes before the first World mutation, and a later failure takes the earlier side effect back (`forget_manifest` when `index_folder` fails), so a failed step leaves the World and the disk as they were.
- `apply` is also redo, so it reads nothing another step could have changed since: what the request resolves (the Asset's size, the Manifest, a fresh `ElementId`) is resolved once and kept on the struct. An Element is addressed by `ElementId` (a `Target`); an entity handle is kept only for an entity this step alone spawns.
- A step that takes an Element off its Layer remembers its index among the Layer's `Children`; `revert` restores it through `Snapshot` and `restored()`, then rebuilds the whole order without it and calls `replace_children` with it inserted at `index.min(order.len())`. _Why_: the Props above it keep their places whatever the children collection does on insert.
- A step that places an Element spawns it through `place::spawn_on_top` and reverts through `place::take_off`, by identity; `revert` of a removal restores first, then the order.

## Example

```rust
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

/// Remove Element: takes the Element off its Layer as one history step; a Wall takes the
/// Portals set into it with it, in the same step.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] when no Element carries the identity, or
/// [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn remove_element(
    world: &mut World,
    command: &RemoveElement,
) -> Result<(), AuthoringError> {
    let entity = command
        .element
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(command.element))?;
    if world.get::<Wall>(entity).is_some() {
        return crate::wall::remove_with_portals(world, command.element);
    }
    crate::record_step(world, Remove::of(command.element))
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
```

## Pitfalls

- Resolving the request inside `apply`: a redo then sees the World as other steps left it and places something else than the first application did.
- Recording with `drs_history::apply` instead of `apply_step`: the Command joins a gesture group that was never closed. Only a gesture's own changes go through `apply` (`crate::record`), as `edit.rs` does.
- Keeping an `Entity` of an Element across steps: another step's undo respawns it under a new handle, and the stale one points at nothing.
