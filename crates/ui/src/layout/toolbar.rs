//! Renders the toolbar at the top of the screen.

use crate::dialogs::Dialogs;
use egui::{Align, Context, Layout, TopBottomPanel};

/// Handles the rendering of the toolbar.
pub(super) fn render(context: &mut Context, dialogs: &mut Dialogs) {
    TopBottomPanel::top("Toolbar").show(context, |ui| {
        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
            ui.style_mut().visuals.button_frame = false;

            if ui.button("New").clicked() {
                dialogs.add(crate::dialogs::NewProject::default());
            }

            let _ = ui.button("Open");
            ui.add_enabled_ui(false, |ui| ui.button("Save"));
        });
    });
}
