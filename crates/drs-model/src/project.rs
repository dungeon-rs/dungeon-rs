//! The Project and its structure: Levels, Layers, the Grid, and the Bounds.

use crate::{AssetReferences, ResolutionTable};
use bevy_ecs::component::Component;
use bevy_ecs::reflect::ReflectComponent;
use bevy_math::{IVec2, UVec2};
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

/// The Author's work: an entity whose children are its Levels.
///
/// The entity also carries the Project's [`Grid`], [`Bounds`], and [`AssetReferences`], and the
/// [`ResolutionTable`] that says where each Asset Reference loads from on this device.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[reflect(Component)]
#[require(Grid, Bounds, AssetReferences, ResolutionTable)]
pub struct Project {
    /// The name the Author knows the Project by: the name of its file, or `Untitled`. It is the
    /// file's, so it is not written into the file; a Project read from a file is named after it.
    #[serde(skip)]
    pub name: String,
}

/// The Project's grid of square cells, which sets the scale of every Level.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Grid {
    /// How many image pixels make one cell: an Asset's natural size in cells is its pixel size
    /// divided by this.
    pub pixels_per_cell: u32,
}

impl Default for Grid {
    /// The convention of the libraries the editor is built for: 256 pixels per cell.
    fn default() -> Self {
        Self {
            pixels_per_cell: 256,
        }
    }
}

/// The rectangle, in Grid cells and shared by all Levels, that decides what is exported.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Bounds {
    /// The cell at the lower-left corner.
    pub origin: IVec2,
    /// The width and height in cells.
    pub size: UVec2,
}

impl Default for Bounds {
    /// Thirty by thirty cells from the origin.
    fn default() -> Self {
        Self {
            origin: IVec2::ZERO,
            size: UVec2::splat(30),
        }
    }
}

/// A complete, independent map within a Project: an entity whose children are its Layers.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Level {
    /// The Level's name.
    pub name: String,
}

/// A named slice of a Level: an entity whose children are its Elements in stacking order.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Layer {
    /// The Layer's name.
    pub name: String,
}
