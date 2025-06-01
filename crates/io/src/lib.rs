#![doc = include_str!("../README.md")]
#![allow(clippy::missing_docs_in_private_items)]

pub(crate) mod document;
mod load_project;
mod plugin;
mod save_project;

pub use load_project::LoadProjectEvent;
pub use plugin::IOPlugin;
pub use save_project::SaveProjectEvent;
