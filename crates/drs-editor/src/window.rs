//! The window itself: its title, which names the Project and marks unsaved changes, and its
//! close button, which asks about unsaved changes instead of closing at once.

use crate::state::{EditorState, Pending, Phase, Question};
use bevy::app::AppExit;
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::ecs::query::With;
use bevy::ecs::system::{Res, ResMut, Single};
use bevy::window::{PrimaryWindow, Window, WindowCloseRequested};
use drs_history::History;
use drs_model::{SavedMark, project_name_of};

/// The editor's name, after the Project's in the title.
const EDITOR: &str = "DungeonRS";
/// What precedes the Project's name while it has unsaved changes.
const UNSAVED: &str = "• ";

/// Keeps the title as `<Project> — DungeonRS`, the Project named by its file or `Untitled`,
/// with the unsaved marker in front while the history stands elsewhere than at the save.
pub(crate) fn title(
    history: Res<History>,
    mark: Res<SavedMark>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    let name = mark
        .file
        .as_deref()
        .map_or_else(|| "Untitled".to_owned(), project_name_of);
    let marker = if history.position() == mark.position {
        ""
    } else {
        UNSAVED
    };
    let title = format!("{marker}{name} — {EDITOR}");
    if window.title != title {
        window.title = title;
    }
}

/// Answers the window's close button: quits at once without unsaved changes, and asks whether
/// to save them otherwise. The Host leaves the window open on the request so this can run.
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
    if history.position() == mark.position {
        exit.write(AppExit::Success);
    } else if state.question.is_none() {
        state.question = Some(Question {
            pending: Pending::Quit,
            phase: Phase::Asking,
            refusal: None,
        });
    }
}
