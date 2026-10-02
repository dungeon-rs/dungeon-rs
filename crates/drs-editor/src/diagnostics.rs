//! What the diagnostics Utility found and left: Show Logs, the bundle directory that was not
//! found, and a crash report another thread left for the main thread to announce.

use crate::state::EditorState;
use bevy::ecs::system::{NonSendMarker, Res, ResMut};
use drs_model::{BundleDirectory, Diagnostics};

/// Opens the log directory in the platform's file manager; a failure is shown in the status
/// line.
pub(crate) fn show_logs(state: &mut EditorState, diagnostics: Option<&Diagnostics>) {
    let Some(diagnostics) = diagnostics else {
        "The log directory is not known".clone_into(&mut state.status);
        return;
    };
    if let Err(error) = drs_diagnostics::reveal_logs(&diagnostics.logs) {
        state.status = format!("The logs could not be shown: {error}");
    }
}

/// Reports in the status line, at start, that no bundle directory is marked as the editor's,
/// naming every location tried.
pub(crate) fn report_bundled_files(
    mut state: ResMut<EditorState>,
    diagnostics: Option<Res<Diagnostics>>,
) {
    let Some(diagnostics) = diagnostics else {
        return;
    };
    if let BundleDirectory::Missing { tried } = &diagnostics.bundle {
        let tried = tried
            .iter()
            .map(|location| location.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        state.status = format!("The editor's Bundled Files were not found; tried {tried}");
    }
}

/// Announces, on the main thread, a crash report another thread left: in the status line and
/// in the crash dialog.
pub(crate) fn announce_crash(_main_thread: NonSendMarker, mut state: ResMut<EditorState>) {
    if let Some(report) = drs_diagnostics::take_pending_report() {
        state.status = format!(
            "The editor crashed on another thread; the report is at {}",
            report.path.display()
        );
        drs_diagnostics::announce(&report);
    }
}
