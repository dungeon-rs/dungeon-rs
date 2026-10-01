//! What the Editor itself keeps: the chosen Asset, the selection, the filter, the status line,
//! the prompt in progress, and the gesture under way. None of it is domain state.

use bevy::ecs::resource::Resource;
use bevy::math::Vec2;
use drs_model::{ChosenAsset, ElementId};
use std::path::PathBuf;

/// The Editor's own state.
#[derive(Resource, Default)]
pub(crate) struct EditorState {
    /// The Asset the next click places, if one is chosen.
    pub chosen: Option<Chosen>,
    /// The selected Prop, if any. Selection is never a history step.
    pub selected: Option<ElementId>,
    /// The text the Assets panel filters by.
    pub filter: String,
    /// What the status line reports.
    pub status: String,
    /// The Canonical Name prompt, while a folder is being added.
    pub prompt: Option<NamePrompt>,
    /// The pointer gesture under way in the viewport.
    pub interaction: Interaction,
}

impl EditorState {
    /// Whether a Prop is being dragged: the pointer went down on it and has moved since.
    pub fn dragging(&self) -> bool {
        matches!(
            self.interaction,
            Interaction::Pressed {
                moved_at: Some(_),
                ..
            }
        )
    }
}

/// The Asset chosen for placing, with its name for the status line.
pub(crate) struct Chosen {
    /// The Asset as a Place Element names it.
    pub asset: ChosenAsset,
    /// The Asset's name.
    pub name: String,
}

/// The Canonical Name prompt for a folder picked in the dialog.
pub(crate) struct NamePrompt {
    /// The folder picked.
    pub path: PathBuf,
    /// The name as typed, proposed from the folder's own name.
    pub name: String,
    /// Why the last attempt was refused, shown until the next.
    pub refusal: Option<String>,
    /// Whether Add Asset Folder was sent and its answer is awaited.
    pub awaiting: bool,
    /// Whether the text field should take focus on the next frame.
    pub focus: bool,
}

/// A pointer gesture in the viewport.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum Interaction {
    /// Nothing is under way.
    #[default]
    Idle,
    /// The view is being dragged.
    Panning {
        /// Where the pointer was when the view last followed it.
        last: Vec2,
    },
    /// The left button went down on a Prop; a drag moves it.
    Pressed {
        /// The Prop under the pointer.
        element: ElementId,
        /// The Prop's centre, in cells, when the button went down.
        origin: Vec2,
        /// The pointer, on screen, when the button went down.
        pointer: Vec2,
        /// The pointer, on screen, when the Prop was last moved; `None` until the drag begins.
        moved_at: Option<Vec2>,
    },
}
