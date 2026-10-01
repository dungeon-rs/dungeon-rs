#![doc = include_str!("../README.md")]

mod assets;
mod directories;
mod element;
mod file;
mod messages;
mod project;
mod resolution;
mod serialisation;
mod viewport;

pub use assets::{
    AssetFolder, AssetFolderReference, AssetKind, AssetReference, AssetReferenceRow,
    AssetReferences, AssetReferencesFull, CanonicalName, Fingerprint, FolderKey, IndexedAsset,
    ScanSkips,
};
pub use directories::{EditorDirectories, NoPlatformDirectories, ResolvedDirectories};
pub use element::{
    Element, ElementId, ElementKindDescriptor, ElementKindName, ElementKindRegistry, PROP, Prop,
};
pub use file::{
    FORMAT_VERSION, LayerRecord, LevelRecord, PROJECT_EXTENSION, ProjectFile, SavedMark,
    project_name_of, with_project_extension,
};
pub use messages::{
    AddFolder, Apply, AssetFolderChanged, ChosenAsset, CommandFailed, EditElement, ElementChange,
    ExportLevel, ExportRefused, FolderAdded, FolderRefusal, FolderRefused, FolderUnavailable,
    Gesture, HistoryFailed, LevelExported, ManagerSystems, MissingAsset, OpenProject, OpenReport,
    PlaceElement, ProjectOpened, ProjectRefused, ProjectRequest, ProjectSaved, Redo, RemoveElement,
    SaveProject, Undo, UnknownKind,
};
pub use project::{Bounds, Grid, Layer, Level, Project};
pub use resolution::{MissingReason, Resolution, ResolutionTable};
pub use serialisation::{
    Envelope, Envelopes, Serialisable, SerialisableComponent, SerialisationError,
    SerialisationRegistry, UnknownComponents, read_only_version,
};
pub use viewport::Viewport;

use bevy_app::{App, Plugin, Update};
use bevy_ecs::schedule::IntoScheduleConfigs;

/// Registers the model's types, messages, the Element kind registry, and the serialisation
/// registry with the model's own components, and orders the Managers' message handling.
pub struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        let mut serialisation = SerialisationRegistry::default();
        serialisation.register::<Project>();
        serialisation.register::<Grid>();
        serialisation.register::<Bounds>();
        serialisation.register::<AssetReferences>();
        serialisation.register::<Level>();
        serialisation.register::<Layer>();
        serialisation.register::<Element>();
        serialisation.register::<Prop>();
        app.register_type::<Project>()
            .register_type::<Grid>()
            .register_type::<Bounds>()
            .register_type::<Level>()
            .register_type::<Layer>()
            .register_type::<Element>()
            .register_type::<ElementId>()
            .register_type::<Prop>()
            .register_type::<AssetReferences>()
            .register_type::<ResolutionTable>()
            .register_type::<UnknownComponents>()
            .register_type::<AssetFolder>()
            .register_type::<EditorDirectories>()
            .register_type::<Viewport>()
            .insert_resource(serialisation)
            .init_resource::<EditorDirectories>()
            .init_resource::<Viewport>()
            .init_resource::<ElementKindRegistry>()
            .add_message::<AddFolder>()
            .add_message::<FolderAdded>()
            .add_message::<FolderRefused>()
            .add_message::<FolderUnavailable>()
            .add_message::<AssetFolderChanged>()
            .add_message::<Apply>()
            .add_message::<CommandFailed>()
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
