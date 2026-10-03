# Editor drag gesture

**Use when**: a press in the viewport may become a drag that changes the Level as it goes, sending Edit Elements as one gesture (an Element moved, a handle dragged, a whole Wall or Room moved by amounts, a set Portal slid, a stroke or one of its points moved). **Not when**: the drag changes only the view (`Interaction::Panning`) or sends nothing until it is released (a Room's rectangle, a stroke being painted); a widget held in the tool strip follows the tool-strip-option guideline.
**Exemplar**: `crates/drs-editor/src/gesture.rs`

## Rules

- The press arms an `Interaction` variant, or the Paint tool's `StrokeDrag`, carrying `drag: Drag::new(cursor)` beside what the drag starts from (an `origin` in cells, the `from` of a whole drag); it never keeps a pointer, a moved-at, or a threshold of its own, and `EditorState::pressing` names the variant, as `PaintTool::step_in_progress` does a stroke's drag, so undo and redo wait from the press.
- While the button is held, a `drag_<what>` function computes the one value the step would send, a position or a total travel in cells, through `Shown` when it snaps, and calls `drag.step(cursor, value)`: `None` sends nothing, and `Begin` or `Continue` is the `gesture` of the Edit Element it sends; it then writes the variant back with the stepped `Drag`. Stepping on the value rather than the cursor lets a step follow a change in what is shown with the pointer still.
- An Edit Element that carries a difference (`ElementChange::MoveBy`) still steps on the total: it reads `drag.sent()` before the step and sends the total less that, so the amounts sum exactly to what was shown.
- On release `finish_gesture` (or a kind's own finish that knows more, as `finish_slide` knows the Portal's Wall and `paint::release` the stroke) sends `drag.sent()` again as `Gesture::End`, or a move by nothing for a difference, so the drag is one history step ending where it was last shown; a press that never became a drag just ends. The release is honoured over a panel and outside the window (`pointer_gone`), so no Begin is left without its End.
- A drag that snaps keeps the Pointer snapping for as long as it is under way, over a panel or out of the window: `write_pointer` treats every interaction but `Idle` as a gesture, so the snapped point a step sends never falls back to the bare pointer.

## Example

```rust
/// How far the pointer travels, in pixels, before a press on an Element, a handle, or a stroke
/// becomes a drag.
pub(crate) const DRAG_THRESHOLD: f32 = 3.0;

/// A press that may become a drag: where the pointer went down, and what the drag last sent, a
/// position or an amount in cells.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Drag {
    /// The pointer, on screen, when the button went down.
    pressed_at: Vec2,
    /// What the last step sent; `None` until the drag begins.
    sent: Option<Vec2>,
}

impl Drag {
    /// A press with the pointer at `cursor`, not yet a drag.
    pub(crate) fn new(cursor: Vec2) -> Self {
        Self {
            pressed_at: cursor,
            sent: None,
        }
    }

    /// The step that sends `next` with the pointer at `cursor`, if there is one, recorded as
    /// sent: the gesture begins once the pointer has travelled farther than the threshold from
    /// the press, and continues whenever `next` differs from what was last sent, whether the
    /// pointer moved or only what it is shown at.
    pub(crate) fn step(&mut self, cursor: Vec2, next: Vec2) -> Option<Gesture> {
        let gesture = match self.sent {
            None if cursor.distance(self.pressed_at) > DRAG_THRESHOLD => Gesture::Begin,
            Some(sent) if sent != next => Gesture::Continue,
            None | Some(_) => return None,
        };
        self.sent = Some(next);
        Some(gesture)
    }
}
```

## Pitfalls

- Stepping when the cursor moves rather than when the value changes: the value a snapping drag sends answers the Pointer of the frame before, so a pointer that stops on the frame it crosses into a new corner leaves the Element on the old one.
- Working the End out again from the cursor and the snapped point of the release frame: the drag then ends somewhere the Author never saw it.
- Sending a Wall's or a Room's new box centre as a position: the difference of two box centres is not the travel the Author made, so points on Grid corners drift off them.
