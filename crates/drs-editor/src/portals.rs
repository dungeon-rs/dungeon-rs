//! The Portal tool and the selected Portal: finding the Wall or the Room's Walls under the
//! pointer, the nearest within reach, and the point of its line nearest the pointer, which is hit-testing as picking
//! is, showing them with its marker, placing a Portal set into a Wall or freestanding, picking a Portal by its turned
//! rectangle, sliding a set Portal along its Wall, flipping, freeing, and setting it, and its
//! options in the tool strip.
//!
//! Every change is a Command: a click places with one Place Element, a slide is an Edit Element
//! gesture, and `F` and `X` send one Command each.

use crate::state::{EditorState, Interaction, Tool};
use crate::viewport::LevelView;
use crate::walls::{NearestPoint, OptionGesture, nearest_on_line};
use bevy::color::Color;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::query::With;
use bevy::ecs::system::{Res, Single};
use bevy::gizmos::gizmos::Gizmos;
use bevy::math::{Isometry2d, Rot2, Vec2, ops};
use bevy::window::{PrimaryWindow, Window};
use drs_model::{
    Apply, EditElement, Element, ElementChange, ElementId, FreePortal, Gesture, PlaceElement,
    Placement, Portal, PortalAnchor, SetPortalIntoWall, Side, Viewport, WallShape,
};

/// How far from a Wall's line, in cells, the Portal tool reaches it however thin the Wall: half a
/// cell.
const HALF_A_CELL: f32 = 0.5;
/// How long the marker's line across the Wall is, in screen pixels beyond the Wall's thickness.
const MARKER_PIXELS: f32 = 12.0;
/// How long the marker's arrowhead is, in screen pixels.
const ARROWHEAD_PIXELS: f32 = 6.0;
/// The colour of the marker.
const MARKER: Color = Color::srgb(1.0, 0.85, 0.2);
/// The slowest a dragged width changes, in cells per pixel.
const WIDTH_SPEED: f32 = 0.01;
/// The narrowest a dragged width reaches, in cells; a typed width is sent as typed, so one not
/// above zero is refused with the reason.
const NARROWEST_DRAGGED: f32 = 0.05;

/// The Portal tool's own state.
#[derive(Debug, Default)]
pub(crate) struct PortalTool {
    /// The option being changed as a gesture, if one is.
    pub(crate) option: Option<OptionGesture>,
}

impl PortalTool {
    /// Whether an option is being changed as a gesture: the width or the rotation is being
    /// dragged.
    pub(crate) fn option_in_progress(&self) -> bool {
        self.option.is_some()
    }
}

/// The Wall under the pointer as the Portal tool finds it: the nearest point of its line within
/// the tool's reach, and where a Portal would be set there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LineUnderPointer {
    /// Where the Portal would be set.
    pub anchor: PortalAnchor,
    /// The point on the line, in cells.
    pub at: Vec2,
    /// The unit direction of the line's chord there.
    pub along: Vec2,
    /// How far the pointer is from the line, in cells.
    pub distance: f32,
    /// How thick the Wall is, in cells.
    pub thickness: f32,
}

/// The side of a chord running along `along` through `at` that `cells` lies on.
fn side_of(along: Vec2, at: Vec2, cells: Vec2) -> Side {
    if along.perp_dot(cells - at) >= 0.0 {
        Side::Left
    } else {
        Side::Right
    }
}

/// The nearest Wall or Room within reach of `cells` of those given bottom first, each with its
/// thickness and the shape of its line, the topmost winning a tie: no farther from its line than
/// half a cell or half its thickness, whichever is more. The side is the side of the line `cells`
/// lies on, or `side` when given.
pub(crate) fn line_under<'a>(
    lines: impl IntoIterator<Item = (ElementId, f32, &'a WallShape)>,
    cells: Vec2,
    side: Option<Side>,
) -> Option<LineUnderPointer> {
    let mut best: Option<LineUnderPointer> = None;
    for (host, thickness, shape) in lines {
        let Some(NearestPoint {
            distance,
            place,
            at,
            along,
        }) = nearest_on_line(shape, cells)
        else {
            continue;
        };
        if distance > HALF_A_CELL.max(thickness / 2.0) {
            continue;
        }
        if best.is_some_and(|best| distance > best.distance) {
            continue;
        }
        best = Some(LineUnderPointer {
            anchor: PortalAnchor {
                host,
                index: place.segment,
                t: place.t.clamp(0.0, 1.0),
                side: side.unwrap_or_else(|| side_of(along, at, cells)),
            },
            at,
            along,
            distance,
            thickness,
        });
    }
    best
}

/// Chooses the Portal tool: the Wall, the Room, or the Paint tool is left, discarding a Wall, an
/// outline, or a stroke being drawn, the selection is dropped, and a chosen Asset is kept as the
/// Portal's image.
pub(crate) fn choose_portal_tool(state: &mut EditorState) {
    state.walls.drawing.clear();
    crate::paint::discard_stroke(state);
    state.rooms.drawing.clear();
    state.selected = None;
    state.handle = None;
    state.tool = Tool::Portal;
}

/// A click with the Portal tool: with no Asset chosen it asks for one in the status line; with a
/// Wall under the pointer it places a Portal set into it at the nearest point of its line, and
/// elsewhere one freestanding, unturned and centred on the click. The tool stays chosen.
pub(crate) fn place_click(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    layer: Option<Entity>,
    under: Option<LineUnderPointer>,
    cells: Vec2,
) {
    let Some(chosen) = &state.chosen else {
        "Choose an Asset in the browser to place it as a Portal".clone_into(&mut state.status);
        return;
    };
    let Some(layer) = layer else {
        return;
    };
    apply.write(Apply::PlaceElement(PlaceElement {
        layer,
        placement: Placement::Portal {
            position: cells,
            asset: chosen.asset.clone(),
            anchor: under.map(|under| under.anchor),
        },
    }));
}

/// Whether a point in cells lies within a Portal's turned rectangle.
pub(crate) fn on_portal(element: &Element, portal: &Portal, cells: Vec2) -> bool {
    let (sine, cosine) = ops::sin_cos(-portal.rotation);
    let offset = cells - element.position;
    let local = Vec2::new(
        offset.x * cosine - offset.y * sine,
        offset.x * sine + offset.y * cosine,
    );
    let half = element.size / 2.0;
    local.x.abs() <= half.x && local.y.abs() <= half.y
}

/// The Edit Element of a slide of a set Portal to the nearest point of its own Wall's line to
/// `cells`.
pub(crate) fn slide(
    element: ElementId,
    shape: &WallShape,
    cells: Vec2,
    gesture: Gesture,
) -> Option<Apply> {
    let place = nearest_on_line(shape, cells)?.place;
    Some(Apply::EditElement(EditElement {
        element,
        change: ElementChange::Along {
            segment: place.segment,
            t: place.t.clamp(0.0, 1.0),
        },
        gesture,
    }))
}

/// `X`: flips the side a Portal that `follows` its Wall faces, or the mirroring of any other,
/// freestanding or lost.
pub(crate) fn flip(element: ElementId, portal: &Portal, follows: bool) -> Apply {
    let change = match portal.anchor {
        Some(anchor) if follows => ElementChange::Side(anchor.side.flipped()),
        Some(_) | None => ElementChange::Mirrored(!portal.mirrored),
    };
    Apply::EditElement(EditElement {
        element,
        change,
        gesture: Gesture::Single,
    })
}

/// `F`: frees a set Portal, or sets a freestanding one into the nearest Wall or Room within reach
/// of its centre, at the nearest point on that line and facing the right when it is mirrored or
/// the left when it is not. With no Wall within reach the status line says so and nothing is
/// sent.
pub(crate) fn free_or_set<'a>(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    element: ElementId,
    centre: Vec2,
    portal: &Portal,
    lines: impl IntoIterator<Item = (ElementId, f32, &'a WallShape)>,
) {
    if portal.anchor.is_some() {
        apply.write(Apply::FreePortal(FreePortal { portal: element }));
        return;
    }
    match line_under(lines, centre, Some(Side::of_mirroring(portal.mirrored))) {
        Some(under) => {
            apply.write(Apply::SetPortalIntoWall(SetPortalIntoWall {
                portal: element,
                anchor: under.anchor,
            }));
        }
        None => {
            "No Wall or Room is within reach of the Portal to set it into"
                .clone_into(&mut state.status);
        }
    }
}

/// Starts a press on a Portal set into a Wall: a drag slides it along its Wall.
pub(crate) fn press_set(state: &mut EditorState, element: ElementId, cursor: Vec2) {
    state.interaction = Interaction::Sliding {
        element,
        pointer: cursor,
        moved_at: None,
    };
}

/// The options of the selected Portal in the tool strip: its width, its rotation in degrees
/// unless it `follows` its Wall, the flip button, and the Free Portal or Set into Wall button, each change
/// sent as one step, a drag of the width or the rotation as one gesture.
pub(crate) fn options<'a>(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    selected: (ElementId, &Element, &Portal, bool),
    lines: impl IntoIterator<Item = (ElementId, f32, &'a WallShape)>,
    apply: &mut MessageWriter<Apply>,
) {
    let (id, element, portal, follows) = selected;
    ui.label("Width");
    let mut width = portal.width;
    let drag = ui.add(
        egui::DragValue::new(&mut width)
            .speed(WIDTH_SPEED)
            .max_decimals(3)
            .suffix(" cells")
            .update_while_editing(false),
    );
    if drag.dragged() {
        width = width.max(NARROWEST_DRAGGED);
    }
    let mut held = drag.dragged();
    let mut change = drag.changed().then_some(ElementChange::Width(width));
    if !follows {
        ui.label("Rotation");
        let mut degrees = portal.rotation.to_degrees();
        let turn = ui.add(
            egui::DragValue::new(&mut degrees)
                .speed(1.0)
                .max_decimals(1)
                .suffix("°")
                .update_while_editing(false),
        );
        if turn.changed() {
            change = Some(ElementChange::Rotation(degrees.to_radians()));
            held = turn.dragged();
        } else {
            held |= turn.dragged();
        }
    }
    if let Some(change) = change {
        crate::walls::send_option(&mut state.portals.option, apply, id, change, held);
    } else if !held {
        crate::walls::end_option(&mut state.portals.option, apply);
    }
    if ui.button("Flip").on_hover_text("X").clicked() {
        apply.write(flip(id, portal, follows));
    }
    let label = if portal.anchor.is_some() {
        "Free Portal"
    } else {
        "Set into Wall"
    };
    if ui.button(label).on_hover_text("F").clicked() {
        free_or_set(state, apply, id, element.position, portal, lines);
    }
}

/// Ends a Portal option's gesture left open when its Portal is no longer shown in the strip.
pub(crate) fn end_options(state: &mut EditorState, apply: &mut MessageWriter<Apply>) {
    crate::walls::end_option(&mut state.portals.option, apply);
}

/// Draws the Portal tool's marker over the Level: a short line across the nearest Wall or Room
/// within reach of the pointer at the nearest point on its line, with an arrowhead to the side the
/// Portal will face. Nothing is drawn while an Export runs or the pointer is off the viewport.
pub(crate) fn draw_marker(
    mut gizmos: Gizmos,
    state: Res<EditorState>,
    viewport: Res<Viewport>,
    window: Single<&Window, With<PrimaryWindow>>,
    level: LevelView,
) {
    if state.exporting || state.tool != Tool::Portal {
        return;
    }
    let Some(cursor) = window
        .cursor_position()
        .filter(|cursor| viewport.contains(*cursor))
    else {
        return;
    };
    let cells = viewport.cells_at(cursor);
    let Some(under) = line_under(level.lines_in_order(), cells, None) else {
        return;
    };
    let across = match under.anchor.side {
        Side::Left => under.along.perp(),
        Side::Right => -under.along.perp(),
    };
    let reach = under.thickness / 2.0 + MARKER_PIXELS / viewport.zoom;
    gizmos.line_2d(under.at - across * reach, under.at, MARKER);
    gizmos
        .arrow_2d(under.at, under.at + across * reach, MARKER)
        .with_tip_length(ARROWHEAD_PIXELS / viewport.zoom);
}

/// The isometry a Portal's outline is drawn with: at its centre, turned by its rotation.
pub(crate) fn outline(element: &Element, portal: &Portal) -> Isometry2d {
    Isometry2d::new(element.position, Rot2::radians(portal.rotation))
}

/// Logs every Portal, for a script to check what placing, sliding, and Wall edits did.
#[cfg(feature = "dev")]
pub(crate) fn describe(
    state: &EditorState,
    portals: &bevy::ecs::system::Query<(&ElementId, &Element, &Portal)>,
    shapes: &bevy::ecs::system::Query<(&ElementId, &WallShape)>,
) {
    bevy::log::info!(
        "describe: portal tool {}, selected {:?}",
        state.tool == Tool::Portal,
        state.selected.map(ElementId::as_raw)
    );
    for (id, element, portal) in portals {
        bevy::log::info!(
            "describe: portal {} at {} size {}, rotation {}, mirrored {}, anchor {:?}",
            id.as_raw(),
            element.position,
            element.size,
            portal.rotation,
            portal.mirrored,
            portal
                .anchor
                .map(|anchor| (anchor.host.as_raw(), anchor.index, anchor.t, anchor.side))
        );
    }
    for (id, shape) in shapes {
        bevy::log::info!(
            "describe: wall {} gives way along {:?}",
            id.as_raw(),
            shape
                .stretches
                .iter()
                .map(|stretch| (
                    (stretch.start.segment, stretch.start.t),
                    (stretch.end.segment, stretch.end.t)
                ))
                .collect::<Vec<_>>()
        );
    }
}
