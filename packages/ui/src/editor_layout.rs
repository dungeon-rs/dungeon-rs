use crate::widgets::toolbar::toolbar;
use crate::UiState;
use bevy::prelude::{warn, EventWriter, Res, With, World};
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContext, EguiContexts};
use bevy_inspector_egui::bevy_inspector;
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
}

pub fn inspector_layout(world: &mut World) {
    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };
    let mut egui_context = egui_context.clone();

    SidePanel::right("right_panel")
        .resizable(true)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                bevy_inspector::ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
}
