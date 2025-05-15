/// The version of DungeonRS.
/// This is used for releases as well as determining the save file versioning.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The number of world units per cell.
/// This determines how "wide" cells in the map grid are.
pub const WORLD_UNITS_PER_CELL: u32 = 100;
