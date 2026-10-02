# Edit that carries anchored Elements

**Use when**: an authoring Command changes an Element that other Elements are anchored to in a way that renumbers what their anchors name (adding or removing a Wall's point), or removes the host with them. **Not when**: the change keeps every anchor valid as it is (moving a point, bending a segment, a new thickness): those go through `SetField`, and deriving moves the anchored Elements.
**Exemplar**: `crates/drs-authoring-manager/src/wall.rs`

## Rules

- Read the host as it is before the edit, then ask `portal::anchored_to` for the Elements anchored to it: those set into it and those it has lost. Whether an Element is set is decided by `portal::sets_into` alone, never by a check of the edit's own.
- The shape Engine says where each anchor goes, from the host as it was (`anchor_portals_through` with the `PointEdit`): `Some` place to move it to, or `None` when its part of the host goes. The Manager never renumbers by hand; a lost anchor goes through the same call when the edit could make it name a real part again, so it keeps naming none.
- Each anchor that moves becomes a `SetField` at the path `"anchor"` through `moved`, which gives `None` for a place the anchor already holds, so the step records no change that is none.
- Record one history group in this order: a `Remove::of` for each Element that goes, then the host's own step, then the anchor moves (`record_together`), so undo restores the host before what is anchored to it. Chain the outcomes with `and_then` and hand the last to `crate::close_group`, which ends the group on success and on failure takes back what was already applied, so a Command is never left half done nor its group open; with nothing anchored, the host's step is a step of its own.
- The work returns the answer naming the Elements that went (`removed` gives `None` when none did) and `handle_apply` writes it; the work never writes a message.
- Where the anchored Elements now stand is not written here: deriving follows their anchors once every Manager has handled the frame, so undo, redo, and Open put them back the same way.

## Example

```rust
pub(crate) fn remove_point(
    world: &mut World,
    element: ElementId,
    index: usize,
) -> Result<Option<PortalsRemoved>, AuthoringError> {
    let before = wall_of(world, element)?;
    let points = before.points.len();
    if index >= points {
        return Err(AuthoringError::NoPoint {
            outline: "Wall",
            index,
            points,
        });
    }
    if points <= 2 {
        return remove_with_portals(world, element);
    }
    let mut wall = before.clone();
    wall.points.remove(index);
    let (portals, _) = anchored_to(world, element);
    let places = anchor_portals_through(&before, PointEdit::Removed { index }, &settings(&portals));
    let mut gone = Vec::new();
    let mut moves = Vec::new();
    for ((portal, anchor, _), place) in portals.iter().zip(places) {
        match place {
            Some(place) => moves.extend(moved(*portal, *anchor, place)?),
            None => gone.push(*portal),
        }
    }
    record_together(
        world,
        gone.clone(),
        Reshape {
            element,
            outline: wall,
            previous: None,
        },
        moves,
    )?;
    Ok(removed(element, gone))
}

pub(crate) fn record_together(
    world: &mut World,
    gone: Vec<ElementId>,
    reshape: impl ReversibleCommand,
    moves: Vec<SetField<ElementId>>,
) -> Result<(), AuthoringError> {
    if gone.is_empty() && moves.is_empty() {
        return crate::record_step(world, reshape);
    }
    crate::history(world)?.begin_group();
    let mut outcome = Ok(());
    for portal in gone {
        outcome = outcome.and_then(|()| crate::record(world, Remove::of(portal)));
    }
    outcome = outcome.and_then(|()| crate::record(world, reshape));
    for field in moves {
        outcome = outcome.and_then(|()| crate::record(world, field));
    }
    crate::close_group(world, outcome)
}

/// The answer naming the Portals set into `host` that a Command removed, when it removed any.
pub(crate) fn removed(host: ElementId, portals: Vec<ElementId>) -> Option<PortalsRemoved> {
    (!portals.is_empty()).then_some(PortalsRemoved { host, portals })
}
```

## Pitfalls

- Removing the anchored Elements after the host's step: undo then restores them before their host exists again, and deriving finds them lost for a frame.
- Asking the Engine with the host as it is after the edit: the old segment numbers no longer describe it, and anchors land on the wrong part.
- Leaving the lost anchors out of an edit that adds a part: an anchor past the old end comes to name the new part, and the Element jumps onto the host.
