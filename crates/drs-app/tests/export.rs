//! Exporting a Level through the headless editor with offscreen rendering: the real plugins of
//! the model, the history, `LibraryAccess`, `LibraryManager`, `ProjectManager`,
//! `AuthoringManager`, and `RenderEngine` under Bevy's default plugins without a window, over one
//! Asset Folder of solid-colour images, exporting to temporary files and asserting on the
//! decoded pixels and the messages answered.
//!
//! These tests draw on the GPU and fail, rather than skip, where no adapter exists.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    clippy::panic,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

use bevy::app::{App, PluginGroup, PluginsState};
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::message::Messages;
use bevy::math::{UVec2, Vec2};
use bevy::render::RenderPlugin;
use bevy::window::{ExitCondition, WindowPlugin};
use bevy::winit::WinitPlugin;
use drs_authoring_manager::AuthoringManagerPlugin;
use drs_history::{History, HistoryPlugin};
use drs_library_access::{LibraryAccessPlugin, register_library_source};
use drs_library_manager::LibraryManagerPlugin;
use drs_model::{
    AddFolder, Apply, CanonicalName, ChosenAsset, CommandFailed, EditorDirectories, Element,
    ElementId, ExportLevel, ExportRefused, FolderAdded, FolderKey, FolderRefused, Layer, Level,
    LevelExported, ModelPlugin, PlaceElement, Prop, SavedMark, Viewport,
};
use drs_project_manager::ProjectManagerPlugin;
use drs_render_engine::RenderEnginePlugin;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tempfile::TempDir;

/// A solid red image of one cell by one cell at the Grid's 256 pixels per cell.
const RED: &str = "red.png";
/// A solid blue image of two cells by one cell.
const BLUE: &str = "blue.png";
/// A solid green image of one cell by one cell.
const GREEN: &str = "green.png";
/// A solid magenta image of one cell by one cell at a place full of symbols, among them the `#`
/// and `?` an asset path would otherwise read as a label and a query.
#[cfg(not(windows))]
const MAGENTA: &str = "odd 'things' & more/caf\u{e9} #1? [v2].png";
/// A solid magenta image of one cell by one cell; Windows forbids `?` in file names.
#[cfg(windows)]
const MAGENTA: &str = "odd 'things' & more/caf\u{e9} #1 [v2].png";
/// The colour of the red image.
const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
/// The colour of the blue image.
const BLUE_PIXEL: [u8; 4] = [0, 0, 255, 255];
/// The colour of the green image.
const GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];
/// The colour of the magenta image.
const MAGENTA_PIXEL: [u8; 4] = [255, 0, 255, 255];
/// The background of an Export.
const BLACK_PIXEL: [u8; 4] = [0, 0, 0, 255];
/// The resolution most tests export at, which makes the default Bounds 240 pixels a side.
const PIXELS_PER_CELL: u32 = 8;
/// The tile size most tests export with.
const TILE: u32 = 128;
/// How many frames an Export may take before the test gives up.
const MOST_FRAMES: u32 = 2_000;

/// The Asset Folder every test places from, the headless editor with it added, and where the
/// Exports go.
struct Fixture {
    /// Keeps the temporary directory alive for the test.
    root: TempDir,
    /// The folder's key once added.
    key: FolderKey,
    /// The headless editor with the folder added.
    app: App,
}

/// Writes an opaque PNG of one colour and `size` pixels at `place` under `folder`.
fn png(folder: &Path, place: &str, size: UVec2, colour: [u8; 4]) {
    let path = folder.join(place);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture folder");
    }
    image::RgbaImage::from_pixel(size.x, size.y, image::Rgba(colour))
        .save(&path)
        .expect("fixture image");
}

/// A headless editor with offscreen rendering whose configuration and cache directories live
/// under `root`, started once: Bevy's default plugins without a window or winit, the `lib://`
/// asset source, and every plugin of the editor but the Editor's own panels.
///
/// The renderer initialises asynchronously and the render thread is set up when the plugins
/// are finished and cleaned up, which `App::run` would do; a test driving `update` by hand
/// does it here.
fn editor(root: &Path) -> App {
    let mut app = App::new();
    app.insert_resource(EditorDirectories::under(root));
    register_library_source(&mut app);
    app.add_plugins(
        bevy::DefaultPlugins
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..AssetPlugin::default()
            })
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                close_when_requested: false,
                ..WindowPlugin::default()
            })
            .set(RenderPlugin {
                synchronous_pipeline_compilation: true,
                ..RenderPlugin::default()
            })
            .disable::<WinitPlugin>(),
    );
    app.add_plugins((
        ModelPlugin,
        HistoryPlugin,
        LibraryAccessPlugin,
        LibraryManagerPlugin,
        ProjectManagerPlugin,
        AuthoringManagerPlugin,
        RenderEnginePlugin,
    ));
    while app.plugins_state() == PluginsState::Adding {
        bevy::tasks::tick_global_task_pools_on_main_thread();
    }
    app.finish();
    app.cleanup();
    app.update();
    app
}

/// Sends Add Asset Folder and returns what came back.
fn add_folder(app: &mut App, path: &Path, name: &str) -> FolderAdded {
    app.world_mut().write_message(AddFolder {
        path: path.to_path_buf(),
        name: CanonicalName(name.to_owned()),
    });
    app.update();
    let world = app.world_mut();
    let refused: Vec<FolderRefused> = world
        .resource_mut::<Messages<FolderRefused>>()
        .drain()
        .collect();
    assert!(
        refused.is_empty(),
        "the fixture folder was refused: {refused:?}"
    );
    world
        .resource_mut::<Messages<FolderAdded>>()
        .drain()
        .next()
        .expect("the fixture folder is added")
}

/// A decoded Export.
struct Picture {
    /// Its width in pixels.
    width: u32,
    /// Its height in pixels.
    height: u32,
    /// Its RGBA8 pixels, rows from the top.
    rgba: Vec<u8>,
}

impl Picture {
    /// Decodes the PNG at `path`.
    fn decode(path: &Path) -> Self {
        let image = image::open(path).expect("the Export decodes").to_rgba8();
        Self {
            width: image.width(),
            height: image.height(),
            rgba: image.into_raw(),
        }
    }

    /// The pixel at a column and a row counted from the top-left corner.
    fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        assert!(x < self.width && y < self.height, "({x}, {y}) is outside");
        let start = ((y * self.width + x) * 4) as usize;
        self.rgba[start..start + 4]
            .try_into()
            .expect("four bytes make a pixel")
    }

    /// The pixel at the centre of a cell, `(column, row)` counted from the Bounds' lower-left
    /// corner, in an Export at [`PIXELS_PER_CELL`].
    fn at_cell(&self, column: u32, row: u32) -> [u8; 4] {
        let x = column * PIXELS_PER_CELL + PIXELS_PER_CELL / 2;
        let y = self.height - (row * PIXELS_PER_CELL + PIXELS_PER_CELL / 2) - 1;
        self.pixel(x, y)
    }

    /// How many pixels have exactly `colour`.
    fn count(&self, colour: [u8; 4]) -> usize {
        self.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| **pixel == colour)
            .count()
    }
}

impl Fixture {
    /// Creates the fixture folder, starts the editor, and adds the folder as `Fixtures`.
    fn new() -> Self {
        let root = TempDir::new().expect("temporary root");
        let folder = root.path().join("fixtures");
        png(&folder, RED, UVec2::splat(256), RED_PIXEL);
        png(&folder, BLUE, UVec2::new(512, 256), BLUE_PIXEL);
        png(&folder, GREEN, UVec2::splat(256), GREEN_PIXEL);
        png(&folder, MAGENTA, UVec2::splat(256), MAGENTA_PIXEL);
        let mut app = editor(root.path());
        let added = add_folder(&mut app, &folder, "Fixtures");
        Self {
            root,
            key: added.key,
            app,
        }
    }

    /// Where an Export named `name` goes.
    fn output(&self, name: &str) -> PathBuf {
        self.root.path().join("exports").join(name)
    }

    /// The one Level of the new Project.
    fn level(&mut self) -> Entity {
        let world = self.app.world_mut();
        world
            .query::<(Entity, &Level)>()
            .single(world)
            .expect("exactly one Level")
            .0
    }

    /// The one Layer of the new Project.
    fn layer(&mut self) -> Entity {
        let world = self.app.world_mut();
        world
            .query::<(Entity, &Layer)>()
            .single(world)
            .expect("exactly one Layer")
            .0
    }

    /// The Place Element Command for a Prop of the Asset at `place` centred on `position`.
    fn placement(&mut self, place: &str, position: Vec2) -> Apply {
        Apply::PlaceElement(PlaceElement {
            layer: self.layer(),
            position,
            asset: ChosenAsset {
                folder: self.key.clone(),
                place: place.to_owned(),
            },
        })
    }

    /// Places a Prop of the Asset at `place` centred on `position` and runs one update, failing
    /// the test if the Command was refused.
    fn place(&mut self, place: &str, position: Vec2) {
        let command = self.placement(place, position);
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

    /// Sends Export Level and runs the editor until it answers.
    ///
    /// # Errors
    ///
    /// The refusal, when the Export was refused or failed.
    fn export(
        &mut self,
        pixels_per_cell: u32,
        name: &str,
        tile_size: u32,
    ) -> Result<LevelExported, ExportRefused> {
        fs::create_dir_all(self.root.path().join("exports")).expect("the exports folder");
        let request = ExportLevel {
            level: self.level(),
            pixels_per_cell,
            path: self.output(name),
            tile_size,
        };
        self.app.world_mut().write_message(request.clone());
        self.await_export(&request)
    }

    /// Runs the editor until the Export is answered, failing the test when it takes too long.
    ///
    /// # Errors
    ///
    /// The refusal, when the Export was refused or failed.
    #[expect(
        clippy::print_stderr,
        reason = "the timing is shown with the test's output, where `--nocapture` puts it"
    )]
    fn await_export(&mut self, request: &ExportLevel) -> Result<LevelExported, ExportRefused> {
        let started = Instant::now();
        for frame in 1..=MOST_FRAMES {
            self.app.update();
            let world = self.app.world_mut();
            if let Some(refused) = world
                .resource_mut::<Messages<ExportRefused>>()
                .drain()
                .next()
            {
                return Err(refused);
            }
            if let Some(exported) = world
                .resource_mut::<Messages<LevelExported>>()
                .drain()
                .next()
            {
                eprintln!(
                    "exported {} by {} pixels in {frame} frames and {:.3} s",
                    exported.width,
                    exported.height,
                    started.elapsed().as_secs_f64()
                );
                return Ok(exported);
            }
        }
        panic!(
            "the Export of {} at {} pixels per cell with tiles of {} was neither written nor \
             refused within {MOST_FRAMES} frames",
            request.path.display(),
            request.pixels_per_cell,
            request.tile_size
        );
    }

    /// Exports at [`PIXELS_PER_CELL`] with tiles of [`TILE`] and decodes the image.
    fn picture(&mut self, name: &str) -> Picture {
        let exported = self
            .export(PIXELS_PER_CELL, name, TILE)
            .expect("the Export is written");
        Picture::decode(&exported.path)
    }

    /// The Props on the Layer in stacking order, bottom first.
    fn props(&mut self) -> Vec<(ElementId, Element, Prop)> {
        let layer = self.layer();
        let world = self.app.world_mut();
        let children: Vec<Entity> = world
            .get::<Children>(layer)
            .map(|children| children.iter().copied().collect())
            .unwrap_or_default();
        children
            .into_iter()
            .map(|entity| {
                (
                    *world
                        .get::<ElementId>(entity)
                        .expect("an Element has an identity"),
                    world
                        .get::<Element>(entity)
                        .expect("a child is an Element")
                        .clone(),
                    world
                        .get::<Prop>(entity)
                        .expect("the Element is a Prop")
                        .clone(),
                )
            })
            .collect()
    }
}

/// Whether the file is a PNG by its signature.
fn is_png(path: &Path) -> bool {
    fs::read(path).is_ok_and(|bytes| bytes.starts_with(b"\x89PNG\r\n\x1a\n"))
}

/// A new Project's Bounds are thirty by thirty cells with their lower-left corner at the
/// Level's origin.
#[test]
fn bounds_to_start_with() {
    let mut fixture = Fixture::new();
    fixture.place(RED, Vec2::new(0.5, 0.5));
    fixture.place(GREEN, Vec2::new(29.5, 29.5));

    let picture = fixture.picture("bounds.png");

    assert_eq!((picture.width, picture.height), (240, 240));
    assert_eq!(picture.at_cell(0, 0), RED_PIXEL, "the cell at the origin");
    assert_eq!(picture.at_cell(29, 29), GREEN_PIXEL, "the last cell");
    assert_eq!(picture.pixel(0, 239), RED_PIXEL, "the lower-left pixel");
    assert_eq!(picture.pixel(239, 0), GREEN_PIXEL, "the upper-right pixel");
}

/// The Export is as many pixels wide as the Bounds' width in cells times the resolution, and
/// as many high as the height times the resolution, and shows exactly the Bounds.
#[test]
fn exactly_the_bounds() {
    let mut fixture = Fixture::new();
    fixture.place(RED, Vec2::new(0.5, 0.5));

    for (pixels_per_cell, side) in [(3, 90), (PIXELS_PER_CELL, 240), (50, 1500)] {
        let exported = fixture
            .export(pixels_per_cell, &format!("{pixels_per_cell}.png"), TILE)
            .expect("the Export is written");
        assert_eq!((exported.width, exported.height), (side, side));
        let picture = Picture::decode(&exported.path);
        assert_eq!((picture.width, picture.height), (side, side));
        assert_eq!(
            picture.count(RED_PIXEL),
            (pixels_per_cell * pixels_per_cell) as usize,
            "one cell of red at {pixels_per_cell} pixels per cell"
        );
        assert_eq!(picture.pixel(0, side - 1), RED_PIXEL);
        assert_eq!(picture.pixel(pixels_per_cell, side - 1), BLACK_PIXEL);
    }
}

/// An Element wholly outside the Bounds appears nowhere in the Export; an Element straddling
/// the edge appears only where it lies inside.
#[test]
fn clipped_at_the_edge() {
    let mut fixture = Fixture::new();
    fixture.place(RED, Vec2::new(-3.0, 5.5));
    fixture.place(GREEN, Vec2::new(10.5, 31.5));
    fixture.place(BLUE, Vec2::new(30.0, 10.5));

    let picture = fixture.picture("clipped.png");

    assert_eq!(picture.count(RED_PIXEL), 0, "wholly outside to the left");
    assert_eq!(picture.count(GREEN_PIXEL), 0, "wholly outside above");
    assert_eq!(
        picture.count(BLUE_PIXEL),
        (PIXELS_PER_CELL * PIXELS_PER_CELL) as usize,
        "the inside half of a two-cell-wide Prop"
    );
    assert_eq!(picture.at_cell(29, 10), BLUE_PIXEL);
    assert_eq!(picture.at_cell(28, 10), BLACK_PIXEL);
}

/// Every Element on the Level appears in the Export at its position and size in cells scaled
/// to the resolution, in stacking order, placeholders included.
#[test]
fn drawn_as_in_the_editor() {
    let mut fixture = Fixture::new();
    fixture.place(RED, Vec2::new(5.5, 5.5));
    fixture.place(BLUE, Vec2::new(6.0, 5.5));
    fixture.place(GREEN, Vec2::new(6.5, 5.5));
    fixture.place(RED, Vec2::new(20.5, 12.5));

    let picture = fixture.picture("stacked.png");

    assert_eq!(picture.at_cell(5, 5), BLUE_PIXEL, "blue over red");
    assert_eq!(picture.at_cell(6, 5), GREEN_PIXEL, "green over blue");
    assert_eq!(picture.at_cell(20, 12), RED_PIXEL, "a Prop on its own");
    assert_eq!(picture.at_cell(4, 5), BLACK_PIXEL);
    assert_eq!(picture.at_cell(7, 5), BLACK_PIXEL);
    assert_eq!(picture.at_cell(5, 6), BLACK_PIXEL);
    assert_eq!(picture.at_cell(5, 4), BLACK_PIXEL);
    let cell = (PIXELS_PER_CELL * PIXELS_PER_CELL) as usize;
    assert_eq!(picture.count(RED_PIXEL), cell, "the lone red Prop");
    assert_eq!(
        picture.count(BLUE_PIXEL),
        cell,
        "the half of blue green leaves"
    );
    assert_eq!(picture.count(GREEN_PIXEL), cell);
    // The lone red Prop's edges sit exactly on cell boundaries.
    let left = 20 * PIXELS_PER_CELL;
    let top = picture.height - 13 * PIXELS_PER_CELL;
    assert_eq!(picture.pixel(left, top), RED_PIXEL);
    assert_eq!(
        picture.pixel(left + PIXELS_PER_CELL - 1, top + PIXELS_PER_CELL - 1),
        RED_PIXEL
    );
    assert_eq!(picture.pixel(left - 1, top), BLACK_PIXEL);
    assert_eq!(picture.pixel(left, top - 1), BLACK_PIXEL);
    assert_eq!(picture.pixel(left + PIXELS_PER_CELL, top), BLACK_PIXEL);
    assert_eq!(picture.pixel(left, top + PIXELS_PER_CELL), BLACK_PIXEL);
}

/// A Prop whose Asset sits at a place holding spaces, quotes, non-ASCII letters, or symbols is
/// drawn from its own image, never as a placeholder.
#[test]
fn any_path_works_in_the_export() {
    let mut fixture = Fixture::new();
    fixture.place(MAGENTA, Vec2::new(3.5, 3.5));

    let picture = fixture.picture("odd.png");

    assert_eq!(picture.at_cell(3, 3), MAGENTA_PIXEL);
    assert_eq!(
        picture.count(MAGENTA_PIXEL),
        (PIXELS_PER_CELL * PIXELS_PER_CELL) as usize
    );
}

/// Every pixel no Element covers is opaque black, and the Export holds no transparent or
/// translucent pixel.
#[test]
fn opaque_background() {
    let mut fixture = Fixture::new();
    fixture.place(RED, Vec2::new(10.5, 10.5));

    let picture = fixture.picture("opaque.png");

    let cell = (PIXELS_PER_CELL * PIXELS_PER_CELL) as usize;
    assert_eq!(picture.count(RED_PIXEL), cell);
    assert_eq!(picture.count(BLACK_PIXEL), 240 * 240 - cell);
    assert!(
        picture
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| pixel[3] == u8::MAX),
        "every pixel is opaque"
    );
}

/// The Export is written as a PNG file at the path the Author chose, with `.png` added when the
/// name lacks it.
#[test]
fn exported_as_png() {
    let mut fixture = Fixture::new();

    let exported = fixture
        .export(PIXELS_PER_CELL, "map.png", TILE)
        .expect("the Export is written");
    assert_eq!(exported.path, fixture.output("map.png"));
    assert!(is_png(&exported.path));

    let exported = fixture
        .export(PIXELS_PER_CELL, "unnamed", TILE)
        .expect("the Export is written");
    assert_eq!(exported.path, fixture.output("unnamed.png"));
    assert!(is_png(&exported.path));
    assert!(!fixture.output("unnamed").exists());
}

/// The Export is captured only once every Asset the Level uses has loaded or failed to load.
#[test]
fn export_waits_for_assets() {
    let mut fixture = Fixture::new();
    fs::create_dir_all(fixture.root.path().join("exports")).expect("the exports folder");
    let placement = fixture.placement(GREEN, Vec2::new(3.5, 3.5));
    let request = ExportLevel {
        level: fixture.level(),
        pixels_per_cell: PIXELS_PER_CELL,
        path: fixture.output("waited.png"),
        tile_size: TILE,
    };

    // Placed and exported in the same frame: the image has not loaded when the Export starts.
    fixture.app.world_mut().write_message(placement);
    fixture.app.world_mut().write_message(request.clone());
    let exported = fixture
        .await_export(&request)
        .expect("the Export is written");

    let picture = Picture::decode(&exported.path);
    assert_eq!(picture.at_cell(3, 3), GREEN_PIXEL);
    assert_eq!(
        picture.count(GREEN_PIXEL),
        (PIXELS_PER_CELL * PIXELS_PER_CELL) as usize
    );
}

/// Two Exports of the same Level at the same resolution are byte-identical.
#[test]
fn same_level_same_image() {
    let mut fixture = Fixture::new();
    fixture.place(RED, Vec2::new(2.5, 3.5));
    fixture.place(BLUE, Vec2::new(12.0, 20.5));
    fixture.place(GREEN, Vec2::new(12.5, 20.5));

    let first = fixture
        .export(PIXELS_PER_CELL, "first.png", TILE)
        .expect("the Export is written");
    let second = fixture
        .export(PIXELS_PER_CELL, "second.png", TILE)
        .expect("the Export is written");

    let first = fs::read(first.path).expect("the first Export");
    let second = fs::read(second.path).expect("the second Export");
    assert!(first == second, "the two Exports differ");
}

/// The Export is the same image whatever the size of the tiles it is assembled from.
#[test]
fn tiles_leave_no_seams() {
    let mut fixture = Fixture::new();
    fixture.place(RED, Vec2::new(2.5, 3.5));
    fixture.place(BLUE, Vec2::new(16.0, 16.5));
    fixture.place(GREEN, Vec2::new(16.5, 16.5));
    fixture.place(RED, Vec2::new(29.5, 0.5));

    let mut exports = Vec::new();
    for tile_size in [64, TILE, 100, 1024] {
        let exported = fixture
            .export(
                PIXELS_PER_CELL,
                &format!("tiles-{tile_size}.png"),
                tile_size,
            )
            .expect("the Export is written");
        exports.push((tile_size, fs::read(exported.path).expect("the Export")));
    }

    let (_, reference) = &exports[0];
    for (tile_size, bytes) in &exports[1..] {
        assert!(
            bytes == reference,
            "the Export with tiles of {tile_size} differs from the one with tiles of 64"
        );
    }
    let picture = Picture::decode(&fixture.output("tiles-100.png"));
    assert_eq!(picture.at_cell(2, 3), RED_PIXEL);
    assert_eq!(picture.at_cell(15, 16), BLUE_PIXEL);
    assert_eq!(picture.at_cell(16, 16), GREEN_PIXEL);
    assert_eq!(picture.at_cell(29, 0), RED_PIXEL);
}

/// An Export changes no Element, the selection, or the view, and the Project has no more unsaved
/// changes after it than before.
#[test]
fn export_changes_nothing() {
    let mut fixture = Fixture::new();
    fixture.place(RED, Vec2::new(2.5, 3.5));
    fixture.place(BLUE, Vec2::new(12.0, 20.5));
    fixture.app.world_mut().resource_mut::<Viewport>().centre = Vec2::new(7.0, -3.0);
    fixture.app.update();
    let props = fixture.props();
    let viewport = *fixture.app.world().resource::<Viewport>();
    let history = fixture.app.world().resource::<History>();
    let (undo_depth, can_redo, position) =
        (history.undo_depth(), history.can_redo(), history.position());
    let mark = fixture.app.world().resource::<SavedMark>().clone();

    fixture
        .export(PIXELS_PER_CELL, "unchanged.png", TILE)
        .expect("the Export is written");

    assert_eq!(fixture.props(), props);
    assert_eq!(*fixture.app.world().resource::<Viewport>(), viewport);
    let history = fixture.app.world().resource::<History>();
    assert_eq!(history.undo_depth(), undo_depth, "an Export is not a step");
    assert_eq!(history.can_redo(), can_redo);
    assert_eq!(history.position(), position);
    assert_eq!(
        *fixture.app.world().resource::<SavedMark>(),
        mark,
        "an Export neither saves the Project nor leaves it with more unsaved changes"
    );
}

/// When the Export cannot be written, the Author is told the reason and no partial file
/// remains; a resolution below 1 or above 1024 pixels per cell is refused before anything is
/// written, with the limits named.
#[test]
fn a_failed_export_is_reported() {
    let mut fixture = Fixture::new();
    fixture.place(RED, Vec2::new(2.5, 3.5));

    for pixels_per_cell in [0, 2_000] {
        let refused = fixture
            .export(pixels_per_cell, "refused.png", TILE)
            .expect_err("a resolution outside the limits is refused");
        assert_eq!(refused.path, fixture.output("refused.png"));
        assert!(
            refused.reason.contains("1 to 1024"),
            "the limits are named: {}",
            refused.reason
        );
        assert!(!fixture.output("refused.png").exists());
        assert!(!fixture.output("refused.png.part").exists());
    }

    let vanished = fixture.root.path().join("vanished").join("map.png");
    let request = ExportLevel {
        level: fixture.level(),
        pixels_per_cell: PIXELS_PER_CELL,
        path: vanished.clone(),
        tile_size: TILE,
    };
    fixture.app.world_mut().write_message(request.clone());
    let refused = fixture
        .await_export(&request)
        .expect_err("a vanished location is refused");
    assert_eq!(refused.path, vanished);
    assert!(!refused.reason.is_empty());
    assert!(!vanished.exists());
    assert!(!fixture.root.path().join("vanished").exists());

    // The editor goes on exporting after a failure.
    let exported = fixture
        .export(PIXELS_PER_CELL, "after.png", TILE)
        .expect("the Export is written");
    assert!(is_png(&exported.path));
}
