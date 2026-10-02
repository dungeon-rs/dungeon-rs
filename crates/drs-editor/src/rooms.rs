//! The Room tool and the selected Room: the outline being drawn point by point or as a rectangle
//! by a drag, its preview, closing it into one Place Element, picking a Room by its floor or its
//! Walls, and the Room's options in the tool strip.
//!
//! The outline being drawn is the Editor's own state until it is closed, when it becomes one
//! Place Element; every change to a placed Room is an Edit Element. A Room's handles are a
//! Wall's, through the outline the Wall tool's handles are seen by.

use crate::state::{EditorState, Interaction, Tool};
use crate::walls::{
    DEFAULT_COLOUR, DEFAULT_THICKNESS, HANDLE_PIXELS, HANDLES, NEAR_THE_LAST, OptionGesture,
    THICKEST, THINNEST_DRAGGED, end_option, on_wall, rgb, send_option,
};
use bevy::color::Color;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::query::With;
use bevy::ecs::system::{Res, Single};
use bevy::gizmos::gizmos::Gizmos;
use bevy::math::{Isometry2d, Vec2};
use bevy::window::{PrimaryWindow, Window};
use drs_model::{
    Apply, Colour, ElementChange, ElementId, LinePoint, PlaceElement, Placement, Room, RoomShape,
    Viewport,
};

/// The floor colour the first Room is drawn with: a light grey.
const DEFAULT_FLOOR: Colour = Colour::rgb(200, 200, 200);
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
    /// The option being changed as a gesture, if one is.
    option: Option<OptionGesture>,
}

impl Default for RoomTool {
    /// An eighth of a cell, the Wall tool's dark grey, and a light grey floor for the next Room.
    fn default() -> Self {
        Self {
            drawing: Vec::new(),
            thickness: DEFAULT_THICKNESS,
            wall_colour: DEFAULT_COLOUR,
            floor_colour: DEFAULT_FLOOR,
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
    state.walls.handle = None;
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

/// A left press with the Room tool: with no point placed it may begin a rectangle, which the
/// release tells from a click; with points placed it is a click.
pub(crate) fn press(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    layer: Option<Entity>,
    viewport: &Viewport,
    cursor: Vec2,
) {
    if state.rooms.drawing.is_empty() {
        state.interaction = Interaction::Outlining {
            pointer: cursor,
            moved_at: None,
        };
    } else {
        click(state, apply, layer, viewport, cursor);
    }
}

/// A click with the Room tool: within a few pixels of the first point it closes the outline once
/// three or more points are placed and does nothing before; within a few pixels of the last
/// point it adds none; anywhere else it adds a point.
fn click(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    layer: Option<Entity>,
    viewport: &Viewport,
    cursor: Vec2,
) {
    let cells = viewport.cells_at(cursor);
    let near = |point: &Vec2, pixels: f32| point.distance(cells) * viewport.zoom <= pixels;
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
    if let Interaction::Outlining { pointer, moved_at } = state.interaction
        && (moved_at.is_some() || (cursor - pointer).length() > threshold)
    {
        state.interaction = Interaction::Outlining {
            pointer,
            moved_at: Some(cursor),
        };
    }
}

/// Ends a press that may have begun a rectangle once its button is up: a drag sends one Place
/// Element of a Room of the rectangle's four corners, from its lower-left corner
/// counter-clockwise, unless it is less than a few pixels wide or high; a press that never
/// became a drag is a click, which adds the first point.
pub(crate) fn release(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    layer: Option<Entity>,
    viewport: &Viewport,
) {
    let Interaction::Outlining { pointer, moved_at } = state.interaction else {
        return;
    };
    state.interaction = Interaction::Idle;
    let Some(released) = moved_at else {
        click(state, apply, layer, viewport, pointer);
        return;
    };
    let across = (released - pointer).abs();
    if across.x < SMALLEST_RECTANGLE || across.y < SMALLEST_RECTANGLE {
        return;
    }
    let (from, to) = (viewport.cells_at(pointer), viewport.cells_at(released));
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

/// Whether a point in cells is on a Room: inside its floor, by the non-zero winding of its
/// closed line around the point, or on its Walls outside the stretches its Portals cover.
pub(crate) fn on_room(room: &Room, shape: &RoomShape, cells: Vec2, zoom: f32) -> bool {
    winds_around(&shape.walls.line, cells) || on_wall(room.thickness, &shape.walls, cells, zoom)
}

/// The wall thickness, wall colour, and floor colour options, of the selected Room or of the
/// next one, a change to a selected Room sent as one Edit Element and a held widget as one
/// gesture.
pub(crate) fn options(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    selected: Option<&(ElementId, Room)>,
    apply: &mut MessageWriter<Apply>,
) {
    let (mut thickness, walls, floor) = selected.map_or(
        (
            state.rooms.thickness,
            state.rooms.wall_colour,
            state.rooms.floor_colour,
        ),
        |(_, room)| (room.thickness, room.wall_colour, room.floor_colour),
    );
    ui.label("Thickness");
    let drag = ui.add(
        egui::DragValue::new(&mut thickness)
            .range(f32::NEG_INFINITY..=THICKEST)
            .clamp_existing_to_range(false)
            .speed(0.005)
            .max_decimals(3)
            .suffix(" cells")
            .update_while_editing(false),
    );
    if drag.dragged() {
        thickness = thickness.max(THINNEST_DRAGGED);
    }
    let picker = |ui: &mut egui::Ui, label: &str, colour: Colour| {
        ui.label(label);
        // The colour button's popup takes the id the button's own next id is salted with.
        let popup = ui.auto_id_with("popup");
        let mut channels = rgb(colour);
        egui::color_picker::color_edit_button_srgb(ui, &mut channels);
        (
            Colour::rgb(channels[0], channels[1], channels[2]),
            egui::Popup::is_id_open(ui.ctx(), popup),
        )
    };
    let (picked_walls, picking_walls) = picker(ui, "Walls", walls);
    let (picked_floor, picking_floor) = picker(ui, "Floor", floor);

    let Some((element, room)) = selected else {
        // A thickness no Room could have leaves the next Room's as it was.
        if thickness > 0.0 && thickness.is_finite() {
            state.rooms.thickness = thickness;
        }
        state.rooms.wall_colour = picked_walls;
        state.rooms.floor_colour = picked_floor;
        end_option(&mut state.rooms.option, apply);
        return;
    };
    if drag.changed() {
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

/// Draws what the Room tool shows over the Level: the outline being drawn as a thin line through
/// its points with rubber bands from the last point to the pointer and from the pointer to the
/// first, and the rectangle being dragged. Nothing is drawn while an Export runs.
pub(crate) fn draw_overlays(
    mut gizmos: Gizmos,
    state: Res<EditorState>,
    viewport: Res<Viewport>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if state.exporting || state.tool != Tool::Room {
        return;
    }
    let pointer = window
        .cursor_position()
        .filter(|cursor| viewport.contains(*cursor));
    let colour = Color::srgb_u8(
        state.rooms.wall_colour.red,
        state.rooms.wall_colour.green,
        state.rooms.wall_colour.blue,
    );
    if state.rooms.drawing_in_progress() {
        let radius = HANDLE_PIXELS / viewport.zoom;
        let at = pointer.map(|cursor| viewport.cells_at(cursor));
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
        pointer: from,
        moved_at: Some(dragged),
    } = state.interaction
    {
        let to = pointer.unwrap_or(dragged);
        let (from, to) = (viewport.cells_at(from), viewport.cells_at(to));
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
        "describe: room tool {}, drawing {:?}, next thickness {} walls {:?} floor {:?}",
        state.tool == Tool::Room,
        state.rooms.drawing,
        state.rooms.thickness,
        rgb(state.rooms.wall_colour),
        rgb(state.rooms.floor_colour)
    );
    for (id, element, room, shape) in rooms {
        bevy::log::info!(
            "describe: room {} at {} size {}, points {:?}, controls {:?}, thickness {}, walls \
             {:?}, floor {:?}, gives way along {:?}",
            id.as_raw(),
            element.position,
            element.size,
            room.points,
            room.edges
                .iter()
                .map(|edge| edge.control)
                .collect::<Vec<_>>(),
            room.thickness,
            rgb(room.wall_colour),
            rgb(room.floor_colour),
            shape.map(|shape| {
                shape
                    .walls
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
