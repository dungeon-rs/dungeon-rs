//! Contains the functionality for rendering the splash screen.

use crate::widgets::create_project_form;
use crate::widgets::create_project_form::FormState;
use bevy::prelude::{Commands, ResMut};
use egui::Context;
use i18n::t;

/// Renders the splash screen.
pub fn render(context: &mut Context, commands: &mut Commands, state: Option<ResMut<FormState>>) {
    let Some(state) = state else {
        commands.init_resource::<FormState>();

        return;
    };

    egui::Window::new(t!("widgets-create_project_form-title")).show(context, |ui| {
        create_project_form::render(ui, commands, state);
    });
}
