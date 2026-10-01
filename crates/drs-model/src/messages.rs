//! The messages the Editor sends to the Managers, and the reports that come back.

use crate::{CanonicalName, ElementId, FolderKey};
use bevy_ecs::entity::Entity;
use bevy_ecs::message::Message;
use bevy_math::Vec2;
use std::path::PathBuf;

/// Add Asset Folder: make a folder's Assets available under a Canonical Name.
///
/// Handled by the library Manager; answered with [`FolderAdded`] or [`FolderRefused`].
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct AddFolder {
    /// The folder, as the Author picked it.
    pub path: PathBuf,
    /// The name the Author gave it.
    pub name: CanonicalName,
}

/// An Asset Folder was added and its Assets indexed.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct FolderAdded {
    /// The entity carrying the folder.
    pub folder: Entity,
    /// The folder's Canonical Name.
    pub name: CanonicalName,
    /// The folder's device-local key.
    pub key: FolderKey,
}

/// An [`AddFolder`] was refused and nothing was recorded.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct FolderRefused {
    /// The folder that was refused.
    pub path: PathBuf,
    /// The name it was given.
    pub name: CanonicalName,
    /// Why.
    pub reason: FolderRefusal,
}

/// A remembered Asset Folder could not be indexed at start; it stays known but shows no Assets.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct FolderUnavailable {
    /// The entity carrying the folder.
    pub folder: Entity,
    /// The folder's Canonical Name.
    pub name: CanonicalName,
    /// The folder's path as the Author gave it.
    pub path: PathBuf,
    /// What went wrong.
    pub reason: String,
}

/// Why an Asset Folder was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FolderRefusal {
    /// The Canonical Name is empty once trimmed.
    #[error("the Canonical Name is blank")]
    BlankName,
    /// The Canonical Name is already used on this device, ignoring case and normalisation.
    #[error("the name {name} is already used by the folder {}", path.display())]
    NameInUse {
        /// The name as the other folder spells it.
        name: CanonicalName,
        /// The folder that holds the name.
        path: PathBuf,
    },
    /// The same folder was already added, however its path was spelled.
    #[error("this folder was already added as {name}")]
    AlreadyAdded {
        /// The Canonical Name it already has.
        name: CanonicalName,
    },
    /// The folder lies inside an added Asset Folder.
    #[error("the folder lies inside the Asset Folder {name} at {}", path.display())]
    InsideAdded {
        /// The Canonical Name of the folder it lies in.
        name: CanonicalName,
        /// That folder's path.
        path: PathBuf,
    },
    /// The folder contains an added Asset Folder.
    #[error("the folder contains the Asset Folder {name} at {}", path.display())]
    ContainsAdded {
        /// The Canonical Name of the folder it contains.
        name: CanonicalName,
        /// That folder's path.
        path: PathBuf,
    },
    /// The folder does not exist or cannot be listed.
    #[error("the folder cannot be read: {reason}")]
    Unreadable {
        /// What went wrong.
        reason: String,
    },
    /// Writing the Manifest or indexing the folder failed.
    #[error("the folder could not be added: {reason}")]
    Failed {
        /// What went wrong.
        reason: String,
    },
}

impl FolderRefusal {
    /// Whether the refusal is about the Canonical Name, so the Author should be asked for another.
    #[must_use]
    pub fn is_about_the_name(&self) -> bool {
        match self {
            FolderRefusal::BlankName | FolderRefusal::NameInUse { .. } => true,
            FolderRefusal::AlreadyAdded { .. }
            | FolderRefusal::InsideAdded { .. }
            | FolderRefusal::ContainsAdded { .. }
            | FolderRefusal::Unreadable { .. }
            | FolderRefusal::Failed { .. } => false,
        }
    }
}

/// An authoring Command for the authoring Manager to apply.
#[derive(Message, Debug, Clone, PartialEq)]
pub enum Apply {
    /// Place Element: add an Element to a Layer.
    PlaceElement(PlaceElement),
    /// Edit Element: change a property of an Element.
    EditElement(EditElement),
    /// Remove Element: take an Element off its Layer.
    RemoveElement(RemoveElement),
}

/// The Asset the Author chose to place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChosenAsset {
    /// The key of the Asset Folder it sits in.
    pub folder: FolderKey,
    /// Its place in that folder.
    pub place: String,
}

/// Place a Prop of an Asset on a Layer, centred on a point.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaceElement {
    /// The Layer to place on.
    pub layer: Entity,
    /// The centre of the new Element in Grid cells.
    pub position: Vec2,
    /// The Asset to place.
    pub asset: ChosenAsset,
}

/// A property change of an Element.
#[derive(Debug, Clone, PartialEq)]
pub enum ElementChange {
    /// Move the Element's centre to a position in Grid cells.
    Position(Vec2),
}

/// How an [`EditElement`] relates to the gesture it belongs to, so a drag is one history step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    /// A change on its own.
    Single,
    /// The first change of a gesture.
    Begin,
    /// A further change of the gesture in progress.
    Continue,
    /// The last change of the gesture in progress.
    End,
}

/// Change a property of an Element.
#[derive(Debug, Clone, PartialEq)]
pub struct EditElement {
    /// Which Element.
    pub element: ElementId,
    /// What changes.
    pub change: ElementChange,
    /// Where in a gesture this change sits.
    pub gesture: Gesture,
}

/// Take an Element off its Layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveElement {
    /// Which Element.
    pub element: ElementId,
}

/// An [`Apply`] could not be carried out and nothing was recorded.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct CommandFailed {
    /// The Command that failed.
    pub command: Apply,
    /// Why, in words the Author can be shown.
    pub reason: String,
}

/// Take the most recent step back.
#[derive(Message, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Undo;

/// Carry the most recently undone step out again.
#[derive(Message, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Redo;
