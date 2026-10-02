#![doc = include_str!("../README.md")]

mod cache;
mod rasterize;

pub use cache::{PaintCache, apply_stroke};
pub use rasterize::rasterize;
