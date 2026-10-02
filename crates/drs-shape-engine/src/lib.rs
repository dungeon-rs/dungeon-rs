#![doc = include_str!("../README.md")]

mod path;
mod portal;
mod room;
mod wall;

pub use path::Path;
pub use portal::{PointEdit, PortalSetting, Standing, anchor_portals, anchor_portals_through};
pub use room::{CombinedOutline, combine_outlines};
pub use wall::{ShapeError, generate_walls, split_wall};
