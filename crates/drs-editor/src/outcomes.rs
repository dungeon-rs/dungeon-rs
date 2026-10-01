//! What the Managers answer, shown in the status line or in the prompt that asked.

use crate::state::EditorState;
use bevy::ecs::message::MessageReader;
use bevy::ecs::system::ResMut;
use drs_model::{
    CommandFailed, FolderAdded, FolderRefused, FolderUnavailable, HistoryFailed, ScanSkips,
};

/// Reports every answer of the current frame.
///
/// A folder refused for its name keeps the prompt open with the reason, so the Author is asked
/// again; any other refusal closes it and is reported in the status line.
pub(crate) fn report(
    mut state: ResMut<EditorState>,
    mut added: MessageReader<FolderAdded>,
    mut refused: MessageReader<FolderRefused>,
    mut unavailable: MessageReader<FolderUnavailable>,
    mut failed: MessageReader<CommandFailed>,
    mut history_failed: MessageReader<HistoryFailed>,
) {
    for FolderAdded { name, skips, .. } in added.read() {
        state.status = format!("Added the Asset Folder {name}{}", skipped(skips));
        if state.prompt.as_ref().is_some_and(|prompt| prompt.awaiting) {
            state.prompt = None;
        }
    }
    for FolderRefused { reason, .. } in refused.read() {
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
    for FolderUnavailable { name, reason, .. } in unavailable.read() {
        state.status = format!("The Asset Folder {name} could not be indexed: {reason}");
    }
    for CommandFailed { reason, .. } in failed.read() {
        state.status.clone_from(reason);
    }
    for HistoryFailed { reason } in history_failed.read() {
        state.status.clone_from(reason);
    }
}

/// What indexing left out, as a clause for the status line; empty when nothing was skipped.
fn skipped(skips: &ScanSkips) -> String {
    let counts = [
        (
            skips.unlisted_folders,
            "folder that could not be listed",
            "folders that could not be listed",
        ),
        (
            skips.unreadable_entries,
            "entry that could not be read",
            "entries that could not be read",
        ),
        (
            skips.non_unicode_names,
            "name that is not valid Unicode",
            "names that are not valid Unicode",
        ),
    ];
    let parts: Vec<String> = counts
        .into_iter()
        .filter(|(count, _, _)| *count > 0)
        .map(|(count, one, many)| format!("{count} {}", if count == 1 { one } else { many }))
        .collect();
    if parts.is_empty() {
        String::new()
    } else {
        format!(" (skipped {})", parts.join(", "))
    }
}
