use egui::load::SizedTexture;
use egui::{Button, Context, TopBottomPanel};

/// Draw the toolbar, depending on the `state` it automatically enables/disables buttons.
pub fn toolbar(context: &mut Context, logo: SizedTexture) {
    TopBottomPanel::top("toolbar")
        .resizable(false)
        .show(context, |ui| {
            ui.style_mut().spacing.item_spacing.x = 10.0;
            ui.style_mut().visuals.button_frame = false;

            ui.horizontal(|ui| {
                ui.image(logo);
                ui.separator();

                ui.button("New").on_hover_text("Create a new map");
                ui.button("Open").on_hover_text("Open an existing map");
                ui.button("Save").on_hover_text("Save the current map");
                ui.add_enabled(false, Button::new("Export"))
                    .on_hover_text("Export the current map as an image");

                ui.separator();
                ui.close_menu();
            });
        });
}
