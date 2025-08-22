//! Top level module for declaring the editor layout.
//! This is shown when the user is actively editing a project.

use crate::notifications::Notifications;
use ::assets::AssetLibrary;
use egui::{Ui, WidgetText};
use egui_dock::TabViewer;
use i18n::t;

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
        match tab {
            EditorPanels::Editor => t!("layout-tabs-editor"),
            EditorPanels::Assets => t!("layout-tabs-assets"),
            EditorPanels::Layers => t!("layout-tabs-layers"),
            EditorPanels::Levels => t!("layout-tabs-levels"),
            EditorPanels::Settings => t!("layout-tabs-settings"),
        }
        .into()
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
            _ => {} // EditorPanels::Assets => assets::render(self, ui),
                    // EditorPanels::Layers => layers::render(self, ui),
                    // EditorPanels::Levels => levels::render(self, ui),
                    // EditorPanels::Settings => settings::render(self, ui),
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
