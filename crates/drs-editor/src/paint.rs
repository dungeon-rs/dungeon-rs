//! The Paint tool: the Brush and its options, the tool's three modes, the stroke being drawn, the
//! selected stroke with its handles, and what the tool shows over the viewport, the Brush's
//! circle at the pointer, the stroke being drawn as a translucent band, and the selected stroke.
//!
//! The stroke being drawn is the Editor's own state until the release, when it becomes one Paint;
//! a laid stroke is changed only through Edit Element. The Brush's settings, the mode, and the
//! selected stroke are the Editor's too, never a history step and never saved.

use crate::bindings;
use crate::state::{EditorState, Interaction, Tool};
use crate::viewport::LevelView;
use crate::walls::{self, OptionGesture};
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::ecs::query::With;
use bevy::ecs::system::{Query, Res, ResMut, Single, SystemParam};
use bevy::math::Vec2;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::EguiContexts;
use drs_model::{
    Apply, AssetAddress, AssetReferences, BrushSettings, EditElement, ElementChange, ElementId,
    Gesture, Layer, Paint, Project, Redo, Resolution, ResolutionTable, Stroke, StrokeChange,
    Terrain, Undo, Viewport,
};

/// The Brush a new editor starts with: two cells across, half hard, at full strength.
const FIRST_BRUSH: BrushSettings = BrushSettings {
    size: 2.0,
    hardness: 0.5,
    strength: 1.0,
};
/// The smallest Brush the options offer, in cells; the smallest a drag of a stroke's size
/// reaches, a typed size being sent as typed.
const SMALLEST: f32 = 0.1;
/// The largest Brush the options offer, in cells.
const LARGEST: f32 = 64.0;
/// The weakest strength the options offer, as a percentage.
const WEAKEST: f32 = 1.0;
/// How far the pointer moves, as a part of the Brush's size, before it adds a point to the path.
const SPACING: f32 = 1.0 / 8.0;
/// How far the pointer travels, in pixels, before a press on a stroke or a handle becomes a drag.
const DRAG_THRESHOLD: f32 = 3.0;
/// How close to a handle, in screen pixels, the pointer is on it; the size handles are drawn at.
const HANDLE_PIXELS: f32 = 6.0;
/// How close to a stroke's path, in screen pixels, the pointer is on it however narrow it is.
const PATH_PIXELS: f32 = 4.0;
/// The colour of the Brush's circle and a stroke's path while painting.
const CIRCLE: egui::Color32 = egui::Color32::from_rgb(90, 190, 255);
/// The colour of the band a stroke that paints is shown as: translucent.
const BAND: egui::Color32 = egui::Color32::from_rgba_unmultiplied_const(90, 190, 255, 90);
/// The colour of the Brush's circle and a stroke's path while erasing: a warm red.
const ERASE_CIRCLE: egui::Color32 = egui::Color32::from_rgb(255, 110, 80);
/// The colour of the band an erase is shown as: translucent.
const ERASE_BAND: egui::Color32 = egui::Color32::from_rgba_unmultiplied_const(255, 110, 80, 90);
/// The colour of the handle being dragged.
const PICKED: egui::Color32 = egui::Color32::from_rgb(255, 217, 51);

/// What the Paint tool does with the pointer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Mode {
    /// A drag lays a stroke that paints.
    #[default]
    Painting,
    /// A drag lays a stroke that erases.
    Erasing,
    /// A click picks a stroke of the current Layer's Terrain and a drag reshapes it.
    EditingStrokes,
}

/// A stroke of a Terrain, by the Terrain's identity and the stroke's number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StrokeRef {
    /// The Terrain.
    pub terrain: ElementId,
    /// The stroke's number, the first laid counted as zero.
    pub stroke: usize,
}

/// What a drag in Edit strokes moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dragged {
    /// One point of the stroke's path, by its number.
    Point(usize),
    /// The whole stroke.
    Stroke,
}

/// A press in Edit strokes on a stroke or one of its handles, which a drag turns into a gesture.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StrokeDrag {
    /// The stroke pressed on.
    target: StrokeRef,
    /// What the drag moves.
    dragged: Dragged,
    /// Where the point, or the centre of the stroke's points, was when the button went down.
    origin: Vec2,
    /// The pointer, in cells, when the button went down.
    pointer: Vec2,
    /// The pointer, on screen, when the button went down.
    screen: Vec2,
    /// The change last sent; `None` until the drag begins.
    sent: Option<ElementChange>,
}

/// The Paint tool's state.
#[derive(Debug)]
pub(crate) struct PaintTool {
    /// The settings the next stroke is laid with.
    pub brush: BrushSettings,
    /// Whether the tool paints, erases, or edits strokes.
    pub mode: Mode,
    /// The path of the stroke being drawn, in cells; empty when none is.
    pub stroke: Vec<Vec2>,
    /// Whether the stroke being drawn erases, as the tool did when it was pressed.
    pub erasing: bool,
    /// The selected stroke, in Edit strokes.
    pub selected: Option<StrokeRef>,
    /// The press or drag on a stroke or a handle under way, in Edit strokes.
    pub drag: Option<StrokeDrag>,
    /// The selected stroke's option being changed as a gesture, if one is.
    option: Option<OptionGesture>,
}

impl Default for PaintTool {
    /// The first Brush, painting, and nothing drawn or selected.
    fn default() -> Self {
        Self {
            brush: FIRST_BRUSH,
            mode: Mode::Painting,
            stroke: Vec::new(),
            erasing: false,
            selected: None,
            drag: None,
            option: None,
        }
    }
}

impl PaintTool {
    /// Whether a step is being made with the Paint tool, or may be about to be: a stroke being
    /// drawn, a press on a stroke or a handle of one, which names the stroke by its number, or
    /// its drag, or an option of the selected stroke held while it changes.
    pub(crate) fn step_in_progress(&self) -> bool {
        !self.stroke.is_empty() || self.drag.is_some() || self.option.is_some()
    }

    /// The point being dragged, if a drag of a point is under way.
    fn dragged_point(&self) -> Option<(StrokeRef, usize)> {
        self.drag.as_ref().and_then(|drag| match drag.dragged {
            Dragged::Point(index) => Some((drag.target, index)),
            Dragged::Stroke => None,
        })
    }
}

/// Chooses the Paint tool painting: the Wall, the Portal, or the Room tool is left, discarding a
/// Wall or an outline being drawn, and the selection and the selected stroke are let go; a chosen
/// Asset stays, as the image the Brush paints with. A stroke being drawn keeps what it does.
pub(crate) fn choose_paint_tool(state: &mut EditorState) {
    choose_mode(state, Mode::Painting);
}

/// Chooses the Paint tool erasing, as [`choose_paint_tool`] chooses it painting.
pub(crate) fn choose_erasing(state: &mut EditorState) {
    choose_mode(state, Mode::Erasing);
}

/// Chooses the Paint tool in `mode`, letting the selected stroke go unless it edits strokes.
fn choose_mode(state: &mut EditorState, mode: Mode) {
    if state.tool != Tool::Paint {
        walls::leave_tool(state);
        state.selected = None;
        state.handle = None;
        state.tool = Tool::Paint;
    }
    if mode != Mode::EditingStrokes {
        state.paint.selected = None;
    }
    state.paint.mode = mode;
}

/// Leaves the Paint tool for the Select tool, discarding a stroke being drawn and letting the
/// selected stroke go.
pub(crate) fn leave_paint_tool(state: &mut EditorState) {
    if state.tool == Tool::Paint {
        state.tool = Tool::Select;
    }
    discard_stroke(state);
}

/// Throws away the stroke being drawn, if any, and lets the selected stroke go, as the Paint
/// tool is left. A drag of a stroke under way goes on to its release, so its gesture ends.
pub(crate) fn discard_stroke(state: &mut EditorState) {
    state.paint.stroke.clear();
    state.paint.selected = None;
    if state.interaction == Interaction::Painting && state.paint.drag.is_none() {
        state.interaction = Interaction::Idle;
    }
}

/// The Terrains the Paint tool paints onto and the images they show.
#[derive(SystemParam)]
pub(crate) struct Terrains<'w, 's> {
    /// Each Layer's Elements in stacking order.
    layers: Query<'w, 's, &'static Children, With<Layer>>,
    /// Every Terrain.
    painted: Query<'w, 's, (&'static ElementId, &'static Terrain)>,
    /// Each Project's Asset References and where they resolve to.
    projects: Query<'w, 's, (&'static AssetReferences, &'static ResolutionTable), With<Project>>,
}

impl Terrains<'_, '_> {
    /// The topmost Terrain on a Layer: the one a Paint there paints onto.
    pub(crate) fn on(&self, layer: Option<Entity>) -> Option<(ElementId, &Terrain)> {
        let children = self.layers.get(layer?).ok()?;
        children.iter().rev().find_map(|child| {
            let (id, terrain) = self.painted.get(*child).ok()?;
            Some((*id, terrain))
        })
    }

    /// The name of the image a Terrain shows.
    fn image_name(&self, terrain: &Terrain) -> Option<String> {
        self.projects.iter().find_map(|(references, _)| {
            references
                .get(terrain.image)
                .map(|reference| reference.name.clone())
        })
    }

    /// Whether a Terrain shows the image of an Asset, as this device finds it.
    fn shows(&self, terrain: &Terrain, asset: &AssetAddress) -> bool {
        self.projects.iter().any(|(_, resolutions)| {
            matches!(
                resolutions.get(terrain.image),
                Some(Resolution::Resolved { folder, place })
                    if *folder == asset.folder && *place == asset.place
            )
        })
    }
}

/// The distance in cells from `cells` to the nearest point of a path: of its segments, or of its
/// single point.
fn distance_to_path(points: &[Vec2], cells: Vec2) -> f32 {
    match points {
        [] => f32::INFINITY,
        [point] => point.distance(cells),
        points => points
            .windows(2)
            .map(|pair| {
                let (from, along) = (pair[0], pair[1] - pair[0]);
                let length = along.length_squared();
                let t = if length > 0.0 {
                    ((cells - from).dot(along) / length).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                (from + along * t).distance(cells)
            })
            .fold(f32::INFINITY, f32::min),
    }
}

/// The latest laid stroke of `terrain` under a point in cells at `zoom`: its path no farther
/// from the point than its radius or four screen pixels, whichever is more, an erase as much as a
/// paint.
fn stroke_at(terrain: &Terrain, cells: Vec2, zoom: f32) -> Option<usize> {
    terrain
        .strokes
        .iter()
        .enumerate()
        .rev()
        .find(|(_, stroke)| {
            let reach = stroke.brush.radius().max(PATH_PIXELS / zoom);
            distance_to_path(&stroke.points, cells) <= reach
        })
        .map(|(index, _)| index)
}

/// The point of a stroke's path nearest a point in cells within a handle's reach at `zoom`.
fn handle_at(stroke: &Stroke, cells: Vec2, zoom: f32) -> Option<(usize, Vec2)> {
    stroke
        .points
        .iter()
        .copied()
        .enumerate()
        .map(|(index, point)| (index, point, point.distance(cells)))
        .filter(|(_, _, distance)| distance * zoom <= HANDLE_PIXELS)
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(index, point, _)| (index, point))
}

/// The selected stroke, when it is a stroke of `terrain`, the current Layer's topmost Terrain.
fn selected_in<'t>(
    state: &EditorState,
    terrain: Option<(ElementId, &'t Terrain)>,
) -> Option<(StrokeRef, &'t Stroke)> {
    let selected = state.paint.selected?;
    let (id, terrain) = terrain?;
    (id == selected.terrain)
        .then(|| terrain.strokes.get(selected.stroke))
        .flatten()
        .map(|stroke| (selected, stroke))
}

/// A left press with the Paint tool on the Level at `cursor`. Painting or erasing, it starts a
/// stroke that does what the tool does now, or, with nothing to paint with or erase from, lays
/// nothing and says why. Editing strokes, it arms a drag of the selected stroke's handle under
/// the pointer, or selects the latest laid stroke of the current Layer's Terrain under it and
/// arms a drag of it, or, where no stroke lies, lets the selected stroke go.
pub(crate) fn press(
    state: &mut EditorState,
    terrain: Option<(ElementId, &Terrain)>,
    viewport: &Viewport,
    cursor: Vec2,
) {
    let cells = viewport.cells_at(cursor);
    match state.paint.mode {
        Mode::Painting | Mode::Erasing => {
            let erasing = state.paint.mode == Mode::Erasing;
            if erasing && terrain.is_none() {
                "Nothing to erase: this Layer has no Terrain".clone_into(&mut state.status);
                return;
            }
            if !erasing && state.chosen.is_none() && terrain.is_none() {
                "Choose an Asset to paint with: this Layer has no Terrain yet"
                    .clone_into(&mut state.status);
                return;
            }
            state.paint.stroke = vec![cells];
            state.paint.erasing = erasing;
            state.interaction = Interaction::Painting;
        }
        Mode::EditingStrokes => {
            let handle = selected_in(state, terrain).and_then(|(target, stroke)| {
                handle_at(stroke, cells, viewport.zoom).map(|(index, at)| (target, index, at))
            });
            let (target, dragged, origin) = if let Some((target, index, at)) = handle {
                (target, Dragged::Point(index), at)
            } else if let Some((id, terrain)) = terrain
                && let Some(index) = stroke_at(terrain, cells, viewport.zoom)
            {
                let target = StrokeRef {
                    terrain: id,
                    stroke: index,
                };
                (target, Dragged::Stroke, terrain.strokes[index].centre())
            } else {
                state.paint.selected = None;
                return;
            };
            state.paint.selected = Some(target);
            state.paint.drag = Some(StrokeDrag {
                target,
                dragged,
                origin,
                pointer: cells,
                screen: cursor,
                sent: None,
            });
            state.interaction = Interaction::Painting;
        }
    }
}

/// Follows the pointer at `cursor` while the button is held: a stroke being drawn takes the
/// pointer into its path once it is more than an eighth of the Brush's size from the last point
/// added, and a press on a stroke or a handle that has travelled far enough to be a drag moves
/// the stroke or the point with the pointer, the first move beginning the gesture and every later
/// one continuing it.
pub(crate) fn moved(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    viewport: &Viewport,
    cursor: Vec2,
) {
    let cells = viewport.cells_at(cursor);
    if let Some(drag) = &mut state.paint.drag {
        let dragging = drag.sent.is_some() || (cursor - drag.screen).length() > DRAG_THRESHOLD;
        if !dragging {
            return;
        }
        let position = drag.origin + (cells - drag.pointer);
        let change = ElementChange::Stroke {
            stroke: drag.target.stroke,
            change: match drag.dragged {
                Dragged::Point(index) => StrokeChange::Point { index, position },
                Dragged::Stroke => StrokeChange::Position(position),
            },
        };
        if drag.sent.as_ref() == Some(&change) {
            return;
        }
        apply.write(Apply::EditElement(EditElement {
            element: drag.target.terrain,
            change: change.clone(),
            gesture: if drag.sent.is_some() {
                Gesture::Continue
            } else {
                Gesture::Begin
            },
        }));
        drag.sent = Some(change);
        return;
    }
    let spacing = state.paint.brush.size * SPACING;
    let far_enough = state
        .paint
        .stroke
        .last()
        .is_none_or(|last| last.distance(cells) > spacing);
    if far_enough {
        state.paint.stroke.push(cells);
    }
}

/// Ends the Paint tool's gesture once the button is up. A drag of a stroke or a point sends its
/// last change again as the gesture's end, so the whole drag is one step; a press that never
/// became a drag just ends. A stroke being drawn ends at the release point, when there is one
/// and it does not lie on the last point, and is sent as one Paint onto `layer` with the Brush's
/// settings and whether it erases, a stroke that paints naming the chosen Asset or, with none,
/// no image, and an erase naming none. The tool stays chosen in the mode it is in.
pub(crate) fn release(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    layer: Option<Entity>,
    at: Option<Vec2>,
) {
    state.interaction = Interaction::Idle;
    if let Some(drag) = state.paint.drag.take() {
        if let Some(change) = drag.sent {
            apply.write(Apply::EditElement(EditElement {
                element: drag.target.terrain,
                change,
                gesture: Gesture::End,
            }));
        }
        return;
    }
    let mut points = std::mem::take(&mut state.paint.stroke);
    if let Some(at) = at
        && points.last().is_some_and(|last| *last != at)
    {
        points.push(at);
    }
    if points.is_empty() {
        return;
    }
    let erase = state.paint.erasing;
    if let Some(layer) = layer {
        apply.write(Apply::Paint(Paint {
            layer,
            stroke: Stroke {
                points,
                brush: state.paint.brush,
                erase,
            },
            asset: if erase {
                None
            } else {
                state.chosen.as_ref().map(|chosen| chosen.asset.clone())
            },
        }));
    }
}

/// Delete with the Paint tool editing strokes: sends an Edit Element removing the selected
/// stroke, as a step of its own, and lets it go. Nothing happens while a step is being made.
pub(crate) fn remove_selected(state: &mut EditorState, apply: &mut MessageWriter<Apply>) {
    if state.tool != Tool::Paint
        || state.paint.mode != Mode::EditingStrokes
        || state.step_under_way()
    {
        return;
    }
    if let Some(selected) = state.paint.selected.take() {
        apply.write(Apply::EditElement(EditElement {
            element: selected.terrain,
            change: ElementChange::Stroke {
                stroke: selected.stroke,
                change: StrokeChange::Remove,
            },
            gesture: Gesture::Single,
        }));
    }
}

/// The Paint tool's options: Paint, Erase, and Edit strokes, the mode it is in chosen; the
/// Brush's size in cells and its hardness and strength as percentages, or with a stroke selected
/// the stroke's, and whether it paints or erases; and the name of the image it paints with, with,
/// when an Asset is chosen and the Layer's Terrain shows another image, a button that makes the
/// Terrain show the chosen Asset.
pub(crate) fn options(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    terrain: Option<(ElementId, &Terrain)>,
    terrains: &Terrains,
    apply: &mut MessageWriter<Apply>,
) {
    let mode = state.paint.mode;
    let ctx = ui.ctx().clone();
    for (choice, label, keys) in [
        (Mode::Painting, "Paint", bindings::PAINT_TOOL),
        (Mode::Erasing, "Erase", bindings::ERASE_TOOL),
        (Mode::EditingStrokes, "Edit strokes", &[][..]),
    ] {
        let button = ui.add(egui::Button::selectable(mode == choice, label));
        let button = if keys.is_empty() {
            button
        } else {
            button.on_hover_text(bindings::shortcut_text(&ctx, keys))
        };
        if button.clicked() {
            choose_mode(state, choice);
        }
    }
    ui.separator();
    match selected_in(state, terrain) {
        Some((selected, stroke)) if state.paint.mode == Mode::EditingStrokes => {
            stroke_options(ui, state, selected, stroke, apply);
        }
        _ => {
            walls::end_option(&mut state.paint.option, apply);
            brush_options(ui, &mut state.paint.brush);
        }
    }
    ui.separator();
    let shown = terrain.and_then(|(_, terrain)| terrains.image_name(terrain));
    let name = state
        .chosen
        .as_ref()
        .map(|chosen| chosen.name.clone())
        .or_else(|| shown.clone());
    match name {
        Some(name) => ui.label(format!("Paints with {name}")),
        None => ui.weak("Choose an Asset to paint with"),
    };
    if let (Some(chosen), Some((element, terrain))) = (&state.chosen, terrain)
        && !terrains.shows(terrain, &chosen.asset)
        && ui
            .button("Use for the Terrain")
            .on_hover_text(format!(
                "Make this Layer's Terrain show {} instead of {}, every stroke kept",
                chosen.name,
                shown.unwrap_or_default()
            ))
            .clicked()
    {
        apply.write(Apply::EditElement(EditElement {
            element,
            change: ElementChange::Material(chosen.asset.clone()),
            gesture: Gesture::Single,
        }));
    }
}

/// The Brush's size, hardness, and strength, written into the Brush.
fn brush_options(ui: &mut egui::Ui, brush: &mut BrushSettings) {
    ui.label("Size");
    ui.add(
        egui::DragValue::new(&mut brush.size)
            .range(SMALLEST..=LARGEST)
            .speed(0.05)
            .max_decimals(2)
            .suffix(" cells"),
    );
    ui.label("Hardness");
    let mut hardness = brush.hardness * 100.0;
    if ui
        .add(
            egui::DragValue::new(&mut hardness)
                .range(0.0..=100.0)
                .speed(0.5)
                .max_decimals(0)
                .suffix(" %"),
        )
        .changed()
    {
        brush.hardness = hardness / 100.0;
    }
    ui.label("Strength");
    let mut strength = brush.strength * 100.0;
    if ui
        .add(
            egui::DragValue::new(&mut strength)
                .range(WEAKEST..=100.0)
                .speed(0.5)
                .max_decimals(0)
                .suffix(" %"),
        )
        .changed()
    {
        brush.strength = strength / 100.0;
    }
}

/// The selected stroke's size, hardness, and strength, sent to it as one change of its Brush
/// settings, one dragged as one gesture and one typed as one step, and a pair of buttons that
/// turn it to painting or erasing, each a step; the Brush stays as it was.
fn stroke_options(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    selected: StrokeRef,
    stroke: &Stroke,
    apply: &mut MessageWriter<Apply>,
) {
    let mut brush = stroke.brush;
    ui.label("Size");
    // The range has no lower end, as egui clamps typed values into it too: a drag is kept at the
    // smallest Brush here instead, and a typed size of zero or less is sent as typed, so that it
    // is refused with the reason.
    let size = ui.add(
        egui::DragValue::new(&mut brush.size)
            .range(f32::NEG_INFINITY..=LARGEST)
            .clamp_existing_to_range(false)
            .speed(0.05)
            .max_decimals(2)
            .suffix(" cells")
            .update_while_editing(false),
    );
    if size.dragged() {
        brush.size = brush.size.max(SMALLEST);
    }
    ui.label("Hardness");
    let mut hardness = brush.hardness * 100.0;
    let hard = ui.add(
        egui::DragValue::new(&mut hardness)
            .range(0.0..=100.0)
            .clamp_existing_to_range(false)
            .speed(0.5)
            .max_decimals(0)
            .suffix(" %")
            .update_while_editing(false),
    );
    if hard.changed() {
        brush.hardness = hardness / 100.0;
    }
    ui.label("Strength");
    let mut strength = brush.strength * 100.0;
    let strong = ui.add(
        egui::DragValue::new(&mut strength)
            .range(WEAKEST..=100.0)
            .clamp_existing_to_range(false)
            .speed(0.5)
            .max_decimals(0)
            .suffix(" %")
            .update_while_editing(false),
    );
    if strong.changed() {
        brush.strength = strength / 100.0;
    }
    let held = size.dragged() || hard.dragged() || strong.dragged();
    if size.changed() || hard.changed() || strong.changed() {
        walls::send_option(
            &mut state.paint.option,
            apply,
            selected.terrain,
            ElementChange::Stroke {
                stroke: selected.stroke,
                change: StrokeChange::Brush(brush),
            },
            held,
        );
    } else if !held {
        walls::end_option(&mut state.paint.option, apply);
    }
    ui.separator();
    for (erase, label) in [(false, "Paints"), (true, "Erases")] {
        if ui
            .add(egui::Button::selectable(stroke.erase == erase, label))
            .clicked()
            && stroke.erase != erase
        {
            apply.write(Apply::EditElement(EditElement {
                element: selected.terrain,
                change: ElementChange::Stroke {
                    stroke: selected.stroke,
                    change: StrokeChange::Erase(erase),
                },
                gesture: Gesture::Single,
            }));
        }
    }
}

/// The status line's hint and report for the Paint tool: that a drag paints, naming the chosen
/// Asset as the one painted with, that a drag erases, or that a click picks a stroke, a drag
/// reshapes it, and Delete removes it, each with Escape stopping.
pub(crate) fn status(ui: &mut egui::Ui, state: &EditorState) {
    match state.paint.mode {
        Mode::Painting => {
            ui.weak("Drag paints, Escape stops");
            ui.label(state.chosen.as_ref().map_or_else(
                || "painting".to_owned(),
                |chosen| format!("painting with {}", chosen.name),
            ));
        }
        Mode::Erasing => {
            ui.weak("Drag erases, Escape stops");
            ui.label("erasing");
        }
        Mode::EditingStrokes => {
            ui.weak("Click picks a stroke, drag reshapes it, Delete removes it, Escape stops");
            ui.label(state.paint.selected.map_or_else(
                || "editing strokes".to_owned(),
                |selected| format!("stroke {} selected", selected.stroke + 1),
            ));
        }
    }
}

/// Draws a path as a translucent band `radius` wide either side, with round ends.
fn band(canvas: &egui::Painter, path: &[egui::Pos2], radius: f32, colour: egui::Color32) {
    let (Some(first), Some(last)) = (path.first(), path.last()) else {
        return;
    };
    canvas.circle_filled(*first, radius, colour);
    if path.len() > 1 {
        canvas.add(egui::Shape::line(
            path.to_vec(),
            egui::Stroke::new(radius * 2.0, colour),
        ));
        canvas.circle_filled(*last, radius, colour);
    }
}

/// Keeps the Paint tool's selection true to the Level: the selected stroke is let go on every
/// undo and redo, when the tool no longer edits strokes, and when the current Layer's Terrain no
/// longer has it, and an option of it held while it changes ends its gesture once the stroke is
/// let go.
pub(crate) fn keep_selection(
    mut state: ResMut<EditorState>,
    level: LevelView,
    mut undo: MessageReader<Undo>,
    mut redo: MessageReader<Redo>,
    mut apply: MessageWriter<Apply>,
) {
    let history = undo.read().count() + redo.read().count() > 0;
    let editing = state.tool == Tool::Paint && state.paint.mode == Mode::EditingStrokes;
    let shown = editing && selected_in(&state, level.current_terrain()).is_some();
    if history || !shown {
        state.paint.selected = None;
        walls::end_option(&mut state.paint.option, &mut apply);
    }
}

/// Draws what the Paint tool shows over the Level.
///
/// Painting or erasing, the stroke being drawn is a translucent band as wide as the Brush along
/// its path to the pointer, with round ends, and a circle as large as the Brush follows the
/// pointer, both in a warm red while erasing. Editing strokes, the selected stroke is a band as
/// wide as its Brush, in the erase colour when it erases, with its path as a thin line and a
/// handle at each point. Nothing is drawn while an Export runs.
pub(crate) fn overlay(
    mut contexts: EguiContexts,
    state: Res<EditorState>,
    viewport: Res<Viewport>,
    window: Single<&Window, With<PrimaryWindow>>,
    level: LevelView,
) {
    if state.exporting || state.tool != Tool::Paint {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let area = egui::Rect::from_min_max(
        egui::pos2(viewport.area.min.x, viewport.area.min.y),
        egui::pos2(viewport.area.max.x, viewport.area.max.y),
    );
    let canvas = ctx
        .layer_painter(egui::LayerId::new(
            egui::Order::Middle,
            egui::Id::new("paint-overlay"),
        ))
        .with_clip_rect(area);
    let on_screen = |cells: Vec2| {
        let point = viewport.screen_at(cells);
        egui::pos2(point.x, point.y)
    };
    let shown = if state.paint.mode == Mode::EditingStrokes {
        selected_in(&state, level.current_terrain())
    } else {
        None
    };
    if let Some((selected, stroke)) = shown {
        let (line, colour) = if stroke.erase {
            (ERASE_CIRCLE, ERASE_BAND)
        } else {
            (CIRCLE, BAND)
        };
        let path: Vec<egui::Pos2> = stroke.points.iter().copied().map(on_screen).collect();
        band(
            &canvas,
            &path,
            stroke.brush.radius() * viewport.zoom,
            colour,
        );
        if path.len() > 1 {
            canvas.add(egui::Shape::line(
                path.clone(),
                egui::Stroke::new(1.5, line),
            ));
        }
        let dragged = state
            .paint
            .dragged_point()
            .and_then(|(target, index)| (target == selected).then_some(index));
        for (index, point) in path.iter().enumerate() {
            let handle = if dragged == Some(index) { PICKED } else { line };
            canvas.circle_stroke(*point, HANDLE_PIXELS, egui::Stroke::new(1.5, handle));
        }
    }
    if state.paint.mode == Mode::EditingStrokes {
        return;
    }
    let circle = if state.paint.mode == Mode::Erasing {
        ERASE_CIRCLE
    } else {
        CIRCLE
    };
    let radius = state.paint.brush.radius() * viewport.zoom;
    let pointer = window
        .cursor_position()
        .filter(|cursor| viewport.contains(*cursor))
        .map(|cursor| viewport.cells_at(cursor));
    if !state.paint.stroke.is_empty() {
        let drawn = if state.paint.erasing {
            ERASE_BAND
        } else {
            BAND
        };
        let path: Vec<egui::Pos2> = state
            .paint
            .stroke
            .iter()
            .copied()
            .chain(pointer)
            .map(on_screen)
            .collect();
        band(&canvas, &path, radius, drawn);
    }
    if let Some(pointer) = pointer {
        canvas.circle_stroke(on_screen(pointer), radius, egui::Stroke::new(1.5, circle));
    }
}

/// Logs the Paint tool, its mode, the selected stroke, and the handle being dragged, and for
/// every Terrain each stroke's number, whether it erases, its Brush settings, and its number of
/// points, for a script to aim at a stroke and check what painting and editing did.
#[cfg(feature = "dev")]
pub(crate) fn describe(
    state: &EditorState,
    terrains: &Query<(&ElementId, &drs_model::Element, &Terrain)>,
) {
    bevy::log::info!(
        "describe: paint tool chosen {}, mode {:?}, brush size {} hardness {} strength {}, \
         stroke of {} points erasing {}, selected stroke {:?}, dragged {:?}",
        state.tool == Tool::Paint,
        state.paint.mode,
        state.paint.brush.size,
        state.paint.brush.hardness,
        state.paint.brush.strength,
        state.paint.stroke.len(),
        state.paint.erasing,
        state
            .paint
            .selected
            .map(|selected| (selected.terrain.as_raw(), selected.stroke)),
        state.paint.drag.as_ref().map(|drag| drag.dragged)
    );
    for (id, element, terrain) in terrains {
        bevy::log::info!(
            "describe: terrain {} at {} size {}, image row {}, {} strokes",
            id.as_raw(),
            element.position,
            element.size,
            terrain.image.0,
            terrain.strokes.len()
        );
        for (number, stroke) in terrain.strokes.iter().enumerate() {
            bevy::log::info!(
                "describe: terrain {} stroke {number} erase {} size {} hardness {} strength {}, \
                 {} points {:?}",
                id.as_raw(),
                stroke.erase,
                stroke.brush.size,
                stroke.brush.hardness,
                stroke.brush.strength,
                stroke.points.len(),
                stroke.points
            );
        }
    }
}
