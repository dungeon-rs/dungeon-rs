//! Renders the status bar at the bottom of the screen.

use data::ProjectQueryItem;
use egui::{Context, TopBottomPanel, warn_if_debug_build};
use i18n::t;

/// Handles the rendering of the toolbar.
pub fn render(context: &mut Context, project: Option<&ProjectQueryItem>) {
    TopBottomPanel::bottom("Status Bar").show(context, |ui| {
        ui.horizontal(|ui| {
            if let Some(project) = project {
                ui.label(
                    t!("widgets-status_bar-project_loaded", "project" => project.name.as_str()),
                );
            }

            warn_if_debug_build(ui);
        });
    });
}
