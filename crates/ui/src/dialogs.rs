//! Contains the trait for defining dialogs, as well as the resource that tracks all "open" dialogs
//! currently active in the UI.

mod new_project;
mod open_project;

use bevy::prelude::Resource;
use egui::Context;

use crate::dialogs::open_project::OpenProject;
pub use new_project::NewProject;

/// Contains all dialogs in the UI.
///
/// A field that's set to `None` will not be rendered.
#[derive(Default, Resource)]
pub struct Dialogs {
    /// An optional reference to the new project dialog to render.
    new_project: Option<NewProject>,
    /// An optional reference to the open project dialog to render.
    open_project: Option<OpenProject>,
}

impl Dialogs {
    /// Calls `render` on each currently active dialog in the queue.
    ///
    /// Also removes dialogs that indicated they shouldn't be kept open.
    pub fn render(&mut self, context: &mut Context) {
        if let Some(new_project) = self.new_project.as_mut()
            && !new_project.render(context)
        {
            self.new_project = None;
        }

        if let Some(open_project) = self.open_project.as_mut()
            && !open_project.render(context)
        {
            self.open_project = None;
        }
    }

    /// Instruct the UI to start rendering the new project dialog if it's not shown yet.
    pub fn show_new_project(&mut self) {
        if self.new_project.is_none() {
            self.new_project = Some(NewProject::default());
        }
    }

    /// Instruct the UI to start rendering the new project dialog if it's not shown yet.
    pub fn show_open_project(&mut self) {
        if self.open_project.is_none() {
            self.open_project = Some(OpenProject::default());
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
