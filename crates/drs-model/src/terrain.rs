//! Terrain: painted ground of one Material, held as the strokes a Brush laid, and the coverage
//! derived from them for drawing.

use crate::{
    AssetReferenceRow, ElementKindName, Serialisable, SerialisationError, Tier, read_only_version,
};
use bevy_ecs::component::Component;
use bevy_ecs::reflect::ReflectComponent;
use bevy_math::{Rect, Vec2};
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::collections::BTreeMap;
use std::sync::Arc;

/// The Terrain kind.
pub const TERRAIN: ElementKindName = ElementKindName::new("terrain");

/// The settings a Brush lays a stroke with. A stroke keeps the settings it was laid with, and
/// the Editor's current Brush is one of these too, so what the Editor sets is what a stroke
/// records.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BrushSettings {
    /// The diameter the Brush covers, in Grid cells; above zero.
    pub size: f32,
    /// How much of the radius shows the full strength, from 0, a fully soft edge, to 1, a hard
    /// one.
    pub hardness: f32,
    /// How much of the Material a stroke shows where it shows the most, above 0 up to 1.
    pub strength: f32,
}

impl BrushSettings {
    /// Half the size, in Grid cells.
    #[must_use]
    pub fn radius(&self) -> f32 {
        self.size / 2.0
    }

    /// Why the settings are not a Brush's, if they are not: a size not above zero or not finite,
    /// a hardness outside 0 to 1, or a strength not above 0 or above 1.
    #[must_use]
    pub fn malformation(&self) -> Option<String> {
        if !(self.size > 0.0 && self.size.is_finite()) {
            return Some(format!(
                "a Brush's size must be above zero, not {}",
                self.size
            ));
        }
        if !(0.0..=1.0).contains(&self.hardness) {
            return Some(format!(
                "a Brush's hardness must be from 0 to 1, not {}",
                self.hardness
            ));
        }
        if !(self.strength > 0.0 && self.strength <= 1.0) {
            return Some(format!(
                "a Brush's strength must be above 0 and at most 1, not {}",
                self.strength
            ));
        }
        None
    }
}

/// One pass of a Brush along a path, from press to release, that paints or erases.
#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    /// The path: one or more points in Grid cells, a straight segment from each to the next; a
    /// single point is a round dab.
    pub points: Vec<Vec2>,
    /// The settings it was laid with.
    pub brush: BrushSettings,
    /// Whether it erases: an erase lowers the coverage the strokes laid before it give, where a
    /// stroke that paints raises it.
    pub erase: bool,
}

impl Stroke {
    /// The centre of the smallest box holding every point of the path: where a stroke is when it
    /// is moved whole, or the origin for a path of no point.
    #[must_use]
    pub fn centre(&self) -> Vec2 {
        let mut points = self.points.iter().copied();
        let Some(first) = points.next() else {
            return Vec2::ZERO;
        };
        let (min, max) = points.fold((first, first), |(min, max), point| {
            (min.min(point), max.max(point))
        });
        min.midpoint(max)
    }

    /// The smallest box holding every point of the path, grown by the Brush's radius on every
    /// side: outside it, the stroke covers nothing.
    #[must_use]
    pub fn reach(&self) -> Rect {
        let mut points = self.points.iter().copied();
        let Some(first) = points.next() else {
            return Rect::default();
        };
        let (min, max) = points.fold((first, first), |(min, max), point| {
            (min.min(point), max.max(point))
        });
        let radius = Vec2::splat(self.brush.radius());
        Rect {
            min: min - radius,
            max: max + radius,
        }
    }

    /// Why the stroke is not one, if it is not: no point, a point that is not finite, or Brush
    /// settings that are not a Brush's.
    #[must_use]
    pub fn malformation(&self) -> Option<String> {
        if self.points.is_empty() {
            return Some("a stroke needs one or more points".to_owned());
        }
        if !self.points.iter().all(|point| point.is_finite()) {
            return Some("a stroke's points must be finite".to_owned());
        }
        self.brush.malformation()
    }
}

/// Painted ground: one Material and the strokes painted and erased with it, in the order they
/// were laid.
///
/// A Terrain keeps its strokes, never pixels: its coverage at a point, how much of the Material
/// shows there, starts at none and is composited from its strokes in order, a stroke that paints
/// raising it to its own coverage where that is higher and an erase lowering it to one minus its
/// own where that is lower, computed afresh at whatever resolution it is drawn at.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Terrain {
    /// The image the Terrain's built-in Material tiles: the row of the Project's Asset Reference
    /// table that names it.
    pub image: AssetReferenceRow,
    /// The strokes, the first laid first.
    pub strokes: Vec<Stroke>,
}

impl Terrain {
    /// The box an Element of this Terrain has: the smallest box holding every point of every
    /// stroke grown by that stroke's radius on every side, or a box of no size at the origin for
    /// a Terrain without strokes.
    #[must_use]
    pub fn element_box(&self) -> Rect {
        let mut reaches = self.strokes.iter().map(Stroke::reach);
        let Some(first) = reaches.next() else {
            return Rect::default();
        };
        reaches.fold(first, |all, reach| all.union(reach))
    }

    /// Why the Terrain is not one, if it is not: a stroke that is not one, named by its number
    /// counted from zero.
    #[must_use]
    pub fn malformation(&self) -> Option<String> {
        self.strokes.iter().enumerate().find_map(|(index, stroke)| {
            stroke
                .malformation()
                .map(|reason| format!("stroke {index}: {reason}"))
        })
    }
}

/// A Terrain as version one of its data holds it, before strokes could erase.
#[derive(Deserialize)]
struct TerrainVersionOne {
    /// The row of its image.
    image: AssetReferenceRow,
    /// The strokes, every one painting.
    strokes: Vec<StrokeVersionOne>,
}

/// A stroke as version one of a Terrain's data holds it: a path and Brush settings.
#[derive(Deserialize)]
struct StrokeVersionOne {
    /// The path.
    points: Vec<Vec2>,
    /// The settings it was laid with.
    brush: BrushSettings,
}

impl From<TerrainVersionOne> for Terrain {
    /// The same Terrain with every stroke painting.
    fn from(old: TerrainVersionOne) -> Self {
        Self {
            image: old.image,
            strokes: old
                .strokes
                .into_iter()
                .map(|stroke| Stroke {
                    points: stroke.points,
                    brush: stroke.brush,
                    erase: false,
                })
                .collect(),
        }
    }
}

impl Serialisable for Terrain {
    const NAME: &'static str = "terrain";
    const VERSION: u32 = 2;
    const TIER: Tier = Tier::Element;

    /// Reads version two, and version one, from before strokes could erase, with every stroke
    /// painting; either is refused for the reason the Terrain's own check gives.
    fn read(version: u32, data: &RawValue) -> Result<Self, SerialisationError> {
        let terrain: Self = if version == 1 {
            serde_json::from_str::<TerrainVersionOne>(data.get())
                .map(Self::from)
                .map_err(|error| SerialisationError::Malformed {
                    component: Self::NAME.to_owned(),
                    reason: error.to_string(),
                })?
        } else {
            read_only_version(version, data)?
        };
        match terrain.malformation() {
            Some(reason) => Err(SerialisationError::Malformed {
                component: Self::NAME.to_owned(),
                reason,
            }),
            None => Ok(terrain),
        }
    }
}

/// How many coverage pixels span one Grid cell in a [`TerrainCoverage`].
pub const COVERAGE_PIXELS_PER_CELL: u32 = 32;

/// How many pixels a side a tile of a [`TerrainCoverage`] has.
pub const COVERAGE_TILE_PIXELS: u32 = 512;

/// How many Grid cells a side a tile of a [`TerrainCoverage`] covers.
pub const COVERAGE_TILE_CELLS: u32 = COVERAGE_TILE_PIXELS / COVERAGE_PIXELS_PER_CELL;

/// The position of a coverage tile in the Level's pixel plane: the tile `(x, y)` covers the
/// cells from `(x, y) × 16` up to the next tile's, negative positions included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TileKey {
    /// The column, counted to the right from the tile whose lower-left corner is the origin.
    pub x: i32,
    /// The row, counted upwards from the tile whose lower-left corner is the origin.
    pub y: i32,
}

impl TileKey {
    /// The lower-left corner of the tile, in Grid cells.
    #[must_use]
    pub fn corner(self) -> Vec2 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "tile positions are far below where f32 loses whole numbers"
        )]
        let cells = |tile: i32| tile as f32 * COVERAGE_TILE_CELLS as f32;
        Vec2::new(cells(self.x), cells(self.y))
    }
}

/// One tile of a Terrain's coverage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageTile {
    /// A number that changes whenever the tile's pixels do, and only then.
    pub revision: u64,
    /// One byte a pixel, `512 × 512` of them, rows from the tile's top; 255 is full coverage.
    /// The buffer is shared with whoever computed it, so publishing it copies no pixels.
    pub pixels: Arc<[u8]>,
}

/// A Terrain's coverage at [`COVERAGE_PIXELS_PER_CELL`], derived from its strokes and never
/// saved: the tiles some stroke covers, by position; an absent tile is empty.
///
/// The authoring Manager writes it whenever the Terrain's strokes change, after every Manager
/// has handled its Commands, Undo, and Redo; the render Engine draws it.
#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct TerrainCoverage {
    /// The tiles, by position.
    pub tiles: BTreeMap<TileKey, CoverageTile>,
}
