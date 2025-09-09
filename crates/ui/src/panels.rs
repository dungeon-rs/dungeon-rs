//! Contains panels that can be reused in docking layouts.

use bevy::prelude::Visibility;
use drs_assets::AssetLibrary;
use drs_data::{DungeonQueries, LevelQueryItem, ProjectQueryItem};
use egui::Ui;

/// Handles rendering the asset panel in a docked layout.
pub fn asset_library(_ui: &mut Ui, _asset_library: &mut AssetLibrary) {}

/// Handles rendering the asset panel in a docked layout.
pub fn asset_browser(_ui: &mut Ui, _asset_library: &mut AssetLibrary) {}

/// Handles rendering the layers panel in a docked layout.
pub fn layers(ui: &mut Ui, query: &DungeonQueries, level: &LevelQueryItem) {
    for layer in query.layers_for_level(level.entity) {
        ui.label(layer.name.as_str());
    }
}

/// Handles rendering the levels panel in a docked layout.
pub fn levels(ui: &mut Ui, query: &DungeonQueries, project: &ProjectQueryItem) {
    for level in query.levels_for_project(project.entity) {
        ui.horizontal(|ui| {
            let mut is_visible = match level.visibility {
                Visibility::Inherited | Visibility::Hidden => false,
                Visibility::Visible => true,
            };
            ui.checkbox(&mut is_visible, level.name.as_str());
        });
    }
}

/// Handles rendering the settings panel in a docked layout.
pub fn settings(_ui: &mut Ui) {}
