#![doc = include_str!("../README.md")]

mod async_ecs;
mod plugin;

pub use async_ecs::*;
pub use core_macros::*;
pub use plugin::CorePlugin;
