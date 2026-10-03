#![doc = include_str!("../README.md")]

mod bands;
mod cache;
mod gpu;
mod rasterize;

pub use cache::{PaintCache, apply_stroke};
pub use gpu::{PaintEnginePlugin, StrokeJobs, StrokeRasterizer};
pub use rasterize::rasterize;
