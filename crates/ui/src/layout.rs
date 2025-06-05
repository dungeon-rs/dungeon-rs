//! This module defines the building blocks for layouting the editor.
//!
//! The current implementation builds an `egui_dock` `TabViewer` and delegates the specific layout
//! to that. We contain the different panels and how to construct them in this module.
use crate::state::UiState;
use bevy::prelude::{With, World};
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContext;
use bevy_inspector_egui::bevy_inspector::ui_for_world;
use egui::{Ui, WidgetText};
use egui_dock::{DockArea, Style, TabViewer};

/// The different panes that can be shown in the editor UI.
/// If a new panel needs to be available for the user in the UI it needs to be added here,
/// if it needs to be shown by default, make sure to add it in [`UiState::default`] as well.
#[derive(Debug)]
pub enum EditorPanels {
    /// The "main" view that shows the underlying Bevy rendered world.
    Editor,
}

/// Contains the data structures that are available to the [`TabViewer`] when rendering the editor layout.
/// See [`EditorLayout::ui`] in particular.
pub struct EditorLayout<'a> {
    /// The Bevy `World` that is rendering the [`EditorLayout`] instance.
    /// We need this to show various editing panels.
    world: &'a mut World,
}

impl TabViewer for EditorLayout<'_> {
    type Tab = EditorPanels;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        format!("{tab:?}").into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            EditorPanels::Editor => {
                ui.label("Editor panel");
                ui_for_world(self.world, ui);
            }
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }
}

/// Handles rendering the [`EditorLayout`] in the `World`.
#[utils::bevy_system]
pub fn render_editor_layout(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    // get an instance of the UiState and a mutable reference to the world to work with.
    world.resource_scope::<UiState, _>(|world, mut state| {
        let context = egui_context.get_mut();
        // construct an `EditorLayout` using our mutable world reference for rendering.
        // the `EditorLayout` struct has a strict lifetime bound to this scope and may not leak.
        let mut viewer = EditorLayout { world };

        // Render the `dock_state` in the `UiState` in a DockArea.
        DockArea::new(&mut state.dock_state)
            .style(Style::from_egui(context.style().as_ref()))
            .show(context, &mut viewer);
    });
}
