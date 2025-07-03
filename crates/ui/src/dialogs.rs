//! Contains the trait for defining dialogs, as well as the resource that tracks all "open" dialogs
//! currently active in the UI.

mod new_project;

use bevy::prelude::Resource;
use egui::Context;

pub use new_project::NewProject;

/// Contains all "active" (=visible) dialogs in the UI.
#[derive(Default, Resource)]
pub struct Dialogs(Vec<Box<dyn RenderableDialog>>);

impl Dialogs {
    /// calls `render` on each currently active dialog in the queue.
    ///
    /// Also removes dialogs that indicated they shouldn't be kept open.
    pub fn render(&mut self, context: &mut Context) {
        self.0.retain_mut(|dialog| {
            let mut keep_open = true;
            dialog.render(context, &mut keep_open);

            keep_open
        });
    }

    /// Adds a new [`RenderableDialog`] to the queue to be rendered.
    pub fn add(&mut self, dialog: impl RenderableDialog) {
        self.0.push(Box::new(dialog));
    }
}

/// A small trait to define that a dialog struct can be rendered.
pub trait RenderableDialog: Send + Sync + 'static {
    /// Called once per render, the struct should render itself as a window in the given `context`.
    ///
    /// If the passed `keep_open` boolean is set to false the dialog will automatically be removed
    /// from the render queue. This is to make the `.open` API of egui's Window more fluent.
    fn render(&mut self, context: &mut Context, keep_open: &mut bool);
}
