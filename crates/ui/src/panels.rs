//! Contains panels that can be reused in docking layouts.

use assets::AssetLibrary;
use egui::Ui;

/// Handles rendering the asset panel in a docked layout.
pub fn asset_library(_ui: &mut Ui, _asset_library: &mut AssetLibrary) {}

/// Handles rendering the asset panel in a docked layout.
pub fn asset_browser(_ui: &mut Ui, _asset_library: &mut AssetLibrary) {}

/// Handles rendering the layers panel in a docked layout.
pub fn layers(_ui: &mut Ui) {}

/// Handles rendering the levels panel in a docked layout.
pub fn levels(_ui: &mut Ui) {}

/// Handles rendering the settings panel in a docked layout.
pub fn settings(_ui: &mut Ui) {}
