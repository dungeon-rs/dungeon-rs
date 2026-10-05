//! The Export dialog: the Bounds as they stand, the resolution in pixels per Grid cell, typed as it
//! is and refused in words while it is outside the limits or makes an image larger than an Export
//! can write, the size of the image it makes, a warning about
//! placeholders, then the platform's save dialog and the Export request.

use crate::files::{ProjectView, choose_export_file};
use crate::outcomes::counted;
use crate::panels::Outgoing;
use crate::state::{EditorState, ExportDialog};
use drs_model::{Bounds, ExportLevel};

/// The resolutions offered as one click.
const PRESETS: [u32; 4] = [50, 100, 200, 300];

/// Opens the dialog with the proposed resolution.
pub(crate) fn begin(state: &mut EditorState) {
    state.export = Some(ExportDialog::default());
}

/// The Export dialog, while it is open; on Export… it runs the save dialog proposing
/// `<Project> - <Level>.png` and sends the Export request, which the Editor then waits for.
pub(crate) fn dialog(
    ctx: &egui::Context,
    state: &mut EditorState,
    view: &ProjectView,
    outgoing: &mut Outgoing,
) {
    let Some(dialog) = &mut state.export else {
        return;
    };
    let Some((level, level_name)) = view.level() else {
        state.export = None;
        return;
    };
    let bounds = view.bounds().unwrap_or_default();
    let placeholders = view.placeholders(level);
    let modal = egui::Modal::new(egui::Id::new("export-level")).show(ctx, |ui| {
        ui.set_width(420.0);
        ui.heading("Export Level");
        ui.label(format!(
            "The Level {} is exported as a PNG covering exactly the Bounds, {} by {} cells from \
             the cell at {}, {}.",
            level_name.name, bounds.size.x, bounds.size.y, bounds.origin.x, bounds.origin.y
        ));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Resolution");
            for preset in PRESETS {
                ui.selectable_value(&mut dialog.pixels_per_cell, preset, preset.to_string());
            }
            ui.add(egui::DragValue::new(&mut dialog.pixels_per_cell).suffix(" px per cell"));
        });
        let within = (ExportLevel::LEAST_PIXELS_PER_CELL..=ExportLevel::MOST_PIXELS_PER_CELL)
            .contains(&dialog.pixels_per_cell);
        if within {
            ui.weak(format!(
                "Any whole number from {} to {} pixels per cell.",
                ExportLevel::LEAST_PIXELS_PER_CELL,
                ExportLevel::MOST_PIXELS_PER_CELL
            ));
        } else {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                format!(
                    "A resolution of {} pixels per cell is outside the limits: the Export \
                     supports {} to {} pixels per cell.",
                    dialog.pixels_per_cell,
                    ExportLevel::LEAST_PIXELS_PER_CELL,
                    ExportLevel::MOST_PIXELS_PER_CELL
                ),
            );
        }
        let fits = image_size(ui, bounds, dialog.pixels_per_cell);
        if placeholders > 0 {
            ui.colored_label(
                egui::Color32::YELLOW,
                format!(
                    "{} will be exported as shown.",
                    counted(placeholders, "placeholder", "placeholders")
                ),
            );
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let export = ui
                .add_enabled(within && fits, egui::Button::new("Export…"))
                .clicked();
            let cancel = ui.button("Cancel").clicked();
            (export, cancel)
        })
        .inner
    });
    let (export, cancel) = modal.inner;
    if cancel || modal.should_close() {
        state.export = None;
    } else if export {
        let pixels_per_cell = dialog.pixels_per_cell;
        let proposed = format!("{} - {}.png", view.name(), level_name.name);
        state.export = None;
        let Some(path) = choose_export_file(&proposed) else {
            return;
        };
        state.exporting = true;
        "Exporting…".clone_into(&mut state.status);
        outgoing.export.write(ExportLevel {
            level,
            pixels_per_cell,
            path,
            tile_size: ExportLevel::DEFAULT_TILE_SIZE,
        });
    }
}

/// Shows the size of the image `bounds` make at `pixels_per_cell` and, when it is larger than an
/// Export can write, why it is refused, naming the largest resolution the Bounds allow; returns
/// whether it fits.
fn image_size(ui: &mut egui::Ui, bounds: Bounds, pixels_per_cell: u32) -> bool {
    let width = u64::from(bounds.size.x) * u64::from(pixels_per_cell);
    let height = u64::from(bounds.size.y) * u64::from(pixels_per_cell);
    ui.label(format!("Image size: {width} × {height} px"));
    let Err(refusal) = ExportLevel::image_size(bounds, pixels_per_cell) else {
        return true;
    };
    let mut sentence = refusal.to_string();
    if let Some(first) = sentence.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    sentence.push('.');
    ui.colored_label(egui::Color32::LIGHT_RED, sentence);
    false
}
