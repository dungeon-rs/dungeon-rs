use bevy::app::App;
use bevy::prelude::Plugin;
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin, egui};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_systems(EguiContextPass, window);
    }
}

pub fn window(mut contexts: EguiContexts) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });
}
