//! `ApplyStroke`: the tiled pixel cache of a Terrain, a resident base band and an overlay band
//! that follows the view, kept up to its strokes and to the view by rasterizing only the tiles
//! that changed.

use crate::bands::{BASE, Span, floor_band, meeting, next_band, reached, within_reach};
use crate::gpu::{Drawn, Gpu, StrokeRasterizer};
use crate::rasterize::{Region, composite};
use bevy_asset::Handle;
use bevy_image::Image;
use bevy_math::UVec2;
use drs_model::{
    COVERAGE_PIXELS_PER_CELL, COVERAGE_TILE_PIXELS, CoverageTile, Stroke, TerrainCoverage,
    TileContent, TileKey, Viewport,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// The coverage of one Terrain's strokes in tiles of [`COVERAGE_TILE_PIXELS`] a side, keyed by
/// their place in the Level's pixel plane at their band, and the strokes it was rasterized from.
///
/// Rasterized on the CPU, it holds the base band alone, and only the tiles some stroke covers, so
/// two caches of the same strokes hold the same tiles however the strokes came to be. Rasterized
/// on the GPU, it holds the base band wherever the strokes reach and, while the zoom calls for a
/// closer band, that band's tiles that meet the view and that some stroke reaches.
#[derive(Debug, Clone)]
pub struct PaintCache {
    /// The strokes the tiles hold, in order.
    strokes: Vec<Stroke>,
    /// The base band, held whatever the zoom.
    base: Band,
    /// The active band when it is not the base.
    overlay: Option<Band>,
    /// Whether the tiles are rasterized on the GPU, fixed when the cache is made.
    on_gpu: bool,
    /// Whether a view has chosen the active band yet.
    banded: bool,
    /// The revision the last rasterized tile was given.
    revision: u64,
}

/// The tiles of one band.
#[derive(Debug, Clone)]
struct Band {
    /// How many pixels one Grid cell spans.
    band: u32,
    /// The tiles held, by place.
    tiles: BTreeMap<TileKey, Held>,
}

impl Band {
    /// A band holding no tile.
    fn new(band: u32) -> Self {
        Self {
            band,
            tiles: BTreeMap::new(),
        }
    }
}

/// One tile held: as the model holds it, and its image when it is on the GPU.
#[derive(Debug, Clone)]
struct Held {
    /// The tile as published.
    tile: CoverageTile,
    /// The image on the GPU, for a tile rasterized there.
    image: Option<Handle<Image>>,
}

impl PaintCache {
    /// A cache of no stroke and no tile, at the base band, that rasterizes on the GPU when
    /// `rasterizer` has one, and otherwise on the CPU, for as long as it lives.
    #[must_use]
    pub fn new(rasterizer: &StrokeRasterizer) -> Self {
        Self::rasterized_on(rasterizer.has_gpu())
    }

    /// A cache of no stroke and no tile, at the base band, rasterized on the GPU when `on_gpu`
    /// says so and otherwise on the CPU.
    fn rasterized_on(on_gpu: bool) -> Self {
        Self {
            strokes: Vec::new(),
            base: Band::new(BASE),
            overlay: None,
            on_gpu,
            banded: false,
            revision: 0,
        }
    }

    /// The coverage as the model holds it: the tiles of the active band, sharing their pixels
    /// rather than copying them.
    #[must_use]
    pub fn coverage(&self) -> TerrainCoverage {
        let active = self.overlay.as_ref().unwrap_or(&self.base);
        TerrainCoverage {
            band: active.band,
            tiles: active
                .tiles
                .iter()
                .map(|(key, held)| (*key, held.tile.clone()))
                .collect(),
            base_tiles: self.base.tiles.len(),
        }
    }

    /// The band shown.
    fn active(&self) -> u32 {
        self.overlay.as_ref().map_or(BASE, |overlay| overlay.band)
    }

    /// The next revision.
    fn next_revision(&mut self) -> u64 {
        self.revision += 1;
        self.revision
    }

    /// Replaces a base tile's pixels on the CPU, giving it a new revision when they differ, and
    /// drops it when nothing is covered; whether the tiles changed.
    fn store(&mut self, key: TileKey, pixels: Vec<u8>) -> bool {
        if pixels.iter().all(|pixel| *pixel == 0) {
            return self.base.tiles.remove(&key).is_some();
        }
        if self
            .base
            .tiles
            .get(&key)
            .and_then(|held| held.tile.pixels())
            .is_some_and(|held| *held == *pixels)
        {
            return false;
        }
        let revision = self.next_revision();
        self.base.tiles.insert(
            key,
            Held {
                tile: CoverageTile {
                    revision,
                    content: TileContent::Pixels(Arc::from(pixels)),
                },
                image: None,
            },
        );
        true
    }

    /// Takes the strokes as they are now, keeping what both ends share.
    fn take_strokes(&mut self, strokes: &[Stroke], diff: &Diff) {
        let end = self.strokes.len() - diff.ending;
        let shared_end = self.strokes.split_off(end);
        self.strokes.truncate(diff.kept);
        self.strokes
            .extend_from_slice(&strokes[diff.kept..strokes.len() - diff.ending]);
        self.strokes.extend(shared_end);
    }
}

/// How the strokes a cache holds and a Terrain's strokes compare from both ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Diff {
    /// How many strokes both share from the first onwards.
    kept: usize,
    /// How many strokes both share from the last backwards, after the shared start.
    ending: usize,
    /// Whether only new strokes follow the shared start, or nothing at all.
    appended: bool,
}

impl Diff {
    /// How `before` and `now` compare.
    fn of(before: &[Stroke], now: &[Stroke]) -> Self {
        let kept = before
            .iter()
            .zip(now)
            .take_while(|(before, now)| before == now)
            .count();
        if kept == before.len() {
            return Self {
                kept,
                ending: 0,
                appended: true,
            };
        }
        let shortest = before.len().min(now.len());
        let ending = before[kept..]
            .iter()
            .rev()
            .zip(now[kept..].iter().rev())
            .take(shortest - kept)
            .take_while(|(before, now)| before == now)
            .count();
        Self {
            kept,
            ending,
            appended: false,
        }
    }

    /// Whether the strokes are the same.
    fn same(&self, before: &[Stroke], now: &[Stroke]) -> bool {
        self.appended && before.len() == now.len()
    }
}

/// `ApplyStroke`: brings `cache` up to `strokes`, a Terrain's strokes in order, and to the view
/// `viewport` shows, rasterizing through `rasterizer`, and says whether anything the published
/// coverage holds changed, so it needs publishing again.
///
/// The strokes the cache holds and `strokes` are compared from both ends: those they share from
/// the first onwards and from the last backwards are kept. When only new strokes follow the
/// shared start, they are drawn onto the tiles they reach, an erase as much as a paint.
/// Otherwise only the tiles reached by the strokes between the shared ends, as they were and as
/// they are, are rasterized again from every stroke that reaches them, so moving, changing, or
/// removing one stroke, and undoing any of these, recomputes that stroke's tiles alone however
/// many strokes there are. Either way the tiles end up holding what rasterizing every stroke
/// afresh gives.
///
/// On the CPU, the cache holds the base band alone and ignores the view. On the GPU it holds the
/// base band wherever the strokes reach and, as the active band, the floor band of the zoom when
/// it is first brought up, kept while the zoom stays within 0.9 to 2.2 times it (the base with no
/// lower bound, the largest band with no upper one); a band other than the base holds its tiles
/// that meet the view and some stroke reaches, rasterized the moment they come to meet it, and
/// lets a tile go once it lies more than a tile's width from the view, and every tile when
/// another band becomes active.
pub fn apply_stroke(
    cache: &mut PaintCache,
    strokes: &[Stroke],
    viewport: &Viewport,
    rasterizer: &mut StrokeRasterizer,
) -> bool {
    if !cache.on_gpu {
        return on_cpu(cache, strokes).changed;
    }
    match rasterizer.gpu() {
        Some(mut gpu) => on_gpu(cache, strokes, viewport, &mut gpu),
        // A cache made for the GPU is only ever brought up where there is one.
        None => false,
    }
}

/// What bringing a cache up on the CPU did.
#[derive(Debug, Default)]
struct Applied {
    /// The tiles rasterized.
    touched: BTreeSet<TileKey>,
    /// Whether any tile's pixels changed, or a tile came or went.
    changed: bool,
}

/// The region a base tile covers.
fn region(key: TileKey) -> Region {
    Region::at(
        key.corner(),
        UVec2::splat(COVERAGE_TILE_PIXELS),
        COVERAGE_PIXELS_PER_CELL,
    )
}

/// Brings `cache` up to `strokes` on the CPU, at the base band alone, and tells which tiles it
/// rasterized.
fn on_cpu(cache: &mut PaintCache, strokes: &[Stroke]) -> Applied {
    let mut applied = Applied::default();
    let diff = Diff::of(&cache.strokes, strokes);
    if diff.appended {
        for stroke in &strokes[diff.kept..] {
            for key in reached(stroke, BASE).keys() {
                let region = region(key);
                let mut pixels = cache
                    .base
                    .tiles
                    .get(&key)
                    .and_then(|held| held.tile.pixels())
                    .map_or_else(
                        || vec![0; (COVERAGE_TILE_PIXELS * COVERAGE_TILE_PIXELS) as usize],
                        <[u8]>::to_vec,
                    );
                composite(&mut pixels, stroke, &region);
                applied.changed |= cache.store(key, pixels);
                applied.touched.insert(key);
            }
        }
        cache.take_strokes(strokes, &diff);
        return applied;
    }
    let end = cache.strokes.len() - diff.ending;
    applied.touched = cache.strokes[diff.kept..end]
        .iter()
        .chain(&strokes[diff.kept..strokes.len() - diff.ending])
        .flat_map(|stroke| reached(stroke, BASE).keys())
        .collect();
    let spans: Vec<Span> = strokes.iter().map(|stroke| reached(stroke, BASE)).collect();
    for key in &applied.touched {
        let region = region(*key);
        let mut pixels = vec![0; (COVERAGE_TILE_PIXELS * COVERAGE_TILE_PIXELS) as usize];
        let reaching = strokes
            .iter()
            .zip(&spans)
            .filter(|(_, span)| span.contains(*key));
        for (stroke, _) in reaching {
            composite(&mut pixels, stroke, &region);
        }
        applied.changed |= cache.store(*key, pixels);
    }
    cache.take_strokes(strokes, &diff);
    applied
}

/// The tiles of one band a change rasterizes.
#[derive(Debug, Default, PartialEq, Eq)]
struct Plan {
    /// The tiles cleared and drawn with every stroke that reaches them.
    afresh: BTreeSet<TileKey>,
    /// The tiles drawn onto with new strokes, by number in order.
    appended: BTreeMap<TileKey, Vec<usize>>,
    /// The tiles no stroke reaches any more.
    dropped: BTreeSet<TileKey>,
}

/// The tiles of `band` a change from `before` to `now` rasterizes: those `held` and those
/// `eligible` that the strokes the change adds, or changes as they were and as they are, reach,
/// with `spans` the tiles each stroke of `now` reaches at the band.
fn plan(
    band: u32,
    held: &BTreeMap<TileKey, Held>,
    eligible: impl Fn(TileKey) -> bool,
    (before, now): (&[Stroke], &[Stroke]),
    diff: &Diff,
    spans: &[Span],
) -> Plan {
    let mut plan = Plan::default();
    if diff.appended {
        for (number, span) in spans.iter().enumerate().skip(diff.kept) {
            for key in span.keys() {
                if held.contains_key(&key) {
                    plan.appended.entry(key).or_default().push(number);
                } else if eligible(key) {
                    plan.afresh.insert(key);
                }
            }
        }
        return plan;
    }
    let touched: BTreeSet<TileKey> = before[diff.kept..before.len() - diff.ending]
        .iter()
        .map(|stroke| reached(stroke, band))
        .chain(spans[diff.kept..now.len() - diff.ending].iter().copied())
        .flat_map(Span::keys)
        .collect();
    for key in touched {
        let is_held = held.contains_key(&key);
        if !is_held && !eligible(key) {
            continue;
        }
        if spans.iter().any(|span| span.contains(key)) {
            plan.afresh.insert(key);
        } else if is_held {
            plan.dropped.insert(key);
        }
    }
    plan
}

/// Brings `cache` up to `strokes` and to the view on the GPU, handing the drawing over; whether
/// anything the published coverage holds changed.
fn on_gpu(cache: &mut PaintCache, strokes: &[Stroke], viewport: &Viewport, gpu: &mut Gpu) -> bool {
    let mut changed = false;
    let diff = Diff::of(&cache.strokes, strokes);
    let view = viewport.view();
    let base_tiles = cache.base.tiles.len();
    let mut drawn = Vec::new();

    let spans: Vec<Span> = strokes.iter().map(|stroke| reached(stroke, BASE)).collect();
    let base_plan = plan(
        BASE,
        &cache.base.tiles,
        |_| true,
        (&cache.strokes, strokes),
        &diff,
        &spans,
    );
    let base_changed = carry_out(cache, Which::Base, base_plan, &spans, gpu, &mut drawn);

    let active = cache.active();
    let band = if cache.banded {
        next_band(active, viewport.zoom)
    } else {
        cache.banded = true;
        floor_band(viewport.zoom)
    };
    let switched = band != active;
    if switched {
        cache.overlay = (band != BASE).then(|| Band::new(band));
        changed = true;
    }
    let mut overlay_changed = false;
    let overlay_plan = cache.overlay.as_mut().map(|overlay| {
        let gone: Vec<TileKey> = overlay
            .tiles
            .keys()
            .copied()
            .filter(|key| !within_reach(*key, view, band))
            .collect();
        for key in &gone {
            overlay.tiles.remove(key);
        }
        overlay_changed |= !gone.is_empty();
        let meets = meeting(view, band);
        let spans: Vec<Span> = strokes.iter().map(|stroke| reached(stroke, band)).collect();
        let mut overlay_plan = plan(
            band,
            &overlay.tiles,
            |key| meets.is_some_and(|meets| meets.contains(key)),
            (&cache.strokes, strokes),
            &diff,
            &spans,
        );
        if let Some(meets) = meets {
            for span in &spans {
                if let Some(both) = span.meet(meets) {
                    overlay_plan
                        .afresh
                        .extend(both.keys().filter(|key| !overlay.tiles.contains_key(key)));
                }
            }
        }
        (overlay_plan, spans)
    });
    if let Some((overlay_plan, spans)) = overlay_plan {
        overlay_changed |= carry_out(cache, Which::Overlay, overlay_plan, &spans, gpu, &mut drawn);
    }

    if !diff.same(&cache.strokes, strokes) {
        cache.take_strokes(strokes, &diff);
    }
    gpu.jobs.hand_over(strokes, drawn);
    let shows_base = cache.overlay.is_none();
    changed
        || overlay_changed
        || (shows_base && base_changed)
        || cache.base.tiles.len() != base_tiles
}

/// One of the cache's two bands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Which {
    /// The base band.
    Base,
    /// The overlay band.
    Overlay,
}

/// Carries a plan out on one band of the cache: drops the tiles no stroke reaches, makes the new
/// tiles' images, gives every tile rasterized a new revision, and adds the drawing to `drawn`,
/// with `spans` the tiles each stroke reaches at the band; whether any tile came, went, or was
/// rasterized.
fn carry_out(
    cache: &mut PaintCache,
    which: Which,
    plan: Plan,
    spans: &[Span],
    gpu: &mut Gpu,
    drawn: &mut Vec<Drawn>,
) -> bool {
    let mut changed = !plan.dropped.is_empty();
    let PaintCache {
        base,
        overlay,
        revision,
        ..
    } = cache;
    let band = match which {
        Which::Base => base,
        Which::Overlay => match overlay.as_mut() {
            Some(overlay) => overlay,
            None => return false,
        },
    };
    for key in &plan.dropped {
        band.tiles.remove(key);
    }
    let mut draw = |key: TileKey, clear: bool, numbers: Vec<usize>, band: &mut Band| {
        let held = band.tiles.get(&key).and_then(|held| held.image.clone());
        let image = if let Some(image) = held {
            image
        } else {
            let Some((image, identity)) = gpu.new_tile() else {
                return false;
            };
            band.tiles.insert(
                key,
                Held {
                    tile: CoverageTile {
                        revision: 0,
                        content: TileContent::Gpu(identity),
                    },
                    image: Some(image.clone()),
                },
            );
            image
        };
        if let Some(held) = band.tiles.get_mut(&key) {
            *revision += 1;
            held.tile.revision = *revision;
        }
        drawn.push(Drawn {
            image,
            key,
            band: band.band,
            clear,
            strokes: numbers,
        });
        true
    };
    for key in &plan.afresh {
        let reaching = spans
            .iter()
            .enumerate()
            .filter(|(_, span)| span.contains(*key))
            .map(|(number, _)| number)
            .collect();
        changed |= draw(*key, true, reaching, band);
    }
    for (key, numbers) in plan.appended {
        if !plan.afresh.contains(&key) {
            changed |= draw(key, false, numbers, band);
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        reason = "a test stops at the first thing that is not as expected"
    )]
    use super::*;
    use crate::gpu::StrokeJobs;
    use crate::rasterize;
    use bevy_asset::Assets;
    use bevy_math::{Rect, Vec2};
    use drs_model::BrushSettings;

    /// The base tiles a cache holds, as the model holds them.
    fn tiles(cache: &PaintCache) -> BTreeMap<TileKey, CoverageTile> {
        cache
            .base
            .tiles
            .iter()
            .map(|(key, held)| (*key, held.tile.clone()))
            .collect()
    }

    /// The base tiles a stroke may cover.
    fn tiles_of(stroke: &Stroke) -> impl Iterator<Item = TileKey> {
        reached(stroke, BASE).keys()
    }

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
        tiles(cache)
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
        for (key, tile) in &tiles(cache) {
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
        tiles(cache)
            .iter()
            .map(|(key, tile)| (*key, tile.pixels().expect("on the CPU").to_vec()))
            .collect()
    }

    /// Strokes composited one by one onto the tiles they touch give exactly what rasterizing
    /// them all afresh gives.
    #[test]
    fn appending_equals_rasterizing() {
        let strokes = strokes();
        let mut cache = PaintCache::rasterized_on(false);
        for laid in 1..=strokes.len() {
            assert!(
                on_cpu(&mut cache, &strokes[..laid]).changed,
                "stroke {laid} shows"
            );
        }
        assert!(!on_cpu(&mut cache, &strokes).changed, "nothing new");
        assert_eq!(pixels(&cache), afresh(&strokes));
        assert!(tiles(&cache).keys().any(|key| key.x < 0), "a negative tile");
        assert!(tiles(&cache).len() >= 4, "{} tiles", tiles(&cache).len());
    }

    /// Undoing a stroke rasterizes the tiles it touched and no other, keeps every other tile's
    /// revision, and leaves the tiles as they were before it was laid.
    #[test]
    fn an_undo_touches_only_its_tiles() {
        let strokes = strokes();
        let mut cache = PaintCache::rasterized_on(false);
        on_cpu(&mut cache, &strokes[..3]);
        let before = pixels(&cache);
        let laid = on_cpu(&mut cache, &strokes).touched;
        let revisions: BTreeMap<TileKey, u64> = tiles(&cache)
            .iter()
            .map(|(key, tile)| (*key, tile.revision))
            .collect();

        let undone = on_cpu(&mut cache, &strokes[..3]).touched;

        assert_eq!(undone, laid);
        assert_eq!(undone, tiles_of(&strokes[3]).collect());
        assert_eq!(pixels(&cache), before);
        for (key, tile) in &tiles(&cache) {
            if !undone.contains(key) {
                assert_eq!(
                    tile.revision, revisions[key],
                    "tile {key:?} kept its revision"
                );
            }
        }
        assert!(
            tiles(&cache).len() > undone.len(),
            "some tile was left alone"
        );
    }

    /// Erases composited one by one onto the tiles they touch give exactly what rasterizing every
    /// stroke afresh gives, and an erase reaching beyond everything painted adds no tile.
    #[test]
    fn appending_an_erase_equals_rasterizing() {
        let strokes = with_erases();
        let mut cache = PaintCache::rasterized_on(false);
        on_cpu(&mut cache, &strokes[..4]);
        let painted = pixels(&cache);
        for laid in 5..=strokes.len() {
            assert!(
                on_cpu(&mut cache, &strokes[..laid]).changed,
                "erase {laid} shows"
            );
            assert_eq!(pixels(&cache), afresh(&strokes[..laid]));
        }
        assert_ne!(pixels(&cache), painted);
        assert!(
            tiles(&cache).keys().all(|key| painted.contains_key(key)),
            "no tile beyond the painted ones"
        );
    }

    /// Moving one stroke rasterizes only the tiles it touched before and touches after, and
    /// every other tile keeps its revision.
    #[test]
    fn an_edit_touches_only_its_tiles() {
        let before = with_erases();
        let mut cache = PaintCache::rasterized_on(false);
        on_cpu(&mut cache, &before);
        let revisions = revisions(&cache);
        let mut after = before.clone();
        after[1].points = vec![Vec2::new(40.0, 20.0)];

        let touched = on_cpu(&mut cache, &after).touched;

        let expected: BTreeSet<TileKey> = tiles_of(&before[1]).chain(tiles_of(&after[1])).collect();
        assert_eq!(touched, expected);
        assert_eq!(pixels(&cache), afresh(&after));
        others_kept(&cache, &revisions, &touched);
        assert!(
            tiles(&cache).keys().any(|key| !touched.contains(key)),
            "some tile was left alone"
        );
    }

    /// Removing a stroke between others, and putting it back, each rasterize only the tiles that
    /// stroke touches, and putting it back leaves the tiles as they were.
    #[test]
    fn a_removal_touches_only_its_tiles() {
        let all = with_erases();
        let mut cache = PaintCache::rasterized_on(false);
        on_cpu(&mut cache, &all);
        let laid = pixels(&cache);
        let revisions = revisions(&cache);
        let mut without = all.clone();
        let removed = without.remove(2);

        let touched = on_cpu(&mut cache, &without).touched;
        assert_eq!(touched, tiles_of(&removed).collect());
        assert_eq!(pixels(&cache), afresh(&without));
        others_kept(&cache, &revisions, &touched);

        let back = on_cpu(&mut cache, &all).touched;
        assert_eq!(back, touched);
        assert_eq!(pixels(&cache), laid);
    }

    /// After each edit of a stroke, a point moved, the stroke moved, its Brush settings changed,
    /// turned to erasing and back, removed, and put back, the tiles hold exactly what rasterizing
    /// every stroke afresh gives.
    #[test]
    fn an_edit_equals_rasterizing() {
        let mut strokes = with_erases();
        let mut cache = PaintCache::rasterized_on(false);
        on_cpu(&mut cache, &strokes);
        let edits: [fn(&mut Vec<Stroke>); 7] = [
            |strokes| strokes[0].points[1] = Vec2::new(10.0, 12.0),
            |strokes| {
                for point in &mut strokes[2].points {
                    *point += Vec2::new(-30.0, 9.0);
                }
            },
            |strokes| {
                strokes[3].brush = BrushSettings {
                    size: 6.0,
                    hardness: 0.9,
                    strength: 0.3,
                };
            },
            |strokes| strokes[0].erase = true,
            |strokes| strokes[4].erase = false,
            |strokes| {
                strokes.remove(1);
            },
            |strokes| strokes.insert(3, strokes[4].clone()),
        ];
        for (number, edit) in edits.iter().enumerate() {
            edit(&mut strokes);
            on_cpu(&mut cache, &strokes);
            assert_eq!(pixels(&cache), afresh(&strokes), "edit {number}");
        }
    }

    /// A viewport showing `cells` cells a side around `centre` at `zoom`.
    fn viewport(centre: Vec2, zoom: f32, cells: f32) -> Viewport {
        Viewport {
            centre,
            zoom,
            area: Rect::new(0.0, 0.0, cells * zoom, cells * zoom),
        }
    }

    /// Brings `cache` up on a GPU of its own, returning whether the coverage changed and the
    /// tiles handed over, by band and place, with whether each is cleared.
    fn on_a_gpu(
        cache: &mut PaintCache,
        images: &mut Assets<Image>,
        strokes: &[Stroke],
        viewport: &Viewport,
    ) -> (bool, BTreeMap<(u32, TileKey), bool>) {
        let mut jobs = StrokeJobs::default();
        let mut gpu = Gpu {
            jobs: &mut jobs,
            images,
        };
        let changed = on_gpu(cache, strokes, viewport, &mut gpu);
        let drawn = jobs
            .passes()
            .iter()
            .map(|pass| ((pass.band, pass.key), pass.clear))
            .collect();
        (changed, drawn)
    }

    /// Every tile at `band` some stroke reaches.
    fn reached_by(strokes: &[Stroke], band: u32) -> BTreeSet<TileKey> {
        strokes
            .iter()
            .flat_map(|stroke| reached(stroke, band).keys())
            .collect()
    }

    /// An edit's plan names exactly the tiles the stroke's old and new reach meet, each to be
    /// drawn afresh, and a tile only the old reach met and no stroke reaches any more to be let
    /// go; an appended stroke's plan draws it onto the tiles held and makes the rest.
    #[test]
    fn a_plan_names_the_tiles_of_a_change() {
        let before = with_erases();
        let mut images = Assets::<Image>::default();
        let mut cache = PaintCache::rasterized_on(true);
        on_a_gpu(
            &mut cache,
            &mut images,
            &before,
            &viewport(Vec2::ZERO, 40.0, 10.0),
        );
        let mut after = before.clone();
        after[2].points = vec![Vec2::new(-40.0, 20.0)];
        let spans: Vec<Span> = after.iter().map(|stroke| reached(stroke, BASE)).collect();
        let diff = Diff::of(&before, &after);

        let planned = plan(
            BASE,
            &cache.base.tiles,
            |_| true,
            (&before, &after),
            &diff,
            &spans,
        );

        let old: BTreeSet<TileKey> = tiles_of(&before[2]).collect();
        let new: BTreeSet<TileKey> = tiles_of(&after[2]).collect();
        let others = reached_by(&[&after[..2], &after[3..]].concat(), BASE);
        let afresh: BTreeSet<TileKey> = old
            .iter()
            .filter(|key| others.contains(key))
            .chain(&new)
            .copied()
            .collect();
        let dropped: BTreeSet<TileKey> = old.difference(&afresh).copied().collect();
        assert!(!dropped.is_empty(), "the stroke had tiles of its own");
        assert_eq!(planned.afresh, afresh);
        assert_eq!(planned.dropped, dropped);
        assert!(planned.appended.is_empty());

        let mut more = after.clone();
        more.push(Stroke {
            points: vec![Vec2::new(16.5, 3.0), Vec2::new(-39.0, 21.0)],
            brush: BrushSettings {
                size: 1.0,
                hardness: 0.5,
                strength: 1.0,
            },
            erase: false,
        });
        on_a_gpu(
            &mut cache,
            &mut images,
            &after,
            &viewport(Vec2::ZERO, 40.0, 10.0),
        );
        let spans: Vec<Span> = more.iter().map(|stroke| reached(stroke, BASE)).collect();
        let planned = plan(
            BASE,
            &cache.base.tiles,
            |_| true,
            (&after, &more),
            &Diff::of(&after, &more),
            &spans,
        );
        let laid: BTreeSet<TileKey> = tiles_of(&more[6]).collect();
        let held: BTreeSet<TileKey> = laid
            .iter()
            .filter(|key| cache.base.tiles.contains_key(key))
            .copied()
            .collect();
        assert_eq!(
            planned.appended.keys().copied().collect::<BTreeSet<_>>(),
            held
        );
        assert!(planned.appended.values().all(|numbers| *numbers == [6]));
        assert_eq!(planned.afresh, laid.difference(&held).copied().collect());
    }

    /// Handing tiles over lays each stroke's segments out once and draws consecutive strokes of
    /// one kind that reach a tile in one run, in order.
    #[test]
    fn handing_over_groups_runs_of_one_kind() {
        let strokes = with_erases();
        let mut images = Assets::<Image>::default();
        let mut jobs = StrokeJobs::default();
        let image = images.add(Image::default());
        let key = TileKey { x: 0, y: 0 };
        let drawn = |strokes: Vec<usize>| Drawn {
            image: image.clone(),
            key,
            band: BASE,
            clear: true,
            strokes,
        };
        jobs.hand_over(
            &strokes,
            vec![drawn(vec![0, 1, 3, 4, 5]), drawn(vec![1, 5])],
        );
        let segments = |stroke: &Stroke| u32::try_from(stroke.points.len().max(2) - 1).unwrap();
        let starts: Vec<u32> = [0, 1, 3, 4, 5]
            .iter()
            .scan(0, |at, number| {
                let start = *at;
                *at += segments(&strokes[*number]);
                Some(start)
            })
            .collect();
        let end = starts[4] + segments(&strokes[5]);
        let runs: Vec<(bool, std::ops::Range<u32>)> = jobs.passes()[0]
            .runs
            .iter()
            .map(|run| (run.erase, run.segments.clone()))
            .collect();
        assert_eq!(runs, vec![(false, 0..starts[3]), (true, starts[3]..end)]);
        let runs: Vec<(bool, std::ops::Range<u32>)> = jobs.passes()[1]
            .runs
            .iter()
            .map(|run| (run.erase, run.segments.clone()))
            .collect();
        assert_eq!(
            runs,
            vec![(false, starts[1]..starts[2]), (true, starts[4]..end)]
        );
    }

    /// On the GPU the base holds every tile a stroke reaches whatever the zoom; a zoom that calls
    /// for another band makes it active with its tiles that meet the view and some stroke
    /// reaches, all drawn afresh, the base drawn into no more; a pan draws the tiles coming to
    /// meet the view and lets go those out of reach; and a zoom back to the base drops the
    /// overlay and shows the base as it was.
    #[test]
    fn the_bands_follow_the_view_on_the_gpu() {
        let strokes = with_erases();
        let mut images = Assets::<Image>::default();
        let mut cache = PaintCache::rasterized_on(true);
        let far = viewport(Vec2::new(8.0, 8.0), 20.0, 40.0);
        let (changed, drawn) = on_a_gpu(&mut cache, &mut images, &strokes, &far);
        assert!(changed);
        let base = reached_by(&strokes, BASE);
        assert_eq!(cache.coverage().band, BASE);
        assert_eq!(
            cache
                .coverage()
                .tiles
                .keys()
                .copied()
                .collect::<BTreeSet<_>>(),
            base
        );
        assert!(drawn.keys().all(|(band, _)| *band == BASE));
        assert_eq!(drawn.len(), base.len());
        let revisions = cache.coverage().tiles;

        let close = viewport(Vec2::new(16.0, 4.0), 300.0, 6.0);
        let (changed, drawn) = on_a_gpu(&mut cache, &mut images, &strokes, &close);
        assert!(changed);
        let coverage = cache.coverage();
        assert_eq!(coverage.band, 256);
        assert_eq!(coverage.base_tiles, base.len());
        let meets = meeting(close.view(), 256).unwrap();
        let shown: BTreeSet<TileKey> = reached_by(&strokes, 256)
            .into_iter()
            .filter(|key| meets.contains(*key))
            .collect();
        assert!(!shown.is_empty());
        assert_eq!(
            coverage.tiles.keys().copied().collect::<BTreeSet<_>>(),
            shown
        );
        assert_eq!(drawn, shown.iter().map(|key| ((256, *key), true)).collect());

        let (changed, drawn) = on_a_gpu(&mut cache, &mut images, &strokes, &close);
        assert!(!changed, "nothing moved");
        assert!(drawn.is_empty());

        let panned = viewport(Vec2::new(17.0, 4.0), 300.0, 6.0);
        let (_, drawn) = on_a_gpu(&mut cache, &mut images, &strokes, &panned);
        let meets_now = meeting(panned.view(), 256).unwrap();
        let entering: BTreeSet<TileKey> = reached_by(&strokes, 256)
            .into_iter()
            .filter(|key| meets_now.contains(*key) && !shown.contains(key))
            .collect();
        assert_eq!(
            drawn,
            entering.iter().map(|key| ((256, *key), true)).collect()
        );
        let away = viewport(Vec2::new(60.0, 60.0), 300.0, 6.0);
        on_a_gpu(&mut cache, &mut images, &strokes, &away);
        assert!(
            cache
                .coverage()
                .tiles
                .keys()
                .all(|key| within_reach(*key, away.view(), 256))
        );

        let (changed, drawn) = on_a_gpu(&mut cache, &mut images, &strokes, &far);
        assert!(changed);
        assert!(drawn.is_empty(), "the base is resident");
        assert_eq!(cache.coverage().band, BASE);
        assert_eq!(cache.coverage().tiles, revisions);
    }
}
