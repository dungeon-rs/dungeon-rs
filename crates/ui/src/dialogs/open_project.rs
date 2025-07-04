//! Contains the code related to the "open project" dialog and form.
use crate::dialogs::RenderableDialog;
use egui::{Context, TextEdit, Window};
use egui_form::garde::{GardeReport, field_path};
use egui_form::{Form, FormField};
use garde::Validate;
use std::path::PathBuf;

/// Defines all form data required for opening an existing project.
#[derive(Default, Debug, Validate)]
pub struct OpenProject {
    /// The path where the save file is located, represented as a string.
    #[garde(ascii, length(min = 3, max = 256), custom(path_exists))]
    pub path: String,
}

/// Internal validator function that validates the given `value` is an existing path to a file.
///
/// # Errors
/// Returns a `garde::Error` when given a path that doesn't exist or refers to a folder.
#[allow(
    clippy::ref_option,
    reason = "Required by Garde's API to pass a reference"
)]
#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "Required by Garde's API to pass a reference"
)]
fn path_exists(path: &String, _context: &&&()) -> garde::Result {
    let path = PathBuf::from(path);

    if !path.exists() {
        return Err(garde::Error::new("Selected file does not exist"));
    } else if path.is_dir() {
        return Err(garde::Error::new(
            "Folders are (currently) not supported. Please select a file instead.",
        ));
    }

    Ok(())
}

impl RenderableDialog for OpenProject {
    fn render(&mut self, context: &mut Context) -> bool {
        let validation = self.validate();
        let is_valid = validation.is_ok();
        let mut form = Form::new().add_report(GardeReport::new(validation));

        let mut keep_open = true;
        // `.open` takes exclusive ownership, so we create a second flag that the buttons/UI can use.
        let mut keep_open_inner = true;

        Window::new("Open Project")
            .open(&mut keep_open)
            .show(context, |ui| {
                ui.horizontal(|ui| {
                    FormField::new(&mut form, field_path!("path"))
                        .label("Location")
                        .ui(ui, TextEdit::singleline(&mut self.path));

                    if ui.button("…").clicked() {
                        self.path = rfd::FileDialog::new()
                            // TODO: add filters for extensions
                            .pick_file()
                            .map_or_else(String::new, |path| path.to_string_lossy().to_string());
                    }
                });

                ui.add_enabled_ui(is_valid, |ui| {
                    if ui.button("Open").clicked() {
                        keep_open_inner = false;
                    }
                });
            });

        keep_open && keep_open_inner
    }
}
