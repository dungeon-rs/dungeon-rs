//! The window itself: its title, which names the Project and marks unsaved changes, and its
//! close button, which asks about unsaved changes instead of closing at once.

use crate::state::{EditorState, Pending, Question};
use bevy::app::AppExit;
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::ecs::query::With;
use bevy::ecs::system::{Res, ResMut, Single};
use bevy::window::{PrimaryWindow, Window, WindowCloseRequested};
use drs_history::History;
use drs_model::SavedMark;

/// The editor's name: the title until a Project names it, and after the Project's name then.
pub(crate) const EDITOR: &str = "DungeonRS";
/// What precedes the Project's name while it has unsaved changes.
const UNSAVED: &str = "• ";

/// Keeps the title as `<Project> — DungeonRS`, the Project named by its file or `Untitled`,
/// with the unsaved marker in front while the history stands elsewhere than at the save.
pub(crate) fn title(
    history: Res<History>,
    mark: Res<SavedMark>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    let name = mark.name();
    let marker = if mark.unsaved(&history) { UNSAVED } else { "" };
    let title = format!("{marker}{name} — {EDITOR}");
    if window.title != title {
        window.title = title;
    }
}

/// Answers the window's close button: quits at once without unsaved changes, and asks whether
/// to save them otherwise; a question already open, asked before opening another Project, now
/// stands in front of quitting instead. The Host leaves the window open on the request so this
/// can run.
pub(crate) fn close_requested(
    mut requests: MessageReader<WindowCloseRequested>,
    history: Res<History>,
    mark: Res<SavedMark>,
    mut state: ResMut<EditorState>,
    mut exit: MessageWriter<AppExit>,
) {
    if requests.read().next().is_none() {
        return;
    }
    if !mark.unsaved(&history) {
        exit.write(AppExit::Success);
    } else if let Some(question) = &mut state.question {
        question.pending = Pending::Quit;
    } else {
        state.question = Some(Question::asking(Pending::Quit));
    }
}
