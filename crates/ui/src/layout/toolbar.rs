//! Renders the toolbar at the top of the screen.

use crate::dialogs::Dialogs;
use egui::{Align, Context, Layout, TopBottomPanel};
use i18n::t;

/// Handles the rendering of the toolbar.
pub(super) fn render(context: &mut Context, dialogs: &mut Dialogs) {
    TopBottomPanel::top("Toolbar").show(context, |ui| {
        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
            ui.style_mut().visuals.button_frame = false;

            if ui.button(t!("layout-toolbar-new-button")).clicked() {
                dialogs.show_new_project();
            }

            if ui.button(t!("layout-toolbar-open-button")).clicked() {
                dialogs.show_open_project();
            }

            ui.add_enabled_ui(false, |ui| ui.button(t!("layout-toolbar-save-button")));
        });
    });
}
