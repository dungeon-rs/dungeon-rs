#![doc = include_str!("../README.md")]

mod portal;
mod room;
mod wall;

pub use portal::{
    PointEdit, PortalSetting, Standing, anchor_portals, anchor_portals_through,
    anchor_room_portals, anchor_room_portals_through,
};
pub use room::{Outline, combine_outlines, generate_room_walls, split_room};
pub use wall::{ShapeError, generate_walls, split_wall};
