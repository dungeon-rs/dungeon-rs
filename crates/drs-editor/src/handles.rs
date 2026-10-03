//! The handles of a selected Wall, Room, or stroke: the points of its line or path, and a Wall's or
//! a Room's control points and the middles of its straight segments or edges, seen through one
//! view of its outline, hit before any Element, dragged as one gesture, and removed or
//! straightened with Delete.
//!
//! Every change a handle makes is an Edit Element; which handle is selected is the Editor's own
//! state.

use crate::state::EditorState;
use crate::walls::{nearest_on_line, on_wall};
use bevy::color::{Alpha, Color, ColorToPacked};
use bevy::ecs::message::MessageWriter;
use bevy::ecs::system::{Query, Res};
use bevy::gizmos::gizmos::Gizmos;
use bevy::math::{Isometry2d, Vec2};
use drs_model::{
    Apply, EditElement, ElementChange, ElementId, Gesture, Room, Stroke, Viewport, Wall, WallShape,
};

/// How close to a handle, in screen pixels, the pointer is on it; the size handles are drawn at.
pub(crate) const HANDLE_PIXELS: f32 = 6.0;
/// How far the pointer travels, in pixels, before a press on an Element, a handle, or a stroke
/// becomes a drag.
pub(crate) const DRAG_THRESHOLD: f32 = 3.0;
/// How wide a control point's square is drawn, against a point's radius.
const CONTROL_SIDE: f32 = 1.6;
/// How opaque the guide lines from a control point to its segment's or edge's points are drawn.
const GUIDE_ALPHA: f32 = 0.5;
/// The nearest a point added by a double-click comes to either end of its segment or edge, as a
/// parameter along it: a part is split strictly between its points, never at one.
const NEAREST_TO_AN_END: f32 = 0.001;
/// The colour of the handles and the guide lines.
pub(crate) const HANDLES: Color = Color::srgb(0.35, 0.75, 1.0);
/// The colour of the selected handle, or of the one being dragged.
pub(crate) const PICKED: Color = Color::srgb(1.0, 0.85, 0.2);

/// A handle of the selected Wall or Room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutlineHandle {
    /// The point of that number.
    Point(usize),
    /// The control point of the curved segment or edge of that number.
    Control(usize),
    /// The middle of the straight segment or edge of that number, which a drag bends.
    Middle(usize),
}

/// A Wall, a Room, or a stroke as its handles and its line are seen: its points, one control point
/// or none per segment or edge, whether its line closes from the last point back to the first, and
/// how thick it is drawn.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Outline<'a> {
    /// The points, in order.
    pub points: &'a [Vec2],
    /// The control point of each segment or edge, or `None` for a straight one.
    pub controls: Vec<Option<Vec2>>,
    /// Whether the last edge runs from the last point back to the first, as a Room's does.
    pub closed: bool,
    /// How thick it is drawn, in cells.
    pub thickness: f32,
}

impl<'a> Outline<'a> {
    /// A Wall's outline: open, a segment between each point and the next.
    pub(crate) fn of_wall(wall: &'a Wall) -> Self {
        Self {
            points: &wall.points,
            controls: wall
                .segments
                .iter()
                .map(|segment| segment.control)
                .collect(),
            closed: false,
            thickness: wall.thickness,
        }
    }

    /// A Room's outline: closed, an edge from each point to the next and from the last back to
    /// the first.
    pub(crate) fn of_room(room: &'a Room) -> Self {
        Self {
            points: &room.points,
            controls: room.edges.iter().map(|edge| edge.control).collect(),
            closed: true,
            thickness: room.thickness,
        }
    }

    /// A stroke's path: open, with no control points, so its only handles are its points.
    pub(crate) fn of_stroke(stroke: &'a Stroke) -> Self {
        Self {
            points: &stroke.points,
            controls: Vec::new(),
            closed: false,
            thickness: stroke.brush.size,
        }
    }

    /// The two points the segment or edge `part` runs between, if it has one.
    pub(crate) fn ends(&self, part: usize) -> Option<(Vec2, Vec2)> {
        let start = *self.points.get(part)?;
        let next = if self.closed && part + 1 == self.points.len() {
            0
        } else {
            part + 1
        };
        Some((start, *self.points.get(next)?))
    }
}

/// The handles of a Wall, a Room, or a stroke in the order they are hit: its points, then its
/// control points, then the middles of its straight segments or edges, the closing edge's
/// included.
pub(crate) fn handles(outline: &Outline) -> Vec<(OutlineHandle, Vec2)> {
    let points = outline
        .points
        .iter()
        .enumerate()
        .map(|(index, point)| (OutlineHandle::Point(index), *point));
    let controls = outline
        .controls
        .iter()
        .enumerate()
        .filter_map(|(index, control)| Some((OutlineHandle::Control(index), (*control)?)));
    let middles = outline
        .controls
        .iter()
        .enumerate()
        .filter(|(_, control)| control.is_none())
        .filter_map(|(index, _)| {
            let (start, end) = outline.ends(index)?;
            Some((OutlineHandle::Middle(index), start.midpoint(end)))
        });
    points.chain(controls).chain(middles).collect()
}

/// The handle of a Wall, a Room, or a stroke within a handle's reach of a point in cells at
/// `zoom`: of the first kind in the order [`handles`] lists them, points before control points
/// before middles, the nearest of that kind.
pub(crate) fn handle_at(
    outline: &Outline,
    cells: Vec2,
    zoom: f32,
) -> Option<(OutlineHandle, Vec2)> {
    let kind = |handle: OutlineHandle| match handle {
        OutlineHandle::Point(_) => 0,
        OutlineHandle::Control(_) => 1,
        OutlineHandle::Middle(_) => 2,
    };
    handles(outline)
        .into_iter()
        .filter(|(_, at)| at.distance(cells) * zoom <= HANDLE_PIXELS)
        .min_by(|(a, at_a), (b, at_b)| {
            kind(*a)
                .cmp(&kind(*b))
                .then(at_a.distance(cells).total_cmp(&at_b.distance(cells)))
        })
}

/// Where a press stands with the pointer at `cursor`, the pointer having gone down at `pointer`
/// and last moved what it drags at `moved_at`: `None` while it is still a click or the pointer has
/// not moved since, the gesture's Begin once the pointer first travels past [`DRAG_THRESHOLD`],
/// and Continue after, so a drag from press to release is one step.
pub(crate) fn drag_gesture(pointer: Vec2, moved_at: Option<Vec2>, cursor: Vec2) -> Option<Gesture> {
    match moved_at {
        Some(last) => (last != cursor).then_some(Gesture::Continue),
        None => ((cursor - pointer).length() > DRAG_THRESHOLD).then_some(Gesture::Begin),
    }
}

/// A colour of the handles as egui spells it, for handles drawn over the viewport by egui.
pub(crate) fn on_egui(colour: Color) -> egui::Color32 {
    let [red, green, blue, alpha] = colour.to_srgba().to_u8_array();
    egui::Color32::from_rgba_unmultiplied(red, green, blue, alpha)
}

/// A left press with the Select tool on the selected Wall or Room, which is hit before any
/// Element: a double-click on its line adds a point at the nearest place on it, and a press on a
/// handle arms a drag of it and selects it, unless it is a straight segment's or edge's middle,
/// which selects nothing more than the Element. Returns whether the press was the selection's.
pub(crate) fn press_selected(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    selected: Option<(ElementId, Outline, Option<&WallShape>)>,
    viewport: &Viewport,
    cursor: Vec2,
    double: bool,
) -> bool {
    let Some((element, outline, shape)) = selected else {
        return false;
    };
    let cells = viewport.cells_at(cursor);
    let handle = handle_at(&outline, cells, viewport.zoom);
    if double
        && !matches!(
            handle,
            Some((OutlineHandle::Point(_) | OutlineHandle::Control(_), _))
        )
        && let Some(shape) = shape
        && on_wall(outline.thickness, shape, cells, viewport.zoom)
        && let Some(nearest) = nearest_on_line(shape, cells)
    {
        apply.write(Apply::EditElement(EditElement {
            element,
            change: ElementChange::AddPoint {
                segment: nearest.place.segment,
                t: nearest
                    .place
                    .t
                    .clamp(NEAREST_TO_AN_END, 1.0 - NEAREST_TO_AN_END),
            },
            gesture: Gesture::Single,
        }));
        state.handle = None;
        return true;
    }
    let Some((handle, origin)) = handle else {
        return false;
    };
    // A straight part's middle is only for dragging: a click on it leaves the Wall or the Room
    // selected with no handle, so Delete removes it rather than doing nothing.
    state.handle = match handle {
        OutlineHandle::Point(_) | OutlineHandle::Control(_) => Some((element, handle)),
        OutlineHandle::Middle(_) => None,
    };
    state.interaction = crate::state::Interaction::Handle {
        element,
        handle,
        origin,
        pointer: cursor,
        moved_at: None,
    };
    true
}

/// The change that puts a handle at a position: a point moves, and a control point or a
/// straight part's middle becomes the part's control point.
pub(crate) fn handle_change(handle: OutlineHandle, position: Vec2) -> ElementChange {
    match handle {
        OutlineHandle::Point(index) => ElementChange::Point { index, position },
        OutlineHandle::Control(segment) | OutlineHandle::Middle(segment) => {
            ElementChange::Control {
                segment,
                position: Some(position),
            }
        }
    }
}

/// The Edit Element Delete sends for the selected handle of a Wall or a Room: the point is
/// removed, or the curved segment or edge of the control point is made straight; `None` when
/// there is nothing to do.
pub(crate) fn delete_handle(outline: &Outline, handle: OutlineHandle) -> Option<ElementChange> {
    match handle {
        OutlineHandle::Point(index) => {
            (index < outline.points.len()).then_some(ElementChange::RemovePoint { index })
        }
        OutlineHandle::Control(segment) => {
            outline
                .controls
                .get(segment)
                .copied()
                .flatten()
                .map(|_| ElementChange::Control {
                    segment,
                    position: None,
                })
        }
        OutlineHandle::Middle(_) => None,
    }
}

/// Whether a handle still names a part of the Wall or the Room: a point it has, or a segment or
/// edge that is curved for a control point and straight for a middle.
pub(crate) fn handle_exists(outline: &Outline, handle: OutlineHandle) -> bool {
    match handle {
        OutlineHandle::Point(index) => index < outline.points.len(),
        OutlineHandle::Control(segment) => {
            outline.controls.get(segment).is_some_and(Option::is_some)
        }
        OutlineHandle::Middle(segment) => {
            outline.controls.get(segment).is_some_and(Option::is_none)
        }
    }
}

/// Draws the handles of the selected Wall or Room over the Level, its control points with guide
/// lines to their segment's or edge's points, the selected handle in its own colour. Nothing is
/// drawn while an Export runs.
pub(crate) fn draw(
    mut gizmos: Gizmos,
    state: Res<EditorState>,
    viewport: Res<Viewport>,
    outlines: Query<(&ElementId, Option<&Wall>, Option<&Room>)>,
) {
    if state.exporting {
        return;
    }
    let radius = HANDLE_PIXELS / viewport.zoom;
    let Some((id, wall, room)) = state
        .selected
        .and_then(|selected| outlines.iter().find(|(id, ..)| **id == selected))
    else {
        return;
    };
    let Some(outline) = wall
        .map(Outline::of_wall)
        .or_else(|| room.map(Outline::of_room))
    else {
        return;
    };
    let picked = state.handle_of(*id);
    for (handle, at) in handles(&outline) {
        let colour = if picked == Some(handle) {
            PICKED
        } else {
            HANDLES
        };
        match handle {
            OutlineHandle::Point(_) => {
                gizmos.circle_2d(Isometry2d::from_translation(at), radius, colour);
            }
            OutlineHandle::Control(segment) => {
                if let Some((start, end)) = outline.ends(segment) {
                    for end in [start, end] {
                        gizmos.line_2d(end, at, HANDLES.with_alpha(GUIDE_ALPHA));
                    }
                }
                gizmos.rect_2d(
                    Isometry2d::from_translation(at),
                    Vec2::splat(radius * CONTROL_SIDE),
                    colour,
                );
            }
            OutlineHandle::Middle(_) => {
                gizmos.circle_2d(Isometry2d::from_translation(at), radius / 2.0, colour);
            }
        }
    }
}
