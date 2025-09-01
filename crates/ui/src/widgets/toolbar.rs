//! Renders the toolbar at the top of the screen.

use bevy::prelude::{Commands, EventWriter};
use data::ProjectQueryItem;
use egui::{Align, Context, Layout, TopBottomPanel};
use i18n::t;
use io::{LoadProjectEvent, SaveProjectEvent};
use native_dialog::DialogBuilder;
use utils::{AsyncComponent, report_progress};

/// Handles the rendering of the toolbar.
pub fn render(
    context: &mut Context,
    project: &ProjectQueryItem,
    mut commands: Commands,
    save_events: &mut EventWriter<SaveProjectEvent>,
) {
    TopBottomPanel::top("Toolbar").show(context, |ui| {
        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
            ui.style_mut().visuals.button_frame = false;

            if ui.button(t!("layout-toolbar-new-button")).clicked() {
                todo!()
            }

            if ui.button(t!("layout-toolbar-open-button")).clicked() {
                commands.spawn(AsyncComponent::new_async(
                    async |sender| {
                        let input = DialogBuilder::file()
                            .set_location("~/Desktop")
                            .open_single_file()
                            .show()?;

                        if let Some(input) = input {
                            let _ = report_progress(&sender, LoadProjectEvent { input });
                        }

                        Ok(())
                    },
                    |_sender, _error| {},
                ));
            }

            if ui.button(t!("layout-toolbar-save-button")).clicked() {
                save_events.write(SaveProjectEvent::new(project.entity));
            }
        });
    });
}
