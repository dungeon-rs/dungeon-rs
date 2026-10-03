//! Interaction in the viewport: placing Props and Portals, drawing Walls and Rooms, painting,
//! selecting, dragging Elements and the handles of a Wall or a Room, sliding Portals, removing,
//! panning, and zooming.
//!
//! Pointer positions come from the window in logical pixels and go through the model's
//! `Viewport` to Level cells, the same conversion the render Engine draws by. Panning and zooming
//! write the Viewport only; everything that changes the Level is a Command sent to the authoring
//! Manager.

use crate::bindings;
use crate::gesture::{DRAG_THRESHOLD, Drag};
use crate::handles::{self, Outline};
use crate::paint;
use crate::portals;
use crate::rooms;
use crate::snapping::Shown;
use crate::state::{EditorState, Interaction, Tool};
use crate::walls;
use bevy::color::Color;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::{ChildOf, Children};
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::ecs::query::With;
use bevy::ecs::system::{Local, Query, Res, ResMut, Single, SystemParam};
use bevy::gizmos::gizmos::Gizmos;
use bevy::input::ButtonInput;
use bevy::input::gestures::PinchGesture;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseButton, MouseScrollUnit};
use bevy::math::{Isometry2d, Rect, Vec2, ops};
use bevy::time::{Real, Time};
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::EguiContexts;
use bevy_egui::input::EguiWantsInput;
use drs_model::{
    Anchoring, Apply, DrawnAs, EditElement, Element, ElementChange, ElementId, ElementKindRegistry,
    Gesture, Layer, Level, PlaceElement, Placement, Pointer, Portal, Redo, RemoveElement, Room,
    RoomShape, SnappedPoint, Terrain, Undo, Viewport, Wall, WallShape,
};

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
    /// The Level each Layer belongs to.
    layer_levels: Query<'w, 's, &'static ChildOf, With<Layer>>,
    /// Every Element's identity and box, its Wall or Room and derived shape when it is one, and
    /// its Portal when it is one.
    elements: Query<'w, 's, Picked>,
    /// The Terrains and the images they show, for the one a Paint adds to.
    pub(crate) terrains: paint::Terrains<'w, 's>,
    /// How each known kind is drawn, which says what is never picked.
    kinds: Option<Res<'w, ElementKindRegistry>>,
}

/// What picking reads of an Element: its identity and box, its Wall and derived shape when it is
/// a Wall, its Portal and whether it follows its host when it is one, and its Room and derived
/// shape when it is a Room.
type Picked = (
    &'static ElementId,
    &'static Element,
    Option<&'static Wall>,
    Option<&'static WallShape>,
    Option<&'static Portal>,
    Option<&'static Anchoring>,
    Option<&'static Room>,
    Option<&'static RoomShape>,
);

impl LevelView<'_, '_> {
    /// The Layer new Props are placed on: the Project's only Layer for now; choosing one among
    /// several is a later concern.
    pub(crate) fn current_layer(&self) -> Option<Entity> {
        self.any_layer.iter().next()
    }

    /// The Level the Author is working on: the current Layer's.
    pub(crate) fn current_level(&self) -> Option<Entity> {
        let layer = self.current_layer()?;
        self.layer_levels.get(layer).ok().map(ChildOf::parent)
    }

    /// Whether the Element with an identity is a Wall or a Room, which a drag moves by amounts.
    fn is_outline(&self, element: ElementId) -> bool {
        self.elements.iter().any(|(id, _, wall, _, _, _, room, _)| {
            *id == element && (wall.is_some() || room.is_some())
        })
    }

    /// The Terrain a Paint on the current Layer adds to, with its identity: the Layer's topmost.
    pub(crate) fn current_terrain(&self) -> Option<(ElementId, &Terrain)> {
        self.terrains.on(self.current_layer())
    }

    /// The topmost Element under a point in cells at `zoom`, with its centre and whether it is a
    /// Portal that follows its Wall or Room: Layers from the top down, and each Layer's Elements
    /// from the last drawn back. A Wall is under the point when its line is near enough outside
    /// the stretches its Portals cover, a Room when the point is inside its floor or so near its
    /// Walls, a Portal when its turned rectangle holds the point, and any other Element when its
    /// box does; an Element drawn as a painted surface, a Terrain, never is, so the ground never
    /// gets in the way of what stands on it.
    fn topmost_at(&self, cells: Vec2, zoom: f32) -> Option<(ElementId, Vec2, bool)> {
        self.levels.iter().find_map(|layers| {
            layers.iter().rev().find_map(|&layer| {
                let (_, elements) = self.layers.get(layer).ok()?;
                elements.iter().rev().find_map(|&element| {
                    let (id, element, wall, shape, portal, anchoring, room, room_shape) =
                        self.elements.get(element).ok()?;
                    if self.painted(element) {
                        return None;
                    }
                    let hit = match (wall, shape, portal, room, room_shape) {
                        (Some(wall), Some(shape), ..) => {
                            walls::on_wall(wall.thickness, shape, cells, zoom)
                        }
                        (Some(_), None, ..) | (None, _, _, Some(_), None) => false,
                        (None, _, _, Some(room), Some(room_shape)) => {
                            rooms::on_room(room, room_shape, cells, zoom)
                        }
                        (None, _, Some(portal), None, _) => {
                            portals::on_portal(element, portal, cells)
                        }
                        (None, _, None, None, _) => {
                            Rect::from_center_size(element.position, element.size).contains(cells)
                        }
                    };
                    let set = hit && portal.is_some_and(|portal| portal.follows(anchoring));
                    hit.then_some((*id, element.position, set))
                })
            })
        })
    }

    /// Whether an Element is drawn as a painted surface, by its kind.
    fn painted(&self, element: &Element) -> bool {
        self.kinds
            .as_deref()
            .and_then(|kinds| kinds.get(&element.kind))
            .is_some_and(|kind| kind.drawn_as == DrawnAs::PaintedSurface)
    }

    /// The selected Element when it is a Wall, with its derived shape once it has one.
    pub(crate) fn selected_wall(
        &self,
        selected: Option<ElementId>,
    ) -> Option<(ElementId, &Wall, Option<&WallShape>)> {
        let selected = selected?;
        self.elements
            .iter()
            .find(|(id, ..)| **id == selected)
            .and_then(|(id, _, wall, shape, ..)| Some((*id, wall?, shape)))
    }

    /// The selected Element when it is a Room.
    pub(crate) fn selected_room(&self, selected: Option<ElementId>) -> Option<(ElementId, &Room)> {
        let selected = selected?;
        self.elements
            .iter()
            .find(|(id, ..)| **id == selected)
            .and_then(|(id, .., room, _)| Some((*id, room?)))
    }

    /// The selected Element when it is a Wall or a Room, as its handles see it, with the derived
    /// shape of its line once it has one.
    pub(crate) fn selected_outline(
        &self,
        selected: Option<ElementId>,
    ) -> Option<(ElementId, Outline<'_>, Option<&WallShape>)> {
        let selected = selected?;
        let (id, _, wall, shape, _, _, room, room_shape) =
            self.elements.iter().find(|(id, ..)| **id == selected)?;
        match (wall, room) {
            (Some(wall), _) => Some((*id, Outline::of_wall(wall), shape)),
            (None, Some(room)) => Some((
                *id,
                Outline::of_room(room),
                room_shape.map(|shape| &shape.walls),
            )),
            (None, None) => None,
        }
    }

    /// The selected Element when it is a Portal, with its box.
    pub(crate) fn selected_portal(
        &self,
        selected: Option<ElementId>,
    ) -> Option<(ElementId, &Element, &Portal)> {
        let selected = selected?;
        self.elements
            .iter()
            .find(|(id, ..)| **id == selected)
            .and_then(|(id, element, _, _, portal, ..)| Some((*id, element, portal?)))
    }

    /// Whether the Portal `portal` follows the Wall or the Room its anchor names, as deriving
    /// found. A lost Portal, anchored to none as an editor that does not know Portals or Rooms
    /// may leave it, is dragged, flipped, and turned as a freestanding one, and never set again
    /// by a drag.
    pub(crate) fn follows_host(&self, portal: ElementId) -> bool {
        self.elements
            .iter()
            .find(|(id, ..)| **id == portal)
            .is_some_and(|(_, _, _, _, portal, anchoring, ..)| {
                portal.is_some_and(|portal| portal.follows(anchoring))
            })
    }

    /// The derived shape of the line of the Wall or the Room with an identity, once it has one.
    fn shape_of(&self, host: ElementId) -> Option<&WallShape> {
        self.elements.iter().find(|(id, ..)| **id == host).and_then(
            |(_, _, _, shape, _, _, _, room_shape)| {
                shape.or_else(|| room_shape.map(|shape| &shape.walls))
            },
        )
    }

    /// Every Wall and every Room that has its derived shape, with its thickness and the shape of
    /// its line, bottom first in the stacking order, as picking sees them, so whatever looks for
    /// the nearest line breaks a tie as a click would.
    pub(crate) fn lines_in_order(&self) -> Vec<(ElementId, f32, &WallShape)> {
        self.levels
            .iter()
            .flat_map(|layers| layers.iter())
            .filter_map(|&layer| self.layers.get(layer).ok())
            .flat_map(|(_, elements)| elements.iter())
            .filter_map(|&element| {
                let (id, _, wall, shape, _, _, room, room_shape) =
                    self.elements.get(element).ok()?;
                match (wall, shape, room, room_shape) {
                    (Some(wall), Some(shape), ..) => Some((*id, wall.thickness, shape)),
                    (_, _, Some(room), Some(room_shape)) => {
                        Some((*id, room.thickness, &room_shape.walls))
                    }
                    _ => None,
                }
            })
            .collect()
    }
}

/// Makes egui let go of a field with the keyboard as soon as the pointer is pressed elsewhere,
/// rather than once the click ends, so that what was typed is sent in the frame a press on the
/// viewport waits for, and a drag that starts there is not held up.
pub(crate) fn let_go_of_fields_on_press(mut contexts: EguiContexts) {
    if let Ok(ctx) = contexts.ctx_mut() {
        ctx.options_mut(|options| {
            options.input_options.surrender_focus_on = egui::SurrenderFocusOn::Presses;
        });
    }
}

/// Ends what is under way once the pointer has left the window: a stroke or a rectangle whose
/// button is released, a drag of an Element or a handle, or a slide.
fn pointer_gone(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    viewport: &Viewport,
    level: &LevelView,
    input: &Input,
    shown: Shown,
) {
    let released = !input.buttons.pressed(MouseButton::Left);
    if state.interaction == Interaction::Painting && released {
        paint::release(state, apply, level.current_layer(), None);
    }
    finish_gesture(state, apply, input);
    finish_slide(state, apply, level);
    if released {
        rooms::release(state, apply, level.current_layer(), viewport, shown);
    }
}

/// Carries the pointer gesture of the frame out: a click adds a point of the Wall being drawn,
/// places, or selects, a drag moves the selected Element or a handle of the selected Wall as one
/// gesture, the middle button or Space drags the view, scrolling pans, and a wheel, a pinch, or
/// a modified scroll zooms around the pointer. An Asset chosen while the Wall tool is chosen
/// leaves the tool, discarding the Wall being drawn.
///
/// A point the Wall or the Room tool adds, a point of a Wall or a Room dragged, and a whole Wall
/// or Room dragged take the snapped point derived from the Pointer written the frame before, the
/// one on screen, whenever it answers that Pointer, and the pointer itself otherwise.
///
/// A gesture starts only with the pointer over the viewport and egui not using it, a frame late
/// while a field has the keyboard; one under way ends wherever the button is released, so no
/// Begin is left without its End. While an
/// Export runs the pointer is ignored, so the image is of the Level as it was asked for.
#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system is spelled out by what it reads and writes"
)]
pub(crate) fn pointer(
    mut input: Input,
    mut state: ResMut<EditorState>,
    mut viewport: ResMut<Viewport>,
    level: LevelView,
    mut apply: MessageWriter<Apply>,
    time: Res<Time<Real>>,
    mut waiting: Local<Option<(Vec2, bool)>>,
    written: Res<Pointer>,
    snapped: Res<SnappedPoint>,
) {
    if state.exporting {
        return;
    }
    if matches!(state.tool, Tool::Wall | Tool::Room) && state.chosen.is_some() {
        walls::leave_tool(&mut state);
    }
    let shown = Shown::of(&snapped, &written);
    let Some(cursor) = input.window.cursor_position() else {
        pointer_gone(&mut state, &mut apply, &viewport, &level, &input, shown);
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
            if let Some((at, double)) = waiting.take() {
                press(&mut state, &mut apply, &viewport, &level, at, double, shown);
            } else if over {
                let now = time.elapsed_secs_f64();
                *waiting = start(
                    &input, &mut state, &mut apply, &viewport, &level, cursor, now, shown,
                );
            }
        }
        Interaction::Painting => {
            let cells = viewport.cells_at(cursor);
            if input.buttons.pressed(MouseButton::Left) {
                paint::moved(&mut state, &mut apply, &viewport, cursor);
            } else {
                paint::release(&mut state, &mut apply, level.current_layer(), Some(cells));
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
                drag_handle(&mut state, &mut apply, &viewport, cursor, shown);
            } else {
                finish_gesture(&mut state, &mut apply, &input);
            }
        }
        Interaction::Sliding { .. } => {
            if input.buttons.pressed(MouseButton::Left) {
                drag_slide(&mut state, &mut apply, &viewport, &level, cursor);
            } else {
                finish_slide(&mut state, &mut apply, &level);
            }
        }
        Interaction::Outlining { .. } => {
            if input.buttons.pressed(MouseButton::Left) {
                rooms::track(&mut state, cursor, DRAG_THRESHOLD);
            } else {
                rooms::release(
                    &mut state,
                    &mut apply,
                    level.current_layer(),
                    &viewport,
                    shown,
                );
            }
        }
        Interaction::Moving { .. } => {
            if input.buttons.pressed(MouseButton::Left) {
                drag_whole(&mut state, &mut apply, &viewport, cursor, shown);
            } else {
                finish_gesture(&mut state, &mut apply, &input);
            }
        }
        Interaction::Pressed { .. } => {
            if input.buttons.pressed(MouseButton::Left) {
                drag_element(&mut state, &mut apply, &viewport, cursor);
            } else {
                finish_gesture(&mut state, &mut apply, &input);
            }
        }
    }
}

/// Starts what a press over the viewport begins while nothing is under way: a drag of the view
/// with the middle button or Space, or a left press, carried out at once, or returned to be carried
/// out a frame late while a field has the keyboard. Such a field sends what was typed into it as
/// egui lets go of it, in this frame's pass, to what the strip shows then, so the press waits until
/// the selection it may change has had what was typed for it.
#[expect(
    clippy::too_many_arguments,
    reason = "a press reads the input, the view, the Level, and what snapping shows"
)]
fn start(
    input: &Input,
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    viewport: &Viewport,
    level: &LevelView,
    cursor: Vec2,
    now: f64,
    shown: Shown,
) -> Option<(Vec2, bool)> {
    let left = input.buttons.just_pressed(MouseButton::Left);
    if input.buttons.just_pressed(MouseButton::Middle)
        || (left && input.keys.pressed(KeyCode::Space))
    {
        state.interaction = Interaction::Panning { last: cursor };
        return None;
    }
    if !left {
        return None;
    }
    let double = state.walls.double_click(now, cursor);
    if input.egui.wants_any_keyboard_input() {
        return Some((cursor, double));
    }
    press(state, apply, viewport, level, cursor, double, shown);
    None
}

/// Moves the Element being dragged, neither a Wall nor a Room, with the pointer, once it has
/// travelled far enough to be a drag: the first move begins the gesture and every later one that
/// puts it elsewhere continues it.
fn drag_element(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    viewport: &Viewport,
    cursor: Vec2,
) {
    let Interaction::Pressed {
        element,
        origin,
        mut drag,
    } = state.interaction
    else {
        return;
    };
    let position = origin + (viewport.cells_at(cursor) - viewport.cells_at(drag.pressed_at()));
    let Some(gesture) = drag.step(cursor, position) else {
        return;
    };
    apply.write(Apply::EditElement(EditElement {
        element,
        change: ElementChange::Position(position),
        gesture,
    }));
    state.interaction = Interaction::Pressed {
        element,
        origin,
        drag,
    };
}

/// The Edit Element sliding the set Portal `element` to the nearest point of its Wall's line
/// to `cells`, as part of `gesture`.
fn slid(level: &LevelView, element: ElementId, cells: Vec2, gesture: Gesture) -> Option<Apply> {
    let (_, _, portal) = level.selected_portal(Some(element))?;
    let shape = level.shape_of(portal.anchor?.host)?;
    portals::slide(element, shape, cells, gesture)
}

/// Slides the Portal being dragged to the nearest point of its Wall's line to the pointer, once
/// the pointer has travelled far enough to be a drag: the first slide begins the gesture and
/// every later one continues it.
fn drag_slide(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    viewport: &Viewport,
    level: &LevelView,
    cursor: Vec2,
) {
    let Interaction::Sliding { element, mut drag } = state.interaction else {
        return;
    };
    let cells = viewport.cells_at(cursor);
    let Some(gesture) = drag.step(cursor, cells) else {
        return;
    };
    if let Some(slide) = slid(level, element, cells, gesture) {
        apply.write(slide);
    }
    state.interaction = Interaction::Sliding { element, drag };
}

/// Ends a slide that is under way once its button is up: the place the pointer last slid the
/// Portal to is sent again as the end of the gesture, so the whole slide is one history step.
fn finish_slide(state: &mut EditorState, apply: &mut MessageWriter<Apply>, level: &LevelView) {
    let Interaction::Sliding { element, drag } = state.interaction else {
        return;
    };
    if let Some(cells) = drag.sent()
        && let Some(slide) = slid(level, element, cells, Gesture::End)
    {
        apply.write(slide);
    }
    state.interaction = Interaction::Idle;
}

/// Moves the handle being dragged with the pointer, once it has travelled far enough to be a
/// drag: the first move begins the gesture and every later one that puts it elsewhere continues
/// it. A point goes where snapping puts the pointer while it snaps, so it moves as soon as the
/// snapped point does, even with the pointer still.
fn drag_handle(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    viewport: &Viewport,
    cursor: Vec2,
    shown: Shown,
) {
    let Interaction::Handle {
        element,
        handle,
        origin,
        mut drag,
    } = state.interaction
    else {
        return;
    };
    let position =
        shown.point_or(origin + (viewport.cells_at(cursor) - viewport.cells_at(drag.pressed_at())));
    let Some(gesture) = drag.step(cursor, position) else {
        return;
    };
    apply.write(Apply::EditElement(EditElement {
        element,
        change: handles::handle_change(handle, position),
        gesture,
    }));
    // A straight segment's middle, once dragged, is the segment's control point.
    let handle = match handle {
        handles::OutlineHandle::Middle(segment) => handles::OutlineHandle::Control(segment),
        handles::OutlineHandle::Point(_) | handles::OutlineHandle::Control(_) => handle,
    };
    state.handle = Some((element, handle));
    state.interaction = Interaction::Handle {
        element,
        handle,
        origin,
        drag,
    };
}

/// Moves the Wall or the Room being dragged with the pointer, once it has travelled far enough to
/// be a drag, by the amount the pointer travelled since the step before, in whole cells while it
/// snaps: the first move begins the gesture, even by nothing, and every later move that goes
/// anywhere continues it, as soon as the snapped travel changes, even with the pointer still.
fn drag_whole(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    viewport: &Viewport,
    cursor: Vec2,
    shown: Shown,
) {
    let Interaction::Moving {
        element,
        from,
        mut drag,
    } = state.interaction
    else {
        return;
    };
    let moved = drag.sent().unwrap_or(Vec2::ZERO);
    let travel = shown.travel_or(viewport.cells_at(cursor) - from);
    let Some(gesture) = drag.step(cursor, travel) else {
        return;
    };
    apply.write(Apply::EditElement(EditElement {
        element,
        change: ElementChange::MoveBy(travel - moved),
        gesture,
    }));
    state.interaction = Interaction::Moving {
        element,
        from,
        drag,
    };
}

/// A left press over the viewport: with the Paint tool, starts a stroke; with the Wall tool, adds
/// a point of the Wall being drawn or finishes it; with the Room tool, adds a point of the outline
/// being drawn, closes it, or may begin a rectangle; with the Portal tool, places a Portal of the
/// chosen Asset into the nearest Wall within reach or freestanding; with an Asset chosen, places
/// a Prop of it centred on the pointer; otherwise picks a handle of the selected Wall or Room or
/// adds a point on its line, or selects the topmost Element under the pointer and arms a drag, or
/// a slide for a Portal set into a Wall, letting any handle go; empty space clears the selection.
/// The Wall and the Room tool put their points where snapping put the pointer.
fn press(
    state: &mut EditorState,
    apply: &mut MessageWriter<Apply>,
    viewport: &Viewport,
    level: &LevelView,
    cursor: Vec2,
    double: bool,
    shown: Shown,
) {
    if state.tool == Tool::Paint {
        paint::press(state, level.current_terrain(), viewport, cursor);
        return;
    }
    if state.tool == Tool::Wall {
        let at = shown.point_or(viewport.cells_at(cursor));
        walls::draw_click(
            state,
            apply,
            level.current_layer(),
            viewport.zoom,
            at,
            double,
        );
        return;
    }
    if state.tool == Tool::Room {
        let at = shown.point_or(viewport.cells_at(cursor));
        rooms::press(
            state,
            apply,
            level.current_layer(),
            viewport.zoom,
            (cursor, at),
        );
        return;
    }
    let cells = viewport.cells_at(cursor);
    if state.tool == Tool::Portal {
        let under = portals::line_under(level.lines_in_order(), cells, None);
        portals::place_click(state, apply, level.current_layer(), under, cells);
        return;
    }
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
    let selected = level.selected_outline(state.selected);
    if handles::press_selected(state, apply, selected, viewport, cursor, double) {
        return;
    }
    let hit = level.topmost_at(cells, viewport.zoom);
    state.selected = hit.map(|(id, ..)| id);
    state.handle = None;
    match hit {
        Some((element, _, true)) => portals::press_set(state, element, cursor),
        Some((element, _, false)) if level.is_outline(element) => {
            state.interaction = Interaction::Moving {
                element,
                from: cells,
                drag: Drag::new(cursor),
            };
        }
        Some((element, origin, false)) => {
            state.interaction = Interaction::Pressed {
                element,
                origin,
                drag: Drag::new(cursor),
            };
        }
        None => {}
    }
}

/// Ends a drag that is under way once its button is up: the position the pointer last moved the
/// Element or the handle to is sent again as the end of the gesture, or for a whole Wall or Room
/// a move by nothing, so the whole drag is one history step and ends where it was last shown. A
/// press that never became a drag just ends.
fn finish_gesture(state: &mut EditorState, apply: &mut MessageWriter<Apply>, input: &Input) {
    match state.interaction {
        // A slide ends through `finish_slide`, which knows the Portal's Wall, and a rectangle
        // through `rooms::release`, which knows the Layer.
        Interaction::Idle
        | Interaction::Painting
        | Interaction::Sliding { .. }
        | Interaction::Outlining { .. } => {}
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
            drag,
            ..
        } => {
            if input.buttons.pressed(MouseButton::Left) {
                return;
            }
            if let Some(position) = drag.sent() {
                apply.write(Apply::EditElement(EditElement {
                    element,
                    change: handles::handle_change(handle, position),
                    gesture: Gesture::End,
                }));
            }
            state.interaction = Interaction::Idle;
        }
        Interaction::Moving { element, drag, .. } => {
            if input.buttons.pressed(MouseButton::Left) {
                return;
            }
            if drag.begun() {
                apply.write(Apply::EditElement(EditElement {
                    element,
                    change: ElementChange::MoveBy(Vec2::ZERO),
                    gesture: Gesture::End,
                }));
            }
            state.interaction = Interaction::Idle;
        }
        Interaction::Pressed { element, drag, .. } => {
            if input.buttons.pressed(MouseButton::Left) {
                return;
            }
            if let Some(position) = drag.sent() {
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

/// The keys: `W` chooses the Wall tool, `P` the Portal tool, `R` the Room tool, `B` the Paint tool
/// painting and `E` erasing, Enter finishes the Wall or closes the Room being drawn, Escape stops
/// placing or leaves the Wall, the Portal, the Room, or the Paint tool, discarding what is being
/// drawn, `X` flips and `F` frees or sets the selected Portal, Delete (and Backspace on macOS)
/// removes the selected point, straightens the selected control point's segment or edge, removes
/// the selected Element, or with the Paint tool editing strokes removes the selected stroke, and
/// the platform's usual shortcuts undo and redo. Nothing happens while egui has the keyboard, so a
/// text field keeps its own editing keys, nor while an Export runs, and undo, redo, flipping, and
/// freeing or setting wait while an Element, a handle, or a stroke is pressed or dragged, a Wall,
/// a Room, or a stroke is being drawn, or an option is held while it changes, since each is one
/// step that is, or may be about to be, still being made.
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
    if bindings::any_pressed(bindings::PORTAL_TOOL, &keys) {
        portals::choose_portal_tool(&mut state);
    }
    if bindings::any_pressed(bindings::ROOM_TOOL, &keys) {
        rooms::choose_room_tool(&mut state);
    }
    // A flip or a freeing in the middle of a slide or an option's drag would land inside the
    // gesture's step, so both wait for it as undo does.
    if !state.step_under_way()
        && let Some((id, element, portal)) = level.selected_portal(state.selected)
    {
        if bindings::any_pressed(bindings::FLIP, &keys) {
            apply.write(portals::flip(id, portal, level.follows_host(id)));
        }
        if bindings::any_pressed(bindings::FREE_OR_SET, &keys) {
            portals::free_or_set(
                &mut state,
                &mut apply,
                id,
                element.position,
                portal,
                level.lines_in_order(),
            );
        }
    }
    if bindings::any_pressed(bindings::PAINT_TOOL, &keys) {
        paint::choose_paint_tool(&mut state);
    }
    if bindings::any_pressed(bindings::ERASE_TOOL, &keys) {
        paint::choose_erasing(&mut state);
    }
    if bindings::any_pressed(bindings::FINISH, &keys) {
        match state.tool {
            Tool::Wall => walls::finish(&mut state, &mut apply, level.current_layer()),
            Tool::Room => rooms::close(&mut state, &mut apply, level.current_layer()),
            Tool::Select | Tool::Portal | Tool::Paint => {}
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        if state.chosen.is_some() {
            state.chosen = None;
        }
        if matches!(state.tool, Tool::Wall | Tool::Portal | Tool::Room) {
            walls::leave_tool(&mut state);
        }
        paint::leave_paint_tool(&mut state);
    }
    if bindings::any_pressed(bindings::REMOVE, &keys) {
        paint::remove_selected(&mut state, &mut apply);
    }
    if bindings::any_pressed(bindings::REMOVE, &keys)
        && let Some(element) = state.selected
    {
        let picked = state.handle_of(element);
        if let (Some((_, outline, _)), Some(handle)) =
            (level.selected_outline(Some(element)), picked)
        {
            if let Some(change) = handles::delete_handle(&outline, handle) {
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
        state.handle = None;
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

/// What outlining the selection reads of an Element: its identity and box, and its Wall, Portal,
/// or Room when it is one.
type Outlined = (
    &'static ElementId,
    &'static Element,
    Option<&'static Wall>,
    Option<&'static Portal>,
    Option<&'static Room>,
);

/// Outlines the selected Element, a Portal turned as it is drawn, drops a selection whose Element
/// is gone, and lets go of a handle the selected Wall or Room no longer has. Nothing is drawn or
/// dropped while an Export runs, so the outline never appears in the image.
pub(crate) fn outline_selection(
    mut gizmos: Gizmos,
    mut state: ResMut<EditorState>,
    elements: Query<Outlined>,
) {
    if state.exporting {
        return;
    }
    // The state is written only when something changes, so it is not marked changed every frame.
    let Some(selected) = state.selected else {
        if state.handle.is_some() {
            state.handle = None;
        }
        return;
    };
    let Some((_, element, wall, portal, room)) = elements.iter().find(|(id, ..)| **id == selected)
    else {
        state.selected = None;
        state.handle = None;
        return;
    };
    let isometry = portal.map_or_else(
        || Isometry2d::from_translation(element.position),
        |portal| portals::outline(element, portal),
    );
    gizmos.rect_2d(isometry, element.size, SELECTION);
    let outline = wall
        .map(Outline::of_wall)
        .or_else(|| room.map(Outline::of_room));
    let gone = match (outline, state.handle_of(selected)) {
        (Some(outline), Some(handle)) => !handles::handle_exists(&outline, handle),
        (None, Some(_)) => true,
        (_, None) => false,
    };
    if gone || state.handle.is_some_and(|(owner, _)| owner != selected) {
        state.handle = None;
    }
}
