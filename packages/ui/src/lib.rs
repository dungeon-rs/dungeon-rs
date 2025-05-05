use bevy::prelude::*;
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin, egui};
use bevy_inspector_egui::{DefaultInspectorConfigPlugin, reflect_inspector};

#[derive(Default)]
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_systems(EguiContextPass, ui_example_system);
    }
}

fn ui_example_system(
    mut contexts: EguiContexts,
    registry: Res<AppTypeRegistry>,
    mut backcolor: ResMut<ClearColor>,
) {
    let Some(context) = contexts.try_ctx_mut() else {
        return;
    };
    let registry = registry.read();

    egui::SidePanel::right("inspector_panel")
        .resizable(true)
        .show(context, |ui| {
            egui::CollapsingHeader::new("Colours").show(ui, |ui| {
                for _ in 1..10 {
                    ui.horizontal(|ui| {
                        ui.label("Background color");
                        reflect_inspector::ui_for_value(&mut backcolor.0, ui, &registry);
                    });
                }
            })
        });
}
