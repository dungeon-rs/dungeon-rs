//! Contains the trait for defining dialogs, as well as the resource that tracks all "open" dialogs
//! currently active in the UI.

mod new_project;

use bevy::prelude::Resource;
use egui::Context;

pub use new_project::NewProject;

/// Contains all dialogs in the UI.
///
/// A field that's set to `None` will not be rendered.
#[derive(Default, Resource)]
pub struct Dialogs {
    /// An optional reference to the new project dialog to render.
    new_project: Option<NewProject>,
}

impl Dialogs {
    /// Calls `render` on each currently active dialog in the queue.
    ///
    /// Also removes dialogs that indicated they shouldn't be kept open.
    pub fn render(&mut self, context: &mut Context) {
        if let Some(new_project) = self.new_project.as_mut() {
            if !new_project.render(context) {
                self.new_project = None;
            }
        }
    }

    /// Instruct the UI to start rendering the new project dialog if it's not shown yet.
    pub fn show_new_project(&mut self) {
        if self.new_project.is_none() {
            self.new_project = Some(NewProject::default());
        }
    }
}

/// A small trait to define that a dialog struct can be rendered.
trait RenderableDialog: Send + Sync + 'static {
    /// Called once per render, the struct should render itself as a window in the given `context`.
    ///
    /// If the returned boolean is false, the dialog will automatically be removed from the render queue.
    fn render(&mut self, context: &mut Context) -> bool;
}
