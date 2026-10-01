#![doc = include_str!("../README.md")]

mod assets;
mod directories;
mod element;
mod messages;
mod project;

pub use assets::{
    AssetFolder, AssetFolderReference, AssetKind, AssetReference, AssetReferenceRow,
    AssetReferences, CanonicalName, Fingerprint, FolderKey, IndexedAsset,
};
pub use directories::EditorDirectories;
pub use element::{
    Element, ElementId, ElementKindDescriptor, ElementKindName, ElementKindRegistry, PROP, Prop,
};
pub use messages::{
    AddFolder, Apply, ChosenAsset, CommandFailed, EditElement, ElementChange, FolderAdded,
    FolderRefusal, FolderRefused, FolderUnavailable, Gesture, PlaceElement, Redo, RemoveElement,
    Undo,
};
pub use project::{Bounds, Grid, Layer, Level, Project};

use bevy_app::{App, Plugin};

/// Registers the model's types, messages, and the Element kind registry.
pub struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Project>()
            .register_type::<Grid>()
            .register_type::<Bounds>()
            .register_type::<Level>()
            .register_type::<Layer>()
            .register_type::<Element>()
            .register_type::<ElementId>()
            .register_type::<Prop>()
            .register_type::<AssetReferences>()
            .register_type::<AssetFolder>()
            .register_type::<EditorDirectories>()
            .init_resource::<EditorDirectories>()
            .init_resource::<ElementKindRegistry>()
            .add_message::<AddFolder>()
            .add_message::<FolderAdded>()
            .add_message::<FolderRefused>()
            .add_message::<FolderUnavailable>()
            .add_message::<Apply>()
            .add_message::<CommandFailed>()
            .add_message::<Undo>()
            .add_message::<Redo>();
    }
}
