//! How long the paint Engine's main-thread work takes to bring a Terrain's tiles up to a change
//! when they are rasterized on the GPU, timed through `ApplyStroke` over a generated Terrain of
//! the measured shape: 200 strokes of 40 segments over a Level of 60 by 60 cells, one in five
//! erasing, shown over a view of 4096 by 4096 screen pixels. The GPU's own time is not measured
//! here: Bevy waits for no GPU work, so only the work handed to it is timed.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    reason = "a test stops at the first thing that is not as expected"
)]

use bevy_asset::Assets;
use bevy_ecs::system::SystemState;
use bevy_ecs::world::World;
use bevy_image::Image;
use bevy_math::{Rect, Vec2, ops};
use drs_model::{BrushSettings, Stroke, Viewport};
use drs_paint_engine::{PaintCache, StrokeJobs, StrokeRasterizer, apply_stroke};
use std::time::{Duration, Instant};

/// The side of the Level the strokes lie on, in cells.
const LEVEL: f32 = 60.0;
/// The side of the view, in screen pixels.
const VIEW: f32 = 4096.0;
/// The zoom the view is shown at.
const ZOOM: f32 = 256.0;

/// A stroke of 40 segments of half to three quarters of a cell each, meandering from a point
/// chosen by `seed` and kept inside the Level, with a soft Brush one cell across, erasing when
/// `erase` says.
fn meandering(seed: u32, erase: bool) -> Stroke {
    let mut state = seed.wrapping_mul(2_654_435_761).wrapping_add(12_345) | 1;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        #[expect(
            clippy::cast_precision_loss,
            reason = "a pseudo-random fraction needs no more precision"
        )]
        let fraction = state as f32 / u32::MAX as f32;
        fraction
    };
    let (low, high) = (Vec2::splat(0.6), Vec2::splat(LEVEL - 0.6));
    let mut point = low + (high - low) * Vec2::new(next(), next());
    let mut angle = next() * std::f32::consts::TAU;
    let mut points = vec![point];
    for _ in 0..40 {
        angle += (next() - 0.5) * 0.9;
        let (sin, cos) = ops::sin_cos(angle);
        point = (point + Vec2::new(cos, sin) * (0.5 + next() * 0.25)).clamp(low, high);
        points.push(point);
    }
    Stroke {
        points,
        brush: BrushSettings {
            size: 1.0,
            hardness: 0.5,
            strength: 1.0,
        },
        erase,
    }
}

/// The Terrain: 200 strokes over the whole Level, every fifth erasing.
fn terrain() -> Vec<Stroke> {
    (0..200)
        .map(|number| meandering(1000 + number, number % 5 == 4))
        .collect()
}

/// The Viewport showing the view around `centre` at `zoom`.
fn viewport(centre: Vec2, zoom: f32) -> Viewport {
    Viewport {
        centre,
        zoom,
        area: Rect::new(0.0, 0.0, VIEW, VIEW),
    }
}

/// What rasterizing on the GPU hands its work to, as the editor's World holds it when it renders,
/// and the cache of a Terrain.
struct Bench {
    /// The World holding the images and the work handed over.
    world: World,
    /// The rasterizer, as a system takes it.
    state: SystemState<StrokeRasterizer<'static>>,
    /// The cache.
    cache: PaintCache,
}

impl Bench {
    /// A fresh World with `cache`.
    fn new(cache: PaintCache) -> Self {
        let mut world = World::new();
        world.init_resource::<Assets<Image>>();
        world.init_resource::<StrokeJobs>();
        let state = SystemState::new(&mut world);
        Self {
            world,
            state,
            cache,
        }
    }

    /// A fresh World with a cache of no stroke made for its GPU.
    fn fresh() -> Self {
        let mut world = World::new();
        world.init_resource::<Assets<Image>>();
        world.init_resource::<StrokeJobs>();
        let mut state = SystemState::<StrokeRasterizer<'static>>::new(&mut world);
        let cache = PaintCache::new(&state.get_mut(&mut world).expect("the rasterizer"));
        Self {
            world,
            state,
            cache,
        }
    }

    /// Brings the cache up to `strokes` and the view, returning whether the coverage changed.
    fn apply(&mut self, strokes: &[Stroke], viewport: &Viewport) -> bool {
        let mut rasterizer = self.state.get_mut(&mut self.world).expect("the rasterizer");
        apply_stroke(&mut self.cache, strokes, viewport, &mut rasterizer)
    }
}

/// The fastest of `runs` runs of `work` over what `setup` makes afresh for each, and what the
/// last run gave: the least disturbed run is the one that says what the code costs. Only `work`
/// is timed.
fn fastest<S, T>(
    runs: usize,
    mut setup: impl FnMut() -> S,
    mut work: impl FnMut(S) -> T,
) -> (Duration, T) {
    let mut best = Duration::MAX;
    let mut last = None;
    for _ in 0..runs {
        let input = setup();
        let start = Instant::now();
        let outcome = work(input);
        best = best.min(start.elapsed());
        last = Some(outcome);
    }
    (best, last.expect("at least one run"))
}

/// The number of a stroke whose path passes near `centre`.
fn near(strokes: &[Stroke], centre: Vec2) -> usize {
    strokes
        .iter()
        .position(|stroke| {
            !stroke.erase
                && stroke
                    .points
                    .iter()
                    .any(|point| point.distance(centre) < 3.0)
        })
        .expect("a stroke near the centre")
}

/// On a Terrain of 200 strokes of 40 segments over 60 by 60 cells, one in five erasing, shown
/// over 4096 by 4096 screen pixels at a zoom of 256, the paint Engine's main-thread work to bring
/// the tiles up to one new stroke takes under 1 ms, and up to one moved stroke under 2 ms, with
/// the paint Engine built as the Author's build builds it.
#[test]
fn painting_costs_the_main_thread_little() {
    let strokes = terrain();
    let segments: usize = strokes.iter().map(|stroke| stroke.points.len() - 1).sum();
    assert_eq!(strokes.len(), 200);
    assert_eq!(segments, 8_000);
    let centre = Vec2::splat(30.0);
    let view = viewport(centre, ZOOM);
    let mut warm = Bench::fresh();
    warm.apply(&strokes, &view);
    let warm = warm.cache;
    assert_eq!(warm.coverage().band, 256);
    assert!(warm.coverage().tiles.len() > 40, "the view shows ground");

    let mut more = strokes.clone();
    let mut laid = meandering(7, false);
    let offset = centre - laid.points[0];
    for point in &mut laid.points {
        *point += offset;
    }
    more.push(laid);
    let (painted, changed) = fastest(
        7,
        || Bench::new(warm.clone()),
        |mut bench| bench.apply(&more, &view),
    );
    eprintln!("one new stroke: {painted:?}");
    assert!(changed, "the new stroke shows");
    assert!(
        painted < Duration::from_millis(1),
        "one new stroke took {painted:?}"
    );

    let mut moved = strokes.clone();
    let number = near(&moved, centre);
    for point in &mut moved[number].points {
        *point += Vec2::new(0.5, 0.3);
    }
    let (edited, changed) = fastest(
        7,
        || Bench::new(warm.clone()),
        |mut bench| bench.apply(&moved, &view),
    );
    eprintln!("one moved stroke: {edited:?}");
    assert!(changed, "the moved stroke shows");
    assert!(
        edited < Duration::from_millis(2),
        "one moved stroke took {edited:?}"
    );
}

/// On that Terrain and view, the paint Engine's main-thread work for a switch from the base to
/// the band of 256 takes under 4 ms, and for a pan of 512 screen pixels at that band under 2 ms,
/// with the paint Engine built as the Author's build builds it.
#[test]
fn zooming_and_panning_cost_the_main_thread_little() {
    let strokes = terrain();
    assert_eq!(strokes.len(), 200);
    let centre = Vec2::splat(30.0);
    let mut far = Bench::fresh();
    far.apply(&strokes, &viewport(centre, 40.0));
    let far = far.cache;
    assert_eq!(far.coverage().band, 32);
    let close = viewport(centre, ZOOM);
    let (switched, changed) = fastest(
        7,
        || Bench::new(far.clone()),
        |mut bench| {
            let changed = bench.apply(&strokes, &close);
            (changed, bench.cache)
        },
    );
    let (changed, cache) = changed;
    eprintln!(
        "a switch to the band of 256 with {} tiles: {switched:?}",
        cache.coverage().tiles.len()
    );
    assert!(changed);
    assert_eq!(cache.coverage().band, 256);
    assert!(cache.coverage().tiles.len() > 40, "the view shows ground");
    assert!(
        switched < Duration::from_millis(4),
        "the switch took {switched:?}"
    );

    let panned = viewport(centre + Vec2::new(512.0 / ZOOM, 0.0), ZOOM);
    let (pan, changed) = fastest(
        7,
        || Bench::new(cache.clone()),
        |mut bench| bench.apply(&strokes, &panned),
    );
    eprintln!("a pan of 512 screen pixels: {pan:?}");
    assert!(changed, "the pan brings tiles in");
    assert!(pan < Duration::from_millis(2), "the pan took {pan:?}");
}
