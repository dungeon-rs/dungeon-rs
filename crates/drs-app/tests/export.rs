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

mod support;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::message::Messages;
use bevy::math::{UVec2, Vec2};
use drs_history::History;
use drs_model::{
    Apply, AssetAddress, Colour, CommandFailed, EditElement, Element, ElementChange, ElementId,
    ExportLevel, ExportRefused, FolderKey, Gesture, Layer, Level, LevelExported, OpenProject,
    PlaceElement, Placement, PortalAnchor, ProjectOpened, ProjectRefused, ProjectSaved, Prop,
    SaveProject, SavedMark, Side, Viewport,
};
use drs_model::{
    Bounds, BrushSettings, Paint, Resolution, ResolutionTable, Stroke, StrokeChange,
    TerrainCoverage,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use support::offscreen::editor;
use support::png;
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
/// A solid grey image of one cell by one cell that a Project comes to miss.
const GONE: &str = "gone.png";
/// A solid cyan door of two cells by half a cell.
const DOOR: &str = "door.png";
/// A door of two cells by one cell, its top half orange and its bottom half purple.
const TWO_TONE: &str = "two-tone.png";
/// The colour of the door.
const CYAN_PIXEL: [u8; 4] = [0, 255, 255, 255];
/// The colour of the two-tone door's top half.
const ORANGE_PIXEL: [u8; 4] = [255, 128, 0, 255];
/// The colour of the two-tone door's bottom half.
const PURPLE_PIXEL: [u8; 4] = [128, 0, 255, 255];
/// An image two cells a side in four quarters: red at the top left, blue at the top right, green
/// at the bottom left, and yellow at the bottom right.
const QUARTERS: &str = "quarters.png";
/// A solid red image of one cell by one cell, half transparent.
const GLASS: &str = "glass.png";
/// A file named as an image that holds no image, which no decoder reads.
const BROKEN: &str = "broken.png";
/// The colour of the grey image, which a Missing Asset never shows.
const GREY_PIXEL: [u8; 4] = [128, 128, 128, 255];
/// The placeholder of a Missing Asset over the Export's black background.
const PLACEHOLDER_PIXEL: [u8; 4] = [173, 68, 80, 255];
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
/// The colour the Walls are drawn in.
const YELLOW: Colour = Colour::rgb(255, 255, 0);
/// The colour of a yellow Wall in the Export.
const YELLOW_PIXEL: [u8; 4] = [255, 255, 0, 255];
/// The colour the Rooms' floors are drawn in.
const WHITE: Colour = Colour::rgb(255, 255, 255);
/// The colour of a white floor in the Export.
const WHITE_PIXEL: [u8; 4] = [255, 255, 255, 255];
/// The resolution the Walls are exported at, fine enough to tell a round cap from a square one.
const WALL_PIXELS_PER_CELL: u32 = 16;
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

    /// The pixel under a point of the Level in cells, in an Export at `pixels_per_cell` whose
    /// Bounds start at the Level's origin.
    fn at_point(&self, point: Vec2, pixels_per_cell: u32) -> [u8; 4] {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            reason = "the points asked for lie inside the image"
        )]
        let (x, y) = (
            (point.x * pixels_per_cell as f32).floor() as u32,
            (point.y * pixels_per_cell as f32).floor() as u32,
        );
        self.pixel(x, self.height - y - 1)
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
        png(&folder, GONE, UVec2::splat(256), GREY_PIXEL);
        png(&folder, DOOR, UVec2::new(512, 128), CYAN_PIXEL);
        image::RgbaImage::from_fn(512, 256, |_, y| {
            image::Rgba(if y < 128 { ORANGE_PIXEL } else { PURPLE_PIXEL })
        })
        .save(folder.join(TWO_TONE))
        .expect("the two-tone image");
        image::RgbaImage::from_fn(512, 512, |x, y| {
            image::Rgba(match (x < 256, y < 256) {
                (true, true) => RED_PIXEL,
                (false, true) => BLUE_PIXEL,
                (true, false) => GREEN_PIXEL,
                (false, false) => YELLOW_PIXEL,
            })
        })
        .save(folder.join(QUARTERS))
        .expect("the quartered image");
        png(&folder, GLASS, UVec2::splat(256), [255, 0, 0, 128]);
        fs::write(folder.join(BROKEN), b"no image here").expect("the broken image");
        let mut app = editor(root.path());
        let added = support::add_folder(&mut app, &folder, "Fixtures");
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
            placement: Placement::Prop {
                position,
                asset: AssetAddress {
                    folder: self.key.clone(),
                    place: place.to_owned(),
                },
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

    /// Places a Wall through `points` at `thickness` in `colour`, bending each segment that has
    /// a control, and runs the editor until it is placed, failing the test if a Command was
    /// refused.
    fn wall(&mut self, points: &[Vec2], controls: &[Option<Vec2>], thickness: f32, colour: Colour) {
        let layer = self.layer();
        self.run(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Wall {
                points: points.to_vec(),
                thickness,
                colour,
            },
        }));
        let element = {
            let world = self.app.world_mut();
            let children: Vec<Entity> = world
                .get::<Children>(layer)
                .map(|children| children.iter().copied().collect())
                .unwrap_or_default();
            *children
                .last()
                .and_then(|last| world.get::<ElementId>(*last))
                .expect("the Wall is the last child of the Layer")
        };
        for (segment, control) in controls.iter().enumerate() {
            if control.is_some() {
                self.run(Apply::EditElement(EditElement {
                    element,
                    change: ElementChange::Control {
                        segment,
                        position: *control,
                    },
                    gesture: Gesture::Single,
                }));
            }
        }
    }

    /// Places a Room through `points` with Walls `thickness` thick in yellow and a white floor,
    /// bending each edge that has a control, and runs the editor until it is placed, failing the
    /// test if a Command was refused. Returns its identity.
    fn room(&mut self, points: &[Vec2], controls: &[Option<Vec2>], thickness: f32) -> ElementId {
        let layer = self.layer();
        self.run(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Room {
                points: points.to_vec(),
                thickness,
                wall_colour: YELLOW,
                floor_colour: WHITE,
                cuts: false,
            },
        }));
        let room = self.last();
        for (segment, control) in controls.iter().enumerate() {
            if control.is_some() {
                self.edit(
                    room,
                    ElementChange::Control {
                        segment,
                        position: *control,
                    },
                );
            }
        }
        room
    }

    /// The identity of the last Element on the Layer.
    fn last(&mut self) -> ElementId {
        let layer = self.layer();
        let world = self.app.world_mut();
        let last = world
            .get::<Children>(layer)
            .and_then(|children| children.iter().last().copied())
            .expect("the Layer has Elements");
        *world.get::<ElementId>(last).expect("an Element")
    }

    /// Places a Portal of the Asset at `place`, set at `anchor` or freestanding on `position`,
    /// returning its identity.
    fn portal(&mut self, place: &str, position: Vec2, anchor: Option<PortalAnchor>) -> ElementId {
        let layer = self.layer();
        self.run(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Portal {
                position,
                asset: AssetAddress {
                    folder: self.key.clone(),
                    place: place.to_owned(),
                },
                anchor,
            },
        }));
        self.last()
    }

    /// Changes an Element on its own, failing the test if the change was refused.
    fn edit(&mut self, element: ElementId, change: ElementChange) {
        self.run(Apply::EditElement(EditElement {
            element,
            change,
            gesture: Gesture::Single,
        }));
    }

    /// Sends a Command and runs one update, failing the test if the Command was refused.
    fn run(&mut self, command: Apply) {
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

    /// Saves the Project as `name` under the root, failing the test on a refusal.
    fn save(&mut self, name: &str) -> PathBuf {
        self.app.world_mut().write_message(SaveProject {
            path: Some(self.root.path().join(name)),
        });
        self.app.update();
        let world = self.app.world_mut();
        let refused: Vec<ProjectRefused> = world
            .resource_mut::<Messages<ProjectRefused>>()
            .drain()
            .collect();
        assert!(refused.is_empty(), "the save was refused: {refused:?}");
        world
            .resource_mut::<Messages<ProjectSaved>>()
            .drain()
            .next()
            .expect("the Project is saved")
            .path
    }

    /// Opens the Project file at `path` and runs one update, failing the test on a refusal.
    fn open(&mut self, path: &Path) {
        self.app.world_mut().write_message(OpenProject {
            path: path.to_path_buf(),
        });
        self.app.update();
        let world = self.app.world_mut();
        let refused: Vec<ProjectRefused> = world
            .resource_mut::<Messages<ProjectRefused>>()
            .drain()
            .collect();
        assert!(refused.is_empty(), "the file was refused: {refused:?}");
        world
            .resource_mut::<Messages<ProjectOpened>>()
            .drain()
            .next()
            .expect("the file is opened");
    }

    /// Paints a stroke through `points` with `brush` and the Asset at `place`, failing the test
    /// if the Paint was refused.
    fn paint(&mut self, place: &str, points: &[Vec2], brush: BrushSettings) {
        let layer = self.layer();
        self.run(Apply::Paint(Paint {
            layer,
            stroke: Stroke {
                points: points.to_vec(),
                brush,
                erase: false,
            },
            asset: Some(AssetAddress {
                folder: self.key.clone(),
                place: place.to_owned(),
            }),
        }));
    }

    /// Lays an erase through `points` with `brush` on the Layer's Terrain, naming no image,
    /// failing the test if the Paint was refused.
    fn erase(&mut self, points: &[Vec2], brush: BrushSettings) {
        let layer = self.layer();
        self.run(Apply::Paint(Paint {
            layer,
            stroke: Stroke {
                points: points.to_vec(),
                brush,
                erase: true,
            },
            asset: None,
        }));
    }

    /// Makes the Bounds `size` cells with their lower-left corner at `origin`, so an Export at a
    /// high resolution stays small. No Command resizes the Bounds yet, so the fixture sets them
    /// in the World directly, as a Command would once there is one.
    fn bounds(&mut self, origin: bevy::math::IVec2, size: UVec2) {
        let world = self.app.world_mut();
        let mut bounds = world
            .query::<&mut Bounds>()
            .single_mut(world)
            .expect("the Project's Bounds");
        *bounds = Bounds { origin, size };
    }

    /// Writes the Viewport as the Editor does, showing 1024 by 768 screen pixels around `centre`
    /// at `zoom`, and runs one update.
    fn look(&mut self, centre: Vec2, zoom: f32) {
        let mut viewport = self
            .app
            .world_mut()
            .get_resource_mut::<Viewport>()
            .expect("the Viewport");
        viewport.centre = centre;
        viewport.zoom = zoom;
        viewport.area = bevy::math::Rect::new(0.0, 0.0, 1024.0, 768.0);
        self.app.update();
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
    fixture.place(GONE, Vec2::new(12.5, 20.5));
    // The saved file is made to record the grey Asset at a place its folder does not hold, so it
    // opens with that Asset Missing.
    let saved = fixture.save("missing.dungeon");
    let text = fs::read_to_string(&saved).expect("the saved Project");
    assert!(text.contains("\"gone.png\""), "the place is recorded");
    fs::write(
        &saved,
        text.replace("\"gone.png\"", "\"elsewhere/gone.png\""),
    )
    .expect("the Project rewritten");
    fixture.open(&saved);

    let picture = fixture.picture("stacked.png");

    assert_eq!(
        picture.at_cell(12, 20),
        PLACEHOLDER_PIXEL,
        "a Missing Asset"
    );
    assert_eq!(picture.count(GREY_PIXEL), 0, "never its image");
    assert_eq!(
        picture.count(PLACEHOLDER_PIXEL),
        (PIXELS_PER_CELL * PIXELS_PER_CELL) as usize,
        "the placeholder at the Missing Asset's size"
    );

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

/// A resolution below 1 or above 1024 pixels per cell is refused before anything is written,
/// with the limits named.
#[test]
fn resolution_within_limits() {
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
    assert_eq!(
        fs::read_dir(fixture.root.path().join("exports"))
            .expect("the exports folder")
            .count(),
        0,
        "nothing was written"
    );
}

/// When the Export cannot be written, the Author is told the reason and no partial file
/// remains, and the editor goes on exporting afterwards.
#[test]
fn a_failed_export_is_reported() {
    let mut fixture = Fixture::new();
    fixture.place(RED, Vec2::new(2.5, 3.5));

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

    let exported = fixture
        .export(PIXELS_PER_CELL, "after.png", TILE)
        .expect("the Export is written");
    assert!(is_png(&exported.path));
}

/// Opening another Project while an Export runs abandons the Export: it is answered as refused
/// because the Project was replaced, nothing of it is left on disk, and the Project opened
/// exports afterwards.
#[test]
fn an_open_refuses_a_running_export() {
    let mut fixture = Fixture::new();
    fixture.place(RED, Vec2::new(2.5, 3.5));
    let file = fixture.save("map.dungeon");
    let exports = fixture.root.path().join("exports");
    fs::create_dir_all(&exports).expect("the exports folder");
    let request = ExportLevel {
        level: fixture.level(),
        pixels_per_cell: 50,
        path: fixture.output("abandoned.png"),
        tile_size: TILE,
    };
    fixture.app.world_mut().write_message(request.clone());
    fixture.app.update();
    assert!(
        fixture
            .app
            .world_mut()
            .resource_mut::<Messages<LevelExported>>()
            .drain()
            .next()
            .is_none(),
        "an Export of 144 tiles takes more than one frame"
    );

    fixture.open(&file);

    let refused: Vec<ExportRefused> = fixture
        .app
        .world_mut()
        .resource_mut::<Messages<ExportRefused>>()
        .drain()
        .collect();
    assert_eq!(
        refused.len(),
        1,
        "the running Export is answered once: {refused:?}"
    );
    assert_eq!(refused[0].path, request.path);
    assert_eq!(refused[0].reason, "the Project was replaced");
    assert_eq!(
        fs::read_dir(&exports).expect("the exports folder").count(),
        0,
        "nothing is left of the abandoned Export"
    );

    let exported = fixture
        .export(PIXELS_PER_CELL, "after.png", TILE)
        .expect("the Export is written");
    assert!(is_png(&exported.path));
}

/// A Wall is drawn centred on its line, as wide as its thickness, with round joins at its points
/// and round caps at its ends, in its colour.
#[test]
fn a_wall_is_drawn_as_a_stroke() {
    let mut fixture = Fixture::new();
    fixture.wall(
        &[
            Vec2::new(5.0, 10.0),
            Vec2::new(15.0, 10.0),
            Vec2::new(15.0, 20.0),
        ],
        &[None, None],
        2.0,
        YELLOW,
    );

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "stroke.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    for x in [5.2, 8.0, 10.0, 14.9] {
        assert_eq!(at(x, 10.0), YELLOW_PIXEL, "along the line at x = {x}");
    }
    assert_eq!(at(10.0, 10.9), YELLOW_PIXEL, "within half the thickness");
    assert_eq!(at(10.0, 11.1), BLACK_PIXEL, "beyond half the thickness");
    assert_eq!(
        at(10.0, 8.9),
        BLACK_PIXEL,
        "beyond half the thickness below"
    );
    assert_eq!(at(4.2, 10.0), YELLOW_PIXEL, "just past the end, in the cap");
    assert_eq!(at(3.9, 10.0), BLACK_PIXEL, "past the cap");
    assert_eq!(
        at(4.15, 10.85),
        BLACK_PIXEL,
        "the corner a square cap would fill"
    );
    assert_eq!(at(15.6, 9.4), YELLOW_PIXEL, "the round join at the corner");
    assert_eq!(
        at(15.85, 9.15),
        BLACK_PIXEL,
        "the corner a mitred join would fill"
    );
    assert_eq!(at(15.0, 19.0), YELLOW_PIXEL, "the second segment");
}

/// A curved Wall follows its quadratic curve, not the chord between its points.
#[test]
fn a_curved_wall_follows_its_curve() {
    let mut fixture = Fixture::new();
    fixture.wall(
        &[Vec2::new(5.0, 5.0), Vec2::new(25.0, 5.0)],
        &[Some(Vec2::new(15.0, 25.0))],
        1.0,
        YELLOW,
    );

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "curve.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(at(15.0, 15.0), YELLOW_PIXEL, "the curve's middle");
    assert_eq!(at(10.0, 12.5), YELLOW_PIXEL, "a quarter along the curve");
    assert_eq!(at(15.0, 5.0), BLACK_PIXEL, "the chord's middle");
    assert_eq!(at(15.0, 16.0), BLACK_PIXEL, "beyond the curve");
}

/// A Wall appears in the Export above the Elements before it and below those after it.
#[test]
fn walls_stack_with_props() {
    let mut fixture = Fixture::new();
    fixture.place(RED, Vec2::new(10.5, 10.5));
    fixture.wall(
        &[Vec2::new(8.0, 10.5), Vec2::new(14.0, 10.5)],
        &[None],
        0.5,
        YELLOW,
    );
    fixture.place(GREEN, Vec2::new(12.5, 10.5));

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "stacked-walls.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(at(10.5, 10.5), YELLOW_PIXEL, "the Wall over the red Prop");
    assert_eq!(at(10.5, 10.9), RED_PIXEL, "the red Prop beside the Wall");
    assert_eq!(at(12.5, 10.5), GREEN_PIXEL, "the green Prop over the Wall");
    assert_eq!(at(9.0, 10.5), YELLOW_PIXEL, "the Wall on its own");
}

/// A Wall appears in the Export only where it lies inside the Bounds: a Wall with a point outside
/// leaves no trace past the edge.
#[test]
fn a_wall_is_clipped_at_the_edge() {
    let mut fixture = Fixture::new();
    fixture.wall(
        &[Vec2::new(-6.0, 15.0), Vec2::new(5.0, 15.0)],
        &[None],
        1.0,
        YELLOW,
    );
    fixture.wall(
        &[Vec2::new(-4.0, 3.0), Vec2::new(-1.0, 3.0)],
        &[None],
        1.0,
        YELLOW,
    );

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "clipped-walls.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(at(0.01, 15.0), YELLOW_PIXEL, "cut at the left edge");
    assert_eq!(at(5.4, 15.0), YELLOW_PIXEL, "the cap inside");
    assert_eq!(at(0.01, 3.0), BLACK_PIXEL, "the Wall wholly outside");
    let width = WALL_PIXELS_PER_CELL as usize;
    let inside = picture.count(YELLOW_PIXEL);
    let at_most = 6 * width * width;
    let at_least = 5 * width * width;
    assert!(
        (at_least..at_most).contains(&inside),
        "{inside} yellow pixels: the five cells inside and half a cap, nothing more"
    );
    let right_of_the_cap = WALL_PIXELS_PER_CELL * 56 / 10;
    for x in right_of_the_cap..picture.width {
        for y in 0..picture.height {
            assert_eq!(picture.pixel(x, y), BLACK_PIXEL, "({x}, {y})");
        }
    }
}

/// Where a Portal set into `host` at `segment` and `t` facing `side` is anchored.
fn anchored(host: ElementId, segment: usize, t: f32, side: Side) -> PortalAnchor {
    PortalAnchor {
        host,
        index: segment,
        t,
        side,
    }
}

/// A Wall is not drawn along any stretch a Portal set into it covers; at each end of such a stretch
/// the stroke ends squarely across the line. A Portal shorter than the Wall is thick stands in a
/// gap with the background beside it.
#[test]
fn a_wall_gives_way_to_its_portal() {
    let mut fixture = Fixture::new();
    fixture.wall(
        &[Vec2::new(5.0, 15.0), Vec2::new(25.0, 15.0)],
        &[None],
        2.0,
        YELLOW,
    );
    let wall = fixture.last();
    fixture.portal(DOOR, Vec2::ZERO, Some(anchored(wall, 0, 0.5, Side::Left)));

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "gap.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(at(15.0, 15.0), CYAN_PIXEL, "the Portal at its centre");
    assert_eq!(at(15.0, 15.6), BLACK_PIXEL, "the gap beside the Portal");
    assert_eq!(at(14.2, 14.2), BLACK_PIXEL, "the gap below the Portal");
    assert_eq!(
        at(16.1, 15.0),
        YELLOW_PIXEL,
        "the Wall just past the stretch"
    );
    assert_eq!(
        at(16.1, 15.9),
        YELLOW_PIXEL,
        "a square end, not a round one"
    );
    assert_eq!(
        at(13.9, 14.1),
        YELLOW_PIXEL,
        "the square end before the stretch"
    );
    assert_eq!(at(4.2, 15.0), YELLOW_PIXEL, "the Wall's own cap");
}

/// A Portal across a point of a Wall leaves out both segments within its stretch, the join at the
/// point included.
#[test]
fn a_gap_follows_the_corner() {
    let mut fixture = Fixture::new();
    fixture.wall(
        &[
            Vec2::new(5.0, 10.0),
            Vec2::new(15.0, 10.0),
            Vec2::new(15.0, 20.0),
        ],
        &[None, None],
        2.0,
        YELLOW,
    );
    let wall = fixture.last();
    fixture.portal(DOOR, Vec2::ZERO, Some(anchored(wall, 0, 0.95, Side::Left)));

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "corner-gap.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(at(14.5, 10.0), CYAN_PIXEL, "the Portal");
    assert_eq!(at(13.8, 9.4), BLACK_PIXEL, "the gap on the first segment");
    assert_eq!(at(15.7, 10.3), BLACK_PIXEL, "the gap on the second segment");
    assert_eq!(at(15.7, 9.3), BLACK_PIXEL, "no join at the point");
    assert_eq!(
        at(13.0, 10.0),
        YELLOW_PIXEL,
        "the first segment before the stretch"
    );
    assert_eq!(
        at(15.0, 11.0),
        YELLOW_PIXEL,
        "the second segment past the stretch"
    );
}

/// A Portal set into a Wall is turned to the Wall's direction with its image's top facing its side,
/// drawn as it is facing the left and mirrored across the line facing the right.
#[test]
fn a_portal_faces_its_side() {
    let mut fixture = Fixture::new();
    fixture.wall(
        &[Vec2::new(15.0, 5.0), Vec2::new(15.0, 25.0)],
        &[None],
        0.5,
        YELLOW,
    );
    let wall = fixture.last();
    let door = fixture.portal(
        TWO_TONE,
        Vec2::ZERO,
        Some(anchored(wall, 0, 0.5, Side::Left)),
    );

    let left = fixture
        .export(WALL_PIXELS_PER_CELL, "left.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&left.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);
    assert_eq!(
        at(14.7, 15.0),
        ORANGE_PIXEL,
        "the top to the left of the Wall"
    );
    assert_eq!(at(15.3, 15.0), PURPLE_PIXEL, "the bottom to the right");
    assert_eq!(at(14.7, 15.9), ORANGE_PIXEL, "the width along the Wall");
    assert_eq!(at(14.7, 16.1), BLACK_PIXEL, "past the Portal's width");

    fixture.edit(door, ElementChange::Side(Side::Right));
    let right = fixture
        .export(WALL_PIXELS_PER_CELL, "right.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&right.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);
    assert_eq!(
        at(15.3, 15.0),
        ORANGE_PIXEL,
        "the top to the right after a flip"
    );
    assert_eq!(at(14.7, 15.0), PURPLE_PIXEL, "the bottom to the left");
}

/// A freestanding Portal is drawn centred on its position, turned counter-clockwise by its
/// rotation.
#[test]
fn a_freestanding_portal_is_turned() {
    let mut fixture = Fixture::new();
    let door = fixture.portal(BLUE, Vec2::new(15.0, 15.0), None);
    fixture.edit(door, ElementChange::Rotation(std::f32::consts::FRAC_PI_2));

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "turned.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(at(15.0, 15.0), BLUE_PIXEL, "its centre");
    assert_eq!(at(15.0, 15.9), BLUE_PIXEL, "its width up the turn");
    assert_eq!(at(15.0, 14.1), BLUE_PIXEL, "its width down the turn");
    assert_eq!(at(15.9, 15.0), BLACK_PIXEL, "beside its height");
    let cells = (WALL_PIXELS_PER_CELL * WALL_PIXELS_PER_CELL) as usize;
    assert_eq!(
        picture.count(BLUE_PIXEL),
        2 * cells,
        "a tall area of two cells"
    );
}

/// Every Portal is drawn at its place in the stacking order, Portals that overlap each other
/// included, and the Wall is left out along what either covers.
#[test]
fn overlapping_portals_stack() {
    let mut fixture = Fixture::new();
    fixture.wall(
        &[Vec2::new(5.0, 15.0), Vec2::new(25.0, 15.0)],
        &[None],
        2.0,
        YELLOW,
    );
    let wall = fixture.last();
    fixture.portal(RED, Vec2::ZERO, Some(anchored(wall, 0, 0.5, Side::Left)));
    fixture.portal(
        GREEN,
        Vec2::ZERO,
        Some(anchored(wall, 0, 0.525, Side::Left)),
    );

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "overlap.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(at(14.6, 15.0), RED_PIXEL, "the first Portal alone");
    assert_eq!(
        at(15.3, 15.0),
        GREEN_PIXEL,
        "the later Portal where they overlap"
    );
    assert_eq!(at(15.9, 15.0), GREEN_PIXEL, "the later Portal alone");
    assert_eq!(at(14.6, 15.8), BLACK_PIXEL, "the gap of the first");
    assert_eq!(at(15.9, 15.8), BLACK_PIXEL, "the gap of the later");
    assert_eq!(at(16.6, 15.0), YELLOW_PIXEL, "the Wall past both");
}

/// A Portal whose Asset is Missing is drawn as the placeholder of its size, turned and set into
/// its Wall as its image would be, and the Wall still gives way along it.
#[test]
fn a_missing_portal_keeps_its_gap() {
    let mut fixture = Fixture::new();
    fixture.wall(
        &[Vec2::new(15.0, 5.0), Vec2::new(15.0, 25.0)],
        &[None],
        2.0,
        YELLOW,
    );
    let wall = fixture.last();
    fixture.portal(DOOR, Vec2::ZERO, Some(anchored(wall, 0, 0.5, Side::Left)));
    // The saved file is made to record the door at a place its folder does not hold, so it
    // opens with that Asset Missing.
    let saved = fixture.save("missing-door.dungeon");
    let text = fs::read_to_string(&saved).expect("the saved Project");
    assert!(text.contains("\"door.png\""), "the place is recorded");
    fs::write(
        &saved,
        text.replace("\"door.png\"", "\"elsewhere/door.png\""),
    )
    .expect("the Project rewritten");
    fixture.open(&saved);

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "missing-door.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(at(15.0, 15.0), PLACEHOLDER_PIXEL, "the placeholder");
    assert_eq!(
        at(15.1, 15.9),
        PLACEHOLDER_PIXEL,
        "its width along the Wall"
    );
    assert_eq!(picture.count(CYAN_PIXEL), 0, "never its image");
    let cells = (WALL_PIXELS_PER_CELL * WALL_PIXELS_PER_CELL) as usize;
    assert_eq!(
        picture.count(PLACEHOLDER_PIXEL),
        cells,
        "two cells along the Wall by half a cell across"
    );
    assert_eq!(at(15.6, 15.0), BLACK_PIXEL, "the gap beside it");
    assert_eq!(at(15.6, 16.1), YELLOW_PIXEL, "the Wall past the stretch");
}

/// A freestanding Portal that is mirrored is drawn flipped across its length: its image's top
/// half below and its bottom half above, at the same width.
#[test]
fn a_mirrored_portal_is_flipped() {
    let mut fixture = Fixture::new();
    let door = fixture.portal(TWO_TONE, Vec2::new(15.0, 15.0), None);

    let upright = fixture
        .export(WALL_PIXELS_PER_CELL, "upright.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&upright.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);
    assert_eq!(at(15.0, 15.3), ORANGE_PIXEL, "the top above");
    assert_eq!(at(15.0, 14.7), PURPLE_PIXEL, "the bottom below");

    fixture.edit(door, ElementChange::Mirrored(true));
    let mirrored = fixture
        .export(WALL_PIXELS_PER_CELL, "mirrored.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&mirrored.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);
    assert_eq!(at(15.0, 14.7), ORANGE_PIXEL, "the top below");
    assert_eq!(at(15.0, 15.3), PURPLE_PIXEL, "the bottom above");
    assert_eq!(at(15.9, 14.7), ORANGE_PIXEL, "the same width");
    assert_eq!(at(16.1, 14.7), BLACK_PIXEL, "past its width");
}

/// A stretch that reaches an end of the Wall leaves no cap there: the Wall is not drawn around
/// its end point, and it resumes squarely past the stretch.
#[test]
fn no_cap_where_a_portal_reaches_the_end() {
    let mut fixture = Fixture::new();
    fixture.wall(
        &[Vec2::new(5.0, 15.0), Vec2::new(25.0, 15.0)],
        &[None],
        2.0,
        YELLOW,
    );
    let wall = fixture.last();
    fixture.portal(DOOR, Vec2::ZERO, Some(anchored(wall, 0, 0.0, Side::Left)));

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "end.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(at(5.0, 15.0), CYAN_PIXEL, "the Portal on the end point");
    assert_eq!(at(4.5, 15.5), BLACK_PIXEL, "no cap beyond the end");
    assert_eq!(at(4.5, 14.5), BLACK_PIXEL, "no cap on the other side");
    assert_eq!(at(5.5, 15.6), BLACK_PIXEL, "the gap along the stretch");
    assert_eq!(at(6.1, 15.9), YELLOW_PIXEL, "a square end past it");
    assert_eq!(at(25.8, 15.0), YELLOW_PIXEL, "the cap at the far end");
}

/// A Brush of `size` cells, `hardness`, and `strength`.
fn brush(size: f32, hardness: f32, strength: f32) -> BrushSettings {
    BrushSettings {
        size,
        hardness,
        strength,
    }
}

/// Whether a pixel lies strictly between the red of the image and the black background: some
/// red, but not all of it, and nothing else.
fn partly_red(pixel: [u8; 4]) -> bool {
    pixel[0] > 0 && pixel[0] < 255 && pixel[1] == 0 && pixel[2] == 0 && pixel[3] == 255
}

/// A Terrain is drawn as its Material over what lies below it, as opaque as its coverage times
/// the image's own opacity: the image where the coverage is full, nothing where there is none,
/// partly in a soft edge, and partly where the image itself is half transparent.
#[test]
fn a_stroke_shows_its_material() {
    let mut fixture = Fixture::new();
    fixture.paint(
        RED,
        &[Vec2::new(5.0, 15.0), Vec2::new(25.0, 15.0)],
        brush(4.0, 0.5, 1.0),
    );

    let picture = fixture.picture("stroke.png");
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), PIXELS_PER_CELL);

    assert_eq!(at(15.0, 15.0), RED_PIXEL, "on the path");
    assert_eq!(at(15.0, 15.9), RED_PIXEL, "within the hardness");
    assert_eq!(at(15.0, 17.6), BLACK_PIXEL, "beyond the radius");
    assert_eq!(at(2.4, 15.0), BLACK_PIXEL, "beyond the round end");
    assert!(
        partly_red(at(15.0, 16.5)),
        "the soft edge: {:?}",
        at(15.0, 16.5)
    );

    let mut glass = Fixture::new();
    glass.paint(
        GLASS,
        &[Vec2::new(5.0, 15.0), Vec2::new(25.0, 15.0)],
        brush(4.0, 1.0, 1.0),
    );
    glass.place(GLASS, Vec2::new(15.5, 5.5));
    let picture = glass.picture("glass.png");
    let middle = picture.at_point(Vec2::new(15.0, 15.0), PIXELS_PER_CELL);
    assert!(
        partly_red(middle),
        "a half-transparent image under full coverage: {middle:?}"
    );
    assert_eq!(
        middle,
        picture.at_point(Vec2::new(15.5, 5.5), PIXELS_PER_CELL),
        "as the image shows as a Prop"
    );
}

/// A stroke of less than full strength shows its image partly over what is below it.
#[test]
fn a_weaker_stroke_shows_partly() {
    let mut fixture = Fixture::new();
    fixture.paint(
        RED,
        &[Vec2::new(5.0, 15.0), Vec2::new(25.0, 15.0)],
        brush(4.0, 1.0, 0.5),
    );

    let picture = fixture.picture("weak.png");
    let middle = picture.at_point(Vec2::new(15.0, 15.0), PIXELS_PER_CELL);

    assert!(
        partly_red(middle),
        "the middle of a half-strength stroke: {middle:?}"
    );
    assert_eq!(
        picture.at_point(Vec2::new(15.0, 17.6), PIXELS_PER_CELL),
        BLACK_PIXEL
    );
}

/// A Terrain's Material shows its image repeated edge to edge from the Level's origin at its
/// natural size, upright, the same across two strokes that meet: one repetition of an image two
/// cells a side covers the cells from an even column and an even row, its top half in the odd
/// row.
#[test]
fn terrain_is_tiled_at_natural_size() {
    let mut fixture = Fixture::new();
    let hard = brush(6.0, 1.0, 1.0);
    fixture.paint(
        QUARTERS,
        &[Vec2::new(2.0, 10.5), Vec2::new(6.0, 10.5)],
        hard,
    );
    fixture.paint(
        QUARTERS,
        &[Vec2::new(6.5, 10.5), Vec2::new(11.0, 10.5)],
        hard,
    );

    let picture = fixture.picture("tiled.png");

    for row in 9..=12 {
        for column in 1..=11 {
            let expected = match (column % 2 == 0, row % 2 == 1) {
                (true, true) => RED_PIXEL,
                (false, true) => BLUE_PIXEL,
                (true, false) => GREEN_PIXEL,
                (false, false) => YELLOW_PIXEL,
            };
            assert_eq!(
                picture.at_cell(column, row),
                expected,
                "cell ({column}, {row})"
            );
        }
    }
    assert_eq!(picture.at_cell(5, 15), BLACK_PIXEL, "beyond the strokes");
}

/// A Terrain lies under the Props and Walls placed before it was painted, and a Prop placed after
/// lies over it.
#[test]
fn terrain_lies_under_props_and_walls() {
    let mut fixture = Fixture::new();
    fixture.place(GREEN, Vec2::new(12.5, 11.5));
    fixture.wall(
        &[Vec2::new(8.0, 12.5), Vec2::new(16.0, 12.5)],
        &[None],
        0.5,
        YELLOW,
    );
    fixture.paint(
        RED,
        &[Vec2::new(6.0, 12.0), Vec2::new(20.0, 12.0)],
        brush(4.0, 1.0, 1.0),
    );
    fixture.place(BLUE, Vec2::new(18.0, 11.5));

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "under.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(at(12.5, 11.5), GREEN_PIXEL, "the Prop placed before");
    assert_eq!(at(10.0, 12.5), YELLOW_PIXEL, "the Wall drawn before");
    assert_eq!(at(10.0, 11.0), RED_PIXEL, "the Terrain on its own");
    assert_eq!(at(18.0, 11.5), BLUE_PIXEL, "the Prop placed after");
}

/// Each pixel of the Export shows the coverage at its centre as the strokes give it at the
/// Export's resolution: a hard dab exported at 200 pixels per cell shows its image at the pixel
/// whose centre lies just inside its radius and the background at the one just outside, which
/// the editor's 32 pixels per cell could not tell apart.
#[test]
fn terrain_at_the_exports_resolution() {
    const FINE: u32 = 200;
    let mut fixture = Fixture::new();
    fixture.bounds(bevy::math::IVec2::new(8, 8), UVec2::new(4, 4));
    // The centre of the pixel 2000 from the origin, a pixel whose centre lies on the dab's.
    let centre = Vec2::splat(2000.5 / 200.0);
    fixture.paint(RED, &[centre], brush(2.0, 1.0, 1.0));

    let exported = fixture
        .export(FINE, "fine.png", 512)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    // The dab's centre is pixel (400, 400) from the Bounds' lower-left corner; a radius of one
    // cell is 200 pixels.
    let row = picture.height - 400 - 1;

    assert_eq!(picture.pixel(400, row), RED_PIXEL, "the centre");
    assert_eq!(picture.pixel(599, row), RED_PIXEL, "just inside the radius");
    assert_eq!(picture.pixel(601, row), BLACK_PIXEL, "just outside");
    assert_eq!(
        picture.pixel(201, row),
        RED_PIXEL,
        "just inside, the other side"
    );
    assert_eq!(
        picture.pixel(199, row),
        BLACK_PIXEL,
        "just outside, the other side"
    );
}

/// A painted Export is the same image whatever the size of the tiles it is assembled from.
#[test]
fn painted_tiles_leave_no_seams() {
    let mut fixture = Fixture::new();
    fixture.paint(
        RED,
        &[
            Vec2::new(3.0, 3.0),
            Vec2::new(16.0, 14.0),
            Vec2::new(27.0, 6.0),
        ],
        brush(5.0, 0.3, 1.0),
    );
    fixture.paint(RED, &[Vec2::new(12.4, 12.6)], brush(7.0, 0.0, 0.7));

    let mut exports = Vec::new();
    for tile_size in [64, 100] {
        let exported = fixture
            .export(
                PIXELS_PER_CELL,
                &format!("painted-tiles-{tile_size}.png"),
                tile_size,
            )
            .expect("the Export is written");
        exports.push(fs::read(exported.path).expect("the Export"));
    }

    assert!(
        exports[0] == exports[1],
        "the Export with tiles of 100 differs from the one with tiles of 64"
    );
    let picture = Picture::decode(&fixture.output("painted-tiles-100.png"));
    assert_eq!(
        picture.at_point(Vec2::new(16.0, 14.0), PIXELS_PER_CELL),
        RED_PIXEL
    );
}

/// Two Exports of the same painted Level at the same resolution are byte-identical.
#[test]
fn painted_levels_export_the_same() {
    let mut fixture = Fixture::new();
    fixture.paint(
        QUARTERS,
        &[Vec2::new(3.0, 3.0), Vec2::new(16.0, 14.0)],
        brush(5.0, 0.5, 0.8),
    );
    fixture.place(GREEN, Vec2::new(8.5, 8.5));

    let first = fixture
        .export(PIXELS_PER_CELL, "painted-first.png", TILE)
        .expect("the Export is written");
    let second = fixture
        .export(PIXELS_PER_CELL, "painted-second.png", TILE)
        .expect("the Export is written");

    let first = fs::read(first.path).expect("the first Export");
    let second = fs::read(second.path).expect("the second Export");
    assert!(first == second, "the two Exports differ");
}

/// The Export computes Terrain on the CPU at its own resolution, never from the tiles the editor
/// shows: the same Level exported with the view at the base band and at the band of 256 is
/// byte-identical.
#[test]
fn the_export_keeps_the_cpu_rasterizer() {
    let mut fixture = Fixture::new();
    fixture.look(Vec2::new(10.0, 10.0), 40.0);
    fixture.paint(
        QUARTERS,
        &[
            Vec2::new(3.0, 3.0),
            Vec2::new(16.0, 14.0),
            Vec2::new(22.0, 5.0),
        ],
        brush(3.0, 1.0, 0.9),
    );
    fixture.erase(
        &[Vec2::new(4.0, 12.0), Vec2::new(20.0, 6.0)],
        brush(1.5, 0.4, 0.7),
    );

    let at_the_base = fixture
        .export(32, "terrain-base.png", 256)
        .expect("the Export is written");
    fixture.look(Vec2::new(12.0, 9.0), 300.0);
    let world = fixture.app.world_mut();
    let shown = world
        .query::<&TerrainCoverage>()
        .single(world)
        .expect("the Terrain's coverage")
        .band;
    assert_eq!(shown, 256, "the editor shows the band of 256");
    let close_up = fixture
        .export(32, "terrain-close.png", 256)
        .expect("the Export is written");

    let at_the_base = fs::read(at_the_base.path).expect("the first Export");
    let close_up = fs::read(close_up.path).expect("the second Export");
    assert!(at_the_base == close_up, "the two Exports differ");
}

/// A Terrain whose image is Missing, or cannot be decoded, is drawn as its coverage in the
/// placeholder's colour, not as a box.
#[test]
fn a_missing_terrain_image_keeps_its_shape() {
    let mut fixture = Fixture::new();
    fixture.paint(
        GONE,
        &[Vec2::new(5.0, 15.0), Vec2::new(25.0, 15.0)],
        brush(4.0, 1.0, 1.0),
    );
    // The saved file is made to record the grey Asset at a place its folder does not hold, so it
    // opens with that Asset Missing.
    let saved = fixture.save("missing-terrain.dungeon");
    let text = fs::read_to_string(&saved).expect("the saved Project");
    fs::write(
        &saved,
        text.replace("\"gone.png\"", "\"elsewhere/gone.png\""),
    )
    .expect("the Project rewritten");
    fixture.open(&saved);

    let picture = fixture.picture("missing-terrain.png");
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), PIXELS_PER_CELL);

    assert_eq!(at(15.0, 15.0), PLACEHOLDER_PIXEL, "on the path");
    assert_eq!(at(15.0, 17.6), BLACK_PIXEL, "beyond the radius");
    assert_eq!(
        at(3.6, 13.6),
        BLACK_PIXEL,
        "in the box, beyond the round end"
    );
    assert_eq!(picture.count(GREY_PIXEL), 0, "never its image");

    // The same file made to record a file at a place the folder holds that is no image.
    fs::write(&saved, text.replace("\"gone.png\"", "\"broken.png\""))
        .expect("the Project rewritten");
    fixture.open(&saved);
    let world = fixture.app.world_mut();
    let rows = world
        .query::<&ResolutionTable>()
        .single(world)
        .expect("the Project's resolutions")
        .rows
        .clone();
    assert!(
        rows.iter()
            .all(|row| matches!(row, Resolution::Resolved { .. })),
        "the file is found, so only decoding it fails: {rows:?}"
    );

    let picture = fixture.picture("broken-terrain.png");
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), PIXELS_PER_CELL);

    assert_eq!(
        at(15.0, 15.0),
        PLACEHOLDER_PIXEL,
        "on the path, undecodable"
    );
    assert_eq!(
        at(15.0, 17.6),
        BLACK_PIXEL,
        "beyond the radius, undecodable"
    );
    assert_eq!(
        at(3.6, 13.6),
        BLACK_PIXEL,
        "in the box, beyond the round end, undecodable"
    );
}

/// An Edit Element setting a Terrain's Material to another image Asset makes every stroke show
/// that image in the Export.
#[test]
fn the_material_stays_editable() {
    let mut fixture = Fixture::new();
    fixture.paint(
        RED,
        &[Vec2::new(5.0, 15.0), Vec2::new(25.0, 15.0)],
        brush(4.0, 1.0, 1.0),
    );
    let terrain = fixture.last();
    let green = AssetAddress {
        folder: fixture.key.clone(),
        place: GREEN.to_owned(),
    };
    fixture.edit(terrain, ElementChange::Material(green));

    let picture = fixture.picture("new-image.png");

    assert_eq!(
        picture.at_point(Vec2::new(15.0, 15.0), PIXELS_PER_CELL),
        GREEN_PIXEL,
        "the new image along the stroke"
    );
    assert_eq!(picture.count(RED_PIXEL), 0, "never the image it had");
}

/// A rectangle ten cells wide and seven high from (5, 5), counter-clockwise from its lower-left
/// corner.
const ROOM: [Vec2; 4] = [
    Vec2::new(5.0, 5.0),
    Vec2::new(15.0, 5.0),
    Vec2::new(15.0, 12.0),
    Vec2::new(5.0, 12.0),
];

/// A Room's floor covers every place its outline winds around, up to the outline's line, in the
/// floor colour, and nothing beyond its Walls.
#[test]
fn a_room_fills_its_floor() {
    let mut fixture = Fixture::new();
    fixture.room(&ROOM, &[], 1.0);

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "floor.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(at(10.0, 8.5), WHITE_PIXEL, "the floor at the centre");
    assert_eq!(at(5.6, 11.4), WHITE_PIXEL, "the floor in a corner");
    assert_eq!(
        at(4.4, 8.5),
        BLACK_PIXEL,
        "outside, beyond half the thickness"
    );
    assert_eq!(
        at(10.0, 12.6),
        BLACK_PIXEL,
        "above, beyond half the thickness"
    );
}

/// A curved edge's floor reaches its curve, not the chord between its points.
#[test]
fn a_curved_room_fills_its_curve() {
    let mut fixture = Fixture::new();
    fixture.room(
        &[
            Vec2::new(5.0, 15.0),
            Vec2::new(25.0, 15.0),
            Vec2::new(25.0, 25.0),
            Vec2::new(5.0, 25.0),
        ],
        &[Some(Vec2::new(15.0, 5.0))],
        0.5,
    );

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "curved-floor.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(
        at(15.0, 12.5),
        WHITE_PIXEL,
        "between the chord and the curve"
    );
    assert_eq!(
        at(15.0, 10.0),
        YELLOW_PIXEL,
        "the Wall on the curve's middle"
    );
    assert_eq!(at(15.0, 9.4), BLACK_PIXEL, "beyond the curve");
}

/// A Room's Walls are its outline drawn centred on the line, as wide as its thickness, with a
/// round join at every point, the first included, in its wall colour.
#[test]
fn a_room_is_walled_all_round() {
    let mut fixture = Fixture::new();
    fixture.room(&ROOM, &[], 1.0);

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "walled.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    for (middle, across) in [
        (Vec2::new(10.0, 5.0), Vec2::Y),
        (Vec2::new(15.0, 8.5), Vec2::X),
        (Vec2::new(10.0, 12.0), Vec2::Y),
        (Vec2::new(5.0, 8.5), Vec2::X),
    ] {
        for offset in [-0.4, 0.0, 0.4] {
            let p = middle + across * offset;
            assert_eq!(at(p.x, p.y), YELLOW_PIXEL, "the Wall at {p}");
        }
    }
    assert_eq!(
        at(4.7, 4.7),
        YELLOW_PIXEL,
        "the round join at the first point"
    );
    assert_eq!(
        at(4.6, 4.6),
        BLACK_PIXEL,
        "the corner a mitred join would fill"
    );
    assert_eq!(
        at(15.3, 12.3),
        YELLOW_PIXEL,
        "the round join at another point"
    );
}

/// A Room is drawn at its place in the stacking order, its floor first and its Walls over it:
/// an Element before it is drawn under both, an Element after it over both.
#[test]
fn a_rooms_floor_lies_under_its_walls() {
    let mut fixture = Fixture::new();
    fixture.place(RED, Vec2::new(8.5, 8.5));
    fixture.room(&ROOM, &[], 1.0);
    fixture.place(GREEN, Vec2::new(12.5, 8.5));

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "floor-stacked.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(
        at(10.0, 5.4),
        YELLOW_PIXEL,
        "the Wall's inner half over the floor"
    );
    assert_eq!(
        at(8.5, 8.5),
        WHITE_PIXEL,
        "the Prop before the Room under its floor"
    );
    assert_eq!(
        at(12.5, 8.5),
        GREEN_PIXEL,
        "the Prop after the Room over its floor"
    );
}

/// Along a stretch a Portal covers, a Room's Wall is left out: the floor still reaches the
/// outline on the inner half, the background shows on the outer half beside a Portal narrower
/// than the Wall is thick, and the Wall stands just past the stretch, ending squarely across the
/// line at either end of it.
#[test]
fn a_room_wall_gives_way_to_its_portal() {
    let mut fixture = Fixture::new();
    let room = fixture.room(
        &[
            Vec2::new(5.0, 5.0),
            Vec2::new(25.0, 5.0),
            Vec2::new(25.0, 15.0),
            Vec2::new(5.0, 15.0),
        ],
        &[],
        2.0,
    );
    fixture.portal(DOOR, Vec2::ZERO, Some(anchored(room, 0, 0.5, Side::Left)));

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "room-gap.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(at(15.0, 5.0), CYAN_PIXEL, "the Portal at its centre");
    assert_eq!(at(15.0, 5.6), WHITE_PIXEL, "the floor on the inner half");
    assert_eq!(
        at(15.0, 4.4),
        BLACK_PIXEL,
        "the background on the outer half"
    );
    assert_eq!(
        at(16.1, 4.4),
        YELLOW_PIXEL,
        "the Wall just past the stretch"
    );
    assert_eq!(at(13.9, 5.6), YELLOW_PIXEL, "the Wall just before it");
    assert_eq!(at(16.1, 4.1), YELLOW_PIXEL, "a square end, not a round one");
    assert_eq!(
        at(13.9, 5.9),
        YELLOW_PIXEL,
        "the square end before the stretch"
    );
    assert_eq!(
        at(15.9, 4.1),
        BLACK_PIXEL,
        "no cap reaching into the stretch"
    );
    assert_eq!(
        at(14.1, 5.9),
        WHITE_PIXEL,
        "no cap over the floor in the stretch"
    );
}

/// A Room appears in the Export only where it lies inside the Bounds: one with points outside is
/// cut at the edge.
#[test]
fn a_room_is_clipped_at_the_edge() {
    let mut fixture = Fixture::new();
    fixture.room(
        &[
            Vec2::new(-5.0, 10.0),
            Vec2::new(5.0, 10.0),
            Vec2::new(5.0, 20.0),
            Vec2::new(-5.0, 20.0),
        ],
        &[],
        1.0,
    );

    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, "clipped-room.png", TILE)
        .expect("the Export is written");
    let picture = Picture::decode(&exported.path);
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);

    assert_eq!(
        at(0.01, 15.0),
        WHITE_PIXEL,
        "the floor cut at the left edge"
    );
    assert_eq!(
        at(0.01, 10.0),
        YELLOW_PIXEL,
        "the Wall cut at the left edge"
    );
    let width = WALL_PIXELS_PER_CELL as usize;
    let floor = picture.count(WHITE_PIXEL);
    assert!(
        (4 * 9 * width * width..5 * 10 * width * width).contains(&floor),
        "{floor} white pixels: the floor inside, nothing more"
    );
    let right_of_the_wall = WALL_PIXELS_PER_CELL * 56 / 10;
    for x in right_of_the_wall..picture.width {
        for y in 0..picture.height {
            assert_eq!(picture.pixel(x, y), BLACK_PIXEL, "({x}, {y})");
        }
    }
}

/// A full-strength erase across a painted stroke shows what lies below the Terrain on its path,
/// here the background, and the stroke's image beyond its radius.
#[test]
fn an_erase_shows_what_lies_below() {
    let mut fixture = Fixture::new();
    fixture.paint(
        RED,
        &[Vec2::new(5.0, 15.0), Vec2::new(25.0, 15.0)],
        brush(4.0, 1.0, 1.0),
    );
    fixture.erase(
        &[Vec2::new(15.0, 10.0), Vec2::new(15.0, 20.0)],
        brush(2.0, 1.0, 1.0),
    );

    let picture = fixture.picture("erased.png");
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), PIXELS_PER_CELL);

    assert_eq!(at(15.0, 15.0), BLACK_PIXEL, "on the erase's path");
    assert_eq!(at(15.6, 16.0), BLACK_PIXEL, "within its radius");
    assert_eq!(at(13.5, 15.0), RED_PIXEL, "beyond its radius");
    assert_eq!(at(17.0, 14.0), RED_PIXEL, "beyond it on the other side");
}

/// A half-strength erase with a sharp joint over full paint leaves the same colour inside the
/// joint, where both its segments reach, as on its straight stretch, and as a half-strength
/// stroke painted alone over the background shows.
#[test]
fn an_erase_leaves_no_build_up() {
    let mut fixture = Fixture::new();
    fixture.paint(
        RED,
        &[Vec2::new(3.0, 15.0), Vec2::new(27.0, 15.0)],
        brush(8.0, 1.0, 1.0),
    );
    fixture.erase(
        &[
            Vec2::new(8.0, 12.0),
            Vec2::new(12.0, 18.0),
            Vec2::new(16.0, 12.0),
        ],
        brush(2.0, 1.0, 0.5),
    );
    let picture = fixture.picture("joint.png");
    let joint = picture.at_point(Vec2::new(12.0, 17.5), PIXELS_PER_CELL);
    let straight = picture.at_point(Vec2::new(10.0, 15.0), PIXELS_PER_CELL);

    let mut alone = Fixture::new();
    alone.paint(
        RED,
        &[Vec2::new(5.0, 15.0), Vec2::new(25.0, 15.0)],
        brush(2.0, 1.0, 0.5),
    );
    let half = alone
        .picture("half.png")
        .at_point(Vec2::new(10.0, 15.0), PIXELS_PER_CELL);

    assert!(partly_red(straight), "half left: {straight:?}");
    assert_eq!(
        joint, straight,
        "inside the joint as on the straight stretch"
    );
    assert_eq!(straight, half, "as a half-strength stroke alone");
}

/// A moved stroke exports at its new place, with the background at its old one, and an erase
/// turned to painting exports with the Terrain's image.
#[test]
fn an_edited_stroke_exports_as_edited() {
    let mut fixture = Fixture::new();
    let hard = brush(2.0, 1.0, 1.0);
    fixture.paint(RED, &[Vec2::new(5.0, 10.0), Vec2::new(10.0, 10.0)], hard);
    let terrain = fixture.last();
    fixture.paint(RED, &[Vec2::new(5.0, 20.0), Vec2::new(10.0, 20.0)], hard);
    fixture.erase(&[Vec2::new(22.0, 8.0), Vec2::new(26.0, 8.0)], hard);
    fixture.edit(
        terrain,
        ElementChange::Stroke {
            stroke: 1,
            change: StrokeChange::Position(Vec2::new(20.0, 22.0)),
        },
    );
    fixture.edit(
        terrain,
        ElementChange::Stroke {
            stroke: 2,
            change: StrokeChange::Erase(false),
        },
    );

    let picture = fixture.picture("edited.png");
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), PIXELS_PER_CELL);

    assert_eq!(
        at(20.0, 22.0),
        RED_PIXEL,
        "the moved stroke at its new place"
    );
    assert_eq!(at(7.5, 20.0), BLACK_PIXEL, "the background at its old one");
    assert_eq!(at(7.5, 10.0), RED_PIXEL, "the stroke left where it was");
    assert_eq!(at(24.0, 8.0), RED_PIXEL, "the erase turned to painting");
}

/// A red for a floor.
const RED_COLOUR: Colour = Colour::rgb(255, 0, 0);
/// A blue for Walls.
const BLUE_COLOUR: Colour = Colour::rgb(0, 0, 255);
/// A green for Walls.
const GREEN_COLOUR: Colour = Colour::rgb(0, 255, 0);
/// A magenta for the Walls round a hole.
const MAGENTA_COLOUR: Colour = Colour::rgb(255, 0, 255);

/// Exports at [`WALL_PIXELS_PER_CELL`] with tiles of [`TILE`] and decodes the image.
fn exported(fixture: &mut Fixture, name: &str) -> Picture {
    let exported = fixture
        .export(WALL_PIXELS_PER_CELL, name, TILE)
        .expect("the Export is written");
    Picture::decode(&exported.path)
}

/// Places a Room through the rectangle from `low` to `high` with Walls `thickness` thick in
/// `walls` and a floor in `floor`, cutting or not, and returns its identity.
fn coloured_room(
    fixture: &mut Fixture,
    low: Vec2,
    high: Vec2,
    thickness: f32,
    (walls, floor): (Colour, Colour),
    cuts: bool,
) -> ElementId {
    let layer = fixture.layer();
    fixture.run(Apply::PlaceElement(PlaceElement {
        layer,
        placement: Placement::Room {
            points: vec![
                low,
                Vec2::new(high.x, low.y),
                high,
                Vec2::new(low.x, high.y),
            ],
            thickness,
            wall_colour: walls,
            floor_colour: floor,
            cuts,
        },
    }));
    fixture.last()
}

/// Two overlapping Rooms share one floor walled round its outside: the later Room's floor in the
/// overlap, its floor where the earlier Room's edge runs inside it, and the wall colour on the
/// outside.
#[test]
fn overlapping_rooms_share_one_floor() {
    let mut fixture = Fixture::new();
    let look = (YELLOW, WHITE);
    coloured_room(
        &mut fixture,
        Vec2::new(5.0, 5.0),
        Vec2::new(15.0, 13.0),
        1.0,
        look,
        false,
    );
    let later = (YELLOW, RED_COLOUR);
    coloured_room(
        &mut fixture,
        Vec2::new(11.0, 8.0),
        Vec2::new(20.0, 16.0),
        1.0,
        later,
        false,
    );

    let picture = exported(&mut fixture, "overlapping.png");
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);
    assert_eq!(at(13.0, 10.0), RED_PIXEL, "the later floor in the overlap");
    assert_eq!(
        at(15.0, 11.0),
        RED_PIXEL,
        "no Wall where the earlier edge runs inside"
    );
    assert_eq!(
        at(11.0, 10.5),
        RED_PIXEL,
        "nor where the later edge runs inside"
    );
    assert_eq!(at(8.0, 9.0), WHITE_PIXEL, "the earlier floor");
    assert_eq!(at(5.0, 9.0), YELLOW_PIXEL, "the Wall on the outside");
    assert_eq!(at(20.0, 12.0), YELLOW_PIXEL, "round both");
}

/// Two Rooms meeting edge to edge keep one Wall between them, and a door set into it stands in
/// its gap with each Room's floor on its own side.
#[test]
fn a_shared_edge_keeps_its_wall_and_door() {
    let mut fixture = Fixture::new();
    let first = coloured_room(
        &mut fixture,
        Vec2::new(5.0, 5.0),
        Vec2::new(15.0, 15.0),
        1.0,
        (YELLOW, WHITE),
        false,
    );
    coloured_room(
        &mut fixture,
        Vec2::new(15.0, 7.0),
        Vec2::new(25.0, 13.0),
        1.0,
        (YELLOW, RED_COLOUR),
        false,
    );
    fixture.portal(DOOR, Vec2::ZERO, Some(anchored(first, 1, 0.5, Side::Left)));

    let picture = exported(&mut fixture, "shared.png");
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);
    assert_eq!(
        at(15.0, 8.0),
        YELLOW_PIXEL,
        "the Wall along the shared edge"
    );
    assert_eq!(at(15.0, 12.5), YELLOW_PIXEL, "on both sides of the door");
    assert_eq!(at(15.0, 10.0), CYAN_PIXEL, "the door at its centre");
    assert_eq!(
        at(14.6, 10.5),
        WHITE_PIXEL,
        "the first floor on its side of the gap"
    );
    assert_eq!(at(15.4, 10.5), RED_PIXEL, "the second floor on its side");
}

/// A Wall that runs along an edge two Rooms share, or along outline edges that coincide on the
/// outside, is drawn once, in the thickness and colour of the later Room.
#[test]
fn a_walled_once_stretch_has_the_last_rooms_look() {
    let mut fixture = Fixture::new();
    for (low, high, thickness, look) in [
        (
            Vec2::new(5.0, 5.0),
            Vec2::new(15.0, 15.0),
            1.0,
            (YELLOW, WHITE),
        ),
        (
            Vec2::new(15.0, 7.0),
            Vec2::new(25.0, 13.0),
            0.5,
            (GREEN_COLOUR, RED_COLOUR),
        ),
        (
            Vec2::new(5.0, 17.0),
            Vec2::new(15.0, 24.0),
            1.0,
            (YELLOW, WHITE),
        ),
        (
            Vec2::new(10.0, 20.0),
            Vec2::new(20.0, 24.0),
            0.5,
            (GREEN_COLOUR, RED_COLOUR),
        ),
    ] {
        coloured_room(&mut fixture, low, high, thickness, look, false);
    }

    let picture = exported(&mut fixture, "walled-once.png");
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);
    assert_eq!(
        at(15.0, 10.0),
        GREEN_PIXEL,
        "the shared edge in the later colour"
    );
    assert_eq!(
        at(15.2, 10.0),
        GREEN_PIXEL,
        "within half the later thickness"
    );
    assert_eq!(
        at(14.6, 10.0),
        WHITE_PIXEL,
        "not as thick as the earlier Room's Wall"
    );
    assert_eq!(at(15.4, 10.0), RED_PIXEL, "the later floor beside it");

    assert_eq!(
        at(12.0, 24.0),
        GREEN_PIXEL,
        "the coincident edge in the later colour"
    );
    assert_eq!(
        at(12.0, 23.6),
        RED_PIXEL,
        "the later floor, not the earlier Wall's thickness"
    );
    assert_eq!(
        at(12.0, 24.4),
        BLACK_PIXEL,
        "nothing beyond the later thickness"
    );
}

/// Where the Wall of one Room ends and that of another goes on, the ends are round: the earlier
/// Room's Wall reaches round the end of the shared edge past the later Room's thinner Wall, as
/// far as a half-thickness and no farther.
#[test]
fn walls_of_two_rooms_join_round() {
    let mut fixture = Fixture::new();
    coloured_room(
        &mut fixture,
        Vec2::new(5.0, 5.0),
        Vec2::new(15.0, 15.0),
        1.0,
        (YELLOW, WHITE),
        false,
    );
    coloured_room(
        &mut fixture,
        Vec2::new(15.0, 7.0),
        Vec2::new(25.0, 13.0),
        0.5,
        (GREEN_COLOUR, RED_COLOUR),
        false,
    );

    let picture = exported(&mut fixture, "joins.png");
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);
    assert_eq!(
        at(15.35, 12.7),
        YELLOW_PIXEL,
        "the earlier Wall's round end reaches past the shared edge"
    );
    assert_eq!(
        at(15.45, 12.55),
        RED_PIXEL,
        "but not as far as a square end would"
    );
}

/// A Room that cuts leaves a hole showing the background, walled in the cut's own colour, with
/// the Room's floor round it.
#[test]
fn a_cut_leaves_a_hole() {
    let mut fixture = Fixture::new();
    coloured_room(
        &mut fixture,
        Vec2::new(5.0, 5.0),
        Vec2::new(25.0, 20.0),
        1.0,
        (YELLOW, WHITE),
        false,
    );
    coloured_room(
        &mut fixture,
        Vec2::new(10.0, 9.0),
        Vec2::new(16.0, 15.0),
        1.0,
        (MAGENTA_COLOUR, RED_COLOUR),
        true,
    );

    let picture = exported(&mut fixture, "hole.png");
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);
    assert_eq!(at(13.0, 12.0), BLACK_PIXEL, "the background in the hole");
    assert_eq!(
        at(10.6, 12.0),
        BLACK_PIXEL,
        "beyond half the cut's thickness"
    );
    assert_eq!(
        at(10.0, 12.0),
        MAGENTA_PIXEL,
        "the cut's Wall on its outline"
    );
    assert_eq!(at(13.0, 9.0), MAGENTA_PIXEL, "all round");
    assert_eq!(at(8.0, 12.0), WHITE_PIXEL, "the Room's floor between");
    assert_eq!(at(5.0, 12.0), YELLOW_PIXEL, "the Room's own Wall");
    assert_eq!(picture.count(RED_PIXEL), 0, "a cut has no floor");
}

/// Each stretch of Wall of combined Rooms keeps the colour of the Room whose edge it runs along.
#[test]
fn each_wall_keeps_its_rooms_colour() {
    let mut fixture = Fixture::new();
    coloured_room(
        &mut fixture,
        Vec2::new(5.0, 5.0),
        Vec2::new(15.0, 13.0),
        1.0,
        (YELLOW, WHITE),
        false,
    );
    coloured_room(
        &mut fixture,
        Vec2::new(11.0, 8.0),
        Vec2::new(20.0, 16.0),
        1.0,
        (GREEN_COLOUR, WHITE),
        false,
    );

    let picture = exported(&mut fixture, "colours.png");
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);
    assert_eq!(at(5.0, 9.0), YELLOW_PIXEL, "the first Room's Wall");
    assert_eq!(at(8.0, 5.0), YELLOW_PIXEL);
    assert_eq!(at(20.0, 12.0), GREEN_PIXEL, "the second Room's Wall");
    assert_eq!(at(16.0, 16.0), GREEN_PIXEL);
}

/// The Walls of a combination lie over every floor of it, drawn at its last Room's place: the
/// earlier Room's Wall shows inside the later Room's floor near where their outlines cross, and a
/// Prop placed between the two Rooms shows over the earlier floor and under the Walls.
#[test]
fn walls_lie_over_the_combinations_floors() {
    let mut fixture = Fixture::new();
    coloured_room(
        &mut fixture,
        Vec2::new(5.0, 5.0),
        Vec2::new(15.0, 13.0),
        1.0,
        (YELLOW, WHITE),
        false,
    );
    fixture.place(GREEN, Vec2::new(5.5, 9.0));
    coloured_room(
        &mut fixture,
        Vec2::new(11.0, 8.0),
        Vec2::new(20.0, 16.0),
        0.25,
        (BLUE_COLOUR, RED_COLOUR),
        false,
    );

    let picture = exported(&mut fixture, "over.png");
    let at = |x: f32, y: f32| picture.at_point(Vec2::new(x, y), WALL_PIXELS_PER_CELL);
    assert_eq!(
        at(11.3, 12.7),
        YELLOW_PIXEL,
        "the earlier Wall's round end over the later floor"
    );
    assert_eq!(at(12.0, 12.0), RED_PIXEL, "the later floor beyond it");
    assert_eq!(at(5.3, 9.0), YELLOW_PIXEL, "the Wall over the Prop");
    assert_eq!(at(5.8, 9.0), GREEN_PIXEL, "the Prop over the earlier floor");
}
