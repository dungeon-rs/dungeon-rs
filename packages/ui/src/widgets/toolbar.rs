use crate::ui_state::UiState;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::{Commands, EventWriter, Rect, Res, default};
use core::prelude::*;
use egui::{Context, TopBottomPanel};
use std::path::PathBuf;

/// Draw the toolbar, depending on the `state` it automatically enables/disables buttons.
pub fn toolbar(
    context: &mut Context,
    commands: &mut Commands,
    diagnostics: Res<DiagnosticsStore>,
    state: &Res<UiState>,
    mut export_writer: EventWriter<ExportRequest>,
    mut save_writer: EventWriter<SaveProjectRequest>,
    mut load_writer: EventWriter<LoadProjectRequest>,
) {
    TopBottomPanel::top("toolbar")
        .resizable(false)
        .show(context, |ui| {
            ui.style_mut().spacing.item_spacing.x = 10.0;
            ui.style_mut().visuals.button_frame = false;

            ui.horizontal(|ui| {
                ui.image(state.logo);
                ui.separator();

                ui.menu_button("New", move |ui| {
                    if ui
                        .button("New Asset Library")
                        .on_hover_text("Create a new asset library")
                        .clicked()
                    {
                        commands.insert_resource::<AssetLibraryBuilder>(default());
                        ui.close_menu();
                    }
                });
                if ui
                    .button("Open")
                    .on_hover_text("Open an existing map")
                    .clicked()
                {
                    load_writer.write(LoadProjectRequest {
                        path: PathBuf::from("output.drs"),
                    });
                }
                if ui
                    .button("Save")
                    .on_hover_text("Save the current map")
                    .clicked()
                {
                    save_writer.write(SaveProjectRequest {
                        path: PathBuf::from("output.drs"),
                    });
                }
                // ui.add_enabled(false, Button::new("Export"))
                //     .on_hover_text("Export the current map as an image");

                if ui
                    .button("Export")
                    .on_hover_text("Export the current map to an image")
                    .clicked()
                {
                    let Ok(request) = ExportRequest::new(
                        PathBuf::from("output.png"),
                        Rect::new(-500., -500., 500., 500.),
                        128,
                    ) else {
                        return;
                    };

                    export_writer.write(request);
                }

                ui.separator();
                ui.add_space(ui.available_width() - 100.0);

                // algorithm taken from the official FPS code
                if let Some(fps) = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(|fps| fps.smoothed())
                {
                    ui.label(format!("{:.0}fps", fps));
                }
            });
        });
}
