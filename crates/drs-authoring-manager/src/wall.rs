//! Walls: placing one, the edits that add and remove its points, and the shape derived from it.

use crate::AuthoringError;
use crate::remove::Remove;
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Commands, Query};
use bevy_ecs::world::World;
use bevy_math::Vec2;
use drs_history::{ReversibleCommand, Target};
use drs_model::{Element, ElementId, WALL, Wall, WallShape};
use drs_shape_engine::{generate_walls, split_wall};

/// The recorded step of placing a Wall: the Element spawned on top of its Layer, keeping its
/// identity so that redo puts it back exactly.
struct PlaceWall {
    /// The Layer to place on.
    layer: Entity,
    /// The Wall as placed.
    wall: Wall,
    /// The identity the Element keeps through undo and redo.
    element: ElementId,
}

impl ReversibleCommand for PlaceWall {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        if world.get_entity(self.layer).is_err() {
            return Err(AuthoringError::NotALayer.into());
        }
        let footprint = self.wall.element_box();
        let entity = world
            .spawn((
                Element {
                    kind: WALL,
                    position: footprint.center(),
                    size: footprint.size(),
                },
                self.wall.clone(),
                self.element,
            ))
            .id();
        // A redone placement is always last too: every step after it has been undone first.
        world
            .get_entity_mut(self.layer)
            .map_err(|_| AuthoringError::NotALayer)?
            .add_child(entity);
        Ok(())
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        let entity = self.element.entity(world)?;
        world.despawn(entity);
        Ok(())
    }
}

/// Places a Wall on top of the Layer as one history step.
///
/// # Errors
///
/// [`AuthoringError::MalformedWall`] for fewer than two points, a thickness not above zero, or a
/// point that is not finite, or [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn place_wall(
    world: &mut World,
    layer: Entity,
    wall: Wall,
) -> Result<(), AuthoringError> {
    well_formed(&wall)?;
    crate::record_step(
        world,
        PlaceWall {
            layer,
            wall,
            element: ElementId::new(),
        },
    )
}

/// The recorded step of adding or removing a point: the Wall as it becomes and as it was, the
/// control points of the segments it joins or splits included, so that undo restores it exactly.
///
/// These are the only steps that renumber a Wall's segments.
struct Reshape {
    /// The identity of the Wall.
    element: ElementId,
    /// The Wall after the step.
    wall: Wall,
    /// The Wall before the step, once applied.
    previous: Option<Wall>,
}

impl ReversibleCommand for Reshape {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        let entity = self.element.entity(world)?;
        let mut wall = world
            .get_mut::<Wall>(entity)
            .ok_or(AuthoringError::NotAWall(self.element))?;
        let previous = std::mem::replace(&mut *wall, self.wall.clone());
        if self.previous.is_none() {
            self.previous = Some(previous);
        }
        Ok(())
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        let Some(previous) = &self.previous else {
            return Ok(());
        };
        let entity = self.element.entity(world)?;
        *world
            .get_mut::<Wall>(entity)
            .ok_or(AuthoringError::NotAWall(self.element))? = previous.clone();
        Ok(())
    }
}

/// Refuses a Wall that would not be one, for the reason [`Wall::malformation`] gives.
///
/// # Errors
///
/// [`AuthoringError::MalformedWall`] with that reason.
pub(crate) fn well_formed(wall: &Wall) -> Result<(), AuthoringError> {
    wall.malformation()
        .map_or(Ok(()), |reason| Err(AuthoringError::MalformedWall(reason)))
}

/// The Wall an Element is.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] when no Element carries the identity, or
/// [`AuthoringError::NotAWall`] when it is no Wall.
pub(crate) fn wall_of(world: &mut World, element: ElementId) -> Result<Wall, AuthoringError> {
    let entity = element
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(element))?;
    world
        .get::<Wall>(entity)
        .cloned()
        .ok_or(AuthoringError::NotAWall(element))
}

/// The Wall moved so that its box is centred on `position`: every point and control point moved
/// by the same amount.
pub(crate) fn translated(wall: &Wall, position: Vec2) -> Wall {
    let offset = position - wall.element_box().center();
    let mut moved = wall.clone();
    for point in &mut moved.points {
        *point += offset;
    }
    for control in moved
        .segments
        .iter_mut()
        .filter_map(|segment| segment.control.as_mut())
    {
        *control += offset;
    }
    moved
}

/// Adds a point on a segment of a Wall, splitting it into two segments of the shape it had, as
/// one history step.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] or [`AuthoringError::NotAWall`] when the Element is no
/// Wall, [`AuthoringError::Shape`] when the segment or the parameter is not on the Wall, or
/// [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn add_point(
    world: &mut World,
    element: ElementId,
    segment: usize,
    t: f32,
) -> Result<(), AuthoringError> {
    let wall = split_wall(&wall_of(world, element)?, segment, t)?;
    crate::record_step(
        world,
        Reshape {
            element,
            wall,
            previous: None,
        },
    )
}

/// Removes a point of a Wall as one history step: the two segments at an inner point join into
/// one straight segment, and an end point takes its segment with it. A Wall of two points is
/// removed whole, in a group of its own.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] or [`AuthoringError::NotAWall`] when the Element is no
/// Wall, [`AuthoringError::NoPoint`] when the Wall has no such point, or
/// [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn remove_point(
    world: &mut World,
    element: ElementId,
    index: usize,
) -> Result<(), AuthoringError> {
    let mut wall = wall_of(world, element)?;
    let points = wall.points.len();
    if index >= points {
        return Err(AuthoringError::NoPoint { index, points });
    }
    if points <= 2 {
        crate::history(world)?.begin_group();
        let outcome = crate::record(world, Remove::of(element));
        crate::history(world)?.end_group();
        return outcome;
    }
    wall.points.remove(index);
    if index == 0 {
        wall.segments.remove(0);
    } else if index == points - 1 {
        wall.segments.pop();
    } else {
        wall.segments.remove(index);
        wall.segments[index - 1].control = None;
    }
    crate::record_step(
        world,
        Reshape {
            element,
            wall,
            previous: None,
        },
    )
}

/// Derives the shape of every Wall that changed since the last frame through the shape Engine,
/// and sets its Element's box around its points.
pub(crate) fn derive_shapes(
    mut commands: Commands,
    mut walls: Query<(Entity, &Wall, &mut Element, Option<&mut WallShape>), Changed<Wall>>,
) {
    for (entity, wall, mut element, shape) in &mut walls {
        let derived = generate_walls(wall);
        match shape {
            Some(mut shape) => *shape = derived,
            None => {
                commands.entity(entity).insert(derived);
            }
        }
        let footprint = wall.element_box();
        if element.position != footprint.center() {
            element.position = footprint.center();
        }
        if element.size != footprint.size() {
            element.size = footprint.size();
        }
    }
}
