//! Renders the toolbar at the top of the screen.

use egui::{Align, Context, Layout, TopBottomPanel};
use i18n::t;

/// Handles the rendering of the toolbar.
pub(super) fn render(context: &mut Context) {
    TopBottomPanel::top("Toolbar").show(context, |ui| {
        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
            ui.style_mut().visuals.button_frame = false;

            if ui.button(t!("layout-toolbar-new-button")).clicked() {
                //
            }

            if ui.button(t!("layout-toolbar-open-button")).clicked() {
                //
            }

            ui.add_enabled_ui(false, |ui| ui.button(t!("layout-toolbar-save-button")));
        });
    });
}
