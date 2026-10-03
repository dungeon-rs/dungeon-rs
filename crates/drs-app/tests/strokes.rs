//! Erasing and editing the strokes of a Terrain through the headless editor: the real plugins of
//! the model, the history, `LibraryAccess`, `LibraryManager`, `ProjectManager`, and
//! `AuthoringManager`, with no window and no render Engine, over one Asset Folder of two texture
//! images and a table for Props, driven by Apply, Undo, and Redo messages and asserted on the
//! Terrain, its Element, its derived coverage, the Layer's children, the answers, and the
//! history.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::message::Messages;
use bevy::math::Vec2;
use drs_authoring_manager::AuthoringManagerPlugin;
use drs_history::{History, HistoryPlugin};
use drs_library_access::LibraryAccessPlugin;
use drs_library_manager::LibraryManagerPlugin;
use drs_model::{
    AddFolder, Apply, AssetAddress, AssetReferences, BrushSettings, COVERAGE_PIXELS_PER_CELL,
    COVERAGE_TILE_PIXELS, CanonicalName, Colour, CommandFailed, EditElement, EditorDirectories,
    Element, ElementChange, ElementId, FolderAdded, FolderKey, Gesture, Layer, ModelPlugin,
    OpenProject, Paint, PlaceElement, Placement, ProjectOpened, ProjectRefused, ProjectSaved, Redo,
    SaveProject, Stroke, Terrain, TerrainCoverage, TileKey, Undo,
};
use drs_project_manager::ProjectManagerPlugin;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A flagstone texture two cells a side at the Grid's 256 pixels per cell.
const FLAGSTONES: &str = "textures/flagstones.png";
/// A grass texture one cell a side.
const GRASS: &str = "textures/grass.png";
/// A table image one cell a side, for Props.
const TABLE: &str = "table.png";
/// The soft Brush most strokes are laid with: two cells across, half hard, at full strength.
const SOFT: BrushSettings = BrushSettings {
    size: 2.0,
    hardness: 0.5,
    strength: 1.0,
};
/// Half a pixel of the coverage, in cells: added to a whole cell, a point lies on a pixel's
/// centre.
const HALF_PIXEL: f32 = 1.0 / 64.0;

/// The headless editor with the fixture folder added.
struct Fixture {
    /// Keeps the temporary directory alive for the test.
    root: TempDir,
    /// The folder's key once added.
    key: FolderKey,
    /// The headless editor.
    app: App,
    /// How deep the history is once the folder is added.
    start: usize,
}

/// A stroke that paints through `points` with `brush`.
fn stroke(points: &[Vec2], brush: BrushSettings) -> Stroke {
    Stroke {
        points: points.to_vec(),
        brush,
        erase: false,
    }
}

/// A stroke that erases through `points` with `brush`.
fn erasing(points: &[Vec2], brush: BrushSettings) -> Stroke {
    Stroke {
        points: points.to_vec(),
        brush,
        erase: true,
    }
}

/// A Brush of `size`, `hardness`, and `strength`.
fn brush(size: f32, hardness: f32, strength: f32) -> BrushSettings {
    BrushSettings {
        size,
        hardness,
        strength,
    }
}

/// The pixels of every tile of a coverage, by place.
type Tiles = BTreeMap<TileKey, Vec<u8>>;

impl Fixture {
    /// Creates the fixture folder, starts the editor, and adds the folder.
    fn new() -> Self {
        let root = TempDir::new().expect("temporary root");
        let folder = root.path().join("fixtures");
        std::fs::create_dir_all(folder.join("textures")).expect("fixture folder");
        for (place, size, colour) in [
            (FLAGSTONES, 512, [90, 90, 100, 255]),
            (GRASS, 256, [40, 140, 50, 255]),
            (TABLE, 256, [120, 80, 40, 255]),
        ] {
            image::RgbaImage::from_pixel(size, size, image::Rgba(colour))
                .save(folder.join(place))
                .expect("fixture image");
        }
        let mut app = editor(root.path());
        app.world_mut().write_message(AddFolder {
            path: folder,
            name: CanonicalName("Fixtures".to_owned()),
        });
        app.update();
        let key = app
            .world_mut()
            .resource_mut::<Messages<FolderAdded>>()
            .drain()
            .next()
            .expect("the fixture folder is added")
            .key;
        let start = app.world().resource::<History>().undo_depth();
        Self {
            root,
            key,
            app,
            start,
        }
    }

    /// The Asset at `place` in the fixture folder.
    fn asset(&self, place: &str) -> AssetAddress {
        AssetAddress {
            folder: self.key.clone(),
            place: place.to_owned(),
        }
    }

    /// The one Layer of the Project.
    fn layer(&mut self) -> Entity {
        let world = self.app.world_mut();
        world
            .query::<(Entity, &Layer)>()
            .single(world)
            .expect("exactly one Layer")
            .0
    }

    /// Sends a Command and runs one update, returning the reasons of any failure.
    fn try_apply(&mut self, command: Apply) -> Vec<String> {
        self.app.world_mut().write_message(command);
        self.app.update();
        self.app
            .world_mut()
            .resource_mut::<Messages<CommandFailed>>()
            .drain()
            .map(|failed| failed.reason)
            .collect()
    }

    /// Sends a Command and runs one update, failing the test if the Command was refused.
    fn apply(&mut self, command: Apply) {
        let failed = self.try_apply(command);
        assert!(failed.is_empty(), "the Command failed: {failed:?}");
    }

    /// The Paint of `stroke` on the Layer with the Asset at `place`, or with none.
    fn paint_command(&mut self, stroke: Stroke, place: Option<&str>) -> Apply {
        Apply::Paint(Paint {
            layer: self.layer(),
            stroke,
            asset: place.map(|place| self.asset(place)),
        })
    }

    /// Paints `stroke` with the Asset at `place`, or with none, failing the test on a refusal.
    fn paint(&mut self, stroke: Stroke, place: Option<&str>) {
        let command = self.paint_command(stroke, place);
        self.apply(command);
    }

    /// Paints `stroke` with the flagstones.
    fn flagstones(&mut self, stroke: Stroke) {
        self.paint(stroke, Some(FLAGSTONES));
    }

    /// Paints `stroke` and returns the refusals.
    fn try_paint(&mut self, stroke: Stroke, place: Option<&str>) -> Vec<String> {
        let command = self.paint_command(stroke, place);
        self.try_apply(command)
    }

    /// Places a Prop of the table centred on `position`, returning its identity.
    fn prop(&mut self, position: Vec2) -> ElementId {
        let layer = self.layer();
        let asset = self.asset(TABLE);
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Prop { position, asset },
        }));
        *self.order().last().expect("the Prop is on top")
    }

    /// Places a grey Wall through `points`, returning its identity.
    fn wall(&mut self, points: &[Vec2]) -> ElementId {
        let layer = self.layer();
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Wall {
                points: points.to_vec(),
                thickness: 0.125,
                colour: Colour::rgb(60, 60, 60),
            },
        }));
        *self.order().last().expect("the Wall is on top")
    }

    /// Sends an Edit Element on its own and returns the refusals.
    fn try_edit(&mut self, element: ElementId, change: ElementChange) -> Vec<String> {
        self.try_apply(Apply::EditElement(EditElement {
            element,
            change,
            gesture: Gesture::Single,
        }))
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

    /// How many steps the history holds since the folder was added.
    fn steps(&self) -> usize {
        self.app.world().resource::<History>().undo_depth() - self.start
    }

    /// The identities of the Elements on the Layer, bottom first.
    fn order(&mut self) -> Vec<ElementId> {
        let layer = self.layer();
        let world = self.app.world_mut();
        world
            .get::<Children>(layer)
            .map(|children| {
                children
                    .iter()
                    .map(|child| *world.get::<ElementId>(*child).expect("an Element"))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Every Terrain on the Layer, bottom first, with its identity and Element.
    fn terrains(&mut self) -> Vec<(ElementId, Element, Terrain)> {
        let layer = self.layer();
        let world = self.app.world_mut();
        let children: Vec<Entity> = world
            .get::<Children>(layer)
            .map(|children| children.iter().copied().collect())
            .unwrap_or_default();
        children
            .into_iter()
            .filter_map(|child| {
                Some((
                    *world.get::<ElementId>(child)?,
                    world.get::<Element>(child)?.clone(),
                    world.get::<Terrain>(child)?.clone(),
                ))
            })
            .collect()
    }

    /// The one Terrain on the Layer.
    fn terrain(&mut self) -> (ElementId, Element, Terrain) {
        let mut terrains = self.terrains();
        assert_eq!(terrains.len(), 1, "one Terrain on the Layer");
        terrains.remove(0)
    }

    /// The pixels of every tile of the one Terrain's coverage.
    fn tiles(&mut self) -> Tiles {
        let (id, ..) = self.terrain();
        let world = self.app.world_mut();
        let coverage = world
            .query::<(&ElementId, &TerrainCoverage)>()
            .iter(world)
            .find(|(element, _)| **element == id)
            .map(|(_, coverage)| coverage.clone())
            .expect("the Terrain has its coverage");
        coverage
            .tiles
            .into_iter()
            .map(|(key, tile)| (key, tile.pixels.to_vec()))
            .collect()
    }

    /// The one Terrain's coverage at the pixel holding `cells`.
    fn coverage_at(&mut self, cells: Vec2) -> u8 {
        at(&self.tiles(), cells)
    }

    /// The Project's Asset Reference table.
    fn references(&mut self) -> AssetReferences {
        let world = self.app.world_mut();
        world
            .query::<&AssetReferences>()
            .single(world)
            .expect("one Project")
            .clone()
    }

    /// Saves the Project under the root and opens the file again.
    fn save_and_open(&mut self) {
        let path = self.save();
        self.open(path);
    }

    /// Saves the Project under the root, returning the file.
    fn save(&mut self) -> PathBuf {
        let path: PathBuf = self.root.path().join("painted.dungeon");
        self.app.world_mut().write_message(SaveProject {
            path: Some(path.clone()),
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
            .expect("the Project is saved");
        path
    }

    /// Opens the Project file at `path`.
    fn open(&mut self, path: PathBuf) {
        self.app.world_mut().write_message(OpenProject { path });
        self.app.update();
        let world = self.app.world_mut();
        let refused: Vec<ProjectRefused> = world
            .resource_mut::<Messages<ProjectRefused>>()
            .drain()
            .collect();
        assert!(refused.is_empty(), "the open was refused: {refused:?}");
        world
            .resource_mut::<Messages<ProjectOpened>>()
            .drain()
            .next()
            .expect("the Project is opened");
    }
}

/// A headless editor whose configuration and cache directories live under `root`, started once.
fn editor(root: &Path) -> App {
    let mut app = App::new();
    app.insert_resource(EditorDirectories::under(root));
    app.add_plugins((
        ModelPlugin,
        HistoryPlugin,
        LibraryAccessPlugin,
        LibraryManagerPlugin,
        ProjectManagerPlugin,
        AuthoringManagerPlugin,
    ));
    app.update();
    app
}

/// The coverage of `tiles` at the pixel holding `cells`; an absent tile is empty.
fn at(tiles: &Tiles, cells: Vec2) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        reason = "the points asked for are a few cells from the origin"
    )]
    let pixel = |cells: f32| (cells * COVERAGE_PIXELS_PER_CELL as f32).floor() as i32;
    let side = i32::try_from(COVERAGE_TILE_PIXELS).expect("a tile side");
    let (x, y) = (pixel(cells.x), pixel(cells.y));
    let key = TileKey {
        x: x.div_euclid(side),
        y: y.div_euclid(side),
    };
    let column = x.rem_euclid(side);
    let row = side - 1 - y.rem_euclid(side);
    tiles.get(&key).map_or(0, |pixels| {
        pixels[usize::try_from(row * side + column).expect("an index")]
    })
}

/// The pixel-wise smallest of several coverages that hold the same tiles.
fn smallest(coverages: &[Tiles]) -> Tiles {
    let mut smallest = coverages.first().cloned().unwrap_or_default();
    for tiles in &coverages[1..] {
        for (key, into) in &mut smallest {
            let pixels = tiles.get(key).expect("the same tiles");
            for (into, pixel) in into.iter_mut().zip(pixels) {
                *into = (*into).min(*pixel);
            }
        }
    }
    smallest
}

/// The value a coverage of `coverage` is held as: times 255, rounded half up.
fn byte(coverage: f32) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a coverage is from 0 to 1"
    )]
    let value = (coverage * 255.0 + 0.5).floor() as u8;
    value
}

/// The coverage a stroke laid with `brush` has at `distance` cells from its path: its strength
/// within the hardness of its radius, nothing from the radius on, and the smooth falloff between.
fn shaped(brush: BrushSettings, distance: f32) -> f32 {
    let radius = brush.size / 2.0;
    let inner = brush.hardness * radius;
    if distance <= inner {
        brush.strength
    } else if distance >= radius {
        0.0
    } else {
        let t = (distance - inner) / (radius - inner);
        brush.strength * (1.0 - 3.0 * t * t + 2.0 * t * t * t)
    }
}

impl Fixture {
    /// Lays the erase `stroke` on the Layer naming no image, failing the test on a refusal.
    fn erase(&mut self, stroke: Stroke) {
        assert!(stroke.erase, "an erase");
        self.paint(stroke, None);
    }

    /// Sends an Edit Element on its own, failing the test on a refusal.
    fn edit(&mut self, element: ElementId, change: ElementChange) {
        let refused = self.try_edit(element, change);
        assert!(refused.is_empty(), "the edit was refused: {refused:?}");
    }

    /// Sends `changes` as one gesture, its first the Begin and its last the End, one update
    /// each, failing the test on a refusal.
    fn gesture(&mut self, element: ElementId, changes: Vec<ElementChange>) {
        let last = changes.len() - 1;
        for (index, change) in changes.into_iter().enumerate() {
            let gesture = match index {
                0 => Gesture::Begin,
                index if index == last => Gesture::End,
                _ => Gesture::Continue,
            };
            self.apply(Apply::EditElement(EditElement {
                element,
                change,
                gesture,
            }));
        }
    }

    /// The one Terrain's strokes.
    fn strokes(&mut self) -> Vec<Stroke> {
        self.terrain().2.strokes
    }
}

/// A Brush that covers fully and sharply: `size` cells across, hard, at full strength.
fn hard(size: f32) -> BrushSettings {
    brush(size, 1.0, 1.0)
}

/// Ground covered fully from (-2, -2) to (22, 12), laid as the first stroke of the Layer.
fn ground(fixture: &mut Fixture) {
    fixture.flagstones(stroke(
        &[Vec2::new(0.0, 5.0), Vec2::new(20.0, 5.0)],
        hard(14.0),
    ));
}

/// An erase lowers a Terrain's coverage at each point to one minus its own coverage there where
/// that is lower and leaves it where it is already as low: a full-strength erase leaves nothing
/// within its hardness, and one of strength `s` leaves at most `1 - s`.
#[test]
fn erasing_caps_what_remains() {
    let mut fixture = Fixture::new();
    // A soft stroke along a row of pixel centres, so a pixel `j` rows above lies `j`
    // thirty-seconds of a cell from its path.
    let y = 4.0 + HALF_PIXEL;
    let paint = brush(4.0, 0.5, 1.0);
    fixture.flagstones(stroke(&[Vec2::new(0.0, y), Vec2::new(20.0, y)], paint));
    let full = brush(2.0, 0.5, 1.0);
    let x = 4.0 + HALF_PIXEL;
    fixture.erase(erasing(&[Vec2::new(x, 0.0), Vec2::new(x, 8.0)], full));

    assert_eq!(
        fixture.coverage_at(Vec2::new(x, y)),
        0,
        "on the erase's path"
    );
    assert_eq!(
        fixture.coverage_at(Vec2::new(x + 0.5, y)),
        0,
        "at the erase's hardness"
    );
    let tiles = fixture.tiles();
    for across in 0..48_u8 {
        for up in 0..72_u8 {
            let (dx, dy) = (f32::from(across) / 32.0, f32::from(up) / 32.0);
            let point = Vec2::new(x + dx, y + dy);
            let expected = byte(shaped(paint, dy)).min(byte(1.0 - shaped(full, dx)));
            assert_eq!(at(&tiles, point), expected, "{dx} across and {dy} up");
        }
    }

    let half = brush(2.0, 1.0, 0.5);
    let x = 10.0 + HALF_PIXEL;
    fixture.erase(erasing(&[Vec2::new(x, 0.0), Vec2::new(x, 8.0)], half));
    assert_eq!(
        fixture.coverage_at(Vec2::new(x, y)),
        byte(0.5),
        "half left of full ground"
    );

    let mut halves = Fixture::new();
    halves.flagstones(stroke(&[Vec2::new(0.0, y), Vec2::new(20.0, y)], half));
    let before = halves.coverage_at(Vec2::new(x, y));
    halves.erase(erasing(&[Vec2::new(x, 0.0), Vec2::new(x, 8.0)], half));
    assert_eq!(before, byte(0.5));
    assert_eq!(
        halves.coverage_at(Vec2::new(x, y)),
        before,
        "half over half is left as it was"
    );
}

/// An erase lowers the coverage at a point by the same however many of its segments pass near
/// the point, at a joint or a self-crossing alike, and passing over ground again with an erase
/// never lowers it below what the strongest erase there leaves.
#[test]
fn no_build_up_along_an_erase() {
    let mut fixture = Fixture::new();
    ground(&mut fixture);
    let half = brush(3.0, 0.2, 0.5);
    let alone = |fixture: &mut Fixture, points: &[Vec2]| {
        fixture.erase(erasing(points, half));
        let tiles = fixture.tiles();
        fixture.undo();
        tiles
    };

    let sharp = [
        Vec2::new(2.0, 2.0),
        Vec2::new(4.0, 8.0),
        Vec2::new(6.0, 2.0),
    ];
    let bent = alone(&mut fixture, &sharp);
    let first = alone(&mut fixture, &sharp[..2]);
    let second = alone(&mut fixture, &sharp[1..]);
    assert_eq!(bent, smallest(&[first.clone(), second.clone()]));
    let inside = Vec2::new(4.0 + HALF_PIXEL, 6.5 + HALF_PIXEL);
    let (one, other) = (at(&first, inside), at(&second, inside));
    assert!(
        one < 255 && other < 255,
        "both segments reach inside the joint, so a product would show: {one} and {other}"
    );
    assert_eq!(at(&bent, inside), one.min(other));
    assert_eq!(at(&bent, sharp[1]), byte(0.5), "the joint left at half");

    let crossing = [
        Vec2::new(10.0, 2.0),
        Vec2::new(16.0, 8.0),
        Vec2::new(16.0, 2.0),
        Vec2::new(10.0, 8.0),
    ];
    let crossed = alone(&mut fixture, &crossing);
    let pieces: Vec<Tiles> = crossing
        .windows(2)
        .map(|pair| alone(&mut fixture, pair))
        .collect();
    assert_eq!(crossed, smallest(&pieces));
    assert_eq!(
        at(&crossed, Vec2::new(13.0, 5.0)),
        byte(0.5),
        "the self-crossing left at half"
    );

    let across = [Vec2::new(1.0, 10.0), Vec2::new(19.0, 10.0)];
    fixture.erase(erasing(&across, brush(2.0, 1.0, 0.5)));
    fixture.erase(erasing(&across, brush(2.0, 1.0, 0.5)));
    fixture.erase(erasing(
        &[Vec2::new(8.0, 9.0), Vec2::new(8.0, 11.0)],
        brush(2.0, 1.0, 0.25),
    ));
    assert_eq!(
        fixture.coverage_at(Vec2::new(8.0 + HALF_PIXEL, 10.0 + HALF_PIXEL)),
        byte(0.5),
        "erased three times, left at what the strongest leaves"
    );
}

/// An erase lowers only the coverage the strokes laid before it give: a stroke laid after it
/// paints over the erased ground as over bare ground, and an erase where no earlier stroke covers
/// anything changes nothing.
#[test]
fn an_erase_takes_from_what_lies_before_it() {
    let mut fixture = Fixture::new();
    let y = 4.0 + HALF_PIXEL;
    let x = 6.0 + HALF_PIXEL;
    fixture.flagstones(stroke(&[Vec2::new(1.0, y), Vec2::new(12.0, y)], hard(2.0)));
    fixture.erase(erasing(&[Vec2::new(x, 1.0), Vec2::new(x, 8.0)], hard(2.0)));
    assert_eq!(fixture.coverage_at(Vec2::new(x, y)), 0, "erased");
    assert_eq!(fixture.coverage_at(Vec2::new(x + 2.0, y)), 255, "beyond it");

    fixture.paint(
        stroke(
            &[Vec2::new(x, 2.0), Vec2::new(x, 7.0)],
            brush(1.0, 1.0, 1.0),
        ),
        None,
    );
    assert_eq!(
        fixture.coverage_at(Vec2::new(x, y)),
        255,
        "laid after the erase, on its path"
    );
    assert_eq!(
        fixture.coverage_at(Vec2::new(x, 6.0 + HALF_PIXEL)),
        255,
        "laid after the erase, where only it paints"
    );

    let tiles = fixture.tiles();
    let (_, element, _) = fixture.terrain();
    fixture.erase(erasing(
        &[Vec2::new(40.0, 30.0), Vec2::new(60.0, 50.0)],
        hard(6.0),
    ));
    fixture.erase(erasing(&[Vec2::new(12.0, 9.0)], hard(3.0)));
    assert_eq!(fixture.tiles(), tiles, "no tile added and none changed");
    assert_ne!(
        fixture.terrain().1,
        element,
        "the box still holds every stroke, erases included"
    );
}

/// A Paint that erases on a Layer with no Terrain is answered with the reason, makes no Terrain,
/// and records no history step, whether it names an image or none.
#[test]
fn nothing_to_erase() {
    let mut fixture = Fixture::new();
    let erase = erasing(&[Vec2::new(2.0, 2.0)], SOFT);
    for place in [Some(FLAGSTONES), None] {
        let refused = fixture.try_paint(erase.clone(), place);
        assert_eq!(refused.len(), 1, "{refused:?}");
        assert!(refused[0].contains("nothing to erase"), "{}", refused[0]);
    }
    fixture.prop(Vec2::new(3.0, 3.0));
    let refused = fixture.try_paint(erase, None);
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert!(fixture.terrains().is_empty(), "no Terrain is made");
    assert_eq!(
        fixture.references().assets.len(),
        1,
        "only the table is recorded"
    );
    assert_eq!(fixture.steps(), 1, "the Prop alone");
}

/// A Paint that erases adds its stroke to the topmost Terrain on its Layer whatever image it
/// names, or none.
#[test]
fn an_erase_needs_no_image() {
    let mut fixture = Fixture::new();
    fixture.flagstones(stroke(&[Vec2::new(1.0, 1.0), Vec2::new(8.0, 1.0)], SOFT));
    let other = erasing(&[Vec2::new(3.0, 1.0)], SOFT);
    let none = erasing(&[Vec2::new(5.0, 1.0)], SOFT);
    fixture.paint(other.clone(), Some(GRASS));
    fixture.paint(none.clone(), None);

    let (_, _, terrain) = fixture.terrain();
    assert_eq!(terrain.strokes[1..], [other, none]);
    let references = fixture.references();
    assert_eq!(references.assets.len(), 1, "the grass is never recorded");
    assert_eq!(references.assets[0].name, "flagstones");
    assert_eq!(fixture.steps(), 3);
}

/// A Paint that erases appends its stroke, whether it erases with it, at the end of the
/// Terrain's strokes as one history step; undo takes exactly that stroke away and redo puts it
/// back at the same place in the order.
#[test]
fn an_erase_is_a_stroke() {
    let mut fixture = Fixture::new();
    let first = stroke(&[Vec2::new(1.0, 1.0), Vec2::new(6.0, 1.0)], SOFT);
    let erase = erasing(
        &[Vec2::new(3.0, 0.0), Vec2::new(3.0, 3.0)],
        brush(1.0, 0.5, 0.7),
    );
    fixture.flagstones(first.clone());
    fixture.erase(erase.clone());
    assert_eq!(fixture.strokes(), vec![first.clone(), erase.clone()]);
    assert_eq!(fixture.steps(), 2);
    let tiles = fixture.tiles();

    fixture.undo();
    assert_eq!(fixture.strokes(), vec![first.clone()]);
    fixture.redo();
    assert_eq!(fixture.strokes(), vec![first, erase]);
    assert_eq!(fixture.tiles(), tiles);
    assert_eq!(fixture.steps(), 2);
}
