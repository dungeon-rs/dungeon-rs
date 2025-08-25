//! Renders the status bar at the bottom of the screen.

use egui::{Context, TopBottomPanel, warn_if_debug_build};

/// Handles the rendering of the toolbar.
pub fn render(context: &mut Context) {
    TopBottomPanel::bottom("Status Bar").show(context, |ui| {
        warn_if_debug_build(ui);
    });
}
