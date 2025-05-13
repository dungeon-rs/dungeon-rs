// use bevy::prelude::Resource;
// use egui::Ui;
// use egui_dock::{DockArea, DockState, Style, TabViewer};
//
// /// A side panel with tabs, reusable for any sort of side panel.
// #[derive(Resource)]
// pub struct SidePanel<T> {
//     dock_state: DockState<T>,
// }
//
// impl<T> SidePanel<T> {
//     /// Create a new `SidePanel<T>` which generates the docking state from `tabs`.
//     pub fn new(tabs: Vec<T>) -> Self {
//         let dock_state = DockState::new(tabs);
//
//         Self { dock_state }
//     }
//
//     /// Renders a `DockArea` inside `ui` and uses `viewer` to render it.
//     /// The `SidePanel<T>` is responsible for maintaining the state of the docked tabs.
//     pub fn ui(&mut self, ui: &mut Ui, viewer: &mut impl TabViewer<Tab = T>) {
//         DockArea::new(&mut self.dock_state)
//             .style(Style::from_egui(ui.style().as_ref()))
//             .show_close_buttons(false)
//             .show_leaf_close_all_buttons(false)
//             .show_inside(ui, viewer);
//     }
// }
