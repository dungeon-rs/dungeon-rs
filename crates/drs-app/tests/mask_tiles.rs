//! The tiles a Terrain is shown from, through the headless editor with offscreen rendering: the
//! real plugins of the model, the history, `LibraryAccess`, `LibraryManager`, `ProjectManager`,
//! `AuthoringManager`, the paint Engine's rasterizing on the GPU, and `RenderEngine` under Bevy's
//! default plugins without a window, over one Asset Folder holding a texture, driven by Apply,
//! Undo, and Redo messages and by writing the Viewport as the Editor does, and asserted on the
//! published coverage, its band and its tiles' places and revisions, and on the tiles read back
//! from the GPU and compared pixel by pixel with the paint Engine's CPU rasterization of the
//! Terrain's strokes over the same region at the same band.
//!
//! These tests draw on the GPU and fail, rather than skip, where no adapter exists.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

mod support;

use bevy::app::App;
use bevy::asset::{AssetId, AssetIndex, Assets};
use bevy::ecs::entity::Entity;
use bevy::ecs::message::Messages;
use bevy::ecs::observer::On;
use bevy::image::Image;
use bevy::math::{Rect, UVec2, Vec2};
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use drs_model::{
    Apply, AssetAddress, BrushSettings, CommandFailed, EditElement, ElementChange, ElementId,
    FolderKey, Gesture, GpuTile, Layer, Paint, Redo, Stroke, StrokeChange, Terrain,
    TerrainCoverage, TileKey, Undo, Viewport, tile_cells,
};
use drs_paint_engine::rasterize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use support::offscreen::editor;
use support::png;
use tempfile::TempDir;

/// The texture every stroke paints with, two cells a side at the Grid's 256 pixels per cell.
const FLAGSTONES: &str = "flagstones.png";
/// The area the Level is shown in, in logical screen pixels, unless a test says otherwise.
const AREA: Vec2 = Vec2::new(1024.0, 768.0);
/// How many pixels a side a tile has.
const SIDE: u32 = 512;
/// How many frames a read-back may take before the test gives up.
const MOST_FRAMES: u32 = 200;

/// The headless editor with the fixture folder added, its stroke pipeline compiled.
struct Fixture {
    /// Keeps the temporary directory alive for the test.
    _root: TempDir,
    /// The folder's key once added.
    key: FolderKey,
    /// The headless editor.
    app: App,
}

/// A Brush of `size`, `hardness`, and `strength`.
fn brush(size: f32, hardness: f32, strength: f32) -> BrushSettings {
    BrushSettings {
        size,
        hardness,
        strength,
    }
}

/// A stroke that paints through `points` with `brush`.
fn paint(points: &[Vec2], brush: BrushSettings) -> Stroke {
    Stroke {
        points: points.to_vec(),
        brush,
        erase: false,
    }
}

/// An erase through `points` with `brush`.
fn erase(points: &[Vec2], brush: BrushSettings) -> Stroke {
    Stroke {
        erase: true,
        ..paint(points, brush)
    }
}

/// The view of a Viewport around `centre` at `zoom` over an area of `area` screen pixels.
fn view(centre: Vec2, zoom: f32, area: Vec2) -> Rect {
    Rect::from_center_half_size(centre, area / 2.0 / zoom)
}

/// The tiles at `band` a stroke reaches: those its reach, a pixel wider either way, meets.
fn reached(stroke: &Stroke, band: u32) -> BTreeSet<TileKey> {
    let reach = stroke.reach();
    let side = tile_cells(band);
    #[expect(clippy::cast_precision_loss, reason = "a band is a small whole number")]
    let margin = 1.0 / band as f32;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the tiles of a test are a few from the origin"
    )]
    let tile = |cells: f32| (cells / side).floor() as i32;
    let (low_x, low_y) = (tile(reach.min.x - margin), tile(reach.min.y - margin));
    let (high_x, high_y) = (tile(reach.max.x + margin), tile(reach.max.y + margin));
    (low_y..=high_y)
        .flat_map(|y| (low_x..=high_x).map(move |x| TileKey { x, y }))
        .collect()
}

/// The tiles at `band` some stroke reaches.
fn reached_by(strokes: &[Stroke], band: u32) -> BTreeSet<TileKey> {
    strokes
        .iter()
        .flat_map(|stroke| reached(stroke, band))
        .collect()
}

/// The tiles at `band` whose square overlaps `view`.
fn meeting(view: Rect, band: u32) -> BTreeSet<TileKey> {
    let side = tile_cells(band);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the tiles of a test are a few from the origin"
    )]
    let (low_x, low_y, high_x, high_y) = (
        (view.min.x / side).floor() as i32,
        (view.min.y / side).floor() as i32,
        (view.max.x / side).ceil() as i32 - 1,
        (view.max.y / side).ceil() as i32 - 1,
    );
    (low_y..=high_y)
        .flat_map(|y| (low_x..=high_x).map(move |x| TileKey { x, y }))
        .collect()
}

/// The keys of a coverage's tiles.
fn keys(coverage: &TerrainCoverage) -> BTreeSet<TileKey> {
    coverage.tiles.keys().copied().collect()
}

/// The revision of each tile of a coverage.
fn revisions(coverage: &TerrainCoverage) -> BTreeMap<TileKey, u64> {
    coverage
        .tiles
        .iter()
        .map(|(key, tile)| (*key, tile.revision))
        .collect()
}

/// The tiles whose revision differs from `before`, or that are new.
fn rasterized(before: &BTreeMap<TileKey, u64>, after: &TerrainCoverage) -> BTreeSet<TileKey> {
    after
        .tiles
        .iter()
        .filter(|(key, tile)| before.get(key) != Some(&tile.revision))
        .map(|(key, _)| *key)
        .collect()
}

/// A soft stroke, a hard one, a one-point dab, a half-strength erase with a sharp joint and a
/// self-crossing, two overlapping erases, and a stroke beyond the Bounds, at negative cells.
fn varied() -> Vec<Stroke> {
    vec![
        paint(
            &[
                Vec2::new(5.13, 6.27),
                Vec2::new(9.71, 7.43),
                Vec2::new(11.07, 10.91),
            ],
            brush(1.7, 0.4, 0.9),
        ),
        paint(
            &[Vec2::new(6.37, 9.23), Vec2::new(10.59, 5.81)],
            brush(1.13, 1.0, 1.0),
        ),
        paint(&[Vec2::new(8.31, 8.77)], brush(2.3, 0.0, 0.6)),
        erase(
            &[
                Vec2::new(5.51, 7.13),
                Vec2::new(10.33, 8.87),
                Vec2::new(6.11, 9.71),
                Vec2::new(9.23, 5.63),
            ],
            brush(0.93, 0.3, 0.5),
        ),
        erase(
            &[Vec2::new(7.03, 6.01), Vec2::new(9.07, 9.53)],
            brush(1.21, 0.5, 1.0),
        ),
        erase(
            &[Vec2::new(7.41, 6.23), Vec2::new(8.83, 9.91)],
            brush(1.03, 0.2, 0.8),
        ),
        paint(
            &[Vec2::new(-3.31, -2.13), Vec2::new(1.27, 0.41)],
            brush(1.51, 0.5, 1.0),
        ),
    ]
}

impl Fixture {
    /// Creates the fixture folder, starts the editor, adds the folder, and waits until a stroke
    /// shows on the GPU, so the stroke pipeline is compiled before the test begins.
    fn new() -> Self {
        let root = TempDir::new().expect("temporary root");
        let folder = root.path().join("fixtures");
        png(&folder, FLAGSTONES, UVec2::splat(512), [90, 90, 100, 255]);
        let mut app = editor(root.path());
        let key = support::add_folder(&mut app, &folder, "Fixtures").key;
        let mut fixture = Self {
            _root: root,
            key,
            app,
        };
        fixture.look(Vec2::new(0.5, 0.5), 40.0);
        fixture.paint(paint(&[Vec2::new(0.5, 0.5)], brush(1.0, 1.0, 1.0)));
        let mut shown = false;
        for _ in 0..MOST_FRAMES {
            let coverage = fixture.coverage();
            let tiles = fixture.read_back(&coverage);
            if tiles
                .values()
                .any(|pixels| pixels.iter().any(|pixel| *pixel > 0))
            {
                shown = true;
                break;
            }
        }
        assert!(shown, "a stroke never showed on the GPU");
        fixture.undo();
        assert!(fixture.terrain().is_none(), "the warm-up stroke is undone");
        fixture
    }

    /// Writes the Viewport as the Editor does, showing [`AREA`] around `centre` at `zoom`, and
    /// runs one update.
    fn look(&mut self, centre: Vec2, zoom: f32) {
        self.look_over(centre, zoom, AREA);
    }

    /// Writes the Viewport, showing `area` screen pixels around `centre` at `zoom`, and runs one
    /// update.
    fn look_over(&mut self, centre: Vec2, zoom: f32, area: Vec2) {
        let mut viewport = self
            .app
            .world_mut()
            .get_resource_mut::<Viewport>()
            .expect("the Viewport");
        viewport.centre = centre;
        viewport.zoom = zoom;
        viewport.area = Rect::from_corners(Vec2::ZERO, area);
        self.app.update();
    }

    /// Sends a Command and runs one update, failing the test if it was refused.
    fn apply(&mut self, command: Apply) {
        self.app.world_mut().write_message(command);
        self.app.update();
        let failed: Vec<CommandFailed> = self
            .app
            .world_mut()
            .resource_mut::<Messages<CommandFailed>>()
            .drain()
            .collect();
        assert!(failed.is_empty(), "the Command failed: {failed:?}");
    }

    /// Lays `stroke` on the one Layer with the flagstones.
    fn paint(&mut self, stroke: Stroke) {
        let world = self.app.world_mut();
        let layer = world
            .query::<(Entity, &Layer)>()
            .single(world)
            .expect("exactly one Layer")
            .0;
        let asset = AssetAddress {
            folder: self.key.clone(),
            place: FLAGSTONES.to_owned(),
        };
        self.apply(Apply::Paint(Paint {
            layer,
            stroke,
            asset: Some(asset),
        }));
    }

    /// Lays every stroke of `strokes` in order.
    fn paint_all(&mut self, strokes: &[Stroke]) {
        for stroke in strokes {
            self.paint(stroke.clone());
        }
    }

    /// Changes stroke `stroke` of the one Terrain as one step.
    fn edit(&mut self, stroke: usize, change: StrokeChange) {
        let (element, _) = self.terrain().expect("a Terrain");
        self.apply(Apply::EditElement(EditElement {
            element,
            change: ElementChange::Stroke { stroke, change },
            gesture: Gesture::Single,
        }));
    }

    /// Sends Undo and runs one update.
    fn undo(&mut self) {
        self.app.world_mut().write_message(Undo);
        self.app.update();
    }

    /// Sends Redo and runs one update.
    fn redo(&mut self) {
        self.app.world_mut().write_message(Redo);
        self.app.update();
    }

    /// The one Terrain's identity and strokes, if there is one.
    fn terrain(&mut self) -> Option<(ElementId, Vec<Stroke>)> {
        let world = self.app.world_mut();
        world
            .query::<(&ElementId, &Terrain)>()
            .iter(world)
            .next()
            .map(|(id, terrain)| (*id, terrain.strokes.clone()))
    }

    /// The one Terrain's strokes.
    fn strokes(&mut self) -> Vec<Stroke> {
        self.terrain().expect("a Terrain").1
    }

    /// The one Terrain's published coverage.
    fn coverage(&mut self) -> TerrainCoverage {
        let world = self.app.world_mut();
        world
            .query::<&TerrainCoverage>()
            .single(world)
            .expect("one Terrain with its coverage")
            .clone()
    }

    /// The band the one Terrain is shown at.
    fn band(&mut self) -> u32 {
        self.coverage().band
    }

    /// Reads every tile of `coverage` back from the GPU, by place.
    ///
    /// A one-shot read-back spawned between updates is let go at the start of the next one,
    /// before it is taken to the render world, so each tile is read back by a lasting one,
    /// despawned once its pixels arrive.
    fn read_back(&mut self, coverage: &TerrainCoverage) -> BTreeMap<TileKey, Vec<u8>> {
        let arrived: Arc<Mutex<BTreeMap<TileKey, Vec<u8>>>> = Arc::default();
        let mut readers = Vec::new();
        for (key, tile) in &coverage.tiles {
            let image = tile.image().expect("the tile is on the GPU");
            let handle = self
                .app
                .world_mut()
                .resource_mut::<Assets<Image>>()
                .get_strong_handle(AssetId::from(AssetIndex::from_bits(image.0)))
                .expect("the tile's image exists");
            let arrived = Arc::clone(&arrived);
            let key = *key;
            let reader = self
                .app
                .world_mut()
                .spawn(Readback::texture(handle))
                .observe(move |done: On<ReadbackComplete>| {
                    arrived
                        .lock()
                        .expect("the tiles read back")
                        .entry(key)
                        .or_insert_with(|| done.data.clone());
                })
                .id();
            readers.push(reader);
        }
        for _ in 0..MOST_FRAMES {
            if arrived.lock().expect("the tiles read back").len() == coverage.tiles.len() {
                break;
            }
            self.app.update();
        }
        for reader in readers {
            self.app.world_mut().despawn(reader);
        }
        let arrived = arrived.lock().expect("the tiles read back").clone();
        assert_eq!(
            arrived.len(),
            coverage.tiles.len(),
            "every tile was read back within {MOST_FRAMES} frames"
        );
        arrived
    }

    /// Asserts that every tile of the one Terrain's coverage, read back from the GPU, holds what
    /// the CPU rasterizes from its strokes over the tile's region at its band, every pixel
    /// within 1/255, and that some pixel is covered; `what` names the case.
    fn assert_matches_the_reference(&mut self, what: &str) {
        let coverage = self.coverage();
        let strokes = self.strokes();
        assert!(!coverage.tiles.is_empty(), "{what}: some tile is shown");
        let read = self.read_back(&coverage);
        let mut covered = 0;
        for (key, pixels) in &read {
            let reference = rasterize(
                &strokes,
                key.corner_at(coverage.band),
                UVec2::splat(SIDE),
                coverage.band,
            );
            assert_eq!(pixels.len(), reference.len(), "{what}: tile {key:?}");
            let (mut worst, mut at) = (0, 0);
            for (index, (gpu, cpu)) in pixels.iter().zip(&reference).enumerate() {
                if gpu.abs_diff(*cpu) > worst {
                    worst = gpu.abs_diff(*cpu);
                    at = index;
                }
            }
            assert!(
                worst <= 1,
                "{what}: tile {key:?} at band {} differs by {worst}/255 at row {} column {}: \
                 {} on the GPU, {} on the CPU",
                coverage.band,
                at / SIDE as usize,
                at % SIDE as usize,
                pixels[at],
                reference[at]
            );
            covered += pixels.iter().filter(|pixel| **pixel > 0).count();
        }
        assert!(covered > 0, "{what}: some pixel is covered");
    }

    /// Whether an image the paint Engine named is still held anywhere.
    fn image_exists(&mut self, image: GpuTile) -> bool {
        self.app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .get(AssetId::from(AssetIndex::from_bits(image.0)))
            .is_some()
    }
}

/// A Terrain's coverage first shows at the largest band not above the zoom, the base below 64,
/// and at zooms above 512 at the band of 256.
#[test]
fn the_band_follows_the_zoom() {
    let mut fixture = Fixture::new();
    for (zoom, band) in [
        (20.0, 32),
        (40.0, 32),
        (64.0, 64),
        (100.0, 64),
        (128.0, 128),
        (300.0, 256),
        (1024.0, 256),
    ] {
        fixture.look(Vec2::new(4.0, 4.0), zoom);
        fixture.paint(paint(
            &[Vec2::new(3.0, 4.0), Vec2::new(5.0, 4.2)],
            brush(1.0, 0.5, 1.0),
        ));
        assert_eq!(fixture.band(), band, "a fresh Terrain at zoom {zoom}");
        fixture.undo();
        assert!(fixture.terrain().is_none(), "the Terrain is undone");
    }
}

/// Once a band is active it stays while the zoom is from 0.9 to 2.2 times it, the base with no
/// lower bound, and otherwise gives way to the floor band of the zoom; a pinch wobbling the zoom
/// by six per cent never switches it, nor rasterizes a tile.
#[test]
fn the_band_holds_through_a_wobble() {
    let mut fixture = Fixture::new();
    let centre = Vec2::new(4.0, 4.0);
    fixture.look(centre, 64.0);
    fixture.paint(paint(
        &[Vec2::new(1.0, 2.0), Vec2::new(7.0, 6.0)],
        brush(1.5, 0.5, 1.0),
    ));
    assert_eq!(fixture.band(), 64);
    for (zoom, band) in [
        (58.0, 64),
        (57.0, 32),
        (70.0, 32),
        (71.0, 64),
        (140.0, 64),
        (141.0, 128),
        (64.0, 64),
    ] {
        fixture.look(centre, zoom);
        assert_eq!(fixture.band(), band, "at zoom {zoom}");
    }
    let before = fixture.coverage();
    for frame in 0..60_u16 {
        let wobble = f32::from(frame % 7) / 6.0 * 0.12 - 0.06;
        fixture.look(centre, 64.0 * (1.0 + wobble));
        let coverage = fixture.coverage();
        assert_eq!(coverage.band, 64, "frame {frame}");
        assert_eq!(revisions(&coverage), revisions(&before), "frame {frame}");
    }
}

/// A stroke painted outside the view at a high zoom is held at the base, and when the zoom drops
/// to the base its tiles hold it, match the reference, and keep the revisions they had, none
/// rasterized by the drop.
#[test]
fn the_base_is_resident() {
    let mut fixture = Fixture::new();
    fixture.look(Vec2::new(2.0, 2.0), 256.0);
    fixture.paint(paint(
        &[Vec2::new(1.5, 1.5), Vec2::new(2.5, 2.2)],
        brush(0.8, 0.5, 1.0),
    ));
    let outside = paint(
        &[Vec2::new(36.3, 33.1), Vec2::new(41.7, 35.9)],
        brush(1.4, 0.7, 1.0),
    );
    fixture.paint(outside.clone());
    // A last stroke in the view, so every tile rasterized so far has a revision no later than
    // the ones it shows.
    fixture.paint(paint(&[Vec2::new(2.2, 1.8)], brush(0.6, 0.5, 0.7)));
    let shown = fixture.coverage();
    assert_eq!(shown.band, 256);
    assert!(
        shown
            .tiles
            .keys()
            .all(|key| !reached(&outside, 256).contains(key)),
        "the stroke outside the view has no tile at the band of 256"
    );
    let latest = shown
        .tiles
        .values()
        .map(|tile| tile.revision)
        .max()
        .expect("tiles in the view");

    fixture.look(Vec2::new(2.0, 2.0), 40.0);

    let base = fixture.coverage();
    assert_eq!(base.band, 32);
    assert_eq!(keys(&base), reached_by(&fixture.strokes(), 32));
    assert!(
        reached(&outside, 32)
            .iter()
            .all(|key| base.tiles.contains_key(key))
    );
    assert!(
        base.tiles.values().all(|tile| tile.revision <= latest),
        "no base tile was rasterized by the drop"
    );
    fixture.assert_matches_the_reference("the base after the drop");
}

/// At a band other than the base, the tiles held are exactly those that meet the view and some
/// stroke reaches, at negative cells too; a pan brings in the tiles coming to meet the view,
/// each matching the reference in its first frame.
#[test]
fn an_overlay_covers_the_view() {
    let mut fixture = Fixture::new();
    let centre = Vec2::new(0.5, 0.5);
    fixture.look(centre, 128.0);
    let strokes = [
        paint(
            &[Vec2::new(-3.7, -1.2), Vec2::new(2.9, 1.3)],
            brush(1.2, 0.5, 1.0),
        ),
        paint(
            &[Vec2::new(3.1, -2.6), Vec2::new(9.4, 2.3)],
            brush(0.9, 0.3, 0.8),
        ),
        paint(&[Vec2::new(30.0, 30.0)], brush(1.0, 0.5, 1.0)),
    ];
    fixture.paint_all(&strokes);
    let held = fixture.coverage();
    assert_eq!(held.band, 128);
    let expected: BTreeSet<TileKey> = meeting(view(centre, 128.0, AREA), 128)
        .intersection(&reached_by(&strokes, 128))
        .copied()
        .collect();
    assert!(expected.iter().any(|key| key.x < 0 && key.y < 0));
    assert_eq!(keys(&held), expected);
    fixture.assert_matches_the_reference("the view");

    let panned = Vec2::new(5.5, 0.5);
    fixture.look(panned, 128.0);
    let now = fixture.coverage();
    let entered: BTreeSet<TileKey> = meeting(view(panned, 128.0, AREA), 128)
        .intersection(&reached_by(&strokes, 128))
        .filter(|key| !held.tiles.contains_key(key))
        .copied()
        .collect();
    assert!(!entered.is_empty(), "the pan brings tiles in");
    assert!(entered.iter().all(|key| now.tiles.contains_key(key)));
    let entering = TerrainCoverage {
        tiles: now
            .tiles
            .iter()
            .filter(|(key, _)| entered.contains(key))
            .map(|(key, tile)| (*key, tile.clone()))
            .collect(),
        ..now.clone()
    };
    let read = fixture.read_back(&entering);
    for (key, pixels) in read {
        let reference = rasterize(&strokes, key.corner_at(128), UVec2::splat(SIDE), 128);
        assert!(
            pixels
                .iter()
                .zip(&reference)
                .all(|(gpu, cpu)| gpu.abs_diff(*cpu) <= 1),
            "tile {key:?} entering the view"
        );
    }
}

/// A pan of half a tile keeps every tile, a pan far away lets the old ones go, a long pan over
/// painted ground never holds more than the bound for the view's area, and a switch to another
/// band lets every tile of the previous one go.
#[test]
fn overlay_tiles_are_let_go_out_of_reach() {
    let mut fixture = Fixture::new();
    let area = Vec2::new(1920.0, 1080.0);
    fixture.look_over(Vec2::new(10.0, 10.0), 256.0, area);
    let ground: Vec<Stroke> = (0_u8..12)
        .map(|row| {
            let y = 1.5 + f32::from(row) * 1.7;
            paint(
                &[Vec2::new(-2.0, y), Vec2::new(45.0, y + 0.6)],
                brush(1.6, 0.5, 1.0),
            )
        })
        .collect();
    fixture.paint_all(&ground);
    let first = fixture.coverage();
    assert_eq!(first.band, 256);
    assert!(!first.tiles.is_empty());

    fixture.look_over(Vec2::new(11.0, 10.0), 256.0, area);
    let half = fixture.coverage();
    assert!(
        first.tiles.keys().all(|key| half.tiles.contains_key(key)),
        "half a tile keeps every tile"
    );

    fixture.look_over(Vec2::new(200.0, 200.0), 256.0, area);
    let away = fixture.coverage();
    assert!(away.tiles.is_empty(), "nothing is held far from the ground");
    for _ in 0..5 {
        fixture.app.update();
    }
    for tile in first.tiles.values() {
        let image = tile.image().expect("on the GPU");
        assert!(
            !fixture.image_exists(image),
            "a tile let go frees its image"
        );
    }

    let bound = |area: Vec2| {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a few tiles"
        )]
        let along = |pixels: f32| (pixels / 460.8).ceil() as usize + 3;
        along(area.x) * along(area.y)
    };
    assert_eq!(bound(area), 48);
    fixture.look_over(Vec2::new(0.0, 10.0), 256.0, area);
    fixture.look_over(Vec2::new(0.0, 10.0), 232.0, area);
    let mut most = 0;
    for step in 0_u8..40 {
        let centre = Vec2::new(f32::from(step) * 512.0 / 232.0, 10.0);
        fixture.look_over(centre, 232.0, area);
        let coverage = fixture.coverage();
        assert_eq!(coverage.band, 256);
        most = most.max(coverage.tiles.len());
        assert!(
            coverage.tiles.len() <= bound(area),
            "{} tiles at step {step}",
            coverage.tiles.len()
        );
    }
    assert!(most > 16, "the pan held {most} tiles at most");

    let before = fixture.coverage();
    fixture.look_over(Vec2::new(10.0, 10.0), 100.0, area);
    assert_eq!(fixture.band(), 64);
    for _ in 0..5 {
        fixture.app.update();
    }
    for tile in before.tiles.values() {
        let image = tile.image().expect("on the GPU");
        assert!(!fixture.image_exists(image), "the previous band is let go");
    }
}

/// Every tile of every band holds what the CPU rasterizes, within 1/255, for soft and hard
/// strokes, a dab, an erase with a sharp joint and a self-crossing, overlapping erases, and a
/// stroke beyond the Bounds.
#[test]
fn every_band_matches_the_reference() {
    let mut fixture = Fixture::new();
    fixture.look(Vec2::new(8.0, 8.0), 40.0);
    fixture.paint_all(&varied());
    assert_eq!(fixture.band(), 32);
    fixture.assert_matches_the_reference("the base");
    for (zoom, band) in [(100.0, 64), (200.0, 128), (300.0, 256)] {
        for centre in [
            Vec2::new(8.0, 8.0),
            Vec2::new(6.5, 9.0),
            Vec2::new(-1.0, -1.0),
        ] {
            fixture.look(centre, zoom);
            assert_eq!(fixture.band(), band);
            fixture.assert_matches_the_reference(&format!("band {band} around {centre}"));
        }
    }
}

/// After a point moved, a stroke moved, an erase turned to painting, and a stroke removed, each
/// undone and redone, the tiles at the base and at an overlay hold what the CPU rasterizes.
#[test]
fn edited_tiles_match_the_reference() {
    let mut fixture = Fixture::new();
    fixture.look(Vec2::new(8.0, 8.0), 40.0);
    fixture.paint_all(&varied());
    let edits = [
        (
            0,
            StrokeChange::Point {
                index: 1,
                position: Vec2::new(7.9, 11.3),
            },
        ),
        (2, StrokeChange::Position(Vec2::new(6.3, 7.7))),
        (3, StrokeChange::Erase(false)),
        (4, StrokeChange::Remove),
    ];
    for (zoom, place) in [(40.0, "the base"), (200.0, "an overlay")] {
        fixture.look(Vec2::new(8.0, 8.0), zoom);
        for (stroke, change) in &edits {
            fixture.edit(*stroke, change.clone());
            fixture.assert_matches_the_reference(&format!("{change:?} at {place}"));
            fixture.undo();
            fixture.assert_matches_the_reference(&format!("{change:?} undone at {place}"));
            fixture.redo();
            fixture.assert_matches_the_reference(&format!("{change:?} redone at {place}"));
            fixture.undo();
        }
    }
}

/// A new stroke, a moved stroke, and the move undone each rasterize exactly the tiles of the
/// band shown that the stroke reaches, as it was and as it is, at the base and at an overlay,
/// and leave every other tile's revision as it was.
#[test]
fn only_the_touched_tiles_recompute() {
    let mut fixture = Fixture::new();
    let centre = Vec2::new(8.0, 8.0);
    let laid = [
        paint(
            &[Vec2::new(1.0, 2.0), Vec2::new(30.0, 20.0)],
            brush(2.0, 0.5, 1.0),
        ),
        paint(
            &[Vec2::new(2.0, 14.0), Vec2::new(14.0, 3.0)],
            brush(1.5, 0.5, 1.0),
        ),
    ];
    let new = paint(
        &[Vec2::new(7.1, 7.3), Vec2::new(8.9, 8.2)],
        brush(0.6, 0.5, 1.0),
    );
    let moved = Vec2::new(9.3, 7.4);
    for (zoom, band) in [(40.0, 32), (200.0, 128)] {
        fixture.look(centre, zoom);
        fixture.paint_all(&laid);
        assert_eq!(fixture.band(), band);
        let shown = |strokes: &[Stroke]| -> BTreeSet<TileKey> {
            let all = reached_by(strokes, band);
            if band == 32 {
                all
            } else {
                all.intersection(&meeting(view(centre, zoom, AREA), band))
                    .copied()
                    .collect()
            }
        };

        let before = revisions(&fixture.coverage());
        fixture.paint(new.clone());
        let after = fixture.coverage();
        let mut with_new = laid.to_vec();
        with_new.push(new.clone());
        assert_eq!(keys(&after), shown(&with_new), "band {band}");
        assert_eq!(
            rasterized(&before, &after),
            shown(std::slice::from_ref(&new)),
            "the new stroke at band {band}"
        );

        let before = revisions(&after);
        fixture.edit(2, StrokeChange::Position(moved));
        let after = fixture.coverage();
        let now = fixture.strokes()[2].clone();
        // A tile only the stroke reached is let go rather than rasterized.
        let touched = |after: &TerrainCoverage| -> BTreeSet<TileKey> {
            shown(std::slice::from_ref(&new))
                .union(&shown(std::slice::from_ref(&now)))
                .filter(|key| after.tiles.contains_key(key))
                .copied()
                .collect()
        };
        assert_eq!(
            rasterized(&before, &after),
            touched(&after),
            "the move at band {band}"
        );

        let before = revisions(&after);
        fixture.undo();
        let after = fixture.coverage();
        assert_eq!(
            rasterized(&before, &after),
            touched(&after),
            "the undo at band {band}"
        );
        fixture.assert_matches_the_reference(&format!("the undo at band {band}"));

        while fixture.terrain().is_some() {
            fixture.undo();
        }
    }
}
