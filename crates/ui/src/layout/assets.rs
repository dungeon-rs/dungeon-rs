//! Contains the rendering logic for the [`crate::layout::EditorPanels::Assets`] pane.
use crate::layout::EditorLayout;
use egui::Ui;
use i18n::t;
use native_dialog::DialogBuilder;

/// Handles the rendering of the [`crate::layout::EditorPanels::Assets`] tab.
pub(super) fn render(viewer: &mut EditorLayout, ui: &mut Ui) {
    ui.collapsing(t!("assets-add_pack-packs-header"), |ui| {
        for (id, _path) in viewer.asset_library.iter() {
            let mut is_loaded = viewer.asset_library.get_pack(id).is_some();
            if ui.checkbox(&mut is_loaded, &id[0..6]).changed() {
                viewer.notifications.warn(t!("not-implemented-yet"));
            }
        }
    });

    if ui.button(t!("assets-add_pack-add_button")).clicked() {
        let dialog = DialogBuilder::file().set_location("~").open_single_dir();

        if let Ok(Some(path)) = dialog.show() {
            match viewer
                .asset_library
                .add_pack(&path, None, None)
                .and_then(|id| viewer.asset_library.save(None).map(|()| id))
            {
                Ok(id) => viewer
                    .notifications
                    .info(t!("assets-add_pack-added-notification", "name" => id)),
                Err(error) => viewer.notifications.error(
                    t!("assets-add_pack-adding_failed-notification", "error" => error.to_string()),
                ),
            }
        }
    }
}
