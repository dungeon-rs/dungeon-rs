//! Renders the toolbar at the top of the screen.

use bevy::prelude::EventWriter;
use data::ProjectQueryItem;
use egui::{Align, Context, Layout, TopBottomPanel};
use i18n::t;
use io::SaveProjectEvent;

/// Handles the rendering of the toolbar.
pub fn render(
    context: &mut Context,
    project: &ProjectQueryItem,
    save_events: &mut EventWriter<SaveProjectEvent>,
) {
    TopBottomPanel::top("Toolbar").show(context, |ui| {
        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
            ui.style_mut().visuals.button_frame = false;

            if ui.button(t!("layout-toolbar-new-button")).clicked() {
                todo!()
            }

            if ui.button(t!("layout-toolbar-open-button")).clicked() {
                todo!()
            }

            if ui.button(t!("layout-toolbar-save-button")).clicked() {
                save_events.write(SaveProjectEvent::new(project.entity));
            }
        });
    });
}
