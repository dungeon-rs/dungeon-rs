#![doc = include_str!("../README.md")]

mod async_ecs;
mod directories;
mod pathbuf;
mod plugin;
mod version;

pub use async_ecs::*;
pub use directories::*;
pub use pathbuf::*;
pub use plugin::CorePlugin;
pub use utils_macros::*;
pub use version::version;

// re-export for the macros
pub use tracing_indicatif::span_ext::IndicatifSpanExt;
pub use tracing_indicatif::style::ProgressStyle;
