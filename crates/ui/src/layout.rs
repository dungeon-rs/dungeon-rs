//! This module defines the building blocks for building the layout of the editor.
//!
//! The current implementation builds an `egui_dock` `TabViewer` and delegates the specific layout
//! to that. We contain the different panels and how to construct them in this module.

mod editor;
mod splash;

use crate::layout::editor::EditorLayout;
pub use crate::layout::editor::EditorPanels;
use crate::state::UiState;
use crate::widgets::create_project_form::FormState;
use crate::widgets::notifications::Notifications;
use crate::widgets::{status_bar, toolbar};
use bevy::prelude::{BevyError, Commands, ResMut, debug_span};
use bevy_egui::EguiContexts;
use data::DungeonQueries;
use drs_assets::AssetLibrary;
use egui_dock::{DockArea, Style};

/// This system is responsible for rendering the splash screen, which is shown when no project is
/// loaded and the editor is waiting for something to work on.
#[utils::bevy_system]
pub fn render_splash_screen(
    mut contexts: EguiContexts,
    mut commands: Commands,
    state: Option<ResMut<FormState>>,
) -> Result<(), BevyError> {
    let _ = debug_span!("render_splash_screen").entered();
    let context = contexts.ctx_mut()?;

    splash::render(context, &mut commands, state);
    toolbar::render(context, None, commands);
    status_bar::render(context, None);
    Ok(())
}

/// This system is responsible for rendering the editor layout.
///
/// Note that this system will only run if there is a loaded project (due to `Single<ProjectQuery>`).
#[utils::bevy_system]
pub fn render_editor_layout(
    mut contexts: EguiContexts,
    commands: Commands,
    mut notifications: ResMut<Notifications>,
    mut asset_library: ResMut<AssetLibrary>,
    query: DungeonQueries,
    mut state: ResMut<UiState>,
) -> Result<(), BevyError> {
    let _ = debug_span!("render_editor_layout").entered();
    let context = contexts.ctx_mut()?;
    let project = query.projects.single()?;

    // Render any pending notifications
    notifications.ui(context);

    toolbar::render(context, Some(&project), commands);
    status_bar::render(context, Some(&project));

    // construct an `EditorLayout` using our mutable world reference for rendering.
    // the `EditorLayout` struct has a strict lifetime bound to this scope and may not leak.
    let mut viewer = EditorLayout {
        asset_library: asset_library.as_mut(),
        query: &query,
        project: &project,
    };

    // Render the `dock_state` in the `UiState` in a DockArea.
    DockArea::new(&mut state.editor_state)
        .style(Style::from_egui(context.style().as_ref()))
        .show(context, &mut viewer);

    Ok(())
}
