//! Top level module for all functionality related to saving and loading data from storage.

mod document;
mod load_project;
mod save_project;

pub use load_project::{LoadProjectCompleteEvent, LoadProjectEvent, LoadProjectFailedEvent};
pub use save_project::{SaveProjectCompleteEvent, SaveProjectEvent, SaveProjectFailedEvent};

pub(crate) use document::*;
pub(crate) use load_project::handle_load_project_event;
pub(crate) use save_project::handle_save_project;
