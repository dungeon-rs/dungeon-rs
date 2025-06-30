//! Contains the rendering logic for the [`crate::layout::EditorPanels::Assets`] pane.
use crate::layout::EditorLayout;
use egui::Ui;

/// Handles the rendering of the [`crate::layout::EditorPanels::Assets`] tab.
pub(super) fn render(_viewer: &mut EditorLayout, ui: &mut Ui) -> anyhow::Result<()> {
    ui.label("Rendering asset library here");

    Ok(())
}
