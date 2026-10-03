//! The Wall tool and the handles of a selected Wall or Room: the tool strip over the viewport
//! with its options, the Wall being drawn, picking a Wall or a Room's Walls by its line outside
//! the stretches its Portals cover, and the points, control points, and segment or edge middles
//! of either as handles.
//!
//! The Wall being drawn is the Editor's own state until it is finished, when it becomes one
//! Place Element; every change to a placed Wall is an Edit Element.

use crate::handles::{HANDLE_PIXELS, HANDLES};
use crate::snapping::{Shown, SnapSwitch};
use crate::state::{EditorState, Tool};
use crate::viewport::LevelView;
use crate::{portals, rooms};
use bevy::color::Color;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::query::With;
use bevy::ecs::system::{Res, ResMut, Single};
use bevy::gizmos::gizmos::Gizmos;
use bevy::math::{Isometry2d, Vec2};
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::EguiContexts;
use drs_model::{
    Apply, Colour, EditElement, ElementChange, ElementId, Gesture, LinePlace, PlaceElement,
    Placement, Pointer, SnappedPoint, Viewport, Wall, WallShape,
};

/// The thickness the first Wall is drawn with: an eighth of a cell.
pub(crate) const DEFAULT_THICKNESS: f32 = 0.125;
/// The colour the first Wall is drawn with: a dark grey.
pub(crate) const DEFAULT_COLOUR: Colour = Colour::rgb(64, 64, 64);
/// How far the tool strip keeps from the viewport's edges, in screen pixels; it wraps onto
/// further rows rather than growing past the viewport's right edge.
const MARGIN: f32 = 8.0;
/// How close to the last point, in screen pixels, a click adds no point.
pub(crate) const NEAR_THE_LAST: f32 = 4.0;
/// How soon after a click, in seconds, a second click is a double-click.
const DOUBLE_CLICK_SECONDS: f64 = 0.5;
/// How close to a click, in screen pixels, a second click is a double-click.
const DOUBLE_CLICK_PIXELS: f32 = 5.0;
/// How close to a line, in screen pixels, the pointer is on it however thin the Wall or the stroke
/// drawn along it.
pub(crate) const LINE_PIXELS: f32 = 4.0;
/// The thinnest Wall a drag of the thickness reaches, in cells; a typed thickness is sent as
/// typed, so one not above zero is refused with the reason.
const THINNEST_DRAGGED: f32 = 0.01;
/// The thickest Wall the options offer, dragged or typed, in cells.
const THICKEST: f32 = 16.0;

/// An option of the tool strip being changed as one gesture on the Element it shows.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OptionGesture {
    /// The Element whose option is being changed.
    element: ElementId,
    /// The change last sent.
    change: ElementChange,
}

/// The Wall tool's state.
#[derive(Debug)]
pub(crate) struct WallTool {
    /// The points of the Wall being drawn, in cells.
    pub drawing: Vec<Vec2>,
    /// The thickness the next Wall is drawn with, in cells.
    pub thickness: f32,
    /// The colour the next Wall is drawn with.
    pub colour: Colour,
    /// When and where the last click on the viewport went down, for double-clicks.
    last_click: Option<(f64, Vec2)>,
    /// The option being changed as a gesture, if one is.
    option: Option<OptionGesture>,
}

impl Default for WallTool {
    /// An eighth of a cell and a dark grey for the next Wall.
    fn default() -> Self {
        Self {
            drawing: Vec::new(),
            thickness: DEFAULT_THICKNESS,
            colour: DEFAULT_COLOUR,
            last_click: None,
            option: None,
        }
    }
}

impl WallTool {
    /// Whether a Wall is being drawn.
    pub(crate) fn drawing_in_progress(&self) -> bool {
        !self.drawing.is_empty()
    }

    /// Records a click going down at `now` and `cursor`, and says whether it is the second click
    /// of a double-click; a third click starts afresh.
    pub(crate) fn double_click(&mut self, now: f64, cursor: Vec2) -> bool {
        let double = self.last_click.is_some_and(|(at, where_)| {
            now - at <= DOUBLE_CLICK_SECONDS && where_.distance(cursor) <= DOUBLE_CLICK_PIXELS
        });
        self.last_click = if double { None } else { Some((now, cursor)) };
        double
    }

    /// Whether an option is being changed as a gesture: the thickness is being dragged, or a
    /// colour was picked while the picker is open.
    pub(crate) fn option_in_progress(&self) -> bool {
        self.option.is_some()
    }
}

/// Chooses the Wall tool: the Paint tool is left, discarding a stroke being drawn, the Room tool
/// is left, discarding an outline being drawn, and the chosen Asset and the selection are
/// dropped.
pub(crate) fn choose_wall_tool(state: &mut EditorState) {
    crate::paint::discard_stroke(state);
    state.chosen = None;
    state.selected = None;
    state.handle = None;
    state.rooms.drawing.clear();
    state.tool = Tool::Wall;
}

/// Goes back to the Select tool from the Wall, the Portal, or the Room tool, discarding a Wall or
/// an outline being drawn.
pub(crate) fn leave_tool(state: &mut EditorState) {
    state.tool = Tool::Select;
    state.walls.drawing.clear();
    state.rooms.drawing.clear();
}

/// Finishes the Wall being drawn: two or more points are sent as one Place Element on `layer`;
/// fewer are discarded without a Command. The tool stays chosen for the next Wall.
pub(crate) fn finish(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    layer: Option<Entity>,
) {
    let points = std::mem::take(&mut state.walls.drawing);
    if points.len() < 2 {
        return;
    }
    if let Some(layer) = layer {
        apply.write(Apply::PlaceElement(PlaceElement {
            layer,
            placement: Placement::Wall {
                points,
                thickness: state.walls.thickness,
                colour: state.walls.colour,
            },
        }));
    }
}

/// A click with the Wall tool at `cells`, where snapping put the pointer, seen at `zoom`: the
/// second click of a double-click finishes the Wall at the point its first click added; any other
/// click adds the point unless it lies within a few pixels of the last one.
pub(crate) fn draw_click(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    layer: Option<Entity>,
    zoom: f32,
    cells: Vec2,
    double: bool,
) {
    if double {
        finish(state, apply, layer);
        return;
    }
    let near_the_last = state
        .walls
        .drawing
        .last()
        .is_some_and(|last| last.distance(cells) * zoom <= NEAR_THE_LAST);
    if !near_the_last {
        state.walls.drawing.push(cells);
    }
}

/// The point of a Wall's line nearest a point in cells.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct NearestPoint {
    /// How far it is from the point, in cells.
    pub distance: f32,
    /// Where along the line it lies, interpolated along the nearest chord.
    pub place: LinePlace,
    /// The point itself, in cells.
    pub at: Vec2,
    /// The unit direction of the chord it lies on.
    pub along: Vec2,
}

/// Where along the segment from `from` to `to` the point nearest `cells` lies, from zero at `from`
/// to one at `to`; zero for a segment of no length.
pub(crate) fn parameter_on_segment(from: Vec2, to: Vec2, cells: Vec2) -> f32 {
    let along = to - from;
    if along.length_squared() > 0.0 {
        ((cells - from).dot(along) / along.length_squared()).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// The point of a Wall's flattened line nearest `cells`.
pub(crate) fn nearest_on_line(shape: &WallShape, cells: Vec2) -> Option<NearestPoint> {
    shape
        .line
        .windows(2)
        .map(|pair| {
            let (from, to) = (pair[0], pair[1]);
            let along = to.position - from.position;
            let s = parameter_on_segment(from.position, to.position, cells);
            let end = if to.segment == from.segment {
                to.t
            } else {
                1.0
            };
            let at = from.position + along * s;
            NearestPoint {
                distance: cells.distance(at),
                place: LinePlace {
                    segment: from.segment,
                    t: from.t + (end - from.t) * s,
                },
                at,
                along: along.normalize_or_zero(),
            }
        })
        .min_by(|a, b| a.distance.total_cmp(&b.distance))
}

/// Whether a point in cells is on a Wall, or on a Room's Walls, `thickness` thick: no farther
/// from its line than half the thickness or four screen pixels, whichever is more, where the
/// nearest point of the line lies in no stretch a Portal covers.
pub(crate) fn on_wall(thickness: f32, shape: &WallShape, cells: Vec2, zoom: f32) -> bool {
    let reach = (thickness / 2.0).max(LINE_PIXELS / zoom);
    nearest_on_line(shape, cells).is_some_and(|nearest| {
        nearest.distance <= reach
            && !shape
                .stretches
                .iter()
                .any(|stretch| stretch.covers(nearest.place))
    })
}

/// The colour of the model as egui spells it.
pub(crate) fn rgb(colour: Colour) -> [u8; 3] {
    [colour.red, colour.green, colour.blue]
}

/// Sends an option's change to the Element it shows: a change made while the widget is held (a
/// drag of the thickness, the colour picker open) is part of a gesture that ends when it is let
/// go, so it is one step; any other change is a step of its own.
pub(crate) fn send_option(
    option: &mut Option<OptionGesture>,
    apply: &mut MessageWriter<Apply>,
    element: ElementId,
    change: ElementChange,
    held: bool,
) {
    let gesture = match (held, option.as_ref()) {
        (false, _) => Gesture::Single,
        (true, Some(open)) if open.element == element => Gesture::Continue,
        (true, _) => Gesture::Begin,
    };
    apply.write(Apply::EditElement(EditElement {
        element,
        change: change.clone(),
        gesture,
    }));
    *option = held.then_some(OptionGesture { element, change });
}

/// Ends an option's gesture once its widget is let go, by sending its last change again as the
/// gesture's end.
pub(crate) fn end_option(option: &mut Option<OptionGesture>, apply: &mut MessageWriter<Apply>) {
    if let Some(OptionGesture { element, change }) = option.take() {
        apply.write(Apply::EditElement(EditElement {
            element,
            change,
            gesture: Gesture::End,
        }));
    }
}

/// The thickness option of the selected Wall or Room or of the next one, showing `thickness`:
/// what the Author leaves it at and the widget's response, held while it is dragged.
///
/// The range has no lower end, as egui clamps typed values into it too: a drag is kept above
/// zero here instead. A thickness already above the range shows as it is, since clamping what is
/// shown would send a change nobody made.
pub(crate) fn thickness_option(ui: &mut egui::Ui, thickness: f32) -> (f32, egui::Response) {
    let mut thickness = thickness;
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
    (thickness, drag)
}

/// A colour option labelled `label`, showing `colour`: what the Author leaves it at, and whether
/// its picker is open, which holds it.
pub(crate) fn colour_option(ui: &mut egui::Ui, label: &str, colour: Colour) -> (Colour, bool) {
    ui.label(label);
    // The colour button's popup takes the id the button's own next id is salted with.
    let popup = ui.auto_id_with("popup");
    let mut channels = rgb(colour);
    egui::color_picker::color_edit_button_srgb(ui, &mut channels);
    (
        Colour::rgb(channels[0], channels[1], channels[2]),
        egui::Popup::is_id_open(ui.ctx(), popup),
    )
}

/// The tool strip over the top-left corner of the viewport: Select, Wall, Portal, Room, and Paint,
/// the Snap switch, shown selected while on and switched by a click that is no history step, then
/// the options. With the Paint tool they are the Brush's; with a Portal selected they are
/// the Portal's own; with a Room selected, or the Room tool chosen and none selected, they are the
/// wall thickness, the wall colour, and the floor colour, of the Room or of the next one;
/// otherwise they are the thickness and the colour, of the selected Wall, a change sent to it as
/// one Edit Element, or with none of the next Wall.
///
/// Choosing the Wall or the Room tool drops the chosen Asset and the selection and leaves the
/// other tools, discarding a Wall, an outline, or a stroke being drawn; choosing the Portal tool
/// leaves the Wall, the Room, and the Paint tool, discarding what is being drawn, and drops the
/// selection; choosing Select leaves any.
pub(crate) fn tool_strip(
    mut contexts: EguiContexts,
    mut state: ResMut<EditorState>,
    mut switch: ResMut<SnapSwitch>,
    viewport: Res<Viewport>,
    level: LevelView,
    mut apply: MessageWriter<Apply>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let ctx = ctx.clone();
    let selected = level
        .selected_wall(state.selected)
        .map(|(id, wall, _)| (id, wall.clone()));
    let room = level
        .selected_room(state.selected)
        .map(|(id, room)| (id, room.clone()));
    let portal = level
        .selected_portal(state.selected)
        .map(|(id, element, portal)| (id, element.clone(), portal.clone()));
    let corner = egui::pos2(viewport.area.min.x, viewport.area.min.y) + egui::Vec2::splat(MARGIN);
    egui::Area::new(egui::Id::new("tool-strip"))
        .fixed_pos(corner)
        .order(egui::Order::Foreground)
        .show(&ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_max_width(strip_width(&viewport, ui.style()));
                ui.horizontal_wrapped(|ui| {
                    tools(ui, &mut state, &mut switch);
                    ui.separator();
                    if state.tool == Tool::Paint {
                        portals::end_options(&mut state, &mut apply);
                        rooms::end_options(&mut state, &mut apply);
                        let terrain = level.current_terrain();
                        crate::paint::options(ui, &mut state, terrain, &level.terrains, &mut apply);
                    } else if let Some((id, element, portal)) = &portal {
                        rooms::end_options(&mut state, &mut apply);
                        portals::options(
                            ui,
                            &mut state,
                            (*id, element, portal, level.follows_host(*id)),
                            level.lines_in_order(),
                            &mut apply,
                        );
                    } else if room.is_some() || state.tool == Tool::Room {
                        portals::end_options(&mut state, &mut apply);
                        end_option(&mut state.walls.option, &mut apply);
                        rooms::options(ui, &mut state, room.as_ref(), &mut apply);
                    } else {
                        portals::end_options(&mut state, &mut apply);
                        rooms::end_options(&mut state, &mut apply);
                        options(ui, &mut state, selected.as_ref(), &mut apply);
                    }
                });
            });
        });
}

/// The width the tool strip's rows may take before they wrap: the viewport's, less the margin
/// either side and the strip's own frame.
fn strip_width(viewport: &Viewport, style: &egui::Style) -> f32 {
    let frame = egui::Frame::popup(style).total_margin().sum().x;
    (viewport.area.width() - 2.0 * MARGIN - frame).max(0.0)
}

/// The tools of the strip, Select, Wall, Portal, Room, and Paint, and the Snap switch after
/// them, none of which can be used while an Export runs.
fn tools(ui: &mut egui::Ui, state: &mut EditorState, switch: &mut SnapSwitch) {
    let tool = state.tool;
    let enabled = !state.exporting;
    if ui
        .add_enabled(
            enabled,
            egui::Button::selectable(tool == Tool::Select, "Select"),
        )
        .clicked()
    {
        leave_tool(state);
        crate::paint::leave_paint_tool(state);
    }
    if ui
        .add_enabled(
            enabled,
            egui::Button::selectable(tool == Tool::Wall, "Wall"),
        )
        .on_hover_text("W")
        .clicked()
    {
        choose_wall_tool(state);
    }
    if ui
        .add_enabled(
            enabled,
            egui::Button::selectable(tool == Tool::Portal, "Portal"),
        )
        .on_hover_text("P")
        .clicked()
    {
        portals::choose_portal_tool(state);
    }
    if ui
        .add_enabled(
            enabled,
            egui::Button::selectable(tool == Tool::Room, "Room"),
        )
        .on_hover_text("R")
        .clicked()
    {
        rooms::choose_room_tool(state);
    }
    if ui
        .add_enabled(
            enabled,
            egui::Button::selectable(tool == Tool::Paint, "Paint"),
        )
        .on_hover_text("B")
        .clicked()
    {
        crate::paint::choose_paint_tool(state);
    }
    if ui
        .add_enabled(enabled, egui::Button::selectable(switch.on, "Snap"))
        .on_hover_text(SNAP_HINT)
        .clicked()
    {
        switch.on = !switch.on;
    }
}

/// What the Snap switch's tooltip says.
const SNAP_HINT: &str = if cfg!(target_os = "macos") {
    "Snap points to the Grid and to the points of Walls and Rooms; hold Option to place freely"
} else {
    "Snap points to the Grid and to the points of Walls and Rooms; hold Alt to place freely"
};

/// The thickness and colour options, of the selected Wall or of the next one.
fn options(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    selected: Option<&(ElementId, Wall)>,
    apply: &mut MessageWriter<Apply>,
) {
    let (thickness, colour) = selected
        .map_or((state.walls.thickness, state.walls.colour), |(_, wall)| {
            (wall.thickness, wall.colour)
        });
    let (thickness, drag) = thickness_option(ui, thickness);
    let (picked, picking) = colour_option(ui, "Colour", colour);

    let Some((element, wall)) = selected else {
        // A thickness no Wall could have leaves the next Wall's as it was.
        if thickness > 0.0 && thickness.is_finite() {
            state.walls.thickness = thickness;
        }
        state.walls.colour = picked;
        end_option(&mut state.walls.option, apply);
        return;
    };
    if drag.changed() {
        send_option(
            &mut state.walls.option,
            apply,
            *element,
            ElementChange::Thickness(thickness),
            drag.dragged(),
        );
    } else if picked != wall.colour {
        send_option(
            &mut state.walls.option,
            apply,
            *element,
            ElementChange::Colour(picked),
            picking,
        );
    } else if !drag.dragged() && !picking {
        end_option(&mut state.walls.option, apply);
    }
}

/// Draws what the Wall tool shows over the Level: the Wall being drawn as a thin line through its
/// points with a rubber band to where the next click lands, the snapped point or the pointer.
/// Nothing is drawn while an Export runs.
pub(crate) fn draw_overlays(
    mut gizmos: Gizmos,
    state: Res<EditorState>,
    viewport: Res<Viewport>,
    window: Single<&Window, With<PrimaryWindow>>,
    pointer: Res<Pointer>,
    snapped: Res<SnappedPoint>,
) {
    if state.exporting {
        return;
    }
    let radius = HANDLE_PIXELS / viewport.zoom;
    if state.walls.drawing_in_progress() {
        let colour = Color::srgb_u8(
            state.walls.colour.red,
            state.walls.colour.green,
            state.walls.colour.blue,
        );
        let shown = Shown::of(&snapped, &pointer);
        let next = window
            .cursor_position()
            .filter(|cursor| viewport.contains(*cursor))
            .map(|cursor| shown.point_or(viewport.cells_at(cursor)));
        gizmos.linestrip_2d(state.walls.drawing.iter().copied().chain(next), HANDLES);
        for point in &state.walls.drawing {
            gizmos.circle_2d(Isometry2d::from_translation(*point), radius / 2.0, colour);
        }
    }
}

/// Logs the Wall tool and every Wall, for a script to check what drawing and editing did.
#[cfg(feature = "dev")]
pub(crate) fn describe(
    state: &EditorState,
    walls: &bevy::ecs::system::Query<(&ElementId, &drs_model::Element, &Wall)>,
) {
    bevy::log::info!(
        "describe: tool {:?}, drawing {:?}, handle {:?}, next thickness {} colour {:?}",
        state.tool,
        state.walls.drawing,
        state.handle.map(|(_, handle)| handle),
        state.walls.thickness,
        rgb(state.walls.colour)
    );
    for (id, element, wall) in walls {
        bevy::log::info!(
            "describe: wall {} at {} size {}, points {:?}, controls {:?}, thickness {}, colour {:?}",
            id.as_raw(),
            element.position,
            element.size,
            wall.points,
            wall.segments
                .iter()
                .map(|segment| segment.control)
                .collect::<Vec<_>>(),
            wall.thickness,
            rgb(wall.colour)
        );
    }
}
