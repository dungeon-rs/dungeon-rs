//! Interaction in the viewport: placing, drawing Walls, selecting, dragging Elements and the
//! handles of a Wall, removing, panning, and zooming.
//!
//! Pointer positions come from the window in logical pixels and go through the model's
//! `Viewport` to Level cells, the same conversion the render Engine draws by. Panning and zooming
//! write the Viewport only; everything that changes the Level is a Command sent to the authoring
//! Manager.

use crate::bindings;
use crate::state::{EditorState, Interaction, Tool};
use crate::walls;
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
use bevy::time::{Real, Time};
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::input::EguiWantsInput;
use drs_model::{
    Apply, EditElement, Element, ElementChange, ElementId, Gesture, Layer, Level, PlaceElement,
    Placement, Redo, RemoveElement, Undo, Viewport, Wall, WallShape,
};

/// How far the pointer travels, in pixels, before a press on an Element or a handle becomes a
/// drag.
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
    /// Every Element's identity and box, and its Wall and derived shape when it is a Wall.
    elements: Query<
        'w,
        's,
        (
            &'static ElementId,
            &'static Element,
            Option<&'static Wall>,
            Option<&'static WallShape>,
        ),
    >,
}

impl LevelView<'_, '_> {
    /// The Layer new Props are placed on: the Project's only Layer for now; choosing one among
    /// several is a later concern.
    fn current_layer(&self) -> Option<Entity> {
        self.any_layer.iter().next()
    }

    /// The topmost Element under a point in cells at `zoom`, with its centre: Layers from the
    /// top down, and each Layer's Elements from the last drawn back. A Wall is under the point
    /// when its line is near enough, any other Element when its box holds the point.
    fn topmost_at(&self, cells: Vec2, zoom: f32) -> Option<(ElementId, Vec2)> {
        self.levels.iter().find_map(|layers| {
            layers.iter().rev().find_map(|&layer| {
                let (_, elements) = self.layers.get(layer).ok()?;
                elements.iter().rev().find_map(|&element| {
                    let (id, element, wall, shape) = self.elements.get(element).ok()?;
                    let hit = match (wall, shape) {
                        (Some(wall), Some(shape)) => walls::on_wall(wall, shape, cells, zoom),
                        (Some(_), None) => false,
                        (None, _) => {
                            Rect::from_center_size(element.position, element.size).contains(cells)
                        }
                    };
                    hit.then_some((*id, element.position))
                })
            })
        })
    }

    /// The selected Element when it is a Wall, with its derived shape once it has one.
    fn selected_wall(
        &self,
        selected: Option<ElementId>,
    ) -> Option<(ElementId, &Wall, Option<&WallShape>)> {
        let selected = selected?;
        self.elements
            .iter()
            .find(|(id, ..)| **id == selected)
            .and_then(|(id, _, wall, shape)| Some((*id, wall?, shape)))
    }
}

/// Carries the pointer gesture of the frame out: a click adds a point of the Wall being drawn,
/// places, or selects, a drag moves the selected Element or a handle of the selected Wall as one
/// gesture, the middle button or Space drags the view, scrolling pans, and a wheel, a pinch, or
/// a modified scroll zooms around the pointer. An Asset chosen while the Wall tool is chosen
/// leaves the tool, discarding the Wall being drawn.
///
/// A gesture starts only with the pointer over the viewport and egui not using it; one under
/// way ends wherever the button is released, so no Begin is left without its End. While an
/// Export runs the pointer is ignored, so the image is of the Level as it was asked for.
pub(crate) fn pointer(
    mut input: Input,
    mut state: ResMut<EditorState>,
    mut viewport: ResMut<Viewport>,
    level: LevelView,
    mut apply: MessageWriter<Apply>,
    time: Res<Time<Real>>,
) {
    if state.exporting {
        return;
    }
    if state.tool == Tool::Wall && state.chosen.is_some() {
        walls::leave_wall_tool(&mut state);
    }
    let Some(cursor) = input.window.cursor_position() else {
        finish_gesture(&mut state, &mut apply, &viewport, &input);
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
                let double = state.walls.double_click(time.elapsed_secs_f64(), cursor);
                press(&mut state, &mut apply, &viewport, &level, cursor, double);
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
        Interaction::Handle { .. } => {
            if input.buttons.pressed(MouseButton::Left) {
                drag_handle(&mut state, &mut apply, &viewport, cursor);
            } else {
                finish_gesture(&mut state, &mut apply, &viewport, &input);
            }
        }
        Interaction::Pressed {
            element,
            origin,
            pointer,
            moved_at,
        } => {
            if !input.buttons.pressed(MouseButton::Left) {
                finish_gesture(&mut state, &mut apply, &viewport, &input);
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

/// Moves the handle being dragged with the pointer, once it has travelled far enough to be a
/// drag: the first move begins the gesture and every later one continues it.
fn drag_handle(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    viewport: &Viewport,
    cursor: Vec2,
) {
    let Interaction::Handle {
        element,
        handle,
        origin,
        pointer,
        moved_at,
    } = state.interaction
    else {
        return;
    };
    let dragging = moved_at.is_some() || (cursor - pointer).length() > DRAG_THRESHOLD;
    if !dragging || moved_at == Some(cursor) {
        return;
    }
    let position = origin + (viewport.cells_at(cursor) - viewport.cells_at(pointer));
    apply.write(Apply::EditElement(EditElement {
        element,
        change: walls::handle_change(handle, position),
        gesture: if moved_at.is_some() {
            Gesture::Continue
        } else {
            Gesture::Begin
        },
    }));
    // A straight segment's middle, once dragged, is the segment's control point.
    let handle = match handle {
        walls::WallHandle::Middle(segment) => walls::WallHandle::Control(segment),
        walls::WallHandle::Point(_) | walls::WallHandle::Control(_) => handle,
    };
    state.walls.handle = Some((element, handle));
    state.interaction = Interaction::Handle {
        element,
        handle,
        origin,
        pointer,
        moved_at: Some(cursor),
    };
}

/// A left press over the viewport: with the Wall tool, adds a point of the Wall being drawn or
/// finishes it; with an Asset chosen, places it centred on the pointer; otherwise picks a handle
/// of the selected Wall or adds a point on its line, or selects the topmost Element under the
/// pointer and arms a drag, letting any handle go; empty space clears the selection.
fn press(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    viewport: &Viewport,
    level: &LevelView,
    cursor: Vec2,
    double: bool,
) {
    if state.tool == Tool::Wall {
        walls::draw_click(
            state,
            apply,
            level.current_layer(),
            viewport,
            cursor,
            double,
        );
        return;
    }
    let cells = viewport.cells_at(cursor);
    if let Some(chosen) = &state.chosen {
        if let Some(layer) = level.current_layer() {
            apply.write(Apply::PlaceElement(PlaceElement {
                layer,
                placement: Placement::Prop {
                    position: cells,
                    asset: chosen.asset.clone(),
                },
            }));
        }
        return;
    }
    let selected = level.selected_wall(state.selected);
    if walls::press_selected(state, apply, selected, viewport, cursor, double) {
        return;
    }
    let hit = level.topmost_at(cells, viewport.zoom);
    state.selected = hit.map(|(id, _)| id);
    state.walls.handle = None;
    if let Some((element, origin)) = hit {
        state.interaction = Interaction::Pressed {
            element,
            origin,
            pointer: cursor,
            moved_at: None,
        };
    }
}

/// Ends a drag that is under way once its button is up: the position the pointer last moved the
/// Element or the handle to is sent again as the end of the gesture, so the whole drag is one
/// history step. A press that never became a drag just ends.
fn finish_gesture(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    viewport: &Viewport,
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
        Interaction::Handle {
            element,
            handle,
            origin,
            pointer,
            moved_at,
        } => {
            if input.buttons.pressed(MouseButton::Left) {
                return;
            }
            if let Some(last) = moved_at {
                let position = origin + (viewport.cells_at(last) - viewport.cells_at(pointer));
                apply.write(Apply::EditElement(EditElement {
                    element,
                    change: walls::handle_change(handle, position),
                    gesture: Gesture::End,
                }));
            }
            state.interaction = Interaction::Idle;
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
                let position = origin + (viewport.cells_at(last) - viewport.cells_at(pointer));
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

/// The keys: `W` chooses the Wall tool, Enter finishes the Wall being drawn, Escape stops
/// placing or leaves the Wall tool, Delete (and Backspace on macOS) removes the selected point,
/// straightens the selected control point's segment, or removes the selected Element, and the
/// platform's usual shortcuts undo and redo. Nothing happens while egui has the keyboard, so a
/// text field keeps its own editing keys, nor while an Export runs, and undo and redo wait while
/// an Element or a handle is being dragged or a Wall is being drawn, since each is one step that
/// is still being made.
pub(crate) fn keys(
    keys: Res<ButtonInput<KeyCode>>,
    egui: Res<EguiWantsInput>,
    mut state: ResMut<EditorState>,
    level: LevelView,
    mut apply: MessageWriter<Apply>,
    mut undo: MessageWriter<Undo>,
    mut redo: MessageWriter<Redo>,
) {
    if egui.wants_any_keyboard_input() || state.exporting {
        return;
    }
    if bindings::any_pressed(bindings::WALL_TOOL, &keys) {
        walls::choose_wall_tool(&mut state);
    }
    if bindings::any_pressed(bindings::FINISH, &keys) && state.tool == Tool::Wall {
        walls::finish(&mut state, &mut apply, level.current_layer());
    }
    if keys.just_pressed(KeyCode::Escape) {
        if state.chosen.is_some() {
            state.chosen = None;
        }
        if state.tool == Tool::Wall {
            walls::leave_wall_tool(&mut state);
        }
    }
    if bindings::any_pressed(bindings::REMOVE, &keys)
        && let Some(element) = state.selected
    {
        let picked = state.walls.handle_of(element);
        if let (Some((_, wall, _)), Some(handle)) = (level.selected_wall(Some(element)), picked) {
            if let Some(change) = walls::delete_handle(wall, handle) {
                apply.write(Apply::EditElement(EditElement {
                    element,
                    change,
                    gesture: Gesture::Single,
                }));
            }
        } else {
            state.selected = None;
            apply.write(Apply::RemoveElement(RemoveElement { element }));
        }
        state.walls.handle = None;
    }
    if state.step_under_way() {
        return;
    }
    if bindings::any_pressed(bindings::UNDO, &keys) {
        undo.write(Undo);
    }
    if bindings::any_pressed(bindings::REDO, &keys) {
        redo.write(Redo);
    }
}

/// Outlines the selected Element, drops a selection whose Element is gone, and lets go of a
/// handle the selected Wall no longer has.
pub(crate) fn outline_selection(
    mut gizmos: Gizmos,
    mut state: ResMut<EditorState>,
    elements: Query<(&ElementId, &Element, Option<&Wall>)>,
) {
    let Some(selected) = state.selected else {
        state.walls.handle = None;
        return;
    };
    let Some((_, element, wall)) = elements.iter().find(|(id, ..)| **id == selected) else {
        state.selected = None;
        state.walls.handle = None;
        return;
    };
    gizmos.rect_2d(
        Isometry2d::from_translation(element.position),
        element.size,
        SELECTION,
    );
    let gone = match (wall, state.walls.handle_of(selected)) {
        (Some(wall), Some(handle)) => !walls::handle_exists(wall, handle),
        (None, Some(_)) => true,
        (_, None) => false,
    };
    if gone
        || state
            .walls
            .handle
            .is_some_and(|(owner, _)| owner != selected)
    {
        state.walls.handle = None;
    }
}
