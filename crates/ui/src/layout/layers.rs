//! Contains the rendering logic for the [`crate::layout::EditorPanels::Layers`] pane.
use crate::layout::EditorLayout;
use egui::Ui;

/// Handles the rendering of the [`crate::layout::EditorPanels::Layers`] tab.
pub(super) fn render(_viewer: &mut EditorLayout, ui: &mut Ui) -> anyhow::Result<()> {
    ui.label("Rendering layers here");

    Ok(())
}
