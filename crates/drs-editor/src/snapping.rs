//! Snapping as the Editor sees it: the Snap switch, Alt to place freely, the Pointer it writes
//! each frame, the snapped point its tools place at, and the marker that shows where a click of
//! the Wall or the Room tool lands.
//!
//! The Editor never snaps itself: it says in the Pointer what should snap, and the authoring
//! Manager answers in the snapped point, which the Editor uses only while it answers the Pointer
//! last written, so what is placed is what was shown.

use crate::handles::{HANDLES, OutlineHandle};
use crate::state::{EditorState, Interaction, Tool};
use crate::viewport::LevelView;
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Res, ResMut, Single};
use bevy::gizmos::gizmos::Gizmos;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::math::{Isometry2d, Vec2};
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::input::EguiWantsInput;
use drs_model::{PointOf, Pointer, Snapped, SnappedPoint, Snapping, Viewport};

/// How close to the pointer, in screen pixels, a point of a Wall or a Room is within reach.
const REACH_PIXELS: f32 = 8.0;
/// The radius of the marker's ring, in screen pixels.
const MARKER_PIXELS: f32 = 5.0;
/// The keys that place freely while held: Alt, Option on macOS, on either side.
const FREE: [KeyCode; 2] = [KeyCode::AltLeft, KeyCode::AltRight];

/// The Snap switch on the tool strip: on when the editor starts, never a history step, and kept
/// neither in the Project nor between runs.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SnapSwitch {
    /// Whether points snap.
    pub on: bool,
}

impl Default for SnapSwitch {
    /// On, as the editor starts.
    fn default() -> Self {
        Self { on: true }
    }
}

/// What the tools place at: the snapped point last derived, while it answers the Pointer the
/// Editor last wrote apart from where the pointer is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Shown(Option<Snapped>);

impl Shown {
    /// The snapped point as it answers `pointer`.
    pub(crate) fn of(snapped: &SnappedPoint, pointer: &Pointer) -> Self {
        Self(snapped.answering(pointer))
    }

    /// Where a point goes: the snapped point, or `free` while nothing snaps a point.
    pub(crate) fn point_or(self, free: Vec2) -> Vec2 {
        match self.0 {
            Some(Snapped::Point { position, .. }) => position,
            Some(Snapped::Move { .. }) | None => free,
        }
    }

    /// How far a whole drag moves, in whole cells, or `free` while nothing snaps a move.
    pub(crate) fn travel_or(self, free: Vec2) -> Vec2 {
        match self.0 {
            Some(Snapped::Move { travel }) => travel,
            Some(Snapped::Point { .. }) | None => free,
        }
    }
}

/// What the pointer snaps while the Editor is in `state`: the point of a selected Wall or Room
/// being dragged, leaving that point out; the whole Wall or Room being dragged; the next point of
/// the Wall or the Room tool; and nothing else, a control point, a middle, a Prop, a Portal, and
/// a stroke included.
fn what_snaps(state: &EditorState) -> Snapping {
    match state.interaction {
        Interaction::Handle {
            element,
            handle: OutlineHandle::Point(index),
            ..
        } => Snapping::Point {
            left_out: Some(PointOf { element, index }),
        },
        Interaction::Moving { from, .. } => Snapping::Move { from },
        Interaction::Idle | Interaction::Outlining { .. }
            if matches!(state.tool, Tool::Wall | Tool::Room) =>
        {
            Snapping::Point { left_out: None }
        }
        Interaction::Idle
        | Interaction::Outlining { .. }
        | Interaction::Handle { .. }
        | Interaction::Painting
        | Interaction::Panning { .. }
        | Interaction::Sliding { .. }
        | Interaction::Pressed { .. } => Snapping::Nothing,
    }
}

/// Writes the Pointer for the frame, after the tools have used the snapped point of the last:
/// the Level the Author is working on, where the pointer is, the reach at the Viewport's zoom,
/// and what snaps. Nothing snaps while the switch is off, Alt is held outside a text field, or an
/// Export runs, nor, with no gesture under way, while the pointer is off the viewport or over a
/// panel. A gesture under way keeps snapping wherever the pointer goes in the window, over a
/// panel included, and where it last was once it leaves the window, so a drag released there
/// lands where it was shown. The Pointer is written only when it changes.
#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system is spelled out by what it reads and writes"
)]
pub(crate) fn write_pointer(
    window: Single<&Window, With<PrimaryWindow>>,
    keys: Res<ButtonInput<KeyCode>>,
    egui: Res<EguiWantsInput>,
    state: Res<EditorState>,
    switch: Res<SnapSwitch>,
    viewport: Res<Viewport>,
    level: LevelView,
    mut pointer: ResMut<Pointer>,
) {
    let under_way = state.interaction != Interaction::Idle;
    let cursor = window.cursor_position().filter(|cursor| {
        under_way || (viewport.contains(*cursor) && !egui.wants_any_pointer_input())
    });
    let free = !egui.wants_any_keyboard_input() && keys.any_pressed(FREE);
    let snapping = if switch.on && !free && !state.exporting && (under_way || cursor.is_some()) {
        what_snaps(&state)
    } else {
        Snapping::Nothing
    };
    let written = Pointer {
        level: level.current_level(),
        cells: cursor.map_or(pointer.cells, |cursor| viewport.cells_at(cursor)),
        reach: REACH_PIXELS / viewport.zoom,
        snapping,
    };
    if *pointer != written {
        *pointer = written;
    }
}

/// Draws the marker where the next click of the Wall or the Room tool lands: a small ring at the
/// snapped point, filled when it lies on another Element's point. Nothing is drawn while
/// nothing snaps or an Export runs.
pub(crate) fn draw_marker(
    mut gizmos: Gizmos,
    state: Res<EditorState>,
    viewport: Res<Viewport>,
    pointer: Res<Pointer>,
    snapped: Res<SnappedPoint>,
) {
    if state.exporting || !matches!(state.tool, Tool::Wall | Tool::Room) {
        return;
    }
    let Some(Snapped::Point { position, on }) = snapped.answering(&pointer) else {
        return;
    };
    let at = Isometry2d::from_translation(position);
    let pixel = 1.0 / viewport.zoom;
    gizmos.circle_2d(at, MARKER_PIXELS * pixel, HANDLES);
    if on.is_some() {
        // Rings a pixel apart fill the marker, which gizmos cannot fill.
        let mut radius = MARKER_PIXELS - 1.0;
        while radius > 0.0 {
            gizmos.circle_2d(at, radius * pixel, HANDLES);
            radius -= 1.0;
        }
    }
}

/// Logs the Snap switch, the Pointer, and the snapped point, for a script to check what snapping
/// did.
#[cfg(feature = "dev")]
pub(crate) fn describe(switch: SnapSwitch, pointer: &Pointer, snapped: &SnappedPoint) {
    bevy::log::info!(
        "describe: snap {}, pointer {:?}, snapped {:?} answering {:?}",
        if switch.on { "on" } else { "off" },
        pointer,
        snapped.snapped,
        snapped.pointer
    );
}
