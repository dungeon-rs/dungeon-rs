mod side_panel;
mod toolbar;

use crate::UiState;
use crate::widgets::toolbar::toolbar;
use bevy::diagnostic::DiagnosticsStore;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::{Commands, EventWriter, Res, ResMut, With, World, info, warn};
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, block_on};
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContext, EguiContexts};
use bevy_inspector_egui::bevy_inspector;
use core::prelude::*;
use egui::{Direction, Frame, Layout, SidePanel, TopBottomPanel};
use rfd::AsyncFileDialog;
use std::path::PathBuf;

/// This system builds the editor layout, positioning all other widgets and panels on the screen.
#[allow(clippy::needless_pass_by_value)]
pub fn editor_layout(
    mut contexts: EguiContexts,
    mut commands: Commands,
    diagnostics: Res<DiagnosticsStore>,
    ui_state: Res<UiState>,
    export_writer: EventWriter<ExportRequest>,
    save_writer: EventWriter<SaveProjectRequest>,
    load_writer: EventWriter<LoadProjectRequest>,
) {
    let Some(context) = contexts.try_ctx_mut() else {
        warn!("Failed to acquire egui context");
        return;
    };

    toolbar(
        context,
        &mut commands,
        diagnostics,
        &ui_state,
        export_writer,
        save_writer,
        load_writer,
    );

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

pub fn create_asset_library(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut builder: ResMut<AssetLibraryBuilder>,
) {
    let Some(context) = contexts.try_ctx_mut() else {
        warn!("Failed to acquire egui context");
        return;
    };

    egui::Window::new("Create a new asset library").show(context, |ui| {
        let name = &mut builder.name;
        ui.horizontal(move |ui| {
            ui.label("Name: ");
            ui.text_edit_singleline(name);
        });

        let path = &mut builder.root;
        ui.horizontal(|ui| {
            ui.label(format!("Location: {}", path.display()));
            if ui.button("...").clicked() {
                // TODO: we don't want this right here in the UI code..
                AsyncCommand::spawn(&mut commands, async move {
                    let dialog = AsyncFileDialog::new();

                    let mut queue = CommandQueue::default();
                    if let Some(folder) = dialog.pick_folder().await {
                        queue.push(move |world: &mut World| {
                            let path = PathBuf::from(folder.path());

                            let mut builder = world.get_resource_mut::<AssetLibraryBuilder>().unwrap();
                            builder.root = path;
                        });
                    }

                    Ok(queue)
                });
            }
        });

        let packs = &mut builder.packs;
        ui.group(move |ui| {
            ui.heading("Asset Packs");

            for pack in packs.iter_mut() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("name: ");
                        ui.text_edit_singleline(&mut pack.name);
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("location: {}", pack.root.display()));
                        let _ = ui.button("...");
                    });
                });
            }

            ui.separator();
            if ui.button("Add asset pack").clicked() {
                packs.push(AssetPack::default());
            }
        });

        if ui.button("Compile library").clicked() {
            info!("{:?}", builder);
        }
    });
}
