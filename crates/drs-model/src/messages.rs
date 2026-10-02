//! The messages the Editor sends to the Managers, and the reports that come back.

use crate::{
    CanonicalName, Colour, ElementId, ElementKindName, FolderKey, MissingReason, ScanSkips,
};
use bevy_ecs::entity::Entity;
use bevy_ecs::message::Message;
use bevy_ecs::schedule::SystemSet;
use bevy_math::Vec2;
use std::path::PathBuf;

/// The order the Managers handle their messages in within a frame: every Command before Undo,
/// and Undo before Redo, so a Command and the Undo sent in the same frame apply in the order the
/// Author gave them whichever Manager handles each.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ManagerSystems {
    /// The Commands and requests: [`AddFolder`], [`Apply`], [`SaveProject`], [`OpenProject`],
    /// [`ExportLevel`], and those to come.
    Commands,
    /// [`Undo`].
    Undo,
    /// [`Redo`].
    Redo,
}

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
    /// What indexing the folder skipped, for the Editor to show.
    pub skips: ScanSkips,
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

/// An Asset as this device finds it: the key of its Asset Folder and its place in that folder.
/// It names the Asset the Author chose to place and the Assets the browser shows.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetAddress {
    /// The key of the Asset Folder it sits in.
    pub folder: FolderKey,
    /// Its place in that folder.
    pub place: String,
}

/// Place an Element on top of a Layer.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaceElement {
    /// The Layer to place on.
    pub layer: Entity,
    /// What to place.
    pub placement: Placement,
}

/// The Element a [`PlaceElement`] places.
#[derive(Debug, Clone, PartialEq)]
pub enum Placement {
    /// A Prop of an Asset, centred on a point.
    Prop {
        /// The centre of the new Element in Grid cells.
        position: Vec2,
        /// The Asset to place.
        asset: AssetAddress,
    },
    /// A Wall through points, every segment straight.
    Wall {
        /// The points in Grid cells, in order; two or more.
        points: Vec<Vec2>,
        /// How wide the Wall is drawn, in Grid cells; above zero.
        thickness: f32,
        /// The colour it is drawn in.
        colour: Colour,
    },
}

/// A property change of an Element.
#[derive(Debug, Clone, PartialEq)]
pub enum ElementChange {
    /// Move the Element's centre to a position in Grid cells; a Wall moves every point and
    /// control point by the same amount.
    Position(Vec2),
    /// Move one point of a Wall, leaving every other point and every control point where it is.
    Point {
        /// Which point, counted from zero.
        index: usize,
        /// Where it goes, in Grid cells.
        position: Vec2,
    },
    /// Bend a segment of a Wall through a control point, or make it straight.
    Control {
        /// Which segment, counted from zero.
        segment: usize,
        /// The control point in Grid cells, or `None` to make the segment straight.
        position: Option<Vec2>,
    },
    /// Add a point on a segment of a Wall, splitting it into two segments of the same shape.
    AddPoint {
        /// Which segment, counted from zero.
        segment: usize,
        /// Where along it, between zero at its first point and one at its second.
        t: f32,
    },
    /// Remove a point of a Wall, joining the segments at it into one straight segment; a Wall of
    /// two points is removed whole.
    RemovePoint {
        /// Which point, counted from zero.
        index: usize,
    },
    /// Set a Wall's thickness in Grid cells; above zero.
    Thickness(f32),
    /// Set a Wall's colour.
    Colour(Colour),
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

/// An [`Undo`] or [`Redo`] could not be carried out; the step stays where it was in the history.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct HistoryFailed {
    /// Why, in words the Author can be shown.
    pub reason: String,
}

/// Take the most recent step back.
#[derive(Message, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Undo;

/// Carry the most recently undone step out again.
#[derive(Message, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Redo;

/// Asset Folder Changed: the Assets of the Asset Folder known by `name` were added, removed, or
/// changed, or the folder itself arrived or went.
///
/// Sent by the library Manager after an Asset Folder is added, after that is undone or redone,
/// and at startup for each remembered folder; the project Manager re-resolves the Asset
/// References recorded against the name.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct AssetFolderChanged {
    /// The folder's Canonical Name.
    pub name: CanonicalName,
}

/// Save: write the Project to its file, or to `path` to make that the Project's file (Save As).
///
/// Handled by the project Manager; answered with [`ProjectSaved`] or [`ProjectRefused`]. The
/// Project extension is added when `path` lacks it.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct SaveProject {
    /// Where to save, or `None` for the file the Project was last saved to or opened from.
    pub path: Option<PathBuf>,
}

/// Open: replace the current Project with the one in the file at `path`.
///
/// Handled by the project Manager; answered with [`ProjectOpened`] or [`ProjectRefused`].
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct OpenProject {
    /// The file to open.
    pub path: PathBuf,
}

/// The Project was written to `path`, which is now its file.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct ProjectSaved {
    /// The file written.
    pub path: PathBuf,
}

/// The Project in the file at `path` replaced the current one.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct ProjectOpened {
    /// The file opened, now the Project's file.
    pub path: PathBuf,
    /// What the Author is told about the opened Project.
    pub report: OpenReport,
}

/// What an Author is told after opening a Project: what is Missing on this device, and what this
/// editor does not know.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenReport {
    /// Each Missing Asset, in the order of the Asset Reference table.
    pub missing_assets: Vec<MissingAsset>,
    /// Each Element kind this editor does not know, ordered by name.
    pub unknown_kinds: Vec<UnknownKind>,
}

/// A Missing Asset, in the terms the Author is shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingAsset {
    /// The Asset's name.
    pub name: String,
    /// The Canonical Name of its Asset Folder.
    pub folder: CanonicalName,
    /// The folder's version as the Project recorded it.
    pub recorded_version: String,
    /// How many Elements use the Asset.
    pub elements: usize,
    /// Why the Asset is Missing on this device.
    pub reason: MissingReason,
}

/// An Element kind this editor does not know, and how many Elements have it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownKind {
    /// The kind's name as the file records it.
    pub kind: ElementKindName,
    /// How many Elements have the kind.
    pub elements: usize,
}

/// Which Project request a [`ProjectRefused`] answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectRequest {
    /// A [`SaveProject`].
    Save {
        /// The path the request named, if any.
        path: Option<PathBuf>,
    },
    /// An [`OpenProject`].
    Open {
        /// The file the request named.
        path: PathBuf,
    },
}

/// A [`SaveProject`] or [`OpenProject`] was refused and nothing changed.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct ProjectRefused {
    /// The request that was refused.
    pub request: ProjectRequest,
    /// Why, in words the Author can be shown.
    pub reason: String,
}

/// Export Level: write an Export of a Level as a PNG covering exactly the Bounds.
///
/// Handled by the project Manager; answered with [`LevelExported`] or [`ExportRefused`]. The
/// Export is captured in tiles of `tile_size` pixels a side, which changes nothing about the
/// image; the Editor passes [`ExportLevel::DEFAULT_TILE_SIZE`].
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct ExportLevel {
    /// The entity carrying the Level. Today the Export draws everything on every Level of the
    /// Project, which has one Level, so the field selects nothing yet; once a Project has several
    /// Levels, the Export is filtered to this one.
    pub level: Entity,
    /// How many image pixels one Grid cell spans, within
    /// [`ExportLevel::LEAST_PIXELS_PER_CELL`] and [`ExportLevel::MOST_PIXELS_PER_CELL`].
    pub pixels_per_cell: u32,
    /// The file to write; `.png` is added when the name lacks it.
    pub path: PathBuf,
    /// The side, in pixels, of the tiles the Export is assembled from.
    pub tile_size: u32,
}

impl ExportLevel {
    /// The lowest resolution an Export may have.
    pub const LEAST_PIXELS_PER_CELL: u32 = 1;
    /// The highest resolution an Export may have.
    pub const MOST_PIXELS_PER_CELL: u32 = 1024;
    /// The resolution proposed to the Author.
    pub const PROPOSED_PIXELS_PER_CELL: u32 = 100;
    /// The tile size the Editor passes.
    pub const DEFAULT_TILE_SIZE: u32 = 1024;
}

/// An Export was written.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct LevelExported {
    /// The Level that was exported.
    pub level: Entity,
    /// The file that was written.
    pub path: PathBuf,
    /// The image's width in pixels.
    pub width: u32,
    /// The image's height in pixels.
    pub height: u32,
}

/// An [`ExportLevel`] was refused or failed; no file was written or left behind.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct ExportRefused {
    /// The Level that was to be exported.
    pub level: Entity,
    /// The file that was to be written.
    pub path: PathBuf,
    /// Why, in words the Author can be shown.
    pub reason: String,
}

/// Browse: what the browser shows, so the library Manager serves it first. Sent by the Editor
/// whenever the set changes.
///
/// Handled by the library Manager; nothing comes back but the thumbnail states it updates.
#[derive(Message, Debug, Clone, Default, PartialEq, Eq)]
pub struct Browse {
    /// The Assets whose thumbnails the browser wants first: those in its laid-out rows and the
    /// rows either side.
    pub wanted: Vec<AssetAddress>,
}

/// Thumbnails cannot be kept on this device for the rest of the session: the thumbnail cache
/// could not be opened or written. Sent once by the library Manager; the browser shows
/// placeholders from then on.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct ThumbnailsUnavailable {
    /// Why, in words the Author can be shown.
    pub reason: String,
}
