#![doc = include_str!("../README.md")]

mod assets;
mod directories;
mod element;
mod file;
mod matches;
mod messages;
mod panics;
mod pointer;
mod portal;
mod project;
mod resolution;
mod room;
mod serialisation;
mod snapshot;
mod terrain;
mod thumbnails;
mod viewport;
mod wall;

pub use assets::{
    AssetFolder, AssetFolderReference, AssetKind, AssetReference, AssetReferenceRow,
    AssetReferences, AssetReferencesFull, CanonicalName, Fingerprint, FolderKey, IndexedAsset,
    ScanSkips,
};
pub use directories::{EditorDirectories, NoPlatformDirectories, ResolvedDirectories};
pub use element::{
    DrawnAs, Element, ElementId, ElementKindDescriptor, ElementKindName, ElementKindRegistry, PROP,
    Prop, ShownAsset, ShownAssetItem,
};
pub use file::{PROJECT_EXTENSION, SavedMark, project_name_of, with_extension_if_missing};
pub use matches::{AssetMatch, SearchMatches};
pub use messages::{
    AddFolder, Apply, AssetAddress, AssetFolderChanged, Browse, CommandFailed, EditElement,
    ElementChange, ExportLevel, ExportRefused, FolderAdded, FolderRefusal, FolderRefused,
    FolderUnavailable, FreePortal, Gesture, HistoryFailed, LevelExported, ManagerSystems,
    MissingAsset, OpenProject, OpenReport, Paint, PlaceElement, Placement, PortalsRemoved,
    ProjectOpened, ProjectRefused, ProjectRequest, ProjectSaved, Redo, RemoveElement, SaveProject,
    SetPortalIntoWall, StrokeChange, ThumbnailsUnavailable, Undo, UnknownKind,
};
pub use panics::CaughtPanics;
pub use pointer::{PointOf, Pointer, SnapSystems, Snapped, SnappedPoint, Snapping};
pub use portal::{Anchoring, PORTAL, Portal, PortalAnchor, Side};
pub use project::{Bounds, Grid, Layer, Level, Project};
pub use resolution::{MissingReason, Resolution, ResolutionTable};
pub use room::{Edge, FillMesh, ROOM, Room, RoomShape};
pub use serialisation::{
    Envelope, Envelopes, Serialisable, SerialisableComponent, SerialisationError,
    SerialisationRegistry, Tier, UnknownComponents, parse_version, read_current_version,
};
pub use snapshot::{LayerSnapshot, LevelSnapshot, ProjectSnapshot};
pub use terrain::{
    BrushSettings, COVERAGE_PIXELS_PER_CELL, COVERAGE_TILE_CELLS, COVERAGE_TILE_PIXELS,
    CoverageTile, Stroke, TERRAIN, Terrain, TerrainCoverage, TileKey,
};
pub use thumbnails::{THUMBNAIL_SOURCE, ThumbnailState, Thumbnails};
pub use viewport::Viewport;
pub use wall::{Colour, LinePlace, LinePoint, Segment, Stretch, StrokeMesh, WALL, Wall, WallShape};

use bevy_app::{App, Plugin, Update};
use bevy_ecs::schedule::IntoScheduleConfigs;

/// Registers the model's types, messages, the Element kind registry, and the serialisation
/// registry with the model's own components, and orders the Managers' message handling.
pub struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        let mut serialisation = SerialisationRegistry::default();
        serialisation::register_all(&mut serialisation);
        app.register_type::<Project>()
            .register_type::<Grid>()
            .register_type::<Bounds>()
            .register_type::<Level>()
            .register_type::<Layer>()
            .register_type::<Element>()
            .register_type::<ElementId>()
            .register_type::<Prop>()
            .register_type::<Wall>()
            .register_type::<Portal>()
            .register_type::<Terrain>()
            .register_type::<Room>()
            .register_type::<AssetReferences>()
            .register_type::<ResolutionTable>()
            .register_type::<UnknownComponents>()
            .register_type::<AssetFolder>()
            .register_type::<Thumbnails>()
            .register_type::<EditorDirectories>()
            .register_type::<Viewport>()
            .insert_resource(serialisation)
            .init_resource::<EditorDirectories>()
            .init_resource::<SearchMatches>()
            .init_resource::<Viewport>()
            .init_resource::<Pointer>()
            .init_resource::<SnappedPoint>()
            .init_resource::<ElementKindRegistry>()
            .add_message::<AddFolder>()
            .add_message::<FolderAdded>()
            .add_message::<FolderRefused>()
            .add_message::<FolderUnavailable>()
            .add_message::<AssetFolderChanged>()
            .add_message::<Browse>()
            .add_message::<ThumbnailsUnavailable>()
            .add_message::<Apply>()
            .add_message::<CommandFailed>()
            .add_message::<PortalsRemoved>()
            .add_message::<HistoryFailed>()
            .add_message::<Undo>()
            .add_message::<Redo>()
            .add_message::<SaveProject>()
            .add_message::<OpenProject>()
            .add_message::<ProjectSaved>()
            .add_message::<ProjectOpened>()
            .add_message::<ProjectRefused>()
            .add_message::<ExportLevel>()
            .add_message::<LevelExported>()
            .add_message::<ExportRefused>()
            .configure_sets(
                Update,
                (
                    ManagerSystems::Commands,
                    ManagerSystems::Undo,
                    ManagerSystems::Redo,
                )
                    .chain(),
            );
    }
}
