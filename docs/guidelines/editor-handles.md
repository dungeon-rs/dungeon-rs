# Editor handles

**Use when**: the Editor lets the Author reshape a selected Element by dragging parts of it shown over the Level (a Wall's or a Room's points, control points, and middles). **Not when**: the Element is only moved, turned, or resized as a whole (a Prop or a freestanding Portal, through `Interaction::Pressed`), or the part slides along another Element (a set Portal, through `Interaction::Sliding`).
**Exemplar**: `crates/drs-editor/src/handles.rs`

## Rules

- A kind with handles is read through one view of what its handles need, `Outline` (its points, one control point or none per segment or edge, whether it closes, its thickness), built by `of_wall` or `of_room`; every handle function takes the view, never the kind's component, so a second kind gains the handles by gaining a constructor. `LevelView::selected_outline` hands it out for the selection.
- `handles` lists them in the order they are hit (points, then control points, then the middles of straight parts, a closed outline's closing edge included through `Outline::ends`), and `handle_at` takes the first within `HANDLE_PIXELS` of the pointer, so a point always wins over a middle drawn under it. `draw` draws the same list, so what is drawn is what is hit.
- With an Element selected, `press_selected` runs before any picking: the selection's handles are hit before any Element above it. A double-click on its line away from a point or control point sends `AddPoint` at the nearest chord's place, clamped `NEAREST_TO_AN_END` inside the part, so it is never refused for lying at an end.
- A press on a handle arms `Interaction::Handle` with where it was and where the pointer was; the drag sends `handle_change` as `Gesture::Begin`, `Continue`, and `End`, so a drag is one step. A middle is only for dragging: pressing it selects no handle, so Delete still removes the Element, and once dragged it is the part's control point.
- The selected handle lives in `EditorState::handle` with the identity of its owner, read through `handle_of`. Delete sends `delete_handle` for it (remove the point, straighten the part) and removes the Element only when no handle is selected; `outline_selection` lets go of a handle whose owner is no longer selected or whose part `handle_exists` no longer finds, so an undo never leaves a handle naming nothing.
- Undo and redo wait while a handle is dragged (`EditorState::dragging`), as for any gesture.

## Example

```rust
/// A Wall or a Room as its handles and its line are seen: its points, one control point or none
/// per segment or edge, whether its line closes from the last point back to the first, and how
/// thick it is drawn.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Outline<'a> {
    /// The points, in order.
    pub points: &'a [Vec2],
    /// The control point of each segment or edge, or `None` for a straight one.
    pub controls: Vec<Option<Vec2>>,
    /// Whether the last edge runs from the last point back to the first, as a Room's does.
    pub closed: bool,
    /// How thick it is drawn, in cells.
    pub thickness: f32,
}

/// The first handle of a Wall or a Room within a handle's reach of a point in cells.
fn handle_at(outline: &Outline, cells: Vec2, zoom: f32) -> Option<(OutlineHandle, Vec2)> {
    handles(outline)
        .into_iter()
        .find(|(_, at)| at.distance(cells) * zoom <= HANDLE_PIXELS)
}

    let Some((handle, origin)) = handle else {
        return false;
    };
    // A straight part's middle is only for dragging: a click on it leaves the Wall or the Room
    // selected with no handle, so Delete removes it rather than doing nothing.
    state.handle = match handle {
        OutlineHandle::Point(_) | OutlineHandle::Control(_) => Some((element, handle)),
        OutlineHandle::Middle(_) => None,
    };
    state.interaction = crate::state::Interaction::Handle {
        element,
        handle,
        origin,
        pointer: cursor,
        moved_at: None,
    };
    true
}

/// The change that puts a handle at a position: a point moves, and a control point or a
/// straight part's middle becomes the part's control point.
pub(crate) fn handle_change(handle: OutlineHandle, position: Vec2) -> ElementChange {
    match handle {
        OutlineHandle::Point(index) => ElementChange::Point { index, position },
        OutlineHandle::Control(segment) | OutlineHandle::Middle(segment) => {
            ElementChange::Control {
                segment,
                position: Some(position),
            }
        }
    }
}
```

## Pitfalls

- Writing a kind's own hit list beside `handles`: the overlay draws one set of handles and the press hits another.
- Selecting a middle on press: Delete then has a handle to act on and does nothing, instead of removing the Element.
- Keeping a selected handle across an undo without `handle_exists`: Delete sends a change for a point the Element no longer has, and the Author sees a refusal for a handle that is not drawn.
