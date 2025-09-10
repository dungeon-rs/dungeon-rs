//! Core functionality, shared between the UI and CLI implementations.
//!
//! This crate is built with an event-driven architecture, exposing the events and implementing
//! listeners. Progress, completion and errors are reported through events also exposed by this crate.

mod persistence;
mod plugin;

pub use persistence::*;
pub use plugin::CorePlugin;
