mod widgets;

use crate::widgets::toolbar::toolbar;
use bevy::prelude::*;
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;

/// The UI plugin handles registering and setting up all user interface elements of the editor.
/// This module is unavailable in headless mode and should not contain any actual functionality,
/// just build the user interface.
#[derive(Default)]
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_systems(EguiContextPass, editor_interface);
    }
}

fn editor_interface(mut contexts: EguiContexts) {
    let Some(context) = contexts.try_ctx_mut() else {
        return;
    };

    toolbar(context);

    egui::SidePanel::right("inspector_panel")
        .resizable(true)
        .show(context, |ui| {
            //         side_panel.ui(ui, &mut Viewer);
        });
}
