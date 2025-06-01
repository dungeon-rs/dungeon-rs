#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::suspicious, clippy::complexity)]

mod plugin;

pub use plugin::UIPlugin;
