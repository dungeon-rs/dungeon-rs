//! We need to share various bits and bops throughout the different UI layers, we define a singleton
//! resource that contains this code.

use crate::layout::EditorPanels;
use bevy::prelude::Resource;
use egui_dock::{DockState, NodeIndex};

/// Holds the internal state of various UI components.
///
/// Most notably, it holds the [`DockState`] used to build the general layout docks.
#[derive(Resource)]
pub struct UiState {
    /// The [`DockState`](https://docs.rs/egui_dock/latest/egui_dock/dock_state/struct.DockState.html)
    ///
    /// that controls most of the editing layout.
    pub editor_state: DockState<EditorPanels>,
}

impl Default for UiState {
    fn default() -> Self {
        let mut state = DockState::new(vec![EditorPanels::Editor]);
        let surface = state.main_surface_mut();
        let [_, _assets] = surface.split_below(NodeIndex::root(), 0.9, vec![EditorPanels::Assets]);
        let [_, layers] = surface.split_right(
            NodeIndex::root(),
            0.8,
            vec![EditorPanels::Layers, EditorPanels::Levels],
        );
        let [_, _settings] = surface.split_below(layers, 0.6, vec![EditorPanels::Settings]);

        Self {
            editor_state: state,
        }
    }
}
