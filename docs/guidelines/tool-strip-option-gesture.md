# Tool-strip option as a gesture

**Use when**: the tool strip shows a property of the selected Element in a widget the Author can hold while it changes (a `DragValue` dragged, a colour picker open), so a whole drag must be one history step. **Not when**: the widget changes the property once per click (the Flip button sends a `Gesture::Single` Edit Element, Free Portal a Command of its own), or the property belongs to the next Element drawn rather than to a placed one (it is written into the tool's own state and sends nothing).
**Exemplar**: `crates/drs-editor/src/walls.rs`

## Rules

- The tool's own state keeps an `option: Option<OptionGesture>` (the Element and the change last sent) and an `option_in_progress()` over it, which `EditorState::step_under_way` includes, so undo, redo, the Edit menu, `X`, and `F` wait while the widget is held.
- Every change goes through `walls::send_option` with `held` saying whether the widget is still held (`drag.dragged()`, the picker's popup open): held, it begins the gesture on the first change and continues it after; let go, it is a step of its own. Nothing builds an `EditElement` for an option by hand.
- A frame with no change and the widget no longer held calls `walls::end_option`, which sends the last change again as `Gesture::End`. So does every path on which the strip stops showing the Element's options (the selection gone or changed to another kind, as `portals::end_options` does), so no Begin is left without its End.
- A `DragValue` is built with `.update_while_editing(false)` and, when it has a range, `.clamp_existing_to_range(false)`, so showing a value never sends a change nobody made. A floor for dragging is applied in code after `dragged()` (`THINNEST_DRAGGED`, `NARROWEST_DRAGGED`); a typed value is sent as typed, and the Manager refuses one the kind's check rejects with its reason.
- Values are shown and sent in the model's units, converted only at the widget (a rotation shown in degrees is sent in radians).

## Example

```rust
/// Sends an option's change to the Element it shows: a change made while the widget is held (a
/// drag of the thickness, the colour picker open) is part of a gesture that ends when it is let
/// go, so it is one step; any other change is a step of its own.
pub(crate) fn send_option(
    option: &mut Option<OptionGesture>,
    apply: &mut MessageWriter<Apply>,
    element: ElementId,
    change: ElementChange,
    held: bool,
) {
    let gesture = match (held, option.as_ref()) {
        (false, _) => Gesture::Single,
        (true, Some(open)) if open.element == element => Gesture::Continue,
        (true, _) => Gesture::Begin,
    };
    apply.write(Apply::EditElement(EditElement {
        element,
        change: change.clone(),
        gesture,
    }));
    *option = held.then_some(OptionGesture { element, change });
}

/// Ends an option's gesture once its widget is let go, by sending its last change again as the
/// gesture's end.
pub(crate) fn end_option(option: &mut Option<OptionGesture>, apply: &mut MessageWriter<Apply>) {
    if let Some(OptionGesture { element, change }) = option.take() {
        apply.write(Apply::EditElement(EditElement {
            element,
            change,
            gesture: Gesture::End,
        }));
    }
}

    let drag = ui.add(
        egui::DragValue::new(&mut thickness)
            .range(f32::NEG_INFINITY..=THICKEST)
            .clamp_existing_to_range(false)
            .speed(0.005)
            .max_decimals(3)
            .suffix(" cells")
            .update_while_editing(false),
    );
    if drag.dragged() {
        thickness = thickness.max(THINNEST_DRAGGED);
    }
    if drag.changed() {
        send_option(
            &mut state.walls.option,
            apply,
            *element,
            ElementChange::Thickness(thickness),
            drag.dragged(),
        );
    } else if picked != wall.colour {
        send_option(
            &mut state.walls.option,
            apply,
            *element,
            ElementChange::Colour(picked),
            picking,
        );
    } else if !drag.dragged() && !picking {
        end_option(&mut state.walls.option, apply);
    }
```

## Pitfalls

- Sending every frame's value whether or not it changed: each frame becomes a step, or a group that never ends.
- Clamping what the widget shows into its range: selecting an Element outside the range sends a change and records a step the Author never made.
- Forgetting `end_option` when the selection changes mid-drag: the option stays in progress, so undo and redo wait for a gesture that never ends, and its history group stays open until another step closes it.
