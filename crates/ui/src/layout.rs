//! This module defines the building blocks for building the layout of the editor.
//!
//! The current implementation builds an `egui_dock` `TabViewer` and delegates the specific layout
//! to that. We contain the different panels and how to construct them in this module.

mod editor;

use crate::layout::editor::EditorLayout;
pub use crate::layout::editor::EditorPanels;
use crate::state::UiState;
use crate::widgets::notifications::Notifications;
use crate::widgets::{status_bar, toolbar};
use ::assets::AssetLibrary;
use bevy::prelude::{BevyError, ResMut, debug_span};
use bevy_egui::EguiContexts;
use egui_dock::{DockArea, Style};

/// This system is responsible for rendering all UI elements.
///
/// Depending on the current UI state, it will render splash screens, editors, loading, and so forth.
#[utils::bevy_system]
pub fn render_editor_layout(
    mut contexts: EguiContexts,
    mut notifications: ResMut<Notifications>,
    mut asset_library: ResMut<AssetLibrary>,
    mut state: ResMut<UiState>,
) -> Result<(), BevyError> {
    let _ = debug_span!("render_editor_layout").entered();
    let context = contexts.ctx_mut()?;

    // Render any pending notifications
    notifications.ui(context);

    toolbar::render(context);
    status_bar::render(context);

    // construct an `EditorLayout` using our mutable world reference for rendering.
    // the `EditorLayout` struct has a strict lifetime bound to this scope and may not leak.
    let mut viewer = EditorLayout {
        asset_library: asset_library.as_mut(),
    };

    // Render the `dock_state` in the `UiState` in a DockArea.
    DockArea::new(&mut state.editor_state)
        .style(Style::from_egui(context.style().as_ref()))
        .show(context, &mut viewer);

    Ok(())
}
