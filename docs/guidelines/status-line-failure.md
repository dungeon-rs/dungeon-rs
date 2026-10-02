# Failure shown in the status line

**Use when**: the Editor learns of a failure or refusal the Author must see without a modal: a Manager's refusal or failure message, a Utility call that returns `Err`, or something the start found wanting. **Not when**: the failure answers a prompt the Author is still in (a refusal about the name goes into `prompt.refusal` and keeps the Canonical Name prompt open), or it needs a modal (the crash dialog, which the diagnostics Utility shows itself).
**Exemplar**: `crates/drs-editor/src/outcomes.rs`

## Rules

- The sentence is written to `state.status` on `EditorState` and nowhere else: no log entry, no message, no panel of its own. The status line draws it with `ui.add(egui::Label::new(&state.status).truncate());` after the right-hand hint has been laid out, so the text is never shortened by hand and the hint stays whole.
- Where it is written: a Manager's answers in `outcomes::report`, which runs first in the `Update` chain, with one `MessageReader<T>` per message gathered per Manager in a `#[derive(SystemParam)]` struct (`LibraryAnswers`, `ProjectAnswers`), one documented field each; a Utility's `Err` where the call is made (`show_logs`); what the start found in a `Startup` system (`report_bundled_files`).
- The sentence says what did not happen, in the domain's words, then the reason after a colon: `"The folder was not added: {reason}"`, `"The Asset Folder {name} could not be indexed: {reason}"`. The reason is the error's `Display`, never its `Debug`. A reason the Manager already phrased for the Author (`CommandFailed`, `HistoryFailed`) is taken as it is: `state.status.clone_from(reason)`.
- One sentence, no trailing full stop; the latest overwrites the previous, and a success is reported the same way (`"Added the Asset Folder {name}"`), so a failure stays only until the next answer.
- What the Author needs most comes first and a path last (`the report is at {}`). _Why_: truncation cuts the end.
- A refusal that does not belong to the open prompt closes it (`state.prompt = None`) before the status is written, so the Author is not left in a prompt about something that was refused.

## Example

```rust
/// The library Manager's answers.
#[derive(SystemParam)]
pub(crate) struct LibraryAnswers<'w, 's> {
    /// An Asset Folder was added.
    added: MessageReader<'w, 's, FolderAdded>,
    /// An Add Asset Folder was refused.
    refused: MessageReader<'w, 's, FolderRefused>,
    /// A remembered Asset Folder could not be indexed.
    unavailable: MessageReader<'w, 's, FolderUnavailable>,
    /// Thumbnails cannot be kept.
    thumbnails: MessageReader<'w, 's, ThumbnailsUnavailable>,
}

/// Reports every answer of the current frame.
///
/// A folder refused for its name keeps the prompt open with the reason, so the Author is asked
/// again; any other refusal closes it and is reported in the status line. A save the unsaved
/// changes question led to lets the question go ahead when it succeeds, and keeps it open with
/// the reason when it is refused.
pub(crate) fn report(
    mut state: ResMut<EditorState>,
    mut library: LibraryAnswers,
    mut failed: MessageReader<CommandFailed>,
    mut history_failed: MessageReader<HistoryFailed>,
    mut project: ProjectAnswers,
) {
    for FolderAdded { name, skips, .. } in library.added.read() {
        state.status = format!("Added the Asset Folder {name}{}", skipped(skips));
        if state.prompt.as_ref().is_some_and(|prompt| prompt.awaiting) {
            state.prompt = None;
        }
    }
    for FolderRefused { reason, .. } in library.refused.read() {
        if reason.is_about_the_name()
            && let Some(prompt) = &mut state.prompt
        {
            prompt.refusal = Some(reason.to_string());
            prompt.awaiting = false;
            prompt.focus = true;
        } else {
            state.prompt = None;
            state.status = format!("The folder was not added: {reason}");
        }
    }
    for FolderUnavailable { name, reason, .. } in library.unavailable.read() {
        state.status = format!("The Asset Folder {name} could not be indexed: {reason}");
    }
    for CommandFailed { reason, .. } in failed.read() {
        state.status.clone_from(reason);
    }
    for HistoryFailed { reason } in history_failed.read() {
        state.status.clone_from(reason);
    }
    for ThumbnailsUnavailable { reason } in library.thumbnails.read() {
        state.status = format!("Thumbnails cannot be kept on this device: {reason}");
    }
}
```

## Pitfalls

- Shortening the text by hand, with a character limit or an ellipsis: the label already truncates to the width that is left, and the hand-cut text loses its reason on a wide window.
- Leaving the prompt open on a refusal that is not about the name: the Author retypes a name for a folder that will be refused again.
- Formatting the reason with `{reason:?}`: the Author reads a Rust enum instead of a sentence.
