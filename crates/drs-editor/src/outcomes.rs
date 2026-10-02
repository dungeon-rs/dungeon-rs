//! What the Managers answer, shown in the status line or in the prompt or question that asked.

use crate::files;
use crate::state::{EditorState, Phase};
use bevy::ecs::message::MessageReader;
use bevy::ecs::system::{ResMut, SystemParam};
use drs_model::{
    CommandFailed, ExportRefused, FolderAdded, FolderRefused, FolderUnavailable, HistoryFailed,
    LevelExported, PortalsRemoved, ProjectOpened, ProjectRefused, ProjectRequest, ProjectSaved,
    ScanSkips, ThumbnailsUnavailable,
};

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

/// The project Manager's answers.
#[derive(SystemParam)]
pub(crate) struct ProjectAnswers<'w, 's> {
    /// The Project was saved.
    saved: MessageReader<'w, 's, ProjectSaved>,
    /// A Project was opened.
    opened: MessageReader<'w, 's, ProjectOpened>,
    /// A Save or Open was refused.
    refused: MessageReader<'w, 's, ProjectRefused>,
    /// An Export was written.
    exported: MessageReader<'w, 's, LevelExported>,
    /// An Export was refused or failed.
    export_refused: MessageReader<'w, 's, ExportRefused>,
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
    mut portals_removed: MessageReader<PortalsRemoved>,
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
    for PortalsRemoved { portals, .. } in portals_removed.read() {
        let stood = if portals.len() == 1 { "it" } else { "they" };
        state.status = format!(
            "Removed {} with the part of the Wall or Room {stood} stood in",
            counted(portals.len(), "Portal", "Portals")
        );
    }
    for ThumbnailsUnavailable { reason } in library.thumbnails.read() {
        state.status = format!("Thumbnails cannot be kept on this device: {reason}");
    }
    for ProjectSaved { path } in project.saved.read() {
        state.status = format!("Saved to {}", path.display());
        if let Some(question) = &mut state.question
            && question.phase == Phase::Saving
        {
            question.phase = Phase::Proceed;
        }
    }
    for ProjectOpened { path, report } in project.opened.read() {
        state.status = format!("Opened {}{}", path.display(), files::summarise(report));
        state.selected = None;
        state.question = None;
        let shown = !report.missing_assets.is_empty() || !report.unknown_kinds.is_empty();
        state.report = shown.then(|| report.clone());
    }
    for ProjectRefused { request, reason } in project.refused.read() {
        match request {
            ProjectRequest::Save { .. } => {
                state.status = format!("The Project was not saved: {reason}");
                if let Some(question) = &mut state.question
                    && question.phase == Phase::Saving
                {
                    question.refusal = Some(reason.clone());
                    question.phase = Phase::Asking;
                }
            }
            ProjectRequest::Open { path } => {
                state.status = format!("{} was not opened: {reason}", path.display());
            }
        }
    }
    for LevelExported {
        path,
        width,
        height,
        ..
    } in project.exported.read()
    {
        state.status = format!("Exported {width}×{height} px to {}", path.display());
        state.exporting = false;
    }
    for ExportRefused { reason, .. } in project.export_refused.read() {
        state.status = format!("The Level was not exported: {reason}");
        state.exporting = false;
    }
}

/// A count in words: `1 Element`, `2 Elements`.
pub(crate) fn counted(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
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
        .map(|(count, one, many)| counted(count, one, many))
        .collect();
    if parts.is_empty() {
        String::new()
    } else {
        format!(" (skipped {})", parts.join(", "))
    }
}
