//! The Assets panel: the Assets of every added Asset Folder, filtered by name, one click to choose.

use crate::state::{Chosen, EditorState};
use bevy::ecs::system::Query;
use drs_model::{AssetFolder, ChosenAsset, IndexedAsset};

/// Draws the Assets panel.
pub(crate) fn show(ui: &mut egui::Ui, state: &mut EditorState, folders: &Query<&AssetFolder>) {
    ui.horizontal(|ui| {
        ui.label("Filter");
        ui.add(
            egui::TextEdit::singleline(&mut state.filter)
                .hint_text("part of a name")
                .desired_width(f32::INFINITY),
        );
    });
    let needle = state.filter.trim().to_lowercase();
    let mut folders: Vec<&AssetFolder> = folders.iter().collect();
    folders.sort_by(|a, b| a.name.cmp(&b.name));
    if folders.is_empty() {
        ui.weak("No Asset Folder is added yet. Use Library → Add Asset Folder…");
        return;
    }
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            for folder in folders {
                let matching: Vec<&IndexedAsset> = folder
                    .assets
                    .iter()
                    .filter(|asset| {
                        needle.is_empty() || asset.name.to_lowercase().contains(&needle)
                    })
                    .collect();
                egui::CollapsingHeader::new(format!("{} ({})", folder.name, matching.len()))
                    .default_open(true)
                    .show(ui, |ui| {
                        if folder.assets.is_empty() {
                            ui.weak("No Assets");
                        } else if matching.is_empty() {
                            ui.weak("No Asset matches the filter");
                        }
                        for asset in matching {
                            asset_row(ui, state, folder, asset);
                        }
                    });
            }
        });
}

/// One Asset: its name, the subfolder it sits in when it has one, and a click to choose it.
fn asset_row(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    folder: &AssetFolder,
    asset: &IndexedAsset,
) {
    let chosen = state.chosen.as_ref().is_some_and(|chosen| {
        chosen.asset.folder == folder.key && chosen.asset.place == asset.place
    });
    let subfolder = asset.place.rsplit_once('/').map(|(directory, _)| directory);
    let response = ui
        .horizontal(|ui| {
            let response = ui.selectable_label(chosen, &asset.name);
            if let Some(subfolder) = subfolder {
                ui.weak(subfolder);
            }
            response
        })
        .inner
        .on_hover_text(&asset.place);
    if response.clicked() {
        state.chosen = Some(Chosen {
            asset: ChosenAsset {
                folder: folder.key.clone(),
                place: asset.place.clone(),
            },
            name: asset.name.clone(),
        });
        state.selected = None;
    }
}
