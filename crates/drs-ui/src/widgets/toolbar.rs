//! Renders the toolbar at the top of the screen.

use bevy::prelude::Commands;
use drs_data::ProjectQueryItem;
use drs_i18n::t;
use drs_io::{LoadProjectEvent, SaveProjectEvent};
use drs_utils::{AsyncComponent, report_progress};
use egui::{Align, Context, Layout, TopBottomPanel};
use native_dialog::DialogBuilder;

/// Handles the rendering of the toolbar.
pub fn render(context: &mut Context, project: Option<&ProjectQueryItem>, mut commands: Commands) {
    TopBottomPanel::top(t!("widgets-toolbar")).show(context, |ui| {
        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
            ui.style_mut().visuals.button_frame = false;

            if ui.button(t!("widgets-toolbar.new_project")).clicked() {
                todo!()
            }

            if ui.button(t!("widgets-toolbar.open_project")).clicked() {
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

            ui.add_enabled_ui(project.is_some(), |ui| {
                if ui.button(t!("widgets-toolbar.save_project")).clicked()
                    && let Some(project) = project
                {
                    commands.send_event(SaveProjectEvent::new(project.entity));
                }
            });
        });
    });
}
