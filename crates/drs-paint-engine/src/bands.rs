//! The bands a Terrain's coverage is held at and the tiles of a band: which band the zoom calls
//! for, which tiles a stroke reaches, which meet the view, and which lie out of its reach.

use bevy_math::Rect;
use drs_model::{COVERAGE_BANDS, COVERAGE_PIXELS_PER_CELL, Stroke, TileKey, tile_cells};

/// The base band: held wherever the strokes reach, whatever the zoom.
pub(crate) const BASE: u32 = COVERAGE_PIXELS_PER_CELL;

/// How far below a band the zoom may go before another band is taken, as a fraction of the band.
const LOWEST: f32 = 0.9;

/// How far above a band the zoom may go before another band is taken, as a multiple of the band.
const HIGHEST: f32 = 2.2;

/// The band the zoom calls for: the largest band not above it, or the base when the zoom is below
/// every band but the base.
pub(crate) fn floor_band(zoom: f32) -> u32 {
    COVERAGE_BANDS
        .iter()
        .rev()
        .copied()
        .find(|band| band_f32(*band) <= zoom)
        .unwrap_or(BASE)
}

/// The band that is active after the zoom changed to `zoom` with `active` active: `active` while
/// the zoom is at least 0.9 times it, the base excepted, and at most 2.2 times it, the largest
/// band excepted; otherwise the floor band of the zoom.
pub(crate) fn next_band(active: u32, zoom: f32) -> u32 {
    let largest = COVERAGE_BANDS[COVERAGE_BANDS.len() - 1];
    let above_low = active == BASE || zoom >= LOWEST * band_f32(active);
    let below_high = active == largest || zoom <= HIGHEST * band_f32(active);
    if above_low && below_high {
        active
    } else {
        floor_band(zoom)
    }
}

/// A band as a number of pixels per cell to compute with.
fn band_f32(band: u32) -> f32 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "a band is a small whole number, exact in f32"
    )]
    let band = band as f32;
    band
}

/// The column or row of the tile holding `cells` along one axis at a tile side of `side` cells.
fn tile_at(cells: f32, side: f32) -> i32 {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the tiles of a Level are far inside the range of i32"
    )]
    let tile = (cells / side).floor() as i32;
    tile
}

/// An inclusive range of tiles, from the lowest to the highest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Span {
    /// The lower-left tile.
    pub(crate) low: TileKey,
    /// The upper-right tile.
    pub(crate) high: TileKey,
}

impl Span {
    /// Whether the span holds `key`.
    pub(crate) fn contains(self, key: TileKey) -> bool {
        (self.low.x..=self.high.x).contains(&key.x) && (self.low.y..=self.high.y).contains(&key.y)
    }

    /// Every tile of the span, row by row from the bottom.
    pub(crate) fn keys(self) -> impl Iterator<Item = TileKey> {
        (self.low.y..=self.high.y)
            .flat_map(move |y| (self.low.x..=self.high.x).map(move |x| TileKey { x, y }))
    }

    /// The tiles both spans hold, if any.
    pub(crate) fn meet(self, other: Self) -> Option<Self> {
        let low = TileKey {
            x: self.low.x.max(other.low.x),
            y: self.low.y.max(other.low.y),
        };
        let high = TileKey {
            x: self.high.x.min(other.high.x),
            y: self.high.y.min(other.high.y),
        };
        (low.x <= high.x && low.y <= high.y).then_some(Self { low, high })
    }
}

/// The tiles at `band` a stroke may cover: those its reach, a pixel wider either way, meets.
pub(crate) fn reached(stroke: &Stroke, band: u32) -> Span {
    let reach = stroke.reach();
    let side = tile_cells(band);
    let margin = 1.0 / band_f32(band);
    Span {
        low: TileKey {
            x: tile_at(reach.min.x - margin, side),
            y: tile_at(reach.min.y - margin, side),
        },
        high: TileKey {
            x: tile_at(reach.max.x + margin, side),
            y: tile_at(reach.max.y + margin, side),
        },
    }
}

/// The tiles at `band` that meet `view`, a rectangle of cells, or `None` for a view of no size.
pub(crate) fn meeting(view: Rect, band: u32) -> Option<Span> {
    if !(view.width() > 0.0 && view.height() > 0.0) {
        return None;
    }
    let side = tile_cells(band);
    // A tile whose edge only touches the view's does not meet it.
    let last = |cells: f32| {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "the tiles of a Level are far inside the range of i32"
        )]
        let tile = (cells / side).ceil() as i32;
        tile - 1
    };
    Some(Span {
        low: TileKey {
            x: tile_at(view.min.x, side),
            y: tile_at(view.min.y, side),
        },
        high: TileKey {
            x: last(view.max.x),
            y: last(view.max.y),
        },
    })
}

/// Whether some part of the tile `key` at `band` lies within one tile's width of `view`.
pub(crate) fn within_reach(key: TileKey, view: Rect, band: u32) -> bool {
    let side = tile_cells(band);
    let corner = key.corner_at(band);
    let grown = view.inflate(side);
    corner.x < grown.max.x
        && corner.x + side > grown.min.x
        && corner.y < grown.max.y
        && corner.y + side > grown.min.y
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        reason = "a test stops at the first thing that is not as expected"
    )]
    use super::*;
    use bevy_math::Vec2;
    use drs_model::BrushSettings;

    /// The floor band is the largest band not above the zoom, the base below 64.
    #[test]
    fn the_floor_band_is_the_largest_not_above_the_zoom() {
        for (zoom, band) in [
            (4.0, 32),
            (20.0, 32),
            (40.0, 32),
            (63.9, 32),
            (64.0, 64),
            (100.0, 64),
            (127.0, 64),
            (128.0, 128),
            (255.0, 128),
            (256.0, 256),
            (300.0, 256),
            (1024.0, 256),
        ] {
            assert_eq!(floor_band(zoom), band, "zoom {zoom}");
        }
    }

    /// A band holds from 0.9 to 2.2 times itself, the base with no lower bound and the band of 256
    /// with no upper bound, and gives way to the floor band of the zoom outside.
    #[test]
    fn a_band_holds_within_its_margin() {
        let steps = [
            (64, 58.0, 64),
            (64, 57.0, 32),
            (32, 70.0, 32),
            (32, 71.0, 64),
            (64, 140.0, 64),
            (64, 141.0, 128),
            (32, 4.0, 32),
            (256, 1024.0, 256),
            (256, 231.0, 256),
            (256, 230.0, 128),
            (128, 600.0, 256),
            (256, 40.0, 32),
        ];
        for (active, zoom, band) in steps {
            assert_eq!(next_band(active, zoom), band, "{active} at zoom {zoom}");
        }
        let mut active = 64;
        for frame in 0..200_u16 {
            let wobble = f32::from(frame % 13) / 12.0 * 0.12 - 0.06;
            active = next_band(active, 64.0 * (1.0 + wobble));
            assert_eq!(active, 64, "frame {frame}");
        }
    }

    /// The tiles that meet a view are those it overlaps, not those whose edge it only touches,
    /// negative ones included, and a view of no size meets none.
    #[test]
    fn the_tiles_meeting_a_view() {
        let view = Rect::new(-1.0, 3.0, 8.0, 12.0);
        assert_eq!(
            meeting(view, 128),
            Some(Span {
                low: TileKey { x: -1, y: 0 },
                high: TileKey { x: 1, y: 2 },
            })
        );
        assert_eq!(
            meeting(view, 64),
            Some(Span {
                low: TileKey { x: -1, y: 0 },
                high: TileKey { x: 0, y: 1 },
            })
        );
        assert_eq!(meeting(Rect::new(2.0, 2.0, 2.0, 5.0), 64), None);
    }

    /// A tile is within reach of a view while some part of it lies less than its side from the
    /// view, and out of reach beyond.
    #[test]
    fn the_tiles_out_of_reach() {
        let view = Rect::new(0.0, 0.0, 8.0, 8.0);
        let band = 128;
        for (key, within) in [
            (TileKey { x: 0, y: 0 }, true),
            (TileKey { x: 2, y: 2 }, true),
            (TileKey { x: 2, y: 3 }, false),
            (TileKey { x: 3, y: 0 }, false),
            (TileKey { x: -1, y: -1 }, true),
            (TileKey { x: -2, y: 0 }, false),
        ] {
            assert_eq!(within_reach(key, view, band), within, "{key:?}");
        }
    }

    /// The tiles a stroke reaches are those its reach a pixel wider meets, at any band.
    #[test]
    fn the_tiles_a_stroke_reaches() {
        let stroke = Stroke {
            points: vec![Vec2::new(-0.5, 3.9), Vec2::new(7.9, 4.2)],
            brush: BrushSettings {
                size: 0.2,
                hardness: 0.5,
                strength: 1.0,
            },
            erase: false,
        };
        assert_eq!(
            reached(&stroke, 128),
            Span {
                low: TileKey { x: -1, y: 0 },
                high: TileKey { x: 2, y: 1 },
            }
        );
        assert_eq!(
            reached(&stroke, BASE),
            Span {
                low: TileKey { x: -1, y: 0 },
                high: TileKey { x: 0, y: 0 },
            }
        );
    }
}
