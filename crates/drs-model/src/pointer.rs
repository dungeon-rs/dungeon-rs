//! Where the Author's pointer is on the Level and what it should snap to, and the point snapping
//! puts it at.

use crate::ElementId;
use bevy_ecs::entity::Entity;
use bevy_ecs::resource::Resource;
use bevy_ecs::schedule::SystemSet;
use bevy_math::Vec2;

/// A point of a Wall or a Room: the Element and the number of the point, counted from zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PointOf {
    /// The Wall or the Room.
    pub element: ElementId,
    /// Which point, counted from zero.
    pub index: usize,
}

/// What the pointer is snapping.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Snapping {
    /// Nothing: no tool or drag that snaps, snapping switched off, Alt held, or the pointer off
    /// the view with no drag under way.
    #[default]
    Nothing,
    /// A point being placed or dragged, which goes to the nearest point of a Wall or a Room
    /// within reach, or else to the nearest corner of a Grid cell.
    Point {
        /// The point being dragged, which is never within reach of itself; `None` while a point
        /// is being placed.
        left_out: Option<PointOf>,
    },
    /// A whole Wall or Room being dragged, which moves by whole cells.
    Move {
        /// Where the pointer was when the drag began, in Grid cells.
        from: Vec2,
    },
}

/// Where the pointer is on the Level and what it should snap: presentation state the Editor
/// writes each frame as it reads the pointer, which the authoring Manager snaps through the
/// shape Engine. It is never a Command, never a history step, and never saved.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Pointer {
    /// The Level the Author is working on, whose Walls and Rooms are within reach.
    pub level: Option<Entity>,
    /// Where the pointer is, in Grid cells.
    pub cells: Vec2,
    /// How far from the pointer, in Grid cells, a point of a Wall or a Room is within reach.
    pub reach: f32,
    /// What is being snapped.
    pub snapping: Snapping,
}

impl Pointer {
    /// Whether the two say the same apart from where the pointer is: the same Level, reach, and
    /// thing being snapped.
    #[must_use]
    pub fn same_but_for_position(&self, other: &Self) -> bool {
        Self {
            cells: other.cells,
            ..*self
        } == *other
    }
}

/// Where snapping puts what the pointer snaps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Snapped {
    /// The point: on a Grid corner, or exactly on a point of another Wall or Room.
    Point {
        /// Where the point goes, in Grid cells.
        position: Vec2,
        /// The Wall or the Room whose point it lies on, or `None` for a Grid corner.
        on: Option<ElementId>,
    },
    /// The move: the pointer's travel since the drag began, in whole cells.
    Move {
        /// The travel, each coordinate a whole number of cells.
        travel: Vec2,
    },
}

/// The point snapping puts the [`Pointer`] at: derived, never saved and never a history step.
/// The authoring Manager writes it through the shape Engine's Snap after every Command, Undo,
/// and Redo of the frame, whenever the Pointer or a Wall or Room changed, and only when the
/// answer differs; the Editor reads it to place and drag what it shows.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct SnappedPoint {
    /// Where snapping puts what is snapped, or `None` while nothing is.
    pub snapped: Option<Snapped>,
    /// The Pointer it answers.
    pub pointer: Pointer,
}

impl SnappedPoint {
    /// The answer, when it answers `pointer` apart from where the pointer is.
    #[must_use]
    pub fn answering(&self, pointer: &Pointer) -> Option<Snapped> {
        self.pointer
            .same_but_for_position(pointer)
            .then_some(self.snapped)
            .flatten()
    }
}

/// The system that derives the [`SnappedPoint`], so the Editor writes the Pointer before it and
/// draws what it answers after it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SnapSystems;
