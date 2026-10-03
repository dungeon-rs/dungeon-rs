#![doc = include_str!("../README.md")]

mod combine;
mod path;
mod portal;
mod room;
mod snap;
mod wall;

pub use combine::{Combination, CombinedOutline, Outline, WallLine, combine_outlines};
pub use path::Path;
pub use portal::{PointEdit, PortalSetting, Standing, anchor_portals, anchor_portals_through};
pub use snap::{SnapTarget, snap};
pub use wall::{ShapeError, generate_walls, split_wall};
