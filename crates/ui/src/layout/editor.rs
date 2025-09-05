//! Top level module for declaring the editor layout.
//! This is shown when the user is actively editing a project.

use crate::panels;
use ::assets::AssetLibrary;
use data::{DungeonQueries, ProjectQueryItem};
use egui::{RichText, Ui, WidgetText};
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
    AssetLibrary,

    /// Shows the assets available in the currently selected libraries.
    AssetBrowser,

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
    /// The asset library resource for querying and modifying assets.
    pub asset_library: &'a mut AssetLibrary,

    /// Provides access to the level / layer hierarchy within the UI.
    pub query: &'a DungeonQueries<'a, 'a>,

    /// The currently active project.
    pub project: &'a ProjectQueryItem<'a>,
}

impl TabViewer for EditorLayout<'_> {
    type Tab = EditorPanels;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            EditorPanels::Editor => t!("layout-tabs-editor"),
            EditorPanels::AssetLibrary => t!("layout-tabs-assetlibrary"),
            EditorPanels::AssetBrowser => t!("layout-tabs-assetbrowser"),
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
            EditorPanels::AssetLibrary => panels::asset_library(ui, self.asset_library),
            EditorPanels::AssetBrowser => panels::asset_browser(ui, self.asset_library),
            EditorPanels::Layers => {
                let level = self
                    .query
                    .levels_for_project(self.project.entity)
                    .find(data::LevelQueryItem::is_visible);

                if let Some(level) = level {
                    panels::layers(ui, self.query, &level);
                } else {
                    ui.label(
                        RichText::new(t!("layout-tabs-levels.no-visible"))
                            .color(ui.visuals().warn_fg_color),
                    );
                }
            }
            EditorPanels::Levels => panels::levels(ui, self.query, self.project),
            EditorPanels::Settings => panels::settings(ui),
        }
    }

    fn is_closeable(&self, _tab: &Self::Tab) -> bool {
        false
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        !matches!(tab, EditorPanels::Editor)
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, EditorPanels::Editor)
    }
}
