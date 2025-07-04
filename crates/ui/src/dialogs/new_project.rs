//! Contains the code related to the "new project" dialog and form.
use crate::dialogs::RenderableDialog;
use egui::{Context, DragValue, TextEdit, Window};
use egui_form::garde::{GardeReport, field_path};
use egui_form::{Form, FormField};
use garde::Validate;

/// Defines all form data required for creating a new project.
#[derive(Default, Debug, Validate)]
pub struct NewProject {
    /// The human-friendly name of the project.
    #[garde(ascii, length(min = 3, max = 100))]
    pub name: String,
    /// The width in grid cells of the map.
    #[garde(range(min = 4, max = 10), custom(multiple_of_4))]
    pub width: u32,
    /// The height in grid cells of the map.
    #[garde(range(min = 4, max = 10), custom(multiple_of_4))]
    pub height: u32,
}

/// Internal validator function that validates the given `value` is a multiple of 4.
///
/// # Errors
/// Returns a `garde::Error` when given a number that is not a multiple of 4.
#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "Required by Garde's API to pass a reference"
)]
fn multiple_of_4(value: &u32, _context: &&&()) -> garde::Result {
    if value % 4 == 0 {
        return Ok(());
    }

    Err(garde::Error::new("Value must be a multiple of 4"))
}

impl RenderableDialog for NewProject {
    fn render(&mut self, context: &mut Context) -> bool {
        let validation = self.validate();
        let mut form = Form::new().add_report(GardeReport::new(validation.clone()));

        let mut keep_open = true;
        // `.open` takes exclusive ownership, so we create a second flag that the buttons/UI can use.
        let mut keep_open_inner = true;
        Window::new("New Project")
            .open(&mut keep_open)
            .resizable(false)
            .show(context, |ui| {
                FormField::new(&mut form, field_path!("name"))
                    .label("Name")
                    .ui(ui, TextEdit::singleline(&mut self.name));

                ui.horizontal(|ui| {
                    FormField::new(&mut form, field_path!("width"))
                        .label("Width")
                        .ui(ui, DragValue::new(&mut self.width).range(4..=400).speed(4));

                    FormField::new(&mut form, field_path!("height"))
                        .label("Height")
                        .ui(ui, DragValue::new(&mut self.height).range(4..=400).speed(4));
                });

                ui.add_space(8.0);
                ui.separator();

                ui.add_enabled_ui(validation.is_ok(), |ui| {
                    if ui.button("Create").clicked() {
                        keep_open_inner = false;
                    }
                });
            });

        keep_open && keep_open_inner
    }
}
