#![doc = include_str!("../README.md")]

mod bands;
mod cache;
mod gpu;
mod rasterize;

pub use cache::{PaintCache, apply_stroke};
pub use gpu::{PaintEnginePlugin, StrokeRasterizer, ready_to_rasterize_on_gpu};
pub use rasterize::rasterize;
