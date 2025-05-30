#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::suspicious, clippy::complexity)]

pub(crate) mod document;
mod plugin;
mod save_project;

pub use plugin::IOPlugin;
pub use save_project::SaveProjectEvent;
