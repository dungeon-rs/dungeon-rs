//! Contains the rendering logic for the [`crate::layout::EditorPanels::Assets`] pane.
use crate::layout::EditorLayout;
use egui::Ui;
use rfd::FileDialog;

/// Handles the rendering of the [`crate::layout::EditorPanels::Assets`] tab.
pub(super) fn render(viewer: &mut EditorLayout, ui: &mut Ui) {
    ui.collapsing("Asset Packs", |ui| {
        for (id, path) in viewer.asset_library.iter() {
            let mut is_loaded = viewer.asset_library.get_pack(id).is_some();
            if ui.checkbox(&mut is_loaded, &id[0..6]).changed() {
                viewer.notifications.warn("Not implemented yet");
            }
        }
    });

    if ui.button("Add pack").clicked() {
        if let Some(path) = FileDialog::new().pick_folder() {
            match viewer
                .asset_library
                .add_pack(&path, None)
                .and_then(|id| viewer.asset_library.save(None).map(|_| id))
            {
                Ok(id) => viewer.notifications.info(format!("Added pack {id}")),
                Err(error) => viewer
                    .notifications
                    .error(format!("Failed to add asset pack: {error}")),
            }
        }
    }
}
