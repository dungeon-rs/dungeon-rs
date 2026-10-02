#![doc = include_str!("../README.md")]

mod wall;

pub use wall::{ShapeError, generate_walls, split_wall};
