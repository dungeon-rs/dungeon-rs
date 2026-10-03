//! The Paint tool: the Brush and its options, the stroke being drawn, and what the tool shows
//! over the viewport, the Brush's circle at the pointer and the stroke as a translucent band.
//!
//! The stroke being drawn is the Editor's own state until the release, when it becomes one Paint;
//! the Brush's settings are the Editor's too, never a history step and never saved.

use crate::state::{EditorState, Interaction, Tool};
use crate::walls;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::query::With;
use bevy::ecs::system::{Query, Res, Single, SystemParam};
use bevy::math::Vec2;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::EguiContexts;
use drs_model::{
    Apply, AssetAddress, AssetReferences, BrushSettings, EditElement, ElementChange, ElementId,
    Gesture, Layer, Paint, Project, Resolution, ResolutionTable, Stroke, Terrain, Viewport,
};

/// The Brush a new editor starts with: two cells across, half hard, at full strength.
const FIRST_BRUSH: BrushSettings = BrushSettings {
    size: 2.0,
    hardness: 0.5,
    strength: 1.0,
};
/// The smallest Brush the options offer, in cells.
const SMALLEST: f32 = 0.1;
/// The largest Brush the options offer, in cells.
const LARGEST: f32 = 64.0;
/// The weakest strength the options offer, as a percentage.
const WEAKEST: f32 = 1.0;
/// How far the pointer moves, as a part of the Brush's size, before it adds a point to the path.
const SPACING: f32 = 1.0 / 8.0;
/// The colour of the Brush's circle.
const CIRCLE: egui::Color32 = egui::Color32::from_rgb(90, 190, 255);
/// The colour of the band a stroke being drawn is shown as: translucent.
const BAND: egui::Color32 = egui::Color32::from_rgba_unmultiplied_const(90, 190, 255, 90);

/// The Paint tool's state.
#[derive(Debug)]
pub(crate) struct PaintTool {
    /// The settings the next stroke is laid with.
    pub brush: BrushSettings,
    /// The path of the stroke being drawn, in cells; empty when none is.
    pub stroke: Vec<Vec2>,
}

impl Default for PaintTool {
    /// The first Brush and no stroke.
    fn default() -> Self {
        Self {
            brush: FIRST_BRUSH,
            stroke: Vec::new(),
        }
    }
}

impl PaintTool {
    /// Whether a stroke is being drawn.
    pub(crate) fn drawing_in_progress(&self) -> bool {
        !self.stroke.is_empty()
    }
}

/// Chooses the Paint tool: the Wall, the Portal, or the Room tool is left, discarding a Wall or an
/// outline being drawn, and the selection is dropped; a chosen Asset stays, as the image the Brush
/// paints with.
pub(crate) fn choose_paint_tool(state: &mut EditorState) {
    walls::leave_tool(state);
    state.selected = None;
    state.handle = None;
    state.tool = Tool::Paint;
}

/// Leaves the Paint tool for the Select tool, discarding a stroke being drawn.
pub(crate) fn leave_paint_tool(state: &mut EditorState) {
    if state.tool == Tool::Paint {
        state.tool = Tool::Select;
    }
    discard_stroke(state);
}

/// Throws away the stroke being drawn, if any.
pub(crate) fn discard_stroke(state: &mut EditorState) {
    state.paint.stroke.clear();
    if state.interaction == Interaction::Painting {
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

/// A left press with the Paint tool: starts a stroke at `cells`, or, with no Asset chosen and no
/// Terrain on the Layer to paint more onto, paints nothing and asks for an Asset.
pub(crate) fn press(state: &mut EditorState, terrain: bool, cells: Vec2) {
    if state.chosen.is_none() && !terrain {
        "Choose an Asset to paint with: this Layer has no Terrain yet"
            .clone_into(&mut state.status);
        return;
    }
    state.paint.stroke = vec![cells];
    state.interaction = Interaction::Painting;
}

/// Adds the pointer at `cells` to the stroke being drawn once it is more than an eighth of the
/// Brush's size from the last point added.
pub(crate) fn extend(state: &mut EditorState, cells: Vec2) {
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

/// Ends the stroke being drawn: the release point, when there is one and it does not lie on the
/// last point, ends the path, and the path is sent as one Paint onto `layer` with the Brush's
/// settings, naming the chosen Asset or, with none, no image. The tool stays chosen.
pub(crate) fn release(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    layer: Option<Entity>,
    at: Option<Vec2>,
) {
    state.interaction = Interaction::Idle;
    let mut points = std::mem::take(&mut state.paint.stroke);
    if let Some(at) = at
        && points.last().is_some_and(|last| *last != at)
    {
        points.push(at);
    }
    if points.is_empty() {
        return;
    }
    if let Some(layer) = layer {
        apply.write(Apply::Paint(Paint {
            layer,
            stroke: Stroke {
                points,
                brush: state.paint.brush,
                erase: false,
            },
            asset: state.chosen.as_ref().map(|chosen| chosen.asset.clone()),
        }));
    }
}

/// The Paint tool's options: the Brush's size in cells, its hardness and strength as
/// percentages, the name of the image it paints with, and, when an Asset is chosen and the
/// Layer's Terrain shows another image, a button that makes the Terrain show the chosen Asset.
pub(crate) fn options(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    terrain: Option<(ElementId, &Terrain)>,
    terrains: &Terrains,
    apply: &mut MessageWriter<Apply>,
) {
    let brush = &mut state.paint.brush;
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

/// Draws what the Paint tool shows over the Level: the stroke being drawn as a translucent band
/// as wide as the Brush along its path to the pointer, with round ends, and a circle as large as
/// the Brush at the pointer. Nothing is drawn while an Export runs.
pub(crate) fn overlay(
    mut contexts: EguiContexts,
    state: Res<crate::state::EditorState>,
    viewport: Res<Viewport>,
    window: Single<&Window, With<PrimaryWindow>>,
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
    let radius = state.paint.brush.radius() * viewport.zoom;
    let pointer = window
        .cursor_position()
        .filter(|cursor| viewport.contains(*cursor))
        .map(|cursor| viewport.cells_at(cursor));
    if state.paint.drawing_in_progress() {
        let path: Vec<egui::Pos2> = state
            .paint
            .stroke
            .iter()
            .copied()
            .chain(pointer)
            .map(on_screen)
            .collect();
        if let (Some(first), Some(last)) = (path.first(), path.last()) {
            canvas.circle_filled(*first, radius, BAND);
            if path.len() > 1 {
                canvas.add(egui::Shape::line(
                    path.clone(),
                    egui::Stroke::new(radius * 2.0, BAND),
                ));
                canvas.circle_filled(*last, radius, BAND);
            }
        }
    }
    if let Some(pointer) = pointer {
        canvas.circle_stroke(on_screen(pointer), radius, egui::Stroke::new(1.5, CIRCLE));
    }
}

/// Logs the Paint tool and every Terrain, for a script to check what painting did.
#[cfg(feature = "dev")]
pub(crate) fn describe(
    state: &EditorState,
    terrains: &Query<(&ElementId, &drs_model::Element, &Terrain)>,
) {
    bevy::log::info!(
        "describe: paint tool chosen {}, brush size {} hardness {} strength {}, stroke of {} \
         points",
        state.tool == Tool::Paint,
        state.paint.brush.size,
        state.paint.brush.hardness,
        state.paint.brush.strength,
        state.paint.stroke.len()
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
    }
}
