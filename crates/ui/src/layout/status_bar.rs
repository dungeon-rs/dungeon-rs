//! Renders the status bar at the bottom of the screen.

use egui::{Context, TopBottomPanel, warn_if_debug_build};

/// Handles the rendering of the toolbar.
pub(super) fn render(context: &mut Context) {
    TopBottomPanel::bottom("Status Bar").show(context, |ui| {
        ui.horizontal(|ui| {
            warn_if_debug_build(ui);
            ui.add_space(ui.available_width() - (ui.available_height() + 2.));
            ui.spinner();
        });
    });
}
