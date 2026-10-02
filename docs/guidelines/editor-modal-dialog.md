# Editor modal dialog

**Use when**: the Editor asks the Author something, or tells them something, that must be answered before the rest of the window is used again: a question, a prompt for a value, a dialog before a request, a report to dismiss. **Not when**: the message fits the status line (nothing to answer), or a native dialog of the platform does the asking (a file or folder picker goes through `files::choose`).
**Exemplar**: `crates/drs-editor/src/files.rs` (the unsaved-changes question)

## Rules

- The dialog's state is one `Option<Struct>` field of `EditorState`, named for what is asked (`question`, `prompt`, `export`, `report`); `Some` is the dialog being open, and `None` closes it. The field is listed in `EditorState::modal_open`, so the menu shortcuts wait while it is open. _Why_: egui redraws every frame, so what stays open must live in the World, not on the stack.
- One `pub(crate) fn <name>(ctx: &egui::Context, state: &mut EditorState, ..)` draws the dialog, opens with `let Some(x) = &mut state.<field> else { return; };`, and is called from `panels::draw` after the dock area, in one fixed order with the others. The `Modal`'s `Id` is a fixed kebab-case string naming the dialog.
- Inside `show`: `ui.set_width(420.0)` (560 for a report with a list), `ui.heading`, the text in the domain's words saying what happens next, `ui.add_space(8.0)`, then the buttons in one `ui.horizontal` whose closure returns their `clicked()` values as a tuple, read back from `modal.inner`. The buttons are acted on after `show` returns, never inside the closure. _Why_: the closure borrows the state the actions write.
- Cancel and `modal.should_close()` (Escape, or a click outside) are one branch that sets the field to `None`; the other buttons change the state's phase or send a message through `Outgoing`, and the one that sends also leaves the dialog or marks it as awaiting.
- A dialog that awaits a Manager's answer keeps a marker (`awaiting: bool`, or a `phase`) and, while it is set, draws its buttons with `add_enabled(false, ..)` and a `ui.weak("Saving…")` line, and returns before reading the buttons. The refusal is kept in `refusal: Option<String>`, drawn with `ui.colored_label(egui::Color32::LIGHT_RED, ..)`, and cleared on the next attempt. `outcomes::report` moves the marker on when the answer arrives: it closes the dialog, lets it proceed, or re-arms it with the reason.

## Example

```rust
pub(crate) fn question(
    ctx: &egui::Context,
    state: &mut EditorState,
    view: &ProjectView,
    outgoing: &mut Outgoing,
) {
    let Some(question) = &mut state.question else {
        return;
    };
    let action = match question.pending {
        Pending::Open => "opening another Project",
        Pending::Quit => "quitting",
    };
    let saving = question.phase == Phase::Saving;
    let modal = egui::Modal::new(egui::Id::new("unsaved-changes")).show(ctx, |ui| {
        ui.set_width(420.0);
        ui.heading("Unsaved changes");
        ui.label(format!(
            "The Project {} has unsaved changes. Save them before {action}?",
            view.name()
        ));
        if let Some(refusal) = &question.refusal {
            ui.colored_label(egui::Color32::LIGHT_RED, refusal);
        }
        if saving {
            ui.weak("Saving…");
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let save = ui.add_enabled(!saving, egui::Button::new("Save")).clicked();
            let discard = ui
                .add_enabled(!saving, egui::Button::new("Discard"))
                .clicked();
            let cancel = ui
                .add_enabled(!saving, egui::Button::new("Cancel"))
                .clicked();
            (save, discard, cancel)
        })
        .inner
    });
    let (save, discard, cancel) = modal.inner;
    if saving {
        return;
    }
    if cancel || modal.should_close() {
        state.question = None;
    } else if discard {
        question.phase = Phase::Proceed;
    } else if save {
        question.refusal = None;
        if self::save(view, outgoing) {
            question.phase = Phase::Saving;
        } else {
            state.question = None;
        }
    }
}
```

## Pitfalls

- Closing the dialog inside the `show` closure: the closure holds `&mut` to the state the dialog is drawn from, so the field cannot be set to `None` until it returns.
- Letting the shortcuts run while the dialog is open: a field left out of `modal_open` lets Command-S or Command-Q act behind the dialog.
- Clearing the dialog as soon as the request is sent: the answer may be a refusal the Author must see in the dialog, so the dialog stays open as awaiting until `outcomes::report` hears back.
