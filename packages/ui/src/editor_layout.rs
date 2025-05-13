use crate::UiState;
use crate::widgets::toolbar::toolbar;
use bevy::prelude::{EventWriter, Res, warn};
use bevy_egui::EguiContexts;
use core::export::ExportRequest;
use egui::{Direction, Layout, SidePanel, TopBottomPanel};

/// This system builds the editor layout, positioning all other widgets and panels on the screen.
pub fn editor_layout(
    mut contexts: EguiContexts,
    ui_state: Res<UiState>,
    export_writer: EventWriter<ExportRequest>,
) {
    let Some(context) = contexts.try_ctx_mut() else {
        warn!("Failed to acquire egui context");
        return;
    };

    toolbar(context, ui_state.logo, export_writer);

    TopBottomPanel::bottom("bottom_panel")
        .frame(egui::Frame::NONE)
        .show_separator_line(false)
        .show(context, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    ui.label("Hello World");
                },
            );
        });

    SidePanel::right("right_panel")
        .resizable(true)
        .show(context, |ui| {
            ui.label("Right panel");
        });
}
