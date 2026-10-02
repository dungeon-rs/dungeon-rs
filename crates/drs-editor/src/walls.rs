//! The Wall tool and the handles of a selected Wall: the tool strip over the viewport with its
//! options, the Wall being drawn, picking a Wall by its line, and its points, control points, and
//! segment middles as handles.
//!
//! The Wall being drawn is the Editor's own state until it is finished, when it becomes one
//! Place Element; every change to a placed Wall is an Edit Element.

use crate::state::{EditorState, Tool};
use bevy::color::{Alpha, Color};
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::query::With;
use bevy::ecs::system::{Query, Res, ResMut, Single};
use bevy::gizmos::gizmos::Gizmos;
use bevy::math::{Isometry2d, Vec2};
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::EguiContexts;
use drs_model::{
    Apply, Colour, EditElement, ElementChange, ElementId, Gesture, PlaceElement, Placement,
    Viewport, Wall, WallShape,
};

/// The thickness the first Wall is drawn with: an eighth of a cell.
const DEFAULT_THICKNESS: f32 = 0.125;
/// The colour the first Wall is drawn with: a dark grey.
const DEFAULT_COLOUR: Colour = Colour::rgb(64, 64, 64);
/// How close to the last point, in screen pixels, a click adds no point.
const NEAR_THE_LAST: f32 = 4.0;
/// How soon after a click, in seconds, a second click is a double-click.
const DOUBLE_CLICK_SECONDS: f64 = 0.5;
/// How close to a click, in screen pixels, a second click is a double-click.
const DOUBLE_CLICK_PIXELS: f32 = 5.0;
/// How close to a handle, in screen pixels, the pointer is on it; the size handles are drawn at.
const HANDLE_PIXELS: f32 = 6.0;
/// How close to a Wall's line, in screen pixels, the pointer is on it however thin the Wall.
const LINE_PIXELS: f32 = 4.0;
/// The thinnest and the thickest Wall the options offer, in cells.
const THICKNESS_RANGE: std::ops::RangeInclusive<f32> = 0.01..=16.0;
/// How wide a control point's square is drawn, against a point's radius.
const CONTROL_SIDE: f32 = 1.6;
/// How opaque the guide lines from a control point to its segment's points are drawn.
const GUIDE_ALPHA: f32 = 0.5;
/// The nearest a point added by a double-click comes to either end of its segment, as a
/// parameter along it: a segment is split strictly between its points, never at one.
const NEAREST_TO_AN_END: f32 = 0.001;
/// The colour of the handles and the guide lines.
const HANDLES: Color = Color::srgb(0.35, 0.75, 1.0);
/// The colour of the selected handle.
const PICKED: Color = Color::srgb(1.0, 0.85, 0.2);

/// A handle of the selected Wall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WallHandle {
    /// The point of that number.
    Point(usize),
    /// The control point of the curved segment of that number.
    Control(usize),
    /// The middle of the straight segment of that number, which a drag bends.
    Middle(usize),
}

/// An option of the Wall tool being changed as one gesture on the Wall it shows.
#[derive(Debug, Clone, PartialEq)]
struct OptionGesture {
    /// The Wall.
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
    /// The selected handle and the Wall it belongs to.
    pub handle: Option<(ElementId, WallHandle)>,
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
            handle: None,
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

    /// The selected handle of `element`, if one is selected.
    pub(crate) fn handle_of(&self, element: ElementId) -> Option<WallHandle> {
        self.handle
            .and_then(|(owner, handle)| (owner == element).then_some(handle))
    }
}

/// Chooses the Wall tool: the chosen Asset and the selection are dropped.
pub(crate) fn choose_wall_tool(state: &mut EditorState) {
    state.chosen = None;
    state.selected = None;
    state.walls.handle = None;
    state.tool = Tool::Wall;
}

/// Leaves the Wall tool for the Select tool, discarding a Wall being drawn.
pub(crate) fn leave_wall_tool(state: &mut EditorState) {
    state.tool = Tool::Select;
    state.walls.drawing.clear();
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

/// A click with the Wall tool: the second click of a double-click finishes the Wall at the point
/// its first click added; any other click adds a point unless it lands within a few pixels of
/// the last one.
pub(crate) fn draw_click(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    layer: Option<Entity>,
    viewport: &Viewport,
    cursor: Vec2,
    double: bool,
) {
    if double {
        finish(state, apply, layer);
        return;
    }
    let cells = viewport.cells_at(cursor);
    let near_the_last = state
        .walls
        .drawing
        .last()
        .is_some_and(|last| last.distance(cells) * viewport.zoom <= NEAR_THE_LAST);
    if !near_the_last {
        state.walls.drawing.push(cells);
    }
}

/// Where along a Wall's line the point nearest `cells` lies: its distance in cells, the segment,
/// and the parameter along the segment, interpolated along the nearest chord.
pub(crate) fn nearest_on_line(shape: &WallShape, cells: Vec2) -> Option<(f32, usize, f32)> {
    shape
        .line
        .windows(2)
        .map(|pair| {
            let (from, to) = (pair[0], pair[1]);
            let along = to.position - from.position;
            let s = if along.length_squared() > 0.0 {
                ((cells - from.position).dot(along) / along.length_squared()).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let end = if to.segment == from.segment {
                to.t
            } else {
                1.0
            };
            let distance = cells.distance(from.position + along * s);
            (distance, from.segment, from.t + (end - from.t) * s)
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
}

/// Whether a point in cells is on a Wall: no farther from its line than half its thickness or
/// four screen pixels, whichever is more.
pub(crate) fn on_wall(wall: &Wall, shape: &WallShape, cells: Vec2, zoom: f32) -> bool {
    let reach = (wall.thickness / 2.0).max(LINE_PIXELS / zoom);
    nearest_on_line(shape, cells).is_some_and(|(distance, ..)| distance <= reach)
}

/// The handles of a Wall in the order they are hit: its points, then its control points, then
/// the middles of its straight segments.
fn handles(wall: &Wall) -> Vec<(WallHandle, Vec2)> {
    let points = wall
        .points
        .iter()
        .enumerate()
        .map(|(index, point)| (WallHandle::Point(index), *point));
    let controls = wall
        .segments
        .iter()
        .enumerate()
        .filter_map(|(index, segment)| Some((WallHandle::Control(index), segment.control?)));
    let middles = wall
        .segments
        .iter()
        .zip(wall.points.windows(2))
        .enumerate()
        .filter(|(_, (segment, _))| segment.control.is_none())
        .map(|(index, (_, ends))| (WallHandle::Middle(index), ends[0].midpoint(ends[1])));
    points.chain(controls).chain(middles).collect()
}

/// The first handle of a Wall within a handle's reach of a point in cells.
fn handle_at(wall: &Wall, cells: Vec2, zoom: f32) -> Option<(WallHandle, Vec2)> {
    handles(wall)
        .into_iter()
        .find(|(_, at)| at.distance(cells) * zoom <= HANDLE_PIXELS)
}

/// A left press with the Select tool on the selected Wall, which is hit before any Element: a
/// double-click on its line adds a point at the nearest place on it, and a press on a handle
/// selects the handle and arms a drag of it. Returns whether the press was the Wall's.
pub(crate) fn press_selected(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    selected: Option<(ElementId, &Wall, Option<&WallShape>)>,
    viewport: &Viewport,
    cursor: Vec2,
    double: bool,
) -> bool {
    let Some((element, wall, shape)) = selected else {
        return false;
    };
    let cells = viewport.cells_at(cursor);
    let handle = handle_at(wall, cells, viewport.zoom);
    if double
        && !matches!(
            handle,
            Some((WallHandle::Point(_) | WallHandle::Control(_), _))
        )
        && let Some(shape) = shape
        && on_wall(wall, shape, cells, viewport.zoom)
        && let Some((_, segment, t)) = nearest_on_line(shape, cells)
    {
        apply.write(Apply::EditElement(EditElement {
            element,
            change: ElementChange::AddPoint {
                segment,
                t: t.clamp(NEAREST_TO_AN_END, 1.0 - NEAREST_TO_AN_END),
            },
            gesture: Gesture::Single,
        }));
        state.walls.handle = None;
        return true;
    }
    let Some((handle, origin)) = handle else {
        return false;
    };
    state.walls.handle = Some((element, handle));
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
/// straight segment's middle becomes the segment's control point.
pub(crate) fn handle_change(handle: WallHandle, position: Vec2) -> ElementChange {
    match handle {
        WallHandle::Point(index) => ElementChange::Point { index, position },
        WallHandle::Control(segment) | WallHandle::Middle(segment) => ElementChange::Control {
            segment,
            position: Some(position),
        },
    }
}

/// The Edit Element Delete sends for the selected handle of a Wall: the point is removed, or the
/// curved segment of the control point is made straight; `None` when there is nothing to do.
pub(crate) fn delete_handle(wall: &Wall, handle: WallHandle) -> Option<ElementChange> {
    match handle {
        WallHandle::Point(index) => {
            (index < wall.points.len()).then_some(ElementChange::RemovePoint { index })
        }
        WallHandle::Control(segment) => wall
            .segments
            .get(segment)
            .and_then(|segment| segment.control)
            .map(|_| ElementChange::Control {
                segment,
                position: None,
            }),
        WallHandle::Middle(_) => None,
    }
}

/// Whether a handle still names a part of the Wall: a point it has, or a segment that is curved
/// for a control point and straight for a middle.
pub(crate) fn handle_exists(wall: &Wall, handle: WallHandle) -> bool {
    match handle {
        WallHandle::Point(index) => index < wall.points.len(),
        WallHandle::Control(segment) => wall
            .segments
            .get(segment)
            .is_some_and(|segment| segment.control.is_some()),
        WallHandle::Middle(segment) => wall
            .segments
            .get(segment)
            .is_some_and(|segment| segment.control.is_none()),
    }
}

/// The colour of the model as egui spells it.
fn rgb(colour: Colour) -> [u8; 3] {
    [colour.red, colour.green, colour.blue]
}

/// Sends an option's change to the Wall it shows: a change made while the widget is held (a
/// drag of the thickness, the colour picker open) is part of a gesture that ends when it is let
/// go, so it is one step; any other change is a step of its own.
fn send_option(
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
fn end_option(option: &mut Option<OptionGesture>, apply: &mut MessageWriter<Apply>) {
    if let Some(OptionGesture { element, change }) = option.take() {
        apply.write(Apply::EditElement(EditElement {
            element,
            change,
            gesture: Gesture::End,
        }));
    }
}

/// The tool strip over the top-left corner of the viewport: Select and Wall, then the tool's
/// options, the thickness and the colour. With a Wall selected the options show its values and
/// a change is sent to it as one Edit Element; with none they set the next Wall's.
///
/// Choosing the Wall tool drops the chosen Asset and the selection; choosing Select leaves the
/// Wall tool, discarding a Wall being drawn.
pub(crate) fn tool_strip(
    mut contexts: EguiContexts,
    mut state: ResMut<EditorState>,
    viewport: Res<Viewport>,
    walls: Query<(&ElementId, &Wall)>,
    mut apply: MessageWriter<Apply>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let ctx = ctx.clone();
    let selected = state
        .selected
        .and_then(|selected| walls.iter().find(|(id, _)| **id == selected))
        .map(|(id, wall)| (*id, wall.clone()));
    let corner = egui::pos2(viewport.area.min.x + 8.0, viewport.area.min.y + 8.0);
    egui::Area::new(egui::Id::new("tool-strip"))
        .fixed_pos(corner)
        .order(egui::Order::Foreground)
        .show(&ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    let tool = state.tool;
                    let enabled = !state.exporting;
                    if ui
                        .add_enabled(
                            enabled,
                            egui::Button::selectable(tool == Tool::Select, "Select"),
                        )
                        .clicked()
                    {
                        leave_wall_tool(&mut state);
                    }
                    if ui
                        .add_enabled(
                            enabled,
                            egui::Button::selectable(tool == Tool::Wall, "Wall"),
                        )
                        .on_hover_text("W")
                        .clicked()
                    {
                        choose_wall_tool(&mut state);
                    }
                    ui.separator();
                    options(ui, &mut state, selected.as_ref(), &mut apply);
                });
            });
        });
}

/// The thickness and colour options, of the selected Wall or of the next one.
fn options(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    selected: Option<&(ElementId, Wall)>,
    apply: &mut MessageWriter<Apply>,
) {
    let (mut thickness, colour) = selected
        .map_or((state.walls.thickness, state.walls.colour), |(_, wall)| {
            (wall.thickness, wall.colour)
        });
    ui.label("Thickness");
    let drag = ui.add(
        egui::DragValue::new(&mut thickness)
            .range(THICKNESS_RANGE)
            .speed(0.005)
            .max_decimals(3)
            .suffix(" cells")
            .update_while_editing(false),
    );
    ui.label("Colour");
    // The colour button's popup takes the id the button's own next id is salted with.
    let popup = ui.auto_id_with("popup");
    let mut channels = rgb(colour);
    egui::color_picker::color_edit_button_srgb(ui, &mut channels);
    let picking = egui::Popup::is_id_open(ui.ctx(), popup);
    let picked = Colour::rgb(channels[0], channels[1], channels[2]);

    let Some((element, wall)) = selected else {
        state.walls.thickness = thickness;
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
/// points with a rubber band to the pointer, and the handles of the selected Wall, its control
/// points with guide lines to their segment's points. Nothing is drawn while an Export runs.
pub(crate) fn draw_overlays(
    mut gizmos: Gizmos,
    state: Res<EditorState>,
    viewport: Res<Viewport>,
    window: Single<&Window, With<PrimaryWindow>>,
    walls: Query<(&ElementId, &Wall)>,
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
        let pointer = window
            .cursor_position()
            .filter(|cursor| viewport.contains(*cursor))
            .map(|cursor| viewport.cells_at(cursor));
        gizmos.linestrip_2d(state.walls.drawing.iter().copied().chain(pointer), HANDLES);
        for point in &state.walls.drawing {
            gizmos.circle_2d(Isometry2d::from_translation(*point), radius / 2.0, colour);
        }
    }
    let Some((id, wall)) = state
        .selected
        .and_then(|selected| walls.iter().find(|(id, _)| **id == selected))
    else {
        return;
    };
    let picked = state.walls.handle_of(*id);
    for (handle, at) in handles(wall) {
        let colour = if picked == Some(handle) {
            PICKED
        } else {
            HANDLES
        };
        match handle {
            WallHandle::Point(_) => {
                gizmos.circle_2d(Isometry2d::from_translation(at), radius, colour);
            }
            WallHandle::Control(segment) => {
                for end in wall.points.iter().skip(segment).take(2) {
                    gizmos.line_2d(*end, at, HANDLES.with_alpha(GUIDE_ALPHA));
                }
                gizmos.rect_2d(
                    Isometry2d::from_translation(at),
                    Vec2::splat(radius * CONTROL_SIDE),
                    colour,
                );
            }
            WallHandle::Middle(_) => {
                gizmos.circle_2d(Isometry2d::from_translation(at), radius / 2.0, colour);
            }
        }
    }
}

/// Logs the Wall tool and every Wall, for a script to check what drawing and editing did.
#[cfg(feature = "dev")]
pub(crate) fn describe(
    state: &EditorState,
    walls: &Query<(&ElementId, &drs_model::Element, &Wall)>,
) {
    bevy::log::info!(
        "describe: tool {:?}, drawing {:?}, handle {:?}, next thickness {} colour {:?}",
        state.tool,
        state.walls.drawing,
        state.walls.handle.map(|(_, handle)| handle),
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
