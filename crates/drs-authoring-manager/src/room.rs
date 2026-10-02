//! Rooms: placing one, the edits only a Room has or that it has as a Wall does, removing it with
//! the Portals set into it, and the shape derived from it and from those Portals.

use crate::AuthoringError;
use crate::place::{spawn_on_top, take_off};
use crate::portal::{Anchored, anchored_to};
use crate::wall::{
    Reshape, carried_past, moved, record_together, remove_with_portals, removed, settings,
};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::query::Without;
use bevy_ecs::system::{Commands, Query};
use bevy_ecs::world::World;
use bevy_math::Vec2;
use drs_history::{ReversibleCommand, SetField, Target};
use drs_model::{
    Edge, Element, ElementChange, ElementId, Portal, PortalsRemoved, ROOM, Room, RoomShape, Wall,
};
use drs_shape_engine::{
    Path, PointEdit, PortalSetting, Standing, anchor_portals, anchor_portals_through,
    combine_outlines, generate_walls, split_wall,
};
use std::collections::BTreeMap;

/// The recorded step of placing a Room: the Element spawned on top of its Layer, keeping its
/// identity so that redo puts it back exactly.
struct PlaceRoom {
    /// The Layer to place on.
    layer: Entity,
    /// The Room as placed.
    room: Room,
    /// The identity the Element keeps through undo and redo.
    element: ElementId,
}

impl ReversibleCommand for PlaceRoom {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        let footprint = self.room.element_box();
        spawn_on_top(
            world,
            self.layer,
            (
                Element {
                    kind: ROOM,
                    position: footprint.center(),
                    size: footprint.size(),
                },
                self.room.clone(),
                self.element,
            ),
        )
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        take_off(world, self.element)
    }
}

/// Places a Room on top of the Layer as one history step.
///
/// # Errors
///
/// [`AuthoringError::MalformedRoom`] for fewer than three points, a thickness not above zero, or
/// a point that is not finite, or [`AuthoringError::History`] when the step could not be
/// recorded.
pub(crate) fn place_room(
    world: &mut World,
    layer: Entity,
    room: Room,
) -> Result<(), AuthoringError> {
    well_formed(&room)?;
    crate::record_step(
        world,
        PlaceRoom {
            layer,
            room,
            element: ElementId::new(),
        },
    )
}

/// Refuses a Room that would not be one, for the reason [`Room::malformation`] gives.
///
/// # Errors
///
/// [`AuthoringError::MalformedRoom`] with that reason.
fn well_formed(room: &Room) -> Result<(), AuthoringError> {
    room.malformation()
        .map_or(Ok(()), |reason| Err(AuthoringError::MalformedRoom(reason)))
}

/// The Room an Element is.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] when no Element carries the identity, or
/// [`AuthoringError::NotARoom`] when it is no Room.
fn room_of(world: &mut World, element: ElementId) -> Result<Room, AuthoringError> {
    let entity = element
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(element))?;
    world
        .get::<Room>(entity)
        .cloned()
        .ok_or(AuthoringError::NotARoom(element))
}

/// The Room moved so that its box is centred on `position`: every point and control point moved
/// by the same amount.
fn translated(room: &Room, position: Vec2) -> Room {
    let offset = position - room.element_box().center();
    let mut moved = room.clone();
    for point in &mut moved.points {
        *point += offset;
    }
    for control in moved
        .edges
        .iter_mut()
        .filter_map(|edge| edge.control.as_mut())
    {
        *control += offset;
    }
    moved
}

/// What an Edit Element of a Room comes to: a field command for the gesture handling to record,
/// or a step of its own already recorded, with the answer naming the Portals it removed.
pub(crate) enum RoomEdit {
    /// The field command to record.
    Field(SetField<ElementId>),
    /// The step was recorded on its own.
    Recorded(Option<PortalsRemoved>),
}

/// An Edit Element of the Room `id`, checked against the Room as it is: its position, a point,
/// an edge's control point, its thickness, its wall colour, or its floor colour as a field
/// command, or a point added or removed as a step of its own that carries the Portals set into
/// it.
///
/// # Errors
///
/// [`AuthoringError::NoPoint`] or [`AuthoringError::NoEdge`] for a point or edge the Room does
/// not have, [`AuthoringError::Shape`] for a point added where the edge cannot be split,
/// [`AuthoringError::MalformedRoom`] for a thickness not above zero or a point that is not
/// finite, [`AuthoringError::NotAPortal`] for a change only a Portal has,
/// [`AuthoringError::NotATerrain`] for a change only a Terrain has, or
/// [`AuthoringError::History`] when the change cannot be addressed or recorded.
pub(crate) fn room_edit(
    world: &mut World,
    id: ElementId,
    change: &ElementChange,
) -> Result<RoomEdit, AuthoringError> {
    let history = |error: drs_history::HistoryError| AuthoringError::History(error.to_string());
    let mut room = room_of(world, id)?;
    let field = match change {
        ElementChange::Position(position) => {
            let moved = translated(&room, *position);
            well_formed(&moved)?;
            SetField::<ElementId>::new::<Room>(id, "", moved)
        }
        ElementChange::Point { index, position } => {
            let points = room.points.len();
            *room.points.get_mut(*index).ok_or(AuthoringError::NoPoint {
                outline: "Room",
                index: *index,
                points,
            })? = *position;
            well_formed(&room)?;
            SetField::<ElementId>::new::<Room>(id, &format!("points[{index}]"), *position)
        }
        ElementChange::Control { segment, position } => {
            let edges = room.edges.len();
            room.edges
                .get_mut(*segment)
                .ok_or(AuthoringError::NoEdge {
                    edge: *segment,
                    edges,
                })?
                .control = *position;
            well_formed(&room)?;
            SetField::<ElementId>::new::<Room>(id, &format!("edges[{segment}].control"), *position)
        }
        ElementChange::Thickness(thickness) => {
            room.thickness = *thickness;
            well_formed(&room)?;
            SetField::<ElementId>::new::<Room>(id, "thickness", *thickness)
        }
        ElementChange::Colour(colour) => {
            SetField::<ElementId>::new::<Room>(id, "wall_colour", *colour)
        }
        ElementChange::FloorColour(colour) => {
            SetField::<ElementId>::new::<Room>(id, "floor_colour", *colour)
        }
        ElementChange::AddPoint { segment, t } => {
            return add_point(world, id, room, *segment, *t).map(|()| RoomEdit::Recorded(None));
        }
        ElementChange::RemovePoint { index } => {
            return remove_point(world, id, &room, *index).map(RoomEdit::Recorded);
        }
        ElementChange::Width(_)
        | ElementChange::Rotation(_)
        | ElementChange::Mirrored(_)
        | ElementChange::Side(_)
        | ElementChange::Along { .. } => return Err(AuthoringError::NotAPortal(id)),
        ElementChange::Material(_) => return Err(AuthoringError::NotATerrain(id)),
    };
    field.map(RoomEdit::Field).map_err(history)
}

/// Adds a point on an edge of a Room, splitting it into two edges of the shape it had, as one
/// history step that also moves each Portal set into the Room to the edge and parameter that
/// keep it where it was, and each Portal anchored at an edge the Room lacks one edge on, so it
/// still names none.
///
/// # Errors
///
/// [`AuthoringError::Shape`] when the edge or the parameter is not on the Room, or
/// [`AuthoringError::History`] when the step could not be recorded.
fn add_point(
    world: &mut World,
    element: ElementId,
    before: Room,
    edge: usize,
    t: f32,
) -> Result<(), AuthoringError> {
    let path = Path::of_room(&before);
    let split = split_wall(&path, edge, t)?;
    let room = Room {
        points: split.points,
        edges: split
            .controls
            .into_iter()
            .map(|control| Edge { control })
            .collect(),
        ..before
    };
    let moves = carried_past(world, element, &path, edge, t)?;
    record_together(
        world,
        Vec::new(),
        Reshape {
            element,
            outline: room,
            previous: None,
        },
        moves,
    )
}

/// Removes a point of a Room as one history step: the edge that ends at it and the edge that
/// starts there join into one straight edge in the place of the first, the joined edge of the
/// first point being the last. A Room of three points is removed whole, with its Portals, in a
/// group of its own. The Portals whose stretch covers the point are removed in the same step,
/// before the point, and the other Portals set into the Room move to the edge and parameter that
/// keep them on their part of it. Returns the answer naming the Portals that went, when any did.
///
/// # Errors
///
/// [`AuthoringError::NoPoint`] when the Room has no such point, or [`AuthoringError::History`]
/// when the step could not be recorded.
fn remove_point(
    world: &mut World,
    element: ElementId,
    before: &Room,
    index: usize,
) -> Result<Option<PortalsRemoved>, AuthoringError> {
    let points = before.points.len();
    if index >= points {
        return Err(AuthoringError::NoPoint {
            outline: "Room",
            index,
            points,
        });
    }
    if points <= 3 {
        return remove_with_portals(world, element);
    }
    let mut room = before.clone();
    room.points.remove(index);
    room.edges.remove(index);
    let joined = if index == 0 { points - 2 } else { index - 1 };
    room.edges[joined] = Edge::default();
    let (portals, _) = anchored_to(world, element);
    let places = anchor_portals_through(
        &Path::of_room(before),
        PointEdit::Removed { index },
        &settings(&portals),
    );
    let mut gone = Vec::new();
    let mut moves = Vec::new();
    for ((portal, anchor, _), place) in portals.iter().zip(places) {
        match place {
            Some(place) => moves.extend(moved(*portal, *anchor, place)?),
            None => gone.push(*portal),
        }
    }
    record_together(
        world,
        gone.clone(),
        Reshape {
            element,
            outline: room,
            previous: None,
        },
        moves,
    )?;
    Ok(removed(element, gone))
}

/// What a Room's shape was last derived from: its points, edges, and thickness, and where the
/// Portals set into it are and how wide. A change that leaves them as they were, a new colour,
/// keeps the shape.
#[derive(Component, Debug, Clone, PartialEq)]
pub(crate) struct DerivedFrom {
    /// The points the shape was derived from.
    points: Vec<Vec2>,
    /// The edges it was derived from.
    edges: Vec<Edge>,
    /// The thickness it was derived at.
    thickness: f32,
    /// The Portals set into the Room, in the order of their identities.
    portals: Vec<PortalSetting>,
}

impl DerivedFrom {
    /// What `room`'s shape is derived from, with the Portals set into it.
    fn of(room: &Room, portals: &[Anchored]) -> Self {
        Self {
            points: room.points.clone(),
            edges: room.edges.clone(),
            thickness: room.thickness,
            portals: settings(portals),
        }
    }
}

/// The Rooms as deriving reads and writes them.
pub(crate) type Rooms<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ElementId,
        &'static Room,
        &'static mut Element,
        Option<&'static mut RoomShape>,
        Option<&'static DerivedFrom>,
    ),
    (Without<Portal>, Without<Wall>),
>;

/// Derives again the shape of every Room whose points, edges, or thickness, or whose set
/// Portals, differ from what its shape was last derived from, its floor and its Walls, leaving
/// out of its Walls the stretches those Portals cover, and sets its Element's box around its
/// points. Returns where each Portal set into those Rooms stands.
pub(crate) fn reshape_rooms(
    commands: &mut Commands,
    rooms: &mut Rooms,
    set: &BTreeMap<ElementId, Vec<Anchored>>,
) -> BTreeMap<ElementId, Standing> {
    let mut standings = BTreeMap::new();
    for (entity, id, room, mut element, room_shape, derived_from) in rooms {
        let into = set.get(id).map_or(&[][..], Vec::as_slice);
        let geometry = DerivedFrom::of(room, into);
        if room_shape.is_some() && derived_from == Some(&geometry) {
            continue;
        }
        let path = Path::of_room(room);
        let placed = anchor_portals(&path, &geometry.portals);
        let stretches: Vec<_> = placed
            .iter()
            .flatten()
            .map(|standing| standing.stretch)
            .collect();
        for ((portal, ..), standing) in into.iter().zip(placed) {
            if let Some(standing) = standing {
                standings.insert(*portal, standing);
            }
        }
        let combined = combine_outlines(&path);
        let shape = RoomShape {
            walls: generate_walls(&combined, room.thickness, &stretches),
            floor: combined.floor,
        };
        match room_shape {
            Some(mut room_shape) => *room_shape = shape,
            None => {
                commands.entity(entity).insert(shape);
            }
        }
        commands.entity(entity).insert(geometry);
        let footprint = room.element_box();
        if element.position != footprint.center() {
            element.position = footprint.center();
        }
        if element.size != footprint.size() {
            element.size = footprint.size();
        }
    }
    standings
}
