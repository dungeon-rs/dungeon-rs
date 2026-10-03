//! The messages the Editor sends to the Managers, and the reports that come back.

use crate::{
    Bounds, BrushSettings, CanonicalName, Colour, ElementId, ElementKindName, FolderKey,
    MissingReason, PortalAnchor, ScanSkips, Side, Stroke,
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
    /// Set Portal into Wall: anchor a Portal to a place along a Wall.
    SetPortalIntoWall(SetPortalIntoWall),
    /// Free Portal: make a Portal set into a Wall freestanding where it stands.
    FreePortal(FreePortal),
    /// Paint: lay a stroke on a Layer's Terrain.
    Paint(Paint),
    /// Resize Bounds: set the Project's Bounds.
    ResizeBounds(ResizeBounds),
}

/// Set the Bounds of the one Project, shared by every Level, to a lower-left corner and a size
/// in whole cells.
///
/// The Bounds are sent whole rather than as a side and an amount, so every step of a gesture is
/// exact and a drag and a typed value send the same message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResizeBounds {
    /// The Bounds to set.
    pub bounds: Bounds,
    /// Where in a gesture this resize sits.
    pub gesture: Gesture,
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
    /// A Portal of an Asset at its image's natural size, set into a Wall when anchored, and
    /// otherwise freestanding, unturned, and centred on a point.
    Portal {
        /// The centre of a freestanding Portal in Grid cells; a set Portal stands where its
        /// anchor says.
        position: Vec2,
        /// The Asset whose image the Portal shows.
        asset: AssetAddress,
        /// Where the Portal is set, or `None` for a freestanding one.
        anchor: Option<PortalAnchor>,
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
    /// A Room through points, closed from the last back to the first, every edge straight, that
    /// cuts or not.
    Room {
        /// The points in Grid cells, in order; three or more.
        points: Vec<Vec2>,
        /// How wide its Walls are drawn, in Grid cells; above zero.
        thickness: f32,
        /// The colour its Walls are drawn in.
        wall_colour: Colour,
        /// The colour its floor is drawn in.
        floor_colour: Colour,
        /// Whether it takes floor away from the Rooms before it on the Layer rather than adding
        /// its own.
        cuts: bool,
    },
}

/// A property change of an Element.
#[derive(Debug, Clone, PartialEq)]
pub enum ElementChange {
    /// Move the Element's centre to a position in Grid cells; a Wall or a Room moves every point
    /// and control point by the same amount.
    Position(Vec2),
    /// Move a Wall or a Room by an amount in Grid cells: every point and control point by that
    /// amount from where it stood when the gesture the change belongs to began, or from where it
    /// stands for a change on its own, so each step of a drag carries the whole travel since the
    /// press.
    MoveBy(Vec2),
    /// Move one point of a Wall or a Room, leaving every other point and every control point
    /// where it is.
    Point {
        /// Which point, counted from zero.
        index: usize,
        /// Where it goes, in Grid cells.
        position: Vec2,
    },
    /// Bend a segment of a Wall or an edge of a Room through a control point, or make it
    /// straight.
    Control {
        /// Which segment or edge, counted from zero.
        segment: usize,
        /// The control point in Grid cells, or `None` to make the segment straight.
        position: Option<Vec2>,
    },
    /// Add a point on a segment of a Wall or an edge of a Room, splitting it into two of the
    /// same shape.
    AddPoint {
        /// Which segment or edge, counted from zero.
        segment: usize,
        /// Where along it, between zero at its first point and one at its second.
        t: f32,
    },
    /// Remove a point of a Wall or a Room, joining the segments or edges at it into one straight
    /// one; a Wall of two points and a Room of three are removed whole.
    RemovePoint {
        /// Which point, counted from zero.
        index: usize,
    },
    /// Set a Wall's or a Room's wall thickness in Grid cells; above zero.
    Thickness(f32),
    /// Set a Wall's colour, or a Room's wall colour.
    Colour(Colour),
    /// Set a Room's floor colour.
    FloorColour(Colour),
    /// Make a Room take floor away from the Rooms before it on its Layer, or add floor of its own.
    Cuts(bool),
    /// Set a Portal's width in Grid cells, its height following its image's proportions; above
    /// zero.
    Width(f32),
    /// Turn a freestanding Portal to a rotation, in radians counter-clockwise.
    Rotation(f32),
    /// Mirror a freestanding Portal across its length, or not.
    Mirrored(bool),
    /// Turn a Portal set into a Wall or a Room to face a side.
    Side(Side),
    /// Slide a Portal set into a Wall or a Room to another place along it.
    Along {
        /// Which segment of the Wall or edge of the Room, counted from zero.
        segment: usize,
        /// Where along it, from zero at its first point to one at its second.
        t: f32,
    },
    /// Make a Terrain's Material show the image of an Asset, every stroke kept as it is.
    Material(AssetAddress),
    /// Change one stroke of a Terrain, named by its number.
    Stroke {
        /// Which stroke, counted from zero, the first laid first.
        stroke: usize,
        /// What changes of it.
        change: StrokeChange,
    },
}

/// A change of one stroke of a Terrain.
#[derive(Debug, Clone, PartialEq)]
pub enum StrokeChange {
    /// Move one point of its path, leaving every other point, its Brush settings and whether it
    /// erases, and every other stroke as they are.
    Point {
        /// Which point of its path, counted from zero.
        index: usize,
        /// Where it goes, in Grid cells.
        position: Vec2,
    },
    /// Move it whole: every point of its path by the difference between the position, in Grid
    /// cells, and the centre of the smallest box around its points.
    Position(Vec2),
    /// Set its Brush settings, all three together.
    Brush(BrushSettings),
    /// Turn it to erasing, or to painting.
    Erase(bool),
    /// Remove it, numbering each later stroke one lower; removing the only stroke removes the
    /// Terrain.
    Remove,
}

/// How an [`EditElement`] or a [`ResizeBounds`] relates to the gesture it belongs to, so a drag is
/// one history step.
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

/// Anchor a Portal, freestanding or set into any Wall or Room, to a place along a Wall or a
/// Room's Walls.
#[derive(Debug, Clone, PartialEq)]
pub struct SetPortalIntoWall {
    /// Which Portal.
    pub portal: ElementId,
    /// Where it is set: the Wall or the Room, the segment or edge, the parameter, and the side.
    pub anchor: PortalAnchor,
}

/// Make a Portal set into a Wall freestanding with the position, rotation, and mirroring it has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreePortal {
    /// Which Portal.
    pub portal: ElementId,
}

/// A Command removed the Portals set into a part of a Wall or a Room that it removed, or into a
/// Room whose Wall at their centre it took away, in the same step.
///
/// Sent by the authoring Manager after the Command, once for each Wall or Room the removed
/// Portals were set into, for the Editor to tell the Author how many went.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct PortalsRemoved {
    /// The Wall or the Room the Portals were set into.
    pub host: ElementId,
    /// The Portals removed.
    pub portals: Vec<ElementId>,
}

/// Add a stroke to the topmost Terrain on a Layer, making the Terrain when the Layer has none
/// and the stroke paints.
#[derive(Debug, Clone, PartialEq)]
pub struct Paint {
    /// The Layer to paint on.
    pub layer: Entity,
    /// The stroke: its path in Grid cells, the Brush settings it is laid with, and whether it
    /// erases.
    pub stroke: Stroke,
    /// The Asset whose image the stroke paints with, or `None` to paint with the Terrain's own
    /// Material; an erase's is not looked at.
    pub asset: Option<AssetAddress>,
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
    /// The most pixels an Export's image may be wide or high.
    pub const MOST_IMAGE_PIXELS: u32 = 100_000;

    /// The highest whole resolution, up to [`ExportLevel::MOST_PIXELS_PER_CELL`], at which
    /// `bounds` make an image no more than [`ExportLevel::MOST_IMAGE_PIXELS`] a side; zero when
    /// not even one pixel per cell fits.
    #[must_use]
    pub fn largest_pixels_per_cell(bounds: Bounds) -> u32 {
        (Self::MOST_IMAGE_PIXELS / bounds.size.max_element().max(1)).min(Self::MOST_PIXELS_PER_CELL)
    }
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

/// Browse: what the browser searches for and shows, so the library Manager answers the search
/// and serves the thumbnails shown first. Sent by the Editor whenever either changes.
///
/// Handled by the library Manager, which answers the search text in
/// [`crate::SearchMatches`] and otherwise only updates the thumbnail states.
#[derive(Message, Debug, Clone, Default, PartialEq, Eq)]
pub struct Browse {
    /// The text typed in the browser's search field, as typed.
    pub search: String,
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
