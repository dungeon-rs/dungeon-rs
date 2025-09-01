//! Contains the functionality for rendering the splash screen.

use bevy::ecs::world::CommandQueue;
use bevy::prelude::{Commands, ResMut, Resource, SpawnRelated, World, children};
use data::Project;
use egui::Context;
use i18n::t;
use native_dialog::DialogBuilder;
use std::path::PathBuf;
use utils::{AsyncComponent, to_string};

/// Contains the form state for the create project form.
#[derive(Resource, Default)]
pub struct CreateProjectFormState {
    /// The path to the project file that will be created.
    pub path: String,
    /// The human-readable name of the project.
    pub name: String,
}

/// Renders the splash screen.
pub fn render(
    context: &mut Context,
    commands: &mut Commands,
    state: Option<ResMut<CreateProjectFormState>>,
) {
    let Some(mut state) = state else {
        commands.init_resource::<CreateProjectFormState>();

        return;
    };

    egui::Window::new("Splash Screen").show(context, |ui| {
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
                                let mut state = world.resource_mut::<CreateProjectFormState>();
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

            commands.spawn((Project::new(path, name), children![]));
        }
    });
}
