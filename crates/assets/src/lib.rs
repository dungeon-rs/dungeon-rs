#![doc = include_str!("../README.md")]

mod library;
mod packs;
mod plugin;
mod scripting;

pub use library::*;
pub use packs::*;
pub use plugin::AssetPlugin;
