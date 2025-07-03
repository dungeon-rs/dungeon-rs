//! Renders the toolbar at the top of the screen.

use bevy::prelude::default;
use egui::{Align, Context, Layout, TextEdit, TopBottomPanel, Window};
use egui_form::{Form, FormField};
use egui_form::garde::{field_path, GardeReport};
use garde::Validate;
use crate::state::{NewMapState, UiState};

/// Handles the rendering of the toolbar.
pub(super) fn render(context: &mut Context, state: &mut UiState) {
    let &mut UiState { ref mut new_map_state, .. } = state;
    show_new_map_dialog(context, new_map_state);

    TopBottomPanel::top("Toolbar").show(context, |ui| {
        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
            ui.style_mut().visuals.button_frame = false;

            if ui.button("New").clicked() {
                *new_map_state = Some(default());
            }

            let _ = ui.button("Open");
            ui.add_enabled_ui(false, |ui| ui.button("Save"));
        });
    });
}

fn show_new_map_dialog(context: &mut Context, state: &mut Option<NewMapState>) {
    let Some(ref mut new_map_state) = *state else {
        return;
    };

    let mut is_open = true;
    let mut form = Form::new().add_report(GardeReport::new(new_map_state.validate()));

    Window::new("new map")
        .open(&mut is_open)
        .show(context, |ui| {
            FormField::new(&mut form, field_path!("name"))
                .label("Name")
                .ui(ui, TextEdit::singleline(&mut new_map_state.name));
        });

    if !is_open {
        *state = None;
    }
}
