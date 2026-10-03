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
    #[cfg(test)]
    fn tiles(&self) -> &BTreeMap<TileKey, CoverageTile> {
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
    /// nothing is covered; whether the tiles changed.
    fn store(&mut self, key: TileKey, pixels: Vec<u8>) -> bool {
        if pixels.iter().all(|pixel| *pixel == 0) {
            return self.tiles.remove(&key).is_some();
        }
        if self
            .tiles
            .get(&key)
            .is_some_and(|tile| *tile.pixels == *pixels)
        {
            return false;
        }
        self.revision += 1;
        self.tiles.insert(
            key,
            CoverageTile {
                revision: self.revision,
                pixels: Arc::from(pixels),
            },
        );
        true
    }
}

/// What bringing a cache up to its strokes did.
#[derive(Debug, Default)]
struct Applied {
    /// The tiles rasterized.
    touched: BTreeSet<TileKey>,
    /// Whether any tile's pixels changed, or a tile came or went.
    changed: bool,
}

/// The region a tile covers.
fn region(key: TileKey) -> Region {
    Region::at(
        key.corner(),
        UVec2::splat(COVERAGE_TILE_PIXELS),
        COVERAGE_PIXELS_PER_CELL,
    )
}

/// The lowest and the highest tile a stroke may cover: those its box, a pixel wider either way,
/// reaches.
fn tile_span(stroke: &Stroke) -> (TileKey, TileKey) {
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
    (
        TileKey {
            x: tile(reach.min.x - margin),
            y: tile(reach.min.y - margin),
        },
        TileKey {
            x: tile(reach.max.x + margin),
            y: tile(reach.max.y + margin),
        },
    )
}

/// The tiles a stroke may cover.
fn tiles_of(stroke: &Stroke) -> impl Iterator<Item = TileKey> {
    let (low, high) = tile_span(stroke);
    (low.y..=high.y).flat_map(move |y| (low.x..=high.x).map(move |x| TileKey { x, y }))
}

/// `ApplyStroke`: brings `cache` up to `strokes`, a Terrain's strokes in order, and says whether
/// any tile changed, so the coverage published from it needs publishing again.
///
/// The strokes the cache holds and `strokes` are compared from both ends: those they share from
/// the first onwards and from the last backwards are kept. When only new strokes follow the
/// shared start, they are composited onto the tiles they touch, an erase as much as a paint.
/// Otherwise only the tiles touched by the strokes between the shared ends, as they were and as
/// they are, are rasterized again from every stroke that reaches them, so moving, changing, or
/// removing one stroke, and undoing any of these, recomputes that stroke's tiles alone however
/// many strokes there are. Either way the tiles end up holding exactly what rasterizing every
/// stroke afresh gives.
pub fn apply_stroke(cache: &mut PaintCache, strokes: &[Stroke]) -> bool {
    bring_up(cache, strokes).changed
}

/// Brings `cache` up to `strokes` as [`apply_stroke`] does, and tells which tiles it rasterized.
fn bring_up(cache: &mut PaintCache, strokes: &[Stroke]) -> Applied {
    let kept = cache
        .strokes
        .iter()
        .zip(strokes)
        .take_while(|(before, now)| before == now)
        .count();
    let mut applied = Applied::default();
    if kept == cache.strokes.len() {
        for stroke in &strokes[kept..] {
            for key in tiles_of(stroke) {
                let region = region(key);
                let mut pixels = cache.tiles.get(&key).map_or_else(
                    || vec![0; (COVERAGE_TILE_PIXELS * COVERAGE_TILE_PIXELS) as usize],
                    |tile| tile.pixels.to_vec(),
                );
                composite(&mut pixels, stroke, &region);
                applied.changed |= cache.store(key, pixels);
                applied.touched.insert(key);
            }
        }
        cache.strokes.extend_from_slice(&strokes[kept..]);
        return applied;
    }
    let shortest = cache.strokes.len().min(strokes.len());
    let ending = cache.strokes[kept..]
        .iter()
        .rev()
        .zip(strokes[kept..].iter().rev())
        .take(shortest - kept)
        .take_while(|(before, now)| before == now)
        .count();
    let end = cache.strokes.len() - ending;
    let is = &strokes[kept..strokes.len() - ending];
    applied.touched = cache.strokes[kept..end]
        .iter()
        .chain(is)
        .flat_map(tiles_of)
        .collect();
    let spans: Vec<(TileKey, TileKey)> = strokes.iter().map(tile_span).collect();
    for key in &applied.touched {
        let region = region(*key);
        let mut pixels = vec![0; (COVERAGE_TILE_PIXELS * COVERAGE_TILE_PIXELS) as usize];
        let reaching = strokes.iter().zip(&spans).filter(|(_, (low, high))| {
            (low.x..=high.x).contains(&key.x) && (low.y..=high.y).contains(&key.y)
        });
        for (stroke, _) in reaching {
            composite(&mut pixels, stroke, &region);
        }
        applied.changed |= cache.store(*key, pixels);
    }
    let shared_end = cache.strokes.split_off(end);
    cache.strokes.truncate(kept);
    cache.strokes.extend_from_slice(is);
    cache.strokes.extend(shared_end);
    applied
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
    use drs_model::BrushSettings;

    /// Four strokes over and across tile edges, negative cells included.
    fn strokes() -> Vec<Stroke> {
        let brush = |size, hardness, strength| BrushSettings {
            size,
            hardness,
            strength,
        };
        vec![
            Stroke {
                points: vec![Vec2::new(-3.0, 2.0), Vec2::new(18.0, 5.5)],
                brush: brush(2.0, 0.5, 1.0),
                erase: false,
            },
            Stroke {
                points: vec![Vec2::new(15.5, 15.5)],
                brush: brush(3.0, 0.0, 0.6),
                erase: false,
            },
            Stroke {
                points: vec![
                    Vec2::new(40.0, -2.0),
                    Vec2::new(41.0, 3.0),
                    Vec2::new(37.0, 1.0),
                ],
                brush: brush(1.0, 1.0, 0.8),
                erase: false,
            },
            Stroke {
                points: vec![Vec2::new(2.0, 4.0), Vec2::new(9.0, 6.0)],
                brush: brush(4.0, 0.3, 0.4),
                erase: false,
            },
        ]
    }

    /// The four strokes and two erases laid after them: one across the first stroke and the
    /// tile edge, and a weaker one over the dab and beyond everything painted.
    fn with_erases() -> Vec<Stroke> {
        let mut all = strokes();
        all.push(Stroke {
            points: vec![Vec2::new(14.0, -1.0), Vec2::new(17.5, 9.0)],
            brush: BrushSettings {
                size: 2.5,
                hardness: 0.4,
                strength: 1.0,
            },
            erase: true,
        });
        all.push(Stroke {
            points: vec![Vec2::new(15.5, 15.5), Vec2::new(60.0, 40.0)],
            brush: BrushSettings {
                size: 2.0,
                hardness: 0.0,
                strength: 0.5,
            },
            erase: true,
        });
        all
    }

    /// The revision of every tile a cache holds.
    fn revisions(cache: &PaintCache) -> BTreeMap<TileKey, u64> {
        cache
            .tiles()
            .iter()
            .map(|(key, tile)| (*key, tile.revision))
            .collect()
    }

    /// Asserts that every tile not in `touched` kept the revision it had in `before`.
    fn others_kept(
        cache: &PaintCache,
        before: &BTreeMap<TileKey, u64>,
        touched: &BTreeSet<TileKey>,
    ) {
        for (key, tile) in cache.tiles() {
            if !touched.contains(key) {
                assert_eq!(
                    Some(&tile.revision),
                    before.get(key),
                    "tile {key:?} kept its revision"
                );
            }
        }
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
            assert!(
                apply_stroke(&mut cache, &strokes[..laid]),
                "stroke {laid} shows"
            );
        }
        assert!(!apply_stroke(&mut cache, &strokes), "nothing new");
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
        bring_up(&mut cache, &strokes[..3]);
        let before = pixels(&cache);
        let laid = bring_up(&mut cache, &strokes).touched;
        let revisions: BTreeMap<TileKey, u64> = cache
            .tiles()
            .iter()
            .map(|(key, tile)| (*key, tile.revision))
            .collect();

        let undone = bring_up(&mut cache, &strokes[..3]).touched;

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

    /// Erases composited one by one onto the tiles they touch give exactly what rasterizing every
    /// stroke afresh gives, and an erase reaching beyond everything painted adds no tile.
    #[test]
    fn appending_an_erase_equals_rasterizing() {
        let strokes = with_erases();
        let mut cache = PaintCache::default();
        bring_up(&mut cache, &strokes[..4]);
        let painted = pixels(&cache);
        for laid in 5..=strokes.len() {
            assert!(
                apply_stroke(&mut cache, &strokes[..laid]),
                "erase {laid} shows"
            );
            assert_eq!(pixels(&cache), afresh(&strokes[..laid]));
        }
        assert_ne!(pixels(&cache), painted);
        assert!(
            cache.tiles().keys().all(|key| painted.contains_key(key)),
            "no tile beyond the painted ones"
        );
    }

    /// Moving one stroke rasterizes only the tiles it touched before and touches after, and
    /// every other tile keeps its revision.
    #[test]
    fn an_edit_touches_only_its_tiles() {
        let before = with_erases();
        let mut cache = PaintCache::default();
        bring_up(&mut cache, &before);
        let revisions = revisions(&cache);
        let mut after = before.clone();
        after[1].points = vec![Vec2::new(40.0, 20.0)];

        let touched = bring_up(&mut cache, &after).touched;

        let expected: BTreeSet<TileKey> = tiles_of(&before[1]).chain(tiles_of(&after[1])).collect();
        assert_eq!(touched, expected);
        assert_eq!(pixels(&cache), afresh(&after));
        others_kept(&cache, &revisions, &touched);
        assert!(
            cache.tiles().keys().any(|key| !touched.contains(key)),
            "some tile was left alone"
        );
    }

    /// Removing a stroke between others, and putting it back, each rasterize only the tiles that
    /// stroke touches, and putting it back leaves the tiles as they were.
    #[test]
    fn a_removal_touches_only_its_tiles() {
        let all = with_erases();
        let mut cache = PaintCache::default();
        bring_up(&mut cache, &all);
        let laid = pixels(&cache);
        let revisions = revisions(&cache);
        let mut without = all.clone();
        let removed = without.remove(2);

        let touched = bring_up(&mut cache, &without).touched;
        assert_eq!(touched, tiles_of(&removed).collect());
        assert_eq!(pixels(&cache), afresh(&without));
        others_kept(&cache, &revisions, &touched);

        let back = bring_up(&mut cache, &all).touched;
        assert_eq!(back, touched);
        assert_eq!(pixels(&cache), laid);
    }

    /// After each edit of a stroke, a point moved, the stroke moved, its Brush settings changed,
    /// turned to erasing and back, removed, and put back, the tiles hold exactly what rasterizing
    /// every stroke afresh gives.
    #[test]
    fn an_edit_equals_rasterizing() {
        let mut strokes = with_erases();
        let mut cache = PaintCache::default();
        bring_up(&mut cache, &strokes);
        let edits: [&dyn Fn(&mut Vec<Stroke>); 7] = [
            &|strokes| strokes[0].points[1] = Vec2::new(10.0, 12.0),
            &|strokes| {
                for point in &mut strokes[2].points {
                    *point += Vec2::new(-30.0, 9.0);
                }
            },
            &|strokes| {
                strokes[3].brush = BrushSettings {
                    size: 6.0,
                    hardness: 0.9,
                    strength: 0.3,
                };
            },
            &|strokes| strokes[0].erase = true,
            &|strokes| strokes[4].erase = false,
            &|strokes| {
                strokes.remove(1);
            },
            &|strokes| strokes.insert(3, strokes[4].clone()),
        ];
        for (number, edit) in edits.iter().enumerate() {
            edit(&mut strokes);
            bring_up(&mut cache, &strokes);
            assert_eq!(pixels(&cache), afresh(&strokes), "edit {number}");
        }
    }
}
