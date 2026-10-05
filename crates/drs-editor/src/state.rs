//! What the Editor itself keeps: the chosen Asset, the selection, the search, the status line,
//! the prompts and dialogs in progress, the Export under way, the gesture under way, and the
//! tool with the Wall, the Room, or the stroke being drawn and the options being changed. None of
//! it is domain state.

use crate::bounds::{BoundsTool, Edges};
use crate::gesture::Drag;
use crate::handles::OutlineHandle;
use crate::paint::PaintTool;
use crate::portals::PortalTool;
use crate::rooms::RoomTool;
use crate::walls::WallTool;
use bevy::ecs::resource::Resource;
use bevy::math::Vec2;
use drs_model::{AssetAddress, Bounds, ElementId, ExportLevel, OpenReport};
use std::path::PathBuf;

/// The Editor's own state.
#[derive(Resource, Default)]
pub(crate) struct EditorState {
    /// The Asset the next click places, if one is chosen.
    pub chosen: Option<Chosen>,
    /// The selected Element, if any. Selection is never a history step.
    pub selected: Option<ElementId>,
    /// The text typed in the Assets panel's search field.
    pub search: String,
    /// What the status line reports.
    pub status: String,
    /// The Canonical Name prompt, while a folder is being added.
    pub prompt: Option<NamePrompt>,
    /// The save, discard, or cancel question, while unsaved changes stand in the way of
    /// opening a file or quitting.
    pub question: Option<Question>,
    /// The report of what an opened Project is missing, until it is dismissed.
    pub report: Option<OpenReport>,
    /// The Export dialog, while the Author chooses a resolution.
    pub export: Option<ExportDialog>,
    /// Whether an Export is being written, so the viewport waits for it.
    pub exporting: bool,
    /// The pointer gesture under way in the viewport.
    pub interaction: Interaction,
    /// The tool the viewport's clicks serve.
    pub tool: Tool,
    /// The selected handle of the selected Wall or Room and the Element it belongs to.
    pub handle: Option<(ElementId, OutlineHandle)>,
    /// The Wall tool's own state.
    pub walls: WallTool,
    /// The Portal tool's own state.
    pub portals: PortalTool,
    /// The Paint tool's own state.
    pub paint: PaintTool,
    /// The Room tool's own state.
    pub rooms: RoomTool,
    /// The Bounds tool's own state.
    pub bounds: BoundsTool,
}

impl EditorState {
    /// Whether a modal dialog of the Editor's own is open, so shortcuts wait.
    pub fn modal_open(&self) -> bool {
        self.prompt.is_some()
            || self.question.is_some()
            || self.report.is_some()
            || self.export.is_some()
    }

    /// The selected handle of `element`, if one is selected.
    pub fn handle_of(&self, element: ElementId) -> Option<OutlineHandle> {
        self.handle
            .and_then(|(owner, handle)| (owner == element).then_some(handle))
    }

    /// Whether the left button is down on an Element, a handle of a Wall or a Room, a Portal set
    /// into its host, or the Bounds' edges, or with the Room tool, whether or not the pointer has
    /// moved since: a press names what it drags by identity and number, so nothing may renumber
    /// them before the release, and a drag of the Bounds is one step from its press.
    pub fn pressing(&self) -> bool {
        matches!(
            self.interaction,
            Interaction::Pressed { .. }
                | Interaction::Handle { .. }
                | Interaction::Sliding { .. }
                | Interaction::Outlining { .. }
                | Interaction::Moving { .. }
                | Interaction::Resizing { .. }
        )
    }

    /// Whether a step is still being made, or may be about to be, by a press or a drag, by a
    /// Wall, a Room, or a stroke being drawn, by a press on a stroke or a handle of one or its
    /// drag, or by an option or a field of the Bounds held while it changes, so undo and redo
    /// wait.
    pub fn step_under_way(&self) -> bool {
        self.pressing()
            || self.walls.drawing_in_progress()
            || self.paint.step_in_progress()
            || self.rooms.drawing_in_progress()
            || self.walls.option_in_progress()
            || self.rooms.option_in_progress()
            || self.portals.option_in_progress()
            || self.paint.option_in_progress()
            || self.bounds.option_in_progress()
    }
}

/// What the viewport's clicks do.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Tool {
    /// Clicks select, or place the chosen Asset.
    #[default]
    Select,
    /// Clicks add the points of a Wall.
    Wall,
    /// Clicks place Portals of the chosen Asset.
    Portal,
    /// Drags lay strokes of Terrain.
    Paint,
    /// Clicks add the points of a Room, and a drag draws a rectangular one.
    Room,
    /// Drags of the Bounds' edges and corners resize them.
    Bounds,
}

/// The Asset chosen for placing, with its name for the status line.
pub(crate) struct Chosen {
    /// The Asset as a Place Element names it.
    pub asset: AssetAddress,
    /// The Asset's name.
    pub name: String,
}

/// The Canonical Name prompt for a folder picked in the dialog.
pub(crate) struct NamePrompt {
    /// The folder picked.
    pub path: PathBuf,
    /// The name as typed, proposed from the folder's own name.
    pub name: String,
    /// Why the last attempt was refused, shown until the next.
    pub refusal: Option<String>,
    /// Whether Add Asset Folder was sent and its answer is awaited.
    pub awaiting: bool,
    /// Whether the text field should take focus on the next frame.
    pub focus: bool,
}

/// The save, discard, or cancel question asked before an action that would lose unsaved changes.
pub(crate) struct Question {
    /// What the Author was about to do.
    pub pending: Pending,
    /// Where the question stands.
    pub phase: Phase,
    /// Why the save it led to was refused, shown until the next attempt.
    pub refusal: Option<String>,
}

impl Question {
    /// The question as first asked, in front of `pending`.
    pub fn asking(pending: Pending) -> Self {
        Self {
            pending,
            phase: Phase::Asking,
            refusal: None,
        }
    }
}

/// The action an unsaved-changes question stands in front of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pending {
    /// Open another Project.
    Open,
    /// Quit the editor.
    Quit,
}

/// Where an unsaved-changes question stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    /// The Author is being asked.
    Asking,
    /// Save was chosen and its answer is awaited.
    Saving,
    /// The pending action goes ahead.
    Proceed,
}

/// The Export dialog: the resolution the Author is choosing.
pub(crate) struct ExportDialog {
    /// How many image pixels one Grid cell spans.
    pub pixels_per_cell: u32,
}

impl Default for ExportDialog {
    /// The proposed resolution.
    fn default() -> Self {
        Self {
            pixels_per_cell: ExportLevel::PROPOSED_PIXELS_PER_CELL,
        }
    }
}

/// A pointer gesture in the viewport.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum Interaction {
    /// Nothing is under way.
    #[default]
    Idle,
    /// The Paint tool's gesture is under way, in whichever mode it is: a stroke being drawn while
    /// painting or erasing, or, editing strokes, a press on a stroke or a handle of one, which a
    /// drag reshapes; the release lays the stroke or ends the drag.
    Painting,
    /// The view is being dragged.
    Panning {
        /// Where the pointer was when the view last followed it.
        last: Vec2,
    },
    /// The left button went down on a handle of the selected Wall or Room; a drag moves the
    /// handle.
    Handle {
        /// The Wall or the Room.
        element: ElementId,
        /// The handle under the pointer.
        handle: OutlineHandle,
        /// Where the handle was, in cells, when the button went down.
        origin: Vec2,
        /// The press, and where the drag last moved the handle to, in cells.
        drag: Drag,
    },
    /// The left button went down on a Portal set into a Wall or a Room; a drag slides it along
    /// its line.
    Sliding {
        /// The Portal.
        element: ElementId,
        /// The press, and the pointer, in cells, the drag last slid the Portal towards.
        drag: Drag,
    },
    /// The left button went down with the Room tool and no point placed; a drag draws a
    /// rectangle, and a release without one is a click.
    Outlining {
        /// The pointer, on screen, when the button went down.
        pointer: Vec2,
        /// Where the press put the rectangle's first corner, or the click its first point, in
        /// cells: where snapping put the pointer, or the pointer itself.
        from: Vec2,
        /// The pointer, on screen, when the rectangle was last dragged; `None` until the drag
        /// begins.
        moved_at: Option<Vec2>,
    },
    /// The left button went down on a Wall or a Room; a drag moves it whole, by the amount the
    /// pointer travelled, in whole cells with snapping.
    Moving {
        /// The Wall or the Room under the pointer.
        element: ElementId,
        /// Where the pointer was, in cells, when the button went down.
        from: Vec2,
        /// The press, and how far the drag has moved it since, in cells.
        drag: Drag,
    },
    /// The left button went down on an edge or a corner of the Bounds with the Bounds tool; a drag
    /// moves the edges there by whole cells.
    Resizing {
        /// The edges the drag moves.
        edges: Edges,
        /// The Bounds when the button went down, brought within the limits.
        from: Bounds,
        /// The press, and where the drag last put the dragged edges, in cells: the vertical
        /// one's place across and the horizontal one's up.
        drag: Drag,
    },
    /// The left button went down on an Element that is neither a Wall nor a Room; a drag moves
    /// it.
    Pressed {
        /// The Element under the pointer.
        element: ElementId,
        /// The Element's centre, in cells, when the button went down.
        origin: Vec2,
        /// The press, and where the drag last moved the Element's centre to, in cells.
        drag: Drag,
    },
}
