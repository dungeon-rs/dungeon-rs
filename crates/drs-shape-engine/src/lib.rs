#![doc = include_str!("../README.md")]

mod portal;
mod wall;

pub use portal::{PointEdit, PortalSetting, Standing, anchor_portals, anchor_portals_through};
pub use wall::{ShapeError, generate_walls, split_wall};
