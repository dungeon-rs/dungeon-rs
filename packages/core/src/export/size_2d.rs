use serde::{Deserialize, Serialize};

/// Represents the size of a frame in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Size2D {
    /// The width in pixels.
    pub width: u32,
    /// The height in pixels.
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
