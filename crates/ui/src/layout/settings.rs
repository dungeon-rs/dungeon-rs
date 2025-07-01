//! Renders the settings tab in the editor layout.

use crate::layout::EditorLayout;
use egui::{global_theme_preference_buttons, Ui};

/// Handles the rendering of the [`crate::layout::EditorPanels::Settings`] tab.
pub(super) fn render(_viewer: &mut EditorLayout, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("Theme:");
        global_theme_preference_buttons(ui);
    });
}
