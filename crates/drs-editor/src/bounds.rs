//! The Bounds in the viewport: their outline, drawn whatever tool is chosen, and the Bounds tool,
//! whose handles at the corners and the middles of the edges, and the edges' lines, drag the
//! edges by whole cells, and whose fields in the tool strip show and set the left and bottom
//! edges, the width, and the height.
//!
//! Every change of the Bounds is a Resize Bounds carrying them whole; the drag under way and the
//! field held are the Editor's own state. A drag rounds the pointer's travel to whole cells here,
//! since the Bounds are whole cells by their type, so neither snapping nor Alt applies.

use crate::gesture::Drag;
use crate::handles::{self, HANDLE_PIXELS, HANDLES, Outline, OutlineHandle, PICKED};
use crate::state::{EditorState, Interaction, Tool};
use crate::viewport::LevelView;
use bevy::color::Color;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::query::With;
use bevy::ecs::system::{Res, Single};
use bevy::gizmos::config::{GizmoConfig, GizmoConfigGroup, GizmoLineConfig};
use bevy::gizmos::gizmos::Gizmos;
use bevy::math::{Isometry2d, Vec2};
use bevy::reflect::Reflect;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::EguiContexts;
use drs_model::{Apply, Bounds, Gesture, ResizeBounds, Viewport};

/// The colour of the outline's middle line.
const LIGHT: Color = Color::WHITE;
/// The colour of the outline's border, so it shows on pale ground as on dark.
const DARK: Color = Color::BLACK;
/// How far, in logical pixels, the outline's light middle reaches either side of the edge: it is
/// two pixels wide.
const LIGHT_REACH: f32 = 1.0;
/// How far, in logical pixels, the outline's dark border reaches either side of the edge: it is
/// four pixels wide, under the light middle.
const DARK_REACH: f32 = 2.0;
/// How many cells a drag of a field moves per pixel the pointer travels.
const FIELD_SPEED: f64 = 0.1;

/// The gizmos the outline of the Bounds is drawn with: lines one physical pixel wide, laid side
/// by side.
#[derive(Default, Reflect, GizmoConfigGroup)]
pub(crate) struct BoundsGizmos;

/// How the outline's lines are drawn: one physical pixel wide, whatever the zoom.
pub(crate) fn config() -> GizmoConfig {
    GizmoConfig {
        line: GizmoLineConfig {
            width: 1.0,
            ..GizmoLineConfig::default()
        },
        ..GizmoConfig::default()
    }
}

/// The Bounds tool's state: the fields' gesture.
#[derive(Debug, Default)]
pub(crate) struct BoundsTool {
    /// The Bounds a held field last sent, while it is held and has sent any.
    fields: Option<Bounds>,
    /// Whether a field is held, whether or not it has changed anything yet.
    holding: bool,
    /// Where each field was laid out in the last frame, for a script to aim at.
    #[cfg(feature = "dev")]
    laid_out: Vec<(&'static str, egui::Rect)>,
}

impl BoundsTool {
    /// Whether a field of the Bounds is held, so undo and redo wait.
    pub(crate) fn option_in_progress(&self) -> bool {
        self.holding || self.fields.is_some()
    }
}

/// A vertical edge of the Bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Vertical {
    /// The left edge.
    Left,
    /// The right edge.
    Right,
}

/// A horizontal edge of the Bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Horizontal {
    /// The bottom edge.
    Bottom,
    /// The top edge.
    Top,
}

/// The edges a drag of the Bounds moves: one vertical edge, one horizontal edge, or one of each
/// for a corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Edges {
    /// The vertical edge moved, if any.
    pub vertical: Option<Vertical>,
    /// The horizontal edge moved, if any.
    pub horizontal: Option<Horizontal>,
}

impl Edges {
    /// No edge: what a drag of a field moves, since it moves none of them by a handle.
    const NONE: Self = Self {
        vertical: None,
        horizontal: None,
    };

    /// The edges a handle of the Bounds' outline moves: a corner its two edges, a middle its edge.
    fn of(handle: OutlineHandle) -> Self {
        let (vertical, horizontal) = match handle {
            OutlineHandle::Point(0) => (Some(Vertical::Left), Some(Horizontal::Bottom)),
            OutlineHandle::Point(1) => (Some(Vertical::Right), Some(Horizontal::Bottom)),
            OutlineHandle::Point(2) => (Some(Vertical::Right), Some(Horizontal::Top)),
            OutlineHandle::Point(_) => (Some(Vertical::Left), Some(Horizontal::Top)),
            OutlineHandle::Middle(0) | OutlineHandle::Control(0) => {
                (None, Some(Horizontal::Bottom))
            }
            OutlineHandle::Middle(1) | OutlineHandle::Control(1) => (Some(Vertical::Right), None),
            OutlineHandle::Middle(2) | OutlineHandle::Control(2) => (None, Some(Horizontal::Top)),
            OutlineHandle::Middle(_) | OutlineHandle::Control(_) => (Some(Vertical::Left), None),
        };
        Self {
            vertical,
            horizontal,
        }
    }

    /// Whether a handle shows one of these edges or the corner between them.
    fn moved_by(self, handle: OutlineHandle) -> bool {
        let of = Self::of(handle);
        match (of.vertical, of.horizontal) {
            (Some(_), Some(_)) => of == self,
            (Some(vertical), None) => self.vertical == Some(vertical) && self.horizontal.is_none(),
            (None, Some(horizontal)) => {
                self.horizontal == Some(horizontal) && self.vertical.is_none()
            }
            (None, None) => false,
        }
    }

    /// The pointer that says a press here resizes the Bounds.
    fn cursor(self) -> egui::CursorIcon {
        match (self.vertical, self.horizontal) {
            (Some(Vertical::Left), Some(Horizontal::Bottom))
            | (Some(Vertical::Right), Some(Horizontal::Top)) => egui::CursorIcon::ResizeNeSw,
            (Some(_), Some(_)) => egui::CursorIcon::ResizeNwSe,
            (Some(_), None) => egui::CursorIcon::ResizeHorizontal,
            (None, _) => egui::CursorIcon::ResizeVertical,
        }
    }
}

/// The edges of `bounds` in cells, left, bottom, right, and top, as the viewport draws them.
#[expect(
    clippy::cast_precision_loss,
    reason = "an edge far from the origin is drawn a fraction of a cell off, as every point there is"
)]
fn corners(bounds: Bounds) -> [Vec2; 4] {
    let [left, bottom, right, top] = bounds.edges().map(|edge| edge as f32);
    [
        Vec2::new(left, bottom),
        Vec2::new(right, bottom),
        Vec2::new(right, top),
        Vec2::new(left, top),
    ]
}

/// The edges a press at a point in cells at `zoom` drags: a corner's within a handle's reach, or
/// else an edge's middle's, or else the edge whose line is nearest, within a handle's reach of
/// it; `None` elsewhere.
fn edges_at(bounds: Bounds, cells: Vec2, zoom: f32) -> Option<Edges> {
    let corners = corners(bounds);
    let outline = Outline::of_bounds(&corners);
    if let Some((handle, _)) = handles::handle_at(&outline, cells, zoom) {
        return Some(Edges::of(handle));
    }
    (0..corners.len())
        .filter_map(|edge| {
            let (start, end) = outline.ends(edge)?;
            let along = crate::walls::parameter_on_segment(start, end, cells);
            let distance = cells.distance(start.lerp(end, along));
            (distance * zoom <= HANDLE_PIXELS).then_some((edge, distance))
        })
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(edge, _)| Edges::of(OutlineHandle::Middle(edge)))
}

/// Chooses the Bounds tool: the chosen Asset and the selection are dropped, and the Wall, the
/// Room, the Portal, or the Paint tool is left, discarding a Wall, an outline, or a stroke being
/// drawn.
pub(crate) fn choose_bounds_tool(state: &mut EditorState) {
    crate::paint::discard_stroke(state);
    state.chosen = None;
    state.selected = None;
    state.handle = None;
    state.walls.drawing.clear();
    state.rooms.drawing.clear();
    state.tool = Tool::Bounds;
}

/// A left press with the Bounds tool: on a corner, an edge's middle, or an edge's line, it arms a
/// drag of the edges there; anywhere else it does nothing, and picks nothing.
pub(crate) fn press(
    state: &mut EditorState,
    bounds: Option<Bounds>,
    viewport: &Viewport,
    cursor: Vec2,
) {
    let Some(bounds) = bounds else {
        return;
    };
    let Some(edges) = edges_at(bounds, viewport.cells_at(cursor), viewport.zoom) else {
        return;
    };
    state.interaction = Interaction::Resizing {
        edges,
        from: within_limits(bounds, edges),
        drag: Drag::new(cursor),
    };
}

/// `bounds` brought within the limits a Resize Bounds is held to, keeping the edges `edges` drags
/// opposite where they are where it can, so a drag of Bounds opened beyond them sends Bounds that
/// are accepted; Bounds within them come back as they are.
fn within_limits(bounds: Bounds, edges: Edges) -> Bounds {
    let [left, bottom, right, top] = bounds.edges();
    let (left, right) = axis(left, right, edges.vertical == Some(Vertical::Left));
    let (bottom, top) = axis(bottom, top, edges.horizontal == Some(Horizontal::Bottom));
    from_edges([left, bottom, right, top]).unwrap_or(bounds)
}

/// The two edges of one axis, `low` below `high`, within reach of the origin and at least one
/// and at most a thousand cells apart, the low edge giving way when it is the one dragged.
fn axis(low: i64, high: i64, low_dragged: bool) -> (i64, i64) {
    let farthest = Bounds::FARTHEST;
    let most = i64::from(Bounds::MOST_SIDE);
    let least = i64::from(Bounds::LEAST_SIDE);
    let mut low = low.clamp(-farthest, farthest - least);
    let mut high = high.clamp(-farthest + least, farthest);
    let span = high - low;
    if span < least || span > most {
        let span = span.clamp(least, most);
        if low_dragged {
            low = high - span;
        } else {
            high = low + span;
        }
    }
    (low, high)
}

/// The Bounds with edges left, bottom, right, and top, when they fit the Bounds' type.
fn from_edges([left, bottom, right, top]: [i64; 4]) -> Option<Bounds> {
    Some(Bounds {
        origin: bevy::math::IVec2::new(i32::try_from(left).ok()?, i32::try_from(bottom).ok()?),
        size: bevy::math::UVec2::new(
            u32::try_from(right - left).ok()?,
            u32::try_from(top - bottom).ok()?,
        ),
    })
}

/// The Bounds a drag of `edges` from `from` gives once the dragged edges are at `at`, the
/// vertical one's place across and the horizontal one's up, in cells.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the places sent are whole cells within reach of the origin"
)]
fn dragged_to(from: Bounds, edges: Edges, at: Vec2) -> Bounds {
    let [mut left, mut bottom, mut right, mut top] = from.edges();
    match edges.vertical {
        Some(Vertical::Left) => left = at.x as i64,
        Some(Vertical::Right) => right = at.x as i64,
        None => {}
    }
    match edges.horizontal {
        Some(Horizontal::Bottom) => bottom = at.y as i64,
        Some(Horizontal::Top) => top = at.y as i64,
        None => {}
    }
    from_edges([left, bottom, right, top]).unwrap_or(from)
}

/// Where a drag puts an edge that was at `edge`, `opposite` the other edge of its axis: moved by
/// `travel` cells rounded to whole cells, a travel exactly halfway rounded away from zero, and
/// stopped one cell from the opposite edge, a thousand cells from it, and ten thousand cells from
/// the origin.
#[expect(
    clippy::cast_possible_truncation,
    reason = "a travel across the view is far within the range of a whole number, and a cast \
              saturates"
)]
fn dragged_edge(edge: i64, opposite: i64, travel: f32) -> i64 {
    let farthest = Bounds::FARTHEST;
    let most = i64::from(Bounds::MOST_SIDE);
    let least = i64::from(Bounds::LEAST_SIDE);
    let moved = edge.saturating_add(travel.round() as i64);
    if edge < opposite {
        moved.clamp((opposite - most).max(-farthest), opposite - least)
    } else {
        moved.clamp(opposite + least, (opposite + most).min(farthest))
    }
}

/// Moves the dragged edges with the pointer, once it has travelled far enough to be a drag: each
/// lies where it was at the press moved by the pointer's travel across it in whole cells, within
/// the limits; the first move begins the gesture and every later one that gives other Bounds
/// continues it.
#[expect(
    clippy::cast_precision_loss,
    reason = "the places stepped on are whole cells within reach of the origin, which a float \
              holds exactly"
)]
pub(crate) fn drag(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    viewport: &Viewport,
    cursor: Vec2,
) {
    let Interaction::Resizing {
        edges,
        from,
        mut drag,
    } = state.interaction
    else {
        return;
    };
    let travel = viewport.cells_at(cursor) - viewport.cells_at(drag.pressed_at());
    let [left, bottom, right, top] = from.edges();
    let across = match edges.vertical {
        Some(Vertical::Left) => dragged_edge(left, right, travel.x),
        Some(Vertical::Right) => dragged_edge(right, left, travel.x),
        None => 0,
    };
    let up = match edges.horizontal {
        Some(Horizontal::Bottom) => dragged_edge(bottom, top, travel.y),
        Some(Horizontal::Top) => dragged_edge(top, bottom, travel.y),
        None => 0,
    };
    let at = Vec2::new(across as f32, up as f32);
    let Some(gesture) = drag.step(cursor, at) else {
        return;
    };
    apply.write(Apply::ResizeBounds(ResizeBounds {
        bounds: dragged_to(from, edges, at),
        gesture,
    }));
    state.interaction = Interaction::Resizing { edges, from, drag };
}

/// Ends a drag of the Bounds once its button is up, over the viewport, a panel, or outside the
/// window: the Bounds it last sent are sent again as the end of the gesture, so the whole drag is
/// one history step ending where it was last shown. A press that never became a drag just ends.
pub(crate) fn finish(state: &mut EditorState, apply: &mut MessageWriter<Apply>) {
    let Interaction::Resizing { edges, from, drag } = state.interaction else {
        return;
    };
    if let Some(at) = drag.sent() {
        apply.write(Apply::ResizeBounds(ResizeBounds {
            bounds: dragged_to(from, edges, at),
            gesture: Gesture::End,
        }));
    }
    state.interaction = Interaction::Idle;
}

/// The edges the drag of the Bounds under way moves, if one is under way.
fn dragging(state: &EditorState) -> Option<Edges> {
    if let Interaction::Resizing { edges, .. } = state.interaction {
        Some(edges)
    } else {
        None
    }
}

/// Draws the Bounds' outline over every Element and the Grid's lines, whatever tool is chosen: a
/// light line two logical pixels wide over a dark one four pixels wide, the same on screen at
/// every zoom. Gizmo lines are as wide as their configuration in physical pixels, so the band is
/// laid as lines one physical pixel wide side by side, none drawn over another. Nothing outside
/// the Bounds is dimmed, and nothing is drawn while an Export runs.
pub(crate) fn draw_outline(
    mut gizmos: Gizmos<BoundsGizmos>,
    state: Res<EditorState>,
    viewport: Res<Viewport>,
    window: Single<&Window, With<PrimaryWindow>>,
    level: LevelView,
) {
    if state.exporting {
        return;
    }
    let Some(bounds) = level.bounds() else {
        return;
    };
    let [bottom_left, _, top_right, _] = corners(bounds);
    let centre = bottom_left.midpoint(top_right);
    let size = top_right - bottom_left;
    let scale = window.scale_factor().max(1.0);
    // How many cells one physical pixel spans.
    let physical = 1.0 / (viewport.zoom * scale);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a few pixels at any scale factor a display has"
    )]
    let lines = (DARK_REACH * scale).round() as u32;
    for line in 0..lines {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a handful of lines counted exactly"
        )]
        let reach = line as f32 + 0.5;
        let colour = if reach < LIGHT_REACH * scale {
            LIGHT
        } else {
            DARK
        };
        for side in [-1.0, 1.0] {
            let grown = (size + Vec2::splat(2.0 * side * reach * physical)).max(Vec2::ZERO);
            gizmos.rect_2d(Isometry2d::from_translation(centre), grown, colour);
        }
    }
}

/// Draws the Bounds tool's handles: a ring at each corner and a smaller one at the middle of
/// each edge, those the drag under way moves in their own colour. Nothing is drawn while an
/// Export runs or another tool is chosen.
pub(crate) fn draw_handles(
    mut gizmos: Gizmos,
    state: Res<EditorState>,
    viewport: Res<Viewport>,
    level: LevelView,
) {
    if state.exporting || state.tool != Tool::Bounds {
        return;
    }
    let Some(bounds) = level.bounds() else {
        return;
    };
    let dragged = dragging(&state);
    let radius = HANDLE_PIXELS / viewport.zoom;
    let corners = corners(bounds);
    for (handle, at) in handles::handles(&Outline::of_bounds(&corners)) {
        let colour = if dragged.is_some_and(|edges| edges.moved_by(handle)) {
            PICKED
        } else {
            HANDLES
        };
        let radius = match handle {
            OutlineHandle::Point(_) => radius,
            OutlineHandle::Middle(_) | OutlineHandle::Control(_) => radius / 2.0,
        };
        gizmos.circle_2d(Isometry2d::from_translation(at), radius, colour);
    }
}

/// Shows a resizing pointer wherever a press of the Bounds tool would drag the Bounds, and for
/// the whole of a drag of them, wherever the pointer goes.
pub(crate) fn pointer_icon(
    mut contexts: EguiContexts,
    state: Res<EditorState>,
    viewport: Res<Viewport>,
    window: Single<&Window, With<PrimaryWindow>>,
    level: LevelView,
) {
    if state.exporting || state.tool != Tool::Bounds {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let hovered = || {
        window
            .cursor_position()
            .filter(|cursor| viewport.contains(*cursor) && !ctx.is_pointer_over_egui())
            .zip(level.bounds())
            .and_then(|(cursor, bounds)| edges_at(bounds, viewport.cells_at(cursor), viewport.zoom))
    };
    let edges = if state.interaction == Interaction::Idle {
        hovered()
    } else {
        dragging(&state)
    };
    if let Some(edges) = edges {
        ctx.set_cursor_icon(edges.cursor());
    }
}

/// Sends the Bounds a field gives: while the field is held, as part of a gesture that ends when
/// it is let go, so a drag of it is one step; otherwise as a step of its own.
fn send_fields(
    tool: &mut BoundsTool,
    apply: &mut MessageWriter<Apply>,
    bounds: Bounds,
    held: bool,
) {
    let gesture = match (held, tool.fields) {
        (false, _) => Gesture::Single,
        (true, Some(_)) => Gesture::Continue,
        (true, None) => Gesture::Begin,
    };
    apply.write(Apply::ResizeBounds(ResizeBounds { bounds, gesture }));
    tool.fields = held.then_some(bounds);
}

/// Ends the fields' gesture once the field held is let go, or the strip stops showing the
/// fields, by sending the Bounds it last sent again as the gesture's end.
pub(crate) fn end_fields(state: &mut EditorState, apply: &mut MessageWriter<Apply>) {
    state.bounds.holding = false;
    if let Some(bounds) = state.bounds.fields.take() {
        apply.write(Apply::ResizeBounds(ResizeBounds {
            bounds,
            gesture: Gesture::End,
        }));
    }
}

/// The Bounds tool's fields in the tool strip: the left and bottom edges, the width, and the
/// height, in whole cells, as the Bounds are. A value typed is sent as the field lets go of the
/// keyboard, as one Resize Bounds keeping the other three, so a width or a height keeps the left
/// or the bottom edge and an edge moves the Bounds whole; a drag of a field changes it by whole
/// cells, stopped at the limits and starting from Bounds brought within them when they were opened
/// beyond them, as one gesture. A value out of the limits is sent as typed, and refused with the
/// reason. Nothing can be changed while an Export runs.
pub(crate) fn options(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    bounds: Option<Bounds>,
    apply: &mut MessageWriter<Apply>,
) {
    let Some(bounds) = bounds else {
        end_fields(state, apply);
        return;
    };
    let enabled = !state.exporting;
    let [left, bottom, right, top] = bounds.edges();
    let labels = ["Left", "Bottom", "Width", "Height"];
    let values = [left, bottom, right - left, top - bottom];
    let mut change = None;
    let mut holding = false;
    #[cfg(feature = "dev")]
    state.bounds.laid_out.clear();
    for (index, (label, value)) in labels.into_iter().zip(values).enumerate() {
        ui.label(label);
        let mut value = value;
        let field = ui.add_enabled(
            enabled,
            egui::DragValue::new(&mut value)
                .speed(FIELD_SPEED)
                .suffix(" cells")
                .update_while_editing(false),
        );
        #[cfg(feature = "dev")]
        state.bounds.laid_out.push((label, field.rect));
        holding |= field.is_pointer_button_down_on() || field.dragged();
        // Bounds opened beyond the limits are brought within them for a drag of a field, as a
        // drag of their edges does, so that what the drag sends is accepted.
        let from = if field.dragged() {
            within_limits(bounds, Edges::NONE)
        } else {
            bounds
        };
        if field.dragged() {
            value = dragged_field(from, index, value);
        }
        if field.changed() && value != values[index] {
            change = Some((typed(from, index, value), field.dragged()));
        }
    }
    state.bounds.holding = holding;
    match change {
        Some((changed, held)) => send_fields(&mut state.bounds, apply, changed, held),
        None if !holding => end_fields(state, apply),
        None => {}
    }
}

/// A field's value dragged to `value`, stopped where the Bounds would leave the limits: a width
/// or a height between one and a thousand cells and its far edge within reach of the origin, an
/// edge within reach with the Bounds' far edge too.
fn dragged_field(bounds: Bounds, field: usize, value: i64) -> i64 {
    let farthest = Bounds::FARTHEST;
    let [left, bottom, right, top] = bounds.edges();
    let (start, span) = match field {
        0 | 2 => (left, right - left),
        _ => (bottom, top - bottom),
    };
    if field < 2 {
        let span = span.min(2 * farthest);
        value.clamp(-farthest, farthest - span)
    } else {
        let most = i64::from(Bounds::MOST_SIDE).min(farthest - start);
        value.clamp(i64::from(Bounds::LEAST_SIDE), most.max(1))
    }
}

/// The Bounds with field `field` (left, bottom, width, height) set to `value` and the other three
/// kept, a value beyond what the Bounds' type holds saturating, so the Manager refuses it with the
/// limits named.
fn typed(bounds: Bounds, field: usize, value: i64) -> Bounds {
    let edge =
        |value: i64| i32::try_from(value).unwrap_or(if value < 0 { i32::MIN } else { i32::MAX });
    let side = |value: i64| u32::try_from(value).unwrap_or(if value < 0 { 0 } else { u32::MAX });
    let mut typed = bounds;
    match field {
        0 => typed.origin.x = edge(value),
        1 => typed.origin.y = edge(value),
        2 => typed.size.x = side(value),
        _ => typed.size.y = side(value),
    }
    typed
}

/// Logs the Bounds, whether the Bounds tool is chosen, the drag under way and the edges it drags,
/// and where each handle and field lies on screen, for a script to aim at them.
#[cfg(feature = "dev")]
pub(crate) fn describe(state: &EditorState, bounds: Option<Bounds>, viewport: &Viewport) {
    let drag = if let Interaction::Resizing { edges, from, drag } = state.interaction {
        Some((edges, from, drag.sent()))
    } else {
        None
    };
    bevy::log::info!(
        "describe: bounds {:?}, bounds tool {}, dragging {:?}, fields held {}",
        bounds,
        state.tool == Tool::Bounds,
        drag,
        state.bounds.option_in_progress()
    );
    let Some(bounds) = bounds else {
        return;
    };
    let corners = corners(bounds);
    for (handle, at) in handles::handles(&Outline::of_bounds(&corners)) {
        let screen = viewport.screen_at(at);
        bevy::log::info!(
            "describe: bounds handle {:?} moving {:?} at [{:.0} {:.0}]",
            handle,
            Edges::of(handle),
            screen.x,
            screen.y
        );
    }
    for (label, rect) in &state.bounds.laid_out {
        bevy::log::info!(
            "describe: bounds field {label} [{:.0} {:.0} {:.0} {:.0}]",
            rect.min.x,
            rect.min.y,
            rect.max.x,
            rect.max.y
        );
    }
}
