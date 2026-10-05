//! The Room tool and the selected Room: the outline being drawn point by point or as a rectangle
//! by a drag, its preview, closing it into one Place Element, picking a Room by its floor or its
//! Walls, and the Room's options in the tool strip.
//!
//! The outline being drawn is the Editor's own state until it is closed, when it becomes one
//! Place Element; every change to a placed Room is an Edit Element. A Room's handles are a
//! Wall's, through the outline the Wall tool's handles are seen by.

use crate::handles::{HANDLE_PIXELS, HANDLES};
use crate::snapping::Shown;
use crate::state::{EditorState, Interaction, Tool};
use crate::viewport::LevelView;
use crate::walls::{
    DEFAULT_COLOUR, DEFAULT_THICKNESS, Lines, NEAR_THE_LAST, OptionGesture, colour_option,
    end_option, on_wall, send_option, thickness_option,
};
use bevy::color::Color;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::query::With;
use bevy::ecs::system::{Query, Res, Single};
use bevy::gizmos::gizmos::Gizmos;
use bevy::math::{Isometry2d, Vec2};
use bevy::window::{PrimaryWindow, Window};
use drs_model::{
    Apply, Colour, EditElement, ElementChange, ElementId, Gesture, LinePoint, PlaceElement,
    Placement, Pointer, Room, RoomShape, SnappedPoint, Viewport,
};

/// The floor colour the first Room is drawn with: a light grey.
const DEFAULT_FLOOR: Colour = Colour::rgb(200, 200, 200);
/// The colour of the thin guide line a Room that cuts is drawn with: an orange that no floor or
/// Wall is likely to share.
const CUT_GUIDE: Color = Color::srgba(1.0, 0.55, 0.1, 0.8);
/// How close to the first point, in screen pixels, a click closes the outline.
const NEAR_THE_FIRST: f32 = 4.0;
/// How wide and how high, in screen pixels, a dragged rectangle must be to place a Room.
const SMALLEST_RECTANGLE: f32 = 4.0;

/// The Room tool's state.
#[derive(Debug)]
pub(crate) struct RoomTool {
    /// The points of the outline being drawn, in cells.
    pub drawing: Vec<Vec2>,
    /// The wall thickness the next Room is drawn with, in cells.
    pub thickness: f32,
    /// The wall colour the next Room is drawn with.
    pub wall_colour: Colour,
    /// The floor colour the next Room is drawn with.
    pub floor_colour: Colour,
    /// Whether the next Room cuts.
    pub cuts: bool,
    /// The option being changed as a gesture, if one is.
    option: Option<OptionGesture>,
}

impl Default for RoomTool {
    /// An eighth of a cell, the Wall tool's dark grey, and a light grey floor for the next Room,
    /// which does not cut.
    fn default() -> Self {
        Self {
            drawing: Vec::new(),
            thickness: DEFAULT_THICKNESS,
            wall_colour: DEFAULT_COLOUR,
            floor_colour: DEFAULT_FLOOR,
            cuts: false,
            option: None,
        }
    }
}

impl RoomTool {
    /// Whether an outline is being drawn.
    pub(crate) fn drawing_in_progress(&self) -> bool {
        !self.drawing.is_empty()
    }

    /// Whether an option is being changed as a gesture: the thickness is being dragged, or a
    /// colour was picked while its picker is open.
    pub(crate) fn option_in_progress(&self) -> bool {
        self.option.is_some()
    }
}

/// Chooses the Room tool: the chosen Asset and the selection are dropped, and the Wall, the
/// Portal, or the Paint tool is left, discarding a Wall or a stroke being drawn.
pub(crate) fn choose_room_tool(state: &mut EditorState) {
    crate::paint::discard_stroke(state);
    state.chosen = None;
    state.selected = None;
    state.handle = None;
    state.walls.drawing.clear();
    state.tool = Tool::Room;
}

/// Sends one Place Element of a Room through `points` on `layer` with the tool's options.
fn place(
    state: &EditorState,
    apply: &mut MessageWriter<Apply>,
    layer: Option<Entity>,
    points: Vec<Vec2>,
) {
    if let Some(layer) = layer {
        apply.write(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Room {
                points,
                thickness: state.rooms.thickness,
                wall_colour: state.rooms.wall_colour,
                floor_colour: state.rooms.floor_colour,
                cuts: state.rooms.cuts,
            },
        }));
    }
}

/// Closes the outline being drawn: three or more points are sent as one Place Element on
/// `layer`; fewer are kept, and nothing is sent. The tool stays chosen for the next Room.
pub(crate) fn close(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    layer: Option<Entity>,
) {
    if state.rooms.drawing.len() < 3 {
        return;
    }
    let points = std::mem::take(&mut state.rooms.drawing);
    place(state, apply, layer, points);
}

/// A left press with the Room tool, the pointer at `cursor` on screen and snapping putting it at
/// `cells`, seen at `zoom`: with no point placed it may begin a rectangle, which the release
/// tells from a click; with points placed it is a click.
pub(crate) fn press(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    layer: Option<Entity>,
    zoom: f32,
    (cursor, cells): (Vec2, Vec2),
) {
    if state.rooms.drawing.is_empty() {
        state.interaction = Interaction::Outlining {
            pointer: cursor,
            from: cells,
            moved_at: None,
        };
    } else {
        click(state, apply, layer, zoom, cells);
    }
}

/// A click with the Room tool at `cells`, where snapping put the pointer, seen at `zoom`: within
/// a few pixels of the first point it closes the outline once three or more points are placed
/// and does nothing before; within a few pixels of the last point it adds none; anywhere else it
/// adds the point.
fn click(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    layer: Option<Entity>,
    zoom: f32,
    cells: Vec2,
) {
    let near = |point: &Vec2, pixels: f32| point.distance(cells) * zoom <= pixels;
    if state
        .rooms
        .drawing
        .first()
        .is_some_and(|first| near(first, NEAR_THE_FIRST))
    {
        close(state, apply, layer);
        return;
    }
    if !state
        .rooms
        .drawing
        .last()
        .is_some_and(|last| near(last, NEAR_THE_LAST))
    {
        state.rooms.drawing.push(cells);
    }
}

/// Follows the pointer while the button that may begin a rectangle is held: once it has
/// travelled farther than `threshold` pixels, the press is a drag.
pub(crate) fn track(state: &mut EditorState, cursor: Vec2, threshold: f32) {
    if let Interaction::Outlining {
        pointer,
        from,
        moved_at,
    } = state.interaction
        && (moved_at.is_some() || (cursor - pointer).length() > threshold)
    {
        state.interaction = Interaction::Outlining {
            pointer,
            from,
            moved_at: Some(cursor),
        };
    }
}

/// Ends a press that may have begun a rectangle once its button is up: a drag sends one Place
/// Element of a Room of the rectangle's four corners, from its lower-left corner
/// counter-clockwise, between where snapping put the press and the release, unless those lie
/// less than a few pixels apart in either direction; its other two corners take one coordinate
/// from each. A press that never became a drag is a click, which adds the first point where the
/// press put it. A press whose tool was left before the release, by Escape, another tool, or a
/// chosen Asset, sends and adds nothing.
pub(crate) fn release(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    layer: Option<Entity>,
    viewport: &Viewport,
    shown: Shown,
) {
    let Interaction::Outlining { from, moved_at, .. } = state.interaction else {
        return;
    };
    state.interaction = Interaction::Idle;
    if state.tool != Tool::Room {
        return;
    }
    let Some(released) = moved_at else {
        click(state, apply, layer, viewport.zoom, from);
        return;
    };
    let to = shown.point_or(viewport.cells_at(released));
    let across = (to - from).abs() * viewport.zoom;
    if across.x < SMALLEST_RECTANGLE || across.y < SMALLEST_RECTANGLE {
        return;
    }
    let (low, high) = (from.min(to), from.max(to));
    place(
        state,
        apply,
        layer,
        vec![
            low,
            Vec2::new(high.x, low.y),
            high,
            Vec2::new(low.x, high.y),
        ],
    );
}

/// Whether `cells` lies inside a closed line, by the non-zero winding of the line around it.
fn winds_around(line: &[LinePoint], cells: Vec2) -> bool {
    let mut winding = 0_i32;
    for pair in line.windows(2) {
        let (a, b) = (pair[0].position, pair[1].position);
        let left = (b - a).perp_dot(cells - a);
        if a.y <= cells.y {
            if b.y > cells.y && left > 0.0 {
                winding += 1;
            }
        } else if b.y <= cells.y && left < 0.0 {
            winding -= 1;
        }
    }
    winding != 0
}

/// Whether a point in cells is on a Room's floor as it is drawn: inside the triangles of its
/// floor, or, for a Room that cuts and so has none, anywhere its whole outline winds around.
pub(crate) fn on_floor(room: &Room, shape: &RoomShape, cells: Vec2) -> bool {
    if room.cuts {
        return winds_around(&shape.outline, cells);
    }
    shape.floor.covers(cells)
}

/// Whether a point in cells is on the Walls drawn in a Room's look, outside the stretches the
/// Portals leave out of them.
pub(crate) fn on_walls(room: &Room, shape: &RoomShape, cells: Vec2, zoom: f32) -> bool {
    on_wall(room.thickness, &Lines::room_walls(shape), cells, zoom)
}

/// The wall thickness, wall colour, floor colour, and Cut options, of the selected Room or of the
/// next one, a change to a selected Room sent as one Edit Element and a held widget as one
/// gesture.
pub(crate) fn options(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    selected: Option<&(ElementId, Room)>,
    apply: &mut MessageWriter<Apply>,
) {
    let (thickness, walls, floor, cuts) = selected.map_or(
        (
            state.rooms.thickness,
            state.rooms.wall_colour,
            state.rooms.floor_colour,
            state.rooms.cuts,
        ),
        |(_, room)| {
            (
                room.thickness,
                room.wall_colour,
                room.floor_colour,
                room.cuts,
            )
        },
    );
    let (thickness, drag) = thickness_option(ui, thickness);
    let (picked_walls, picking_walls) = colour_option(ui, "Walls", walls);
    let (picked_floor, picking_floor) = colour_option(ui, "Floor", floor);
    let mut picked_cuts = cuts;
    ui.checkbox(&mut picked_cuts, "Cut")
        .on_hover_text("A Room that cuts takes floor away from the Rooms before it on its Layer");

    let Some((element, room)) = selected else {
        // A thickness no Room could have leaves the next Room's as it was.
        if thickness > 0.0 && thickness.is_finite() {
            state.rooms.thickness = thickness;
        }
        state.rooms.wall_colour = picked_walls;
        state.rooms.floor_colour = picked_floor;
        state.rooms.cuts = picked_cuts;
        end_option(&mut state.rooms.option, apply);
        return;
    };
    if picked_cuts != room.cuts {
        end_option(&mut state.rooms.option, apply);
        apply.write(Apply::EditElement(EditElement {
            element: *element,
            change: ElementChange::Cuts(picked_cuts),
            gesture: Gesture::Single,
        }));
    } else if drag.changed() {
        send_option(
            &mut state.rooms.option,
            apply,
            *element,
            ElementChange::Thickness(thickness),
            drag.dragged(),
        );
    } else if picked_walls != room.wall_colour {
        send_option(
            &mut state.rooms.option,
            apply,
            *element,
            ElementChange::Colour(picked_walls),
            picking_walls,
        );
    } else if picked_floor != room.floor_colour {
        send_option(
            &mut state.rooms.option,
            apply,
            *element,
            ElementChange::FloorColour(picked_floor),
            picking_floor,
        );
    } else if !drag.dragged() && !picking_walls && !picking_floor {
        end_option(&mut state.rooms.option, apply);
    }
}

/// Ends a Room option's gesture left open when its Room is no longer shown in the strip.
pub(crate) fn end_options(state: &mut EditorState, apply: &mut MessageWriter<Apply>) {
    end_option(&mut state.rooms.option, apply);
}

/// Draws every Room of the Level being worked on that cuts as a thin guide line along its whole
/// outline, whether or not it takes floor away, so a cut is found where nothing else shows it.
/// Nothing is drawn while an Export runs, so no Export holds it.
pub(crate) fn draw_cuts(
    mut gizmos: Gizmos,
    state: Res<EditorState>,
    level: LevelView,
    rooms: Query<(&ChildOf, &Room, &RoomShape)>,
) {
    if state.exporting {
        return;
    }
    let current = level.current_level();
    for (layer, room, shape) in &rooms {
        if room.cuts && level.level_of(layer.parent()) == current {
            gizmos.linestrip_2d(shape.outline.iter().map(|point| point.position), CUT_GUIDE);
        }
    }
}

/// Draws what the Room tool shows over the Level: the outline being drawn as a thin line through
/// its points with rubber bands from the last point to where the next click lands, the snapped
/// point or the pointer, and from there to the first, and the rectangle being dragged between
/// where snapping put its corners. Nothing is drawn while an Export runs.
pub(crate) fn draw_overlays(
    mut gizmos: Gizmos,
    state: Res<EditorState>,
    viewport: Res<Viewport>,
    window: Single<&Window, With<PrimaryWindow>>,
    pointer: Res<Pointer>,
    snapped: Res<SnappedPoint>,
) {
    if state.exporting || state.tool != Tool::Room {
        return;
    }
    let shown = Shown::of(&snapped, &pointer);
    let cursor = window
        .cursor_position()
        .filter(|cursor| viewport.contains(*cursor));
    let colour = Color::srgb_u8(
        state.rooms.wall_colour.red,
        state.rooms.wall_colour.green,
        state.rooms.wall_colour.blue,
    );
    if state.rooms.drawing_in_progress() {
        let radius = HANDLE_PIXELS / viewport.zoom;
        let at = cursor.map(|cursor| shown.point_or(viewport.cells_at(cursor)));
        let first = state.rooms.drawing.first().copied();
        gizmos.linestrip_2d(
            state.rooms.drawing.iter().copied().chain(at).chain(first),
            HANDLES,
        );
        for point in &state.rooms.drawing {
            gizmos.circle_2d(Isometry2d::from_translation(*point), radius / 2.0, colour);
        }
    }
    if let Interaction::Outlining {
        from,
        moved_at: Some(dragged),
        ..
    } = state.interaction
    {
        let to = shown.point_or(viewport.cells_at(cursor.unwrap_or(dragged)));
        gizmos.rect_2d(
            Isometry2d::from_translation(from.midpoint(to)),
            (to - from).abs(),
            HANDLES,
        );
    }
}

/// Logs the Room tool and every Room, for a script to check what drawing and editing did.
#[cfg(feature = "dev")]
pub(crate) fn describe(
    state: &EditorState,
    rooms: &bevy::ecs::system::Query<(&ElementId, &drs_model::Element, &Room, Option<&RoomShape>)>,
) {
    bevy::log::info!(
        "describe: room tool {}, drawing {:?}, next thickness {} walls {:?} floor {:?} cuts {}",
        state.tool == Tool::Room,
        state.rooms.drawing,
        state.rooms.thickness,
        crate::walls::rgb(state.rooms.wall_colour),
        crate::walls::rgb(state.rooms.floor_colour),
        state.rooms.cuts
    );
    for (id, element, room, shape) in rooms {
        let walls = shape.map(|shape| {
            shape
                .walls
                .iter()
                .map(|line| {
                    line.iter()
                        .map(|point| (point.position, point.segment))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        });
        bevy::log::info!(
            "describe: room {} at {} size {}, points {:?}, controls {:?}, thickness {}, walls \
             {:?}, floor {:?}, cuts {}, floor triangles {:?}, walls drawn at {:?}, walls along \
             {walls:?}, gives way along {:?}",
            id.as_raw(),
            element.position,
            element.size,
            room.points,
            room.edges
                .iter()
                .map(|edge| edge.control)
                .collect::<Vec<_>>(),
            room.thickness,
            crate::walls::rgb(room.wall_colour),
            crate::walls::rgb(room.floor_colour),
            room.cuts,
            shape.map(|shape| shape.floor.indices.len() / 3),
            shape.map(|shape| shape.drawn_at.as_raw()),
            shape.map(|shape| {
                shape
                    .stretches
                    .iter()
                    .map(|stretch| {
                        (
                            (stretch.start.segment, stretch.start.t),
                            (stretch.end.segment, stretch.end.t),
                        )
                    })
                    .collect::<Vec<_>>()
            })
        );
    }
}
