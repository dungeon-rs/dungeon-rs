//! We need to share various bits and bops throughout the different UI layers, we define a singleton
//! resource that contains this code.
use crate::layout::EditorPanels;
use bevy::prelude::Resource;
use egui_dock::DockState;

/// Holds the internal state of various UI components.
///
/// Most notably, it holds the [`DockState`] used to build the general layout docks.
#[derive(Resource)]
pub struct UiState {
    /// The [`DockState`](https://docs.rs/egui_dock/latest/egui_dock/dock_state/struct.DockState.html)
    /// that controls most of the general layout.
    pub dock_state: DockState<EditorPanels>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            dock_state: DockState::new(vec![EditorPanels::Editor]),
        }
    }
}
