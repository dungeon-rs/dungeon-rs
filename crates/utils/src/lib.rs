#![doc = include_str!("../README.md")]

mod async_ecs;
mod logging;
mod plugin;

pub use async_ecs::*;
pub use logging::log_plugin;
pub use plugin::CorePlugin;
pub use utils_macros::*;
