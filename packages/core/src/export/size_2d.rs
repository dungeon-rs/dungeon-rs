use serde::{Deserialize, Serialize};

/// Represents a size in world units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Size2D {
    /// The width in world units.
    pub width: u32,
    /// The height in world units.
    pub height: u32,
}

impl Size2D {
    /// Creates a new [`Size2D`] from width and height.
    #[inline(always)]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Creates a new [`Size2D`] with the same width and height.
    #[inline(always)]
    pub const fn splat(size: u32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }
}
