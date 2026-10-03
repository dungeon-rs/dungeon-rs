# Editor handles

**Use when**: the Editor lets the Author reshape a selected Element, or a selected part of one, by dragging parts of it shown over the Level (a Wall's or a Room's points, control points, and middles; the points of a Terrain's selected stroke). **Not when**: the Element is only moved, turned, or resized as a whole (a Prop or a freestanding Portal, through `Interaction::Pressed`), or the part slides along another Element (a set Portal, through `Interaction::Sliding`).
**Exemplar**: `crates/drs-editor/src/handles.rs`

## Rules

- What has handles is read through one view of what its handles need, `Outline` (its points, one control point or none per segment or edge, whether it closes, its thickness), built by `of_wall`, `of_room`, or `of_stroke`, a stroke's path being open with no control points, so its handles are its points alone; every handle function takes the view, never the kind's component, so another kind gains the handles by gaining a constructor. `LevelView::selected_outline` hands it out for the selected Wall or Room, the Paint tool's `selected_in` for the selected stroke.
- `handles` lists them in the order they are hit (points, then control points, then the middles of straight parts, a closed outline's closing edge included through `Outline::ends`), and `handle_at` takes, of the first kind with one within `HANDLE_PIXELS` of the pointer, the nearest, so a point always wins over a middle drawn under it and the nearest of a stroke's close-set points wins. What is drawn is that same list: `draw` draws a Wall's or a Room's with gizmos, the Paint tool's overlay a stroke's with egui, the handle selected or dragged in `PICKED` (through `on_egui` for egui).
- With a Wall or a Room selected, `press_selected` runs before any picking: the selection's handles are hit before any Element above it. A double-click on its line away from a point or control point sends `AddPoint` at the nearest chord's place, clamped `NEAREST_TO_AN_END` inside the part, so it is never refused for lying at an end. With the Paint tool editing strokes, its press tests the selected stroke's handles before picking a stroke.
- A press on a handle arms a drag with where it was and where the pointer was (`Interaction::Handle`, or the Paint tool's `StrokeDrag`). Every drag, of an Element, a handle, a slide, or a stroke, asks `drag_gesture` whether the pointer now begins it, past `DRAG_THRESHOLD`, or continues it, sends the change at the pointer, and on release sends the change at the pointer's last position as `Gesture::End`, so a drag is one step. A middle is only for dragging: pressing it selects no handle, so Delete still removes the Element, and once dragged it is the part's control point.
- The selected handle of a Wall or a Room lives in `EditorState::handle` with the identity of its owner, read through `handle_of`. Delete sends `delete_handle` for it (remove the point, straighten the part) and removes the Element only when no handle is selected; `outline_selection` lets go of a handle whose owner is no longer selected or whose part `handle_exists` no longer finds, so an undo never leaves a handle naming nothing. A stroke has no selected handle, only the one being dragged.
- Undo and redo wait from the press to the release (`EditorState::pressing`, and `PaintTool::step_in_progress` for a stroke), not only once the pointer has moved: the press names what it drags by number, and an undo in between could renumber it.

## Example

```rust
/// A Wall, a Room, or a stroke as its handles and its line are seen: its points, one control point
/// or none per segment or edge, whether its line closes from the last point back to the first, and
/// how thick it is drawn.
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

    /// A stroke's path: open, with no control points, so its only handles are its points.
    pub(crate) fn of_stroke(stroke: &'a Stroke) -> Self {
        Self {
            points: &stroke.points,
            controls: Vec::new(),
            closed: false,
            thickness: stroke.brush.size,
        }
    }

/// The handle of a Wall, a Room, or a stroke within a handle's reach of a point in cells at
/// `zoom`: of the first kind in the order [`handles`] lists them, points before control points
/// before middles, the nearest of that kind.
pub(crate) fn handle_at(
    outline: &Outline,
    cells: Vec2,
    zoom: f32,
) -> Option<(OutlineHandle, Vec2)> {
    let kind = |handle: OutlineHandle| match handle {
        OutlineHandle::Point(_) => 0,
        OutlineHandle::Control(_) => 1,
        OutlineHandle::Middle(_) => 2,
    };
    handles(outline)
        .into_iter()
        .filter(|(_, at)| at.distance(cells) * zoom <= HANDLE_PIXELS)
        .min_by(|(a, at_a), (b, at_b)| {
            kind(*a)
                .cmp(&kind(*b))
                .then(at_a.distance(cells).total_cmp(&at_b.distance(cells)))
        })
}

pub(crate) fn drag_gesture(pointer: Vec2, moved_at: Option<Vec2>, cursor: Vec2) -> Option<Gesture> {
    match moved_at {
        Some(last) => (last != cursor).then_some(Gesture::Continue),
        None => ((cursor - pointer).length() > DRAG_THRESHOLD).then_some(Gesture::Begin),
    }
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

- Writing a kind's own hit list beside `handles`, or drawing its handles from its points directly: the overlay draws one set of handles and the press hits another.
- Taking the first point within reach of a stroke's path rather than the nearest: a stroke's points lie an eighth of its Brush apart, so at a low zoom several are within reach and the press grabs the wrong one.
- Selecting a middle on press: Delete then has a handle to act on and does nothing, instead of removing the Element.
- Keeping a selected handle across an undo without `handle_exists`: Delete sends a change for a point the Element no longer has, and the Author sees a refusal for a handle that is not drawn.
- Counting only a moved press as a step under way: an undo between the press and the drag renumbers the stroke or the point, and the drag begins on another one.
