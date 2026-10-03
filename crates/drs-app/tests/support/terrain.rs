//! The Terrain the seam tests of painting and of editing strokes share: a headless editor over
//! one Asset Folder of two texture images of known pixel size and a table for Props, Commands
//! that paint, place, and edit on the one Layer, and readers of its Terrain, the Terrain's
//! coverage, and the Project's Asset References, saving and opening the Project on the way.

use super::{
    add_folder, apply, edit, editor, first_layer, history, order, png, redo, try_apply, undo,
};
use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::message::Messages;
use bevy::math::{Rect, UVec2, Vec2};
use drs_model::{
    Apply, AssetAddress, AssetReferenceRow, AssetReferences, BrushSettings,
    COVERAGE_PIXELS_PER_CELL, COVERAGE_TILE_PIXELS, CanonicalName, Colour, Element, ElementChange,
    ElementId, FolderKey, Gesture, OpenProject, Paint, PlaceElement, Placement, ProjectOpened,
    ProjectRefused, ProjectSaved, SaveProject, Stroke, Terrain, TerrainCoverage, TileKey, Viewport,
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tempfile::TempDir;

/// A flagstone texture two cells a side at the Grid's 256 pixels per cell.
pub const FLAGSTONES: &str = "textures/flagstones.png";
/// A grass texture one cell a side.
pub const GRASS: &str = "textures/grass.png";
/// A table image one cell a side, for Props.
pub const TABLE: &str = "table.png";
/// The soft Brush most strokes are laid with: two cells across, half hard, at full strength.
pub const SOFT: BrushSettings = BrushSettings {
    size: 2.0,
    hardness: 0.5,
    strength: 1.0,
};
/// Half a pixel of the coverage, in cells: added to a whole cell, a point lies on a pixel's
/// centre.
pub const HALF_PIXEL: f32 = 1.0 / 64.0;

/// The headless editor with the fixture folder added.
pub struct Fixture {
    /// Keeps the temporary directory alive for the test.
    pub root: TempDir,
    /// The folder's key once added.
    pub key: FolderKey,
    /// The headless editor.
    pub app: App,
    /// How deep the history is once the folder is added.
    pub start: usize,
}

/// A stroke through `points` with `brush`.
pub fn stroke(points: &[Vec2], brush: BrushSettings) -> Stroke {
    Stroke {
        points: points.to_vec(),
        brush,
        erase: false,
    }
}

/// A Brush of `size`, `hardness`, and `strength`.
pub fn brush(size: f32, hardness: f32, strength: f32) -> BrushSettings {
    BrushSettings {
        size,
        hardness,
        strength,
    }
}

/// The pixels of every tile of a coverage, by place.
pub type Tiles = BTreeMap<TileKey, Vec<u8>>;

impl Fixture {
    /// Creates the fixture folder, starts the editor, and adds the folder.
    pub fn new() -> Self {
        let root = TempDir::new().expect("temporary root");
        let folder = root.path().join("fixtures");
        for (place, size, colour) in [
            (FLAGSTONES, 512, [90, 90, 100, 255]),
            (GRASS, 256, [40, 140, 50, 255]),
            (TABLE, 256, [120, 80, 40, 255]),
        ] {
            png(&folder, place, UVec2::splat(size), colour);
        }
        let mut app = editor(root.path());
        let key = add_folder(&mut app, &folder, "Fixtures").key;
        let start = history(&app).undo_depth();
        Self {
            root,
            key,
            app,
            start,
        }
    }

    /// The Asset at `place` in the fixture folder.
    pub fn asset(&self, place: &str) -> AssetAddress {
        AssetAddress {
            folder: self.key.clone(),
            place: place.to_owned(),
        }
    }

    /// The one Layer of the Project.
    pub fn layer(&mut self) -> Entity {
        first_layer(&mut self.app)
    }

    /// Sends a Command and runs one update, returning the reasons of any failure.
    pub fn try_apply(&mut self, command: Apply) -> Vec<String> {
        try_apply(&mut self.app, command)
    }

    /// Sends a Command and runs one update, failing the test if the Command was refused.
    pub fn apply(&mut self, command: Apply) {
        apply(&mut self.app, command);
    }

    /// The Paint of `stroke` on the Layer with the Asset at `place`, or with none.
    pub fn paint_command(&mut self, stroke: Stroke, place: Option<&str>) -> Apply {
        Apply::Paint(Paint {
            layer: self.layer(),
            stroke,
            asset: place.map(|place| self.asset(place)),
        })
    }

    /// Paints `stroke` with the Asset at `place`, or with none, failing the test on a refusal.
    pub fn paint(&mut self, stroke: Stroke, place: Option<&str>) {
        let command = self.paint_command(stroke, place);
        self.apply(command);
    }

    /// Paints `stroke` with the flagstones.
    pub fn flagstones(&mut self, stroke: Stroke) {
        self.paint(stroke, Some(FLAGSTONES));
    }

    /// Paints `stroke` and returns the refusals.
    pub fn try_paint(&mut self, stroke: Stroke, place: Option<&str>) -> Vec<String> {
        let command = self.paint_command(stroke, place);
        self.try_apply(command)
    }

    /// Places a Prop of the table centred on `position`, returning its identity.
    pub fn prop(&mut self, position: Vec2) -> ElementId {
        let layer = self.layer();
        let asset = self.asset(TABLE);
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Prop { position, asset },
        }));
        *self.order().last().expect("the Prop is on top")
    }

    /// Places a grey Wall through `points`, returning its identity.
    pub fn wall(&mut self, points: &[Vec2]) -> ElementId {
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

    /// Places a grey Room with a light floor through `points`, returning its identity.
    pub fn room(&mut self, points: &[Vec2]) -> ElementId {
        let layer = self.layer();
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Room {
                points: points.to_vec(),
                thickness: 0.25,
                wall_colour: Colour::rgb(60, 60, 60),
                floor_colour: Colour::rgb(200, 200, 200),
            },
        }));
        *self.order().last().expect("the Room is on top")
    }

    /// Places a freestanding Portal of the table centred on `position`, returning its identity.
    pub fn portal(&mut self, position: Vec2) -> ElementId {
        let layer = self.layer();
        let asset = self.asset(TABLE);
        self.apply(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Portal {
                position,
                asset,
                anchor: None,
            },
        }));
        *self.order().last().expect("the Portal is on top")
    }

    /// Sends an Edit Element on its own and returns the refusals.
    pub fn try_edit(&mut self, element: ElementId, change: ElementChange) -> Vec<String> {
        try_apply(&mut self.app, edit(element, change, Gesture::Single))
    }

    /// Sends Undo and runs one update.
    pub fn undo(&mut self) {
        undo(&mut self.app);
    }

    /// Sends Redo and runs one update.
    pub fn redo(&mut self) {
        redo(&mut self.app);
    }

    /// How many steps the history holds since the folder was added.
    pub fn steps(&self) -> usize {
        history(&self.app).undo_depth() - self.start
    }

    /// The identities of the Elements on the Layer, bottom first.
    pub fn order(&mut self) -> Vec<ElementId> {
        let layer = self.layer();
        order(&mut self.app, layer)
    }

    /// Every Terrain on the Layer, bottom first, with its identity and Element.
    pub fn terrains(&mut self) -> Vec<(ElementId, Element, Terrain)> {
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
    pub fn terrain(&mut self) -> (ElementId, Element, Terrain) {
        let mut terrains = self.terrains();
        assert_eq!(terrains.len(), 1, "one Terrain on the Layer");
        terrains.remove(0)
    }

    /// The one Terrain's coverage.
    pub fn coverage(&mut self) -> TerrainCoverage {
        let (id, ..) = self.terrain();
        let world = self.app.world_mut();
        world
            .query::<(&ElementId, &TerrainCoverage)>()
            .iter(world)
            .find(|(element, _)| **element == id)
            .map(|(_, coverage)| coverage.clone())
            .expect("the Terrain has its coverage")
    }

    /// Writes the Viewport as the Editor does, showing an area of `area` screen pixels around
    /// `centre` at `zoom`, and runs one update.
    pub fn look(&mut self, centre: Vec2, zoom: f32, area: Vec2) {
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

    /// The pixels of every tile of the one Terrain's coverage.
    pub fn tiles(&mut self) -> Tiles {
        self.coverage()
            .tiles
            .into_iter()
            .map(|(key, tile)| (key, tile.pixels().expect("on the CPU").to_vec()))
            .collect()
    }

    /// The one Terrain's coverage at the pixel holding `cells`.
    pub fn coverage_at(&mut self, cells: Vec2) -> u8 {
        at(&self.tiles(), cells)
    }

    /// The Project's Asset Reference table.
    pub fn references(&mut self) -> AssetReferences {
        let world = self.app.world_mut();
        world
            .query::<&AssetReferences>()
            .single(world)
            .expect("one Project")
            .clone()
    }

    /// The row of the Asset Reference table that records the Asset at `place`.
    pub fn row(&mut self, place: &str) -> AssetReferenceRow {
        self.references()
            .row_of(&CanonicalName("Fixtures".to_owned()), place)
            .expect("the Asset is recorded")
    }

    /// Saves the Project under the root and opens the file again.
    pub fn save_and_open(&mut self) {
        let path = self.save();
        self.open(path);
    }

    /// Saves the Project under the root, returning the file.
    pub fn save(&mut self) -> PathBuf {
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
    pub fn open(&mut self, path: PathBuf) {
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

/// The coverage of `tiles` at the pixel holding `cells`; an absent tile is empty.
pub fn at(tiles: &Tiles, cells: Vec2) -> u8 {
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
