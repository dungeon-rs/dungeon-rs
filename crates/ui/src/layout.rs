//! This module defines the building blocks for building the layout of the editor.
//!
//! The current implementation builds an `egui_dock` `TabViewer` and delegates the specific layout
//! to that. We contain the different panels and how to construct them in this module.

mod assets;
mod layers;
mod levels;
mod settings;
mod status_bar;
mod toolbar;

use crate::dialogs::Dialogs;
use crate::notifications::Notifications;
use crate::state::UiState;
use ::assets::AssetLibrary;
use bevy::prelude::{BevyError, ResMut};
use bevy_egui::EguiContexts;
use egui::{Ui, WidgetText};
use egui_dock::{DockArea, Style, TabViewer};

/// The different panels that can be shown in the editor UI.
/// If a new panel needs to be available for the user in the UI it needs to be added here,
/// if it needs to be shown by default, make sure to add it in [`UiState::default`] as well.
#[derive(Debug)]
pub enum EditorPanels {
    /// The "main" view that shows the underlying Bevy rendered world.
    Editor,
    /// Shows the assets available in the currently selected libraries.
    Assets,
    /// Shows the layers available in the currently selected level.
    Layers,
    /// Shows the levels available in the currently selected project.
    Levels,
    /// Shows the settings related to the UI and application.
    Settings,
}

/// Contains the data structures that are available to the [`TabViewer`] when rendering the editor layout.
/// See [`EditorLayout::ui`] in particular.
pub struct EditorLayout<'a> {
    /// The notifications resource to dispatch toasts in the UI
    pub notifications: &'a mut Notifications,
    /// The asset library resource for querying and modifying assets.
    pub asset_library: &'a mut AssetLibrary,
}

impl TabViewer for EditorLayout<'_> {
    type Tab = EditorPanels;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        format!("{tab:?}").into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            EditorPanels::Editor => {
                // we don't render anything in the editor view.
                // Instead, in `clear_background` we make the background transparent so the underlying
                // Bevy render is visible.

                // Later on, we'd want to get the rectangle that this pane is shown in and then update
                // the Bevy camera to only render to this. That would prevent the camera shifting around
                // when we move the pane.
            }
            EditorPanels::Assets => assets::render(self, ui),
            EditorPanels::Layers => layers::render(self, ui),
            EditorPanels::Levels => levels::render(self, ui),
            EditorPanels::Settings => settings::render(self, ui),
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, EditorPanels::Editor)
    }
}

/// Handles rendering the [`EditorLayout`] in the `World`.
#[utils::bevy_system]
pub fn render_editor_layout(
    mut contexts: EguiContexts,
    mut notifications: ResMut<Notifications>,
    mut asset_library: ResMut<AssetLibrary>,
    mut dialogs: ResMut<Dialogs>,
    mut state: ResMut<UiState>,
) -> Result<(), BevyError> {
    let context = contexts.ctx_mut()?;

    // Render any pending notifications
    notifications.ui(context);

    // Render any dialogs that are open
    dialogs.render(context);

    toolbar::render(context, dialogs.as_mut());
    status_bar::render(context);

    // construct an `EditorLayout` using our mutable world reference for rendering.
    // the `EditorLayout` struct has a strict lifetime bound to this scope and may not leak.
    let mut viewer = EditorLayout {
        notifications: notifications.as_mut(),
        asset_library: asset_library.as_mut(),
    };

    // Render the `dock_state` in the `UiState` in a DockArea.
    DockArea::new(&mut state.dock_state)
        .style(Style::from_egui(context.style().as_ref()))
        .show(context, &mut viewer);

    Ok(())
}
