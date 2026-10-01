//! Interaction in the viewport: placing, selecting, dragging, removing, panning, and zooming.
//!
//! Pointer positions come from the window in logical pixels and go through the model's
//! `Viewport` to Level cells, the same conversion the render Engine draws by. Panning and zooming
//! write the Viewport only; everything that changes the Level is a Command sent to the authoring
//! Manager.

use crate::state::{EditorState, Interaction};
use bevy::color::Color;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::ecs::query::With;
use bevy::ecs::system::{Query, Res, ResMut, Single, SystemParam};
use bevy::gizmos::gizmos::Gizmos;
use bevy::input::ButtonInput;
use bevy::input::gestures::PinchGesture;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseButton, MouseScrollUnit};
use bevy::math::{Isometry2d, Rect, Vec2, ops};
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::input::EguiWantsInput;
use drs_model::{
    Apply, EditElement, Element, ElementChange, ElementId, Gesture, Layer, Level, PlaceElement,
    Redo, RemoveElement, Undo, Viewport,
};

/// How far the pointer travels, in pixels, before a press on a Prop becomes a drag.
const DRAG_THRESHOLD: f32 = 3.0;
/// The zoom factor of one line of a mouse wheel.
const WHEEL_STEP: f32 = 1.1;
/// The zoom factor of one pixel of a modified trackpad scroll.
const PIXEL_STEP: f32 = 1.01;
/// The colour of the selection outline.
const SELECTION: Color = Color::srgb(0.35, 0.75, 1.0);

/// The pointer and the keys, and whether egui is using them.
#[derive(SystemParam)]
pub(crate) struct Input<'w, 's> {
    /// The window, for the pointer's position.
    window: Single<'w, 's, &'static Window, With<PrimaryWindow>>,
    /// The mouse buttons.
    buttons: Res<'w, ButtonInput<MouseButton>>,
    /// The keys, for modifiers.
    keys: Res<'w, ButtonInput<KeyCode>>,
    /// This frame's scrolling.
    scroll: Res<'w, AccumulatedMouseScroll>,
    /// This frame's pinching.
    pinches: MessageReader<'w, 's, PinchGesture>,
    /// Whether egui wants the pointer or the keyboard, so the viewport leaves them alone.
    egui: Res<'w, EguiWantsInput>,
}

/// The Level as the viewport reads it: the Layers in order and the Elements on them.
#[derive(SystemParam)]
pub(crate) struct LevelView<'w, 's> {
    /// Each Level's Layers in stacking order.
    levels: Query<'w, 's, &'static Children, With<Level>>,
    /// Each Layer's Elements in stacking order.
    layers: Query<'w, 's, (Entity, &'static Children), With<Layer>>,
    /// Every Layer, for the one to place on.
    any_layer: Query<'w, 's, Entity, With<Layer>>,
    /// Every Element's identity and shape.
    elements: Query<'w, 's, (&'static ElementId, &'static Element)>,
}

impl LevelView<'_, '_> {
    /// The Layer new Props are placed on: the first one.
    fn current_layer(&self) -> Option<Entity> {
        self.any_layer.iter().next()
    }

    /// The topmost Prop under a point in cells, with its centre: Layers from the top down, and
    /// each Layer's Elements from the last drawn back.
    fn topmost_at(&self, cells: Vec2) -> Option<(ElementId, Vec2)> {
        self.levels.iter().find_map(|layers| {
            layers.iter().rev().find_map(|&layer| {
                let (_, elements) = self.layers.get(layer).ok()?;
                elements.iter().rev().find_map(|&element| {
                    let (id, element) = self.elements.get(element).ok()?;
                    Rect::from_center_size(element.position, element.size)
                        .contains(cells)
                        .then_some((*id, element.position))
                })
            })
        })
    }

    /// The centre of the Element with an identity, if it exists.
    fn position_of(&self, id: ElementId) -> Option<Vec2> {
        self.elements
            .iter()
            .find(|(candidate, _)| **candidate == id)
            .map(|(_, element)| element.position)
    }
}

/// Carries the pointer gesture of the frame out: a click places or selects, a drag moves the
/// selected Prop as one gesture, the middle button or Space drags the view, scrolling pans, and
/// a wheel, a pinch, or a modified scroll zooms around the pointer.
///
/// A gesture starts only with the pointer over the viewport and egui not using it; one under
/// way ends wherever the button is released, so no Begin is left without its End.
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Bevy system takes its parameters by value"
)]
pub(crate) fn pointer(
    mut input: Input,
    mut state: ResMut<EditorState>,
    mut viewport: ResMut<Viewport>,
    level: LevelView,
    mut apply: MessageWriter<Apply>,
) {
    let Some(cursor) = input.window.cursor_position() else {
        finish_gesture(&mut state, &mut apply, &viewport, &level, &input);
        return;
    };
    let over = viewport.contains(cursor) && !input.egui.wants_any_pointer_input();
    if over {
        zoom_and_scroll(&mut input, &mut viewport, cursor);
    }
    let pan_held = input.buttons.pressed(MouseButton::Middle)
        || (input.buttons.pressed(MouseButton::Left) && input.keys.pressed(KeyCode::Space));
    match state.interaction {
        Interaction::Idle => {
            if !over {
                return;
            }
            let pan_pressed = input.buttons.just_pressed(MouseButton::Middle)
                || (input.buttons.just_pressed(MouseButton::Left)
                    && input.keys.pressed(KeyCode::Space));
            if pan_pressed {
                state.interaction = Interaction::Panning { last: cursor };
            } else if input.buttons.just_pressed(MouseButton::Left) {
                press(&mut state, &mut apply, &viewport, &level, cursor);
            }
        }
        Interaction::Panning { last } => {
            if pan_held {
                viewport.pan_by(cursor - last);
                state.interaction = Interaction::Panning { last: cursor };
            } else {
                state.interaction = Interaction::Idle;
            }
        }
        Interaction::Pressed {
            element,
            origin,
            pointer,
            moved_at,
        } => {
            if !input.buttons.pressed(MouseButton::Left) {
                finish_gesture(&mut state, &mut apply, &viewport, &level, &input);
                return;
            }
            let dragging = moved_at.is_some() || (cursor - pointer).length() > DRAG_THRESHOLD;
            if dragging && moved_at != Some(cursor) {
                let position = origin + (viewport.cells_at(cursor) - viewport.cells_at(pointer));
                apply.write(Apply::EditElement(EditElement {
                    element,
                    change: ElementChange::Position(position),
                    gesture: if moved_at.is_some() {
                        Gesture::Continue
                    } else {
                        Gesture::Begin
                    },
                }));
                state.interaction = Interaction::Pressed {
                    element,
                    origin,
                    pointer,
                    moved_at: Some(cursor),
                };
            }
        }
    }
}

/// A left press over the viewport: places the chosen Asset centred on the pointer, or selects
/// the topmost Prop under it and arms a drag; empty space clears the selection.
fn press(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    viewport: &Viewport,
    level: &LevelView,
    cursor: Vec2,
) {
    let cells = viewport.cells_at(cursor);
    if let Some(chosen) = &state.chosen {
        if let Some(layer) = level.current_layer() {
            apply.write(Apply::PlaceElement(PlaceElement {
                layer,
                position: cells,
                asset: chosen.asset.clone(),
            }));
        }
        return;
    }
    let hit = level.topmost_at(cells);
    state.selected = hit.map(|(id, _)| id);
    if let Some((element, origin)) = hit {
        state.interaction = Interaction::Pressed {
            element,
            origin,
            pointer: cursor,
            moved_at: None,
        };
    }
}

/// Ends a drag that is under way once its button is up: the last position is sent as the end of
/// the gesture so the whole drag is one history step. A press that never became a drag just ends.
fn finish_gesture(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    viewport: &Viewport,
    level: &LevelView,
    input: &Input,
) {
    match state.interaction {
        Interaction::Idle => {}
        Interaction::Panning { .. } => {
            if !input.buttons.pressed(MouseButton::Middle)
                && !input.buttons.pressed(MouseButton::Left)
            {
                state.interaction = Interaction::Idle;
            }
        }
        Interaction::Pressed {
            element,
            origin,
            pointer,
            moved_at,
        } => {
            if input.buttons.pressed(MouseButton::Left) {
                return;
            }
            if let Some(last) = moved_at {
                let position = level.position_of(element).unwrap_or_else(|| {
                    origin + (viewport.cells_at(last) - viewport.cells_at(pointer))
                });
                apply.write(Apply::EditElement(EditElement {
                    element,
                    change: ElementChange::Position(position),
                    gesture: Gesture::End,
                }));
            }
            state.interaction = Interaction::Idle;
        }
    }
}

/// Zooms around the pointer for a wheel, a pinch, or a scroll with Command or Control held, and
/// pans for a plain trackpad scroll.
fn zoom_and_scroll(input: &mut Input, viewport: &mut Viewport, cursor: Vec2) {
    let mut factor = 1.0;
    let mut zoomed = false;
    for pinch in input.pinches.read() {
        factor *= 1.0 + pinch.0;
        zoomed = true;
    }
    let delta = input.scroll.delta;
    if delta != Vec2::ZERO {
        let modified = input.keys.any_pressed([
            KeyCode::SuperLeft,
            KeyCode::SuperRight,
            KeyCode::ControlLeft,
            KeyCode::ControlRight,
        ]);
        match input.scroll.unit {
            MouseScrollUnit::Line => {
                factor *= ops::powf(WHEEL_STEP, delta.y);
                zoomed = true;
            }
            MouseScrollUnit::Pixel if modified => {
                factor *= ops::powf(PIXEL_STEP, delta.y);
                zoomed = true;
            }
            MouseScrollUnit::Pixel => viewport.pan_by(delta),
        }
    }
    if zoomed {
        viewport.zoom_by(factor, cursor);
    }
}

/// The keys: Escape stops placing, Delete and Backspace remove the selected Prop, and the
/// platform's usual shortcuts undo and redo. Nothing happens while egui has the keyboard, so a
/// text field keeps its own editing keys.
#[expect(
    clippy::needless_pass_by_value,
    reason = "a Bevy system takes its parameters by value"
)]
pub(crate) fn keys(
    keys: Res<ButtonInput<KeyCode>>,
    egui: Res<EguiWantsInput>,
    mut state: ResMut<EditorState>,
    mut apply: MessageWriter<Apply>,
    mut undo: MessageWriter<Undo>,
    mut redo: MessageWriter<Redo>,
) {
    if egui.wants_any_keyboard_input() {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) && state.chosen.is_some() {
        state.chosen = None;
    }
    if keys.any_just_pressed([KeyCode::Delete, KeyCode::Backspace])
        && let Some(element) = state.selected.take()
    {
        apply.write(Apply::RemoveElement(RemoveElement { element }));
    }
    let command = if cfg!(target_os = "macos") {
        keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
    } else {
        keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
    };
    if !command {
        return;
    }
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if keys.just_pressed(KeyCode::KeyZ) {
        if shift {
            redo.write(Redo);
        } else {
            undo.write(Undo);
        }
    }
    if keys.just_pressed(KeyCode::KeyY) && !cfg!(target_os = "macos") {
        redo.write(Redo);
    }
}

/// Outlines the selected Prop, and drops a selection whose Prop is gone.
pub(crate) fn outline_selection(
    mut gizmos: Gizmos,
    mut state: ResMut<EditorState>,
    elements: Query<(&ElementId, &Element)>,
) {
    let Some(selected) = state.selected else {
        return;
    };
    match elements.iter().find(|(id, _)| **id == selected) {
        Some((_, element)) => gizmos.rect_2d(
            Isometry2d::from_translation(element.position),
            element.size,
            SELECTION,
        ),
        None => state.selected = None,
    }
}
