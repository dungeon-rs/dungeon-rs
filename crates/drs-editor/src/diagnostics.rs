//! What the diagnostics Utility found and left: Show Logs, the bundle directory that was not
//! found, and a crash report another thread left for the main thread to announce.

use crate::state::EditorState;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{NonSendMarker, Res, ResMut};
use drs_diagnostics::Started;

/// What the diagnostics Utility set up and found at start, as the Host handed it to the plugin.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub(crate) struct Diagnostics(pub(crate) Started);

/// Opens the log directory in the platform's file manager; a failure is shown in the status
/// line.
pub(crate) fn show_logs(state: &mut EditorState, diagnostics: &Diagnostics) {
    if let Err(error) = drs_diagnostics::reveal_logs(&diagnostics.0.logs) {
        state.status = format!("The logs could not be shown: {error}");
    }
}

/// Reports in the status line, at start, that no bundle directory is marked as the editor's,
/// naming every location tried.
pub(crate) fn report_bundled_files(mut state: ResMut<EditorState>, diagnostics: Res<Diagnostics>) {
    if let Err(missing) = &diagnostics.0.bundled_files {
        state.status = format!("The editor's Bundled Files were not found: {missing}");
    }
}

/// Announces, on the main thread, a crash report another thread left: in the status line and
/// in the crash dialog.
pub(crate) fn announce_crash(_main_thread: NonSendMarker, mut state: ResMut<EditorState>) {
    if let Some(report) = drs_diagnostics::announce_pending() {
        state.status = format!(
            "The editor crashed on another thread; the report is at {}",
            report.path.display()
        );
    }
}
