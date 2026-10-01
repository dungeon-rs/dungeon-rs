//! The Export dialog: the resolution in pixels per Grid cell, the size of the image it makes,
//! a warning about placeholders, then the platform's save dialog and the Export request.

use crate::files::{ProjectView, choose_save_file};
use crate::panels::Outgoing;
use crate::state::{EditorState, ExportDialog};
use drs_model::ExportLevel;

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
            "The Level {} is exported as a PNG covering exactly the Bounds, {} by {} cells.",
            level_name.name, bounds.size.x, bounds.size.y
        ));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Resolution");
            for preset in PRESETS {
                ui.selectable_value(&mut dialog.pixels_per_cell, preset, preset.to_string());
            }
            ui.add(
                egui::DragValue::new(&mut dialog.pixels_per_cell)
                    .range(ExportLevel::LEAST_PIXELS_PER_CELL..=ExportLevel::MOST_PIXELS_PER_CELL)
                    .clamp_existing_to_range(true)
                    .suffix(" px per cell"),
            );
        });
        ui.weak(format!(
            "Any whole number from {} to {} pixels per cell.",
            ExportLevel::LEAST_PIXELS_PER_CELL,
            ExportLevel::MOST_PIXELS_PER_CELL
        ));
        let within = (ExportLevel::LEAST_PIXELS_PER_CELL..=ExportLevel::MOST_PIXELS_PER_CELL)
            .contains(&dialog.pixels_per_cell);
        let width = u64::from(bounds.size.x) * u64::from(dialog.pixels_per_cell);
        let height = u64::from(bounds.size.y) * u64::from(dialog.pixels_per_cell);
        ui.label(format!("Image size: {width} × {height} px"));
        if placeholders > 0 {
            ui.colored_label(
                egui::Color32::YELLOW,
                format!(
                    "{placeholders} {} will be exported as shown.",
                    if placeholders == 1 {
                        "placeholder"
                    } else {
                        "placeholders"
                    }
                ),
            );
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let export = ui
                .add_enabled(within, egui::Button::new("Export…"))
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
        let Some(path) = choose_save_file("Export Level", "PNG image", "png", &proposed) else {
            return;
        };
        state.exporting = Some(path.clone());
        "Exporting…".clone_into(&mut state.status);
        outgoing.export.write(ExportLevel {
            level,
            pixels_per_cell,
            path,
            tile_size: ExportLevel::DEFAULT_TILE_SIZE,
        });
    }
}
