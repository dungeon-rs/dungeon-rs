//! Contains the functionality for rendering the splash screen.

use egui::Context;

/// Renders the splash screen.
pub fn render(context: &mut Context) {
    egui::Window::new("Splash Screen").show(context, |ui| {
        ui.label("Welcome to DungeonRS!");
    });
}
