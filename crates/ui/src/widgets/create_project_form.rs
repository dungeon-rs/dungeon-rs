//! A widget that renders a form that creates a new project.
//!
//! Note that this widget does not check if a project is currently loaded, calling code should make sure
//! no more than one project is active at once.
use bevy::ecs::children;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::{Commands, ResMut, Resource, World};
use bevy::prelude::{SpawnRelated, Transform};
use drs_data::{Layer, Level, Project};
use drs_i18n::t;
use egui::Ui;
use native_dialog::DialogBuilder;
use std::path::PathBuf;
use utils::{AsyncComponent, to_string};

/// Contains the form state for the create project form.
#[derive(Resource, Default)]
pub struct FormState {
    /// The path to the project file that will be created.
    pub path: String,
    /// The human-readable name of the project.
    pub name: String,
}

/// Renders the "create project" form in the given container context.
pub fn render(ui: &mut Ui, commands: &mut Commands, mut state: ResMut<FormState>) {
    ui.horizontal(|ui| {
        ui.label(t!("layout-splash-path_label"));
        ui.text_edit_singleline(&mut state.path);
        if ui.button(t!("layout-splash-path_button")).clicked() {
            commands.spawn(AsyncComponent::new_async(
                async |sender| {
                    let output = DialogBuilder::file()
                        .set_location("~/Desktop")
                        .save_single_file()
                        .show()?;

                    // TODO: provide utility method for this kind of access similar to `report_progress`?
                    if let Some(path) = output {
                        let mut queue = CommandQueue::default();
                        queue.push(move |world: &mut World| {
                            let mut state = world.resource_mut::<FormState>();
                            state.path = to_string(&path);
                        });

                        let _ = sender.send(queue);
                    }
                    Ok(())
                },
                |_sender, _error| {
                    // TODO: error handling
                },
            ));
        }
    });

    ui.horizontal(|ui| {
        ui.label(t!("layout-splash-name_label"));
        ui.text_edit_singleline(&mut state.name);
    });

    if ui.button(t!("layout-splash-create_button")).clicked() {
        // TODO: validation
        let path = PathBuf::from(&state.path);
        let name = state.name.clone();

        commands.remove_resource::<FormState>();
        commands.spawn((
            Project::new(path, name),
            children![(
                Level::new("default"),
                children![(Layer::new("default", Transform::default()), children![],)]
            )],
        ));
    }
}
