//! Contains the functionality for rendering the splash screen.

use crate::widgets::create_project_form;
use crate::widgets::create_project_form::CreateProjectFormState;
use bevy::prelude::{Commands, ResMut};
use egui::Context;
use i18n::t;

/// Renders the splash screen.
pub fn render(
    context: &mut Context,
    commands: &mut Commands,
    state: Option<ResMut<CreateProjectFormState>>,
) {
    let Some(state) = state else {
        commands.init_resource::<CreateProjectFormState>();

        return;
    };

    egui::Window::new(t!("widgets-create_project_form-title")).show(context, |ui| {
        create_project_form::render(ui, commands, state);
    });
}
