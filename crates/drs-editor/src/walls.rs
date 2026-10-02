//! The Wall tool and the handles of a selected Wall or Room: the tool strip over the viewport
//! with its options, the Wall being drawn, picking a Wall or a Room's Walls by its line outside
//! the stretches its Portals cover, and the points, control points, and segment or edge middles
//! of either as handles.
//!
//! The Wall being drawn is the Editor's own state until it is finished, when it becomes one
//! Place Element; every change to a placed Wall is an Edit Element.

use crate::state::{EditorState, Tool};
use crate::viewport::LevelView;
use crate::{portals, rooms};
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
    Apply, Colour, EditElement, ElementChange, ElementId, Gesture, LinePlace, PlaceElement,
    Placement, Room, Viewport, Wall, WallShape,
};

/// The thickness the first Wall is drawn with: an eighth of a cell.
pub(crate) const DEFAULT_THICKNESS: f32 = 0.125;
/// The colour the first Wall is drawn with: a dark grey.
pub(crate) const DEFAULT_COLOUR: Colour = Colour::rgb(64, 64, 64);
/// How close to the last point, in screen pixels, a click adds no point.
pub(crate) const NEAR_THE_LAST: f32 = 4.0;
/// How soon after a click, in seconds, a second click is a double-click.
const DOUBLE_CLICK_SECONDS: f64 = 0.5;
/// How close to a click, in screen pixels, a second click is a double-click.
const DOUBLE_CLICK_PIXELS: f32 = 5.0;
/// How close to a handle, in screen pixels, the pointer is on it; the size handles are drawn at.
pub(crate) const HANDLE_PIXELS: f32 = 6.0;
/// How close to a Wall's line, in screen pixels, the pointer is on it however thin the Wall.
const LINE_PIXELS: f32 = 4.0;
/// The thinnest Wall a drag of the thickness reaches, in cells; a typed thickness is sent as
/// typed, so one not above zero is refused with the reason.
const THINNEST_DRAGGED: f32 = 0.01;
/// The thickest Wall the options offer, dragged or typed, in cells.
const THICKEST: f32 = 16.0;
/// How wide a control point's square is drawn, against a point's radius.
const CONTROL_SIDE: f32 = 1.6;
/// How opaque the guide lines from a control point to its segment's points are drawn.
const GUIDE_ALPHA: f32 = 0.5;
/// The nearest a point added by a double-click comes to either end of its segment, as a
/// parameter along it: a segment is split strictly between its points, never at one.
const NEAREST_TO_AN_END: f32 = 0.001;
/// The colour of the handles and the guide lines.
pub(crate) const HANDLES: Color = Color::srgb(0.35, 0.75, 1.0);
/// The colour of the selected handle.
const PICKED: Color = Color::srgb(1.0, 0.85, 0.2);

/// A handle of the selected Wall or Room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WallHandle {
    /// The point of that number.
    Point(usize),
    /// The control point of the curved segment or edge of that number.
    Control(usize),
    /// The middle of the straight segment or edge of that number, which a drag bends.
    Middle(usize),
}

/// A Wall or a Room as its handles and its line are seen: its points, one control point or none
/// per segment or edge, whether its line closes from the last point back to the first, and how
/// thick it is drawn.
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

    /// Whether an option is being changed as a gesture: the thickness is being dragged, or a
    /// colour was picked while the picker is open.
    pub(crate) fn option_in_progress(&self) -> bool {
        self.option.is_some()
    }

    /// The selected handle of `element`, if one is selected.
    pub(crate) fn handle_of(&self, element: ElementId) -> Option<WallHandle> {
        self.handle
            .and_then(|(owner, handle)| (owner == element).then_some(handle))
    }
}

/// Chooses the Wall tool: the Paint tool is left, discarding a stroke being drawn, the Room tool
/// is left, discarding an outline being drawn, and the chosen Asset and the selection are
/// dropped.
pub(crate) fn choose_wall_tool(state: &mut EditorState) {
    crate::paint::discard_stroke(state);
    state.chosen = None;
    state.selected = None;
    state.walls.handle = None;
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

/// The point of a Wall's flattened line nearest `cells`.
pub(crate) fn nearest_on_line(shape: &WallShape, cells: Vec2) -> Option<NearestPoint> {
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

/// The handles of a Wall or a Room in the order they are hit: its points, then its control
/// points, then the middles of its straight segments or edges, the closing edge's included.
pub(crate) fn handles(outline: &Outline) -> Vec<(WallHandle, Vec2)> {
    let points = outline
        .points
        .iter()
        .enumerate()
        .map(|(index, point)| (WallHandle::Point(index), *point));
    let controls = outline
        .controls
        .iter()
        .enumerate()
        .filter_map(|(index, control)| Some((WallHandle::Control(index), (*control)?)));
    let middles = outline
        .controls
        .iter()
        .enumerate()
        .filter(|(_, control)| control.is_none())
        .filter_map(|(index, _)| {
            let (start, end) = outline.ends(index)?;
            Some((WallHandle::Middle(index), start.midpoint(end)))
        });
    points.chain(controls).chain(middles).collect()
}

/// The first handle of a Wall or a Room within a handle's reach of a point in cells.
fn handle_at(outline: &Outline, cells: Vec2, zoom: f32) -> Option<(WallHandle, Vec2)> {
    handles(outline)
        .into_iter()
        .find(|(_, at)| at.distance(cells) * zoom <= HANDLE_PIXELS)
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
            Some((WallHandle::Point(_) | WallHandle::Control(_), _))
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
        state.walls.handle = None;
        return true;
    }
    let Some((handle, origin)) = handle else {
        return false;
    };
    // A straight segment's middle is only for dragging: a click on it leaves the Wall selected
    // with no handle, so Delete removes the Wall rather than doing nothing.
    state.walls.handle = match handle {
        WallHandle::Point(_) | WallHandle::Control(_) => Some((element, handle)),
        WallHandle::Middle(_) => None,
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

/// The Edit Element Delete sends for the selected handle of a Wall or a Room: the point is
/// removed, or the curved segment or edge of the control point is made straight; `None` when
/// there is nothing to do.
pub(crate) fn delete_handle(outline: &Outline, handle: WallHandle) -> Option<ElementChange> {
    match handle {
        WallHandle::Point(index) => {
            (index < outline.points.len()).then_some(ElementChange::RemovePoint { index })
        }
        WallHandle::Control(segment) => {
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
        WallHandle::Middle(_) => None,
    }
}

/// Whether a handle still names a part of the Wall or the Room: a point it has, or a segment or
/// edge that is curved for a control point and straight for a middle.
pub(crate) fn handle_exists(outline: &Outline, handle: WallHandle) -> bool {
    match handle {
        WallHandle::Point(index) => index < outline.points.len(),
        WallHandle::Control(segment) => outline.controls.get(segment).is_some_and(Option::is_some),
        WallHandle::Middle(segment) => outline.controls.get(segment).is_some_and(Option::is_none),
    }
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
/// then the options. With the Paint tool they are the Brush's; with a Portal selected they are
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
                        leave_tool(&mut state);
                        crate::paint::leave_paint_tool(&mut state);
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
                    if ui
                        .add_enabled(
                            enabled,
                            egui::Button::selectable(tool == Tool::Portal, "Portal"),
                        )
                        .on_hover_text("P")
                        .clicked()
                    {
                        portals::choose_portal_tool(&mut state);
                    }
                    if ui
                        .add_enabled(
                            enabled,
                            egui::Button::selectable(tool == Tool::Room, "Room"),
                        )
                        .on_hover_text("R")
                        .clicked()
                    {
                        rooms::choose_room_tool(&mut state);
                    }
                    if ui
                        .add_enabled(
                            enabled,
                            egui::Button::selectable(tool == Tool::Paint, "Paint"),
                        )
                        .on_hover_text("B")
                        .clicked()
                    {
                        crate::paint::choose_paint_tool(&mut state);
                    }
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
/// points with a rubber band to the pointer, and the handles of the selected Wall or Room, its
/// control points with guide lines to their segment's or edge's points. Nothing is drawn while
/// an Export runs.
pub(crate) fn draw_overlays(
    mut gizmos: Gizmos,
    state: Res<EditorState>,
    viewport: Res<Viewport>,
    window: Single<&Window, With<PrimaryWindow>>,
    outlines: Query<(&ElementId, Option<&Wall>, Option<&Room>)>,
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
    let Some((id, outline)) = state.selected.and_then(|selected| {
        outlines.iter().find_map(|(id, wall, room)| {
            let outline = wall
                .map(Outline::of_wall)
                .or_else(|| room.map(Outline::of_room))?;
            (*id == selected).then_some((*id, outline))
        })
    }) else {
        return;
    };
    let picked = state.walls.handle_of(id);
    for (handle, at) in handles(&outline) {
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
