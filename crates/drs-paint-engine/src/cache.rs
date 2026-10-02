//! `ApplyStroke`: the tiled pixel cache of a Terrain at the resident base band, kept up to its
//! strokes by rasterizing only the tiles the strokes that changed touch.

use crate::rasterize::{Region, composite};
use bevy_math::UVec2;
use drs_model::{
    COVERAGE_PIXELS_PER_CELL, COVERAGE_TILE_CELLS, COVERAGE_TILE_PIXELS, CoverageTile, Stroke,
    TerrainCoverage, TileKey,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// The coverage of one Terrain's strokes at [`COVERAGE_PIXELS_PER_CELL`], in tiles of
/// [`COVERAGE_TILE_PIXELS`] a side keyed by their place in the Level's pixel plane, and the
/// strokes it was rasterized from.
///
/// Only tiles some stroke covers are held; an absent tile is empty, so two caches of the same
/// strokes hold the same tiles however the strokes came to be.
#[derive(Debug, Clone, Default)]
pub struct PaintCache {
    /// The strokes the tiles hold, in order.
    strokes: Vec<Stroke>,
    /// The tiles some stroke covers.
    tiles: BTreeMap<TileKey, CoverageTile>,
    /// The revision the last changed tile was given.
    revision: u64,
}

impl PaintCache {
    /// The tiles some stroke covers, by place.
    #[must_use]
    pub fn tiles(&self) -> &BTreeMap<TileKey, CoverageTile> {
        &self.tiles
    }

    /// The coverage as the model holds it, sharing the tiles' pixels rather than copying them.
    #[must_use]
    pub fn coverage(&self) -> TerrainCoverage {
        TerrainCoverage {
            tiles: self.tiles.clone(),
        }
    }

    /// Replaces a tile's pixels, giving it a new revision when they differ, and drops it when
    /// nothing is covered.
    fn store(&mut self, key: TileKey, pixels: Vec<u8>) {
        if pixels.iter().all(|pixel| *pixel == 0) {
            self.tiles.remove(&key);
            return;
        }
        if self
            .tiles
            .get(&key)
            .is_some_and(|tile| *tile.pixels == *pixels)
        {
            return;
        }
        self.revision += 1;
        self.tiles.insert(
            key,
            CoverageTile {
                revision: self.revision,
                pixels: Arc::from(pixels),
            },
        );
    }
}

/// The region a tile covers.
fn region(key: TileKey) -> Region {
    Region::at(
        key.corner(),
        UVec2::splat(COVERAGE_TILE_PIXELS),
        COVERAGE_PIXELS_PER_CELL,
    )
}

/// The tiles a stroke may cover: those its box, a pixel wider either way, reaches.
fn tiles_of(stroke: &Stroke) -> impl Iterator<Item = TileKey> {
    let reach = stroke.reach();
    #[expect(
        clippy::cast_precision_loss,
        reason = "a tile's cells and the cells' pixels are small whole numbers, exact in f32"
    )]
    let (cells, margin) = (
        COVERAGE_TILE_CELLS as f32,
        1.0 / COVERAGE_PIXELS_PER_CELL as f32,
    );
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the tiles a stroke reaches are far inside the range of i32"
    )]
    let tile = |cells_at: f32| (cells_at / cells).floor() as i32;
    let (left, right) = (tile(reach.min.x - margin), tile(reach.max.x + margin));
    let (bottom, top) = (tile(reach.min.y - margin), tile(reach.max.y + margin));
    (bottom..=top).flat_map(move |y| (left..=right).map(move |x| TileKey { x, y }))
}

/// `ApplyStroke`: brings `cache` up to `strokes`, a Terrain's strokes in order, and returns the
/// tiles it rasterized.
///
/// Strokes appended since the cache last ran are composited onto the tiles they touch. When an
/// earlier stroke changed or went, as an undo makes it go, only the tiles touched by the strokes
/// that differ, before or after, are rasterized again from every stroke, so undoing a stroke
/// recomputes that stroke's tiles alone. Either way the tiles end up holding exactly what
/// rasterizing every stroke afresh gives.
pub fn apply_stroke(cache: &mut PaintCache, strokes: &[Stroke]) -> BTreeSet<TileKey> {
    let kept = cache
        .strokes
        .iter()
        .zip(strokes)
        .take_while(|(before, now)| before == now)
        .count();
    let mut touched = BTreeSet::new();
    if kept == cache.strokes.len() {
        for stroke in &strokes[kept..] {
            for key in tiles_of(stroke) {
                let region = region(key);
                let mut pixels = cache.tiles.get(&key).map_or_else(
                    || vec![0; (COVERAGE_TILE_PIXELS * COVERAGE_TILE_PIXELS) as usize],
                    |tile| tile.pixels.to_vec(),
                );
                composite(&mut pixels, stroke, &region);
                cache.store(key, pixels);
                touched.insert(key);
            }
        }
    } else {
        touched = cache.strokes[kept..]
            .iter()
            .chain(&strokes[kept..])
            .flat_map(tiles_of)
            .collect();
        for key in &touched {
            let region = region(*key);
            let mut pixels = vec![0; (COVERAGE_TILE_PIXELS * COVERAGE_TILE_PIXELS) as usize];
            for stroke in strokes {
                composite(&mut pixels, stroke, &region);
            }
            cache.store(*key, pixels);
        }
    }
    cache.strokes = strokes.to_vec();
    touched
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        reason = "a test stops at the first thing that is not as expected"
    )]
    use super::*;
    use crate::rasterize;
    use bevy_math::Vec2;
    use drs_model::Brush;

    /// Four strokes over and across tile edges, negative cells included.
    fn strokes() -> Vec<Stroke> {
        let brush = |size, hardness, strength| Brush {
            size,
            hardness,
            strength,
        };
        vec![
            Stroke {
                points: vec![Vec2::new(-3.0, 2.0), Vec2::new(18.0, 5.5)],
                brush: brush(2.0, 0.5, 1.0),
            },
            Stroke {
                points: vec![Vec2::new(15.5, 15.5)],
                brush: brush(3.0, 0.0, 0.6),
            },
            Stroke {
                points: vec![
                    Vec2::new(40.0, -2.0),
                    Vec2::new(41.0, 3.0),
                    Vec2::new(37.0, 1.0),
                ],
                brush: brush(1.0, 1.0, 0.8),
            },
            Stroke {
                points: vec![Vec2::new(2.0, 4.0), Vec2::new(9.0, 6.0)],
                brush: brush(4.0, 0.3, 0.4),
            },
        ]
    }

    /// Every tile `strokes` cover, rasterized afresh over the whole of each tile.
    fn afresh(strokes: &[Stroke]) -> BTreeMap<TileKey, Vec<u8>> {
        strokes
            .iter()
            .flat_map(tiles_of)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|key| {
                (
                    key,
                    rasterize(
                        strokes,
                        key.corner(),
                        UVec2::splat(COVERAGE_TILE_PIXELS),
                        COVERAGE_PIXELS_PER_CELL,
                    ),
                )
            })
            .filter(|(_, pixels)| pixels.iter().any(|pixel| *pixel > 0))
            .collect()
    }

    /// The pixels of every tile a cache holds.
    fn pixels(cache: &PaintCache) -> BTreeMap<TileKey, Vec<u8>> {
        cache
            .tiles()
            .iter()
            .map(|(key, tile)| (*key, tile.pixels.to_vec()))
            .collect()
    }

    /// Strokes composited one by one onto the tiles they touch give exactly what rasterizing
    /// them all afresh gives.
    #[test]
    fn appending_equals_rasterizing() {
        let strokes = strokes();
        let mut cache = PaintCache::default();
        for laid in 1..=strokes.len() {
            apply_stroke(&mut cache, &strokes[..laid]);
        }
        assert_eq!(pixels(&cache), afresh(&strokes));
        assert!(cache.tiles().keys().any(|key| key.x < 0), "a negative tile");
        assert!(cache.tiles().len() >= 4, "{} tiles", cache.tiles().len());
    }

    /// Undoing a stroke rasterizes the tiles it touched and no other, keeps every other tile's
    /// revision, and leaves the tiles as they were before it was laid.
    #[test]
    fn an_undo_touches_only_its_tiles() {
        let strokes = strokes();
        let mut cache = PaintCache::default();
        apply_stroke(&mut cache, &strokes[..3]);
        let before = pixels(&cache);
        let laid = apply_stroke(&mut cache, &strokes);
        let revisions: BTreeMap<TileKey, u64> = cache
            .tiles()
            .iter()
            .map(|(key, tile)| (*key, tile.revision))
            .collect();

        let undone = apply_stroke(&mut cache, &strokes[..3]);

        assert_eq!(undone, laid);
        assert_eq!(undone, tiles_of(&strokes[3]).collect());
        assert_eq!(pixels(&cache), before);
        for (key, tile) in cache.tiles() {
            if !undone.contains(key) {
                assert_eq!(
                    tile.revision, revisions[key],
                    "tile {key:?} kept its revision"
                );
            }
        }
        assert!(
            cache.tiles().len() > undone.len(),
            "some tile was left alone"
        );
    }
}
