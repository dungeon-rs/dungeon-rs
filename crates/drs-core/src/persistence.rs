//! Top level module for all functionality related to saving and loading data from storage.

mod document;
mod load_project;
mod save_project;

pub use load_project::{LoadProjectCompleteMessage, LoadProjectFailedMessage, LoadProjectMessage};
pub use save_project::{SaveProjectCompleteMessage, SaveProjectFailedMessage, SaveProjectMessage};

pub(crate) use document::*;
pub(crate) use load_project::handle_load_project_message;
pub(crate) use save_project::handle_save_project;
