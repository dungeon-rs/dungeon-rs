//! Walls: placing one, the edits that add and remove its points and carry the Portals set into
//! it, and the shape derived from it and from those Portals.

use crate::AuthoringError;
use crate::place::{spawn_on_top, take_off};
use crate::portal::{lost_in, set_into};
use crate::remove::Remove;
use bevy_ecs::change_detection::DetectChanges;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::lifecycle::RemovedComponents;
use bevy_ecs::query::{Changed, With, Without};
use bevy_ecs::system::{Commands, Query};
use bevy_ecs::world::World;
use bevy_math::Vec2;
use drs_history::{ReversibleCommand, SetField, Target};
use drs_model::{
    AssetReferences, Element, ElementId, Level, Portal, PortalAnchor, PortalsRemoved, Segment,
    WALL, Wall, WallShape,
};
use drs_shape_engine::{
    PointEdit, PortalSetting, anchor_portals, anchor_portals_through, generate_walls, split_wall,
};
use std::collections::BTreeMap;

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
        let footprint = self.wall.element_box();
        spawn_on_top(
            world,
            self.layer,
            (
                Element {
                    kind: WALL,
                    position: footprint.center(),
                    size: footprint.size(),
                },
                self.wall.clone(),
                self.element,
            ),
        )
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        take_off(world, self.element)
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
/// one history step that also moves each Portal set into the Wall to the segment and parameter
/// that keep it where it was, and each Portal anchored at a segment the Wall lacks one segment
/// on, so it still names none.
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
    let before = wall_of(world, element)?;
    let wall = split_wall(&before, segment, t)?;
    let portals = set_into(world, element);
    let places = anchor_portals_through(
        &before,
        PointEdit::Added { segment, t },
        &settings(&portals),
    );
    let mut moves = Vec::new();
    for ((portal, anchor, _), place) in portals.iter().zip(places) {
        if let Some(place) = place {
            moves.push(moved(*portal, *anchor, place.segment, place.t)?);
        }
    }
    // A Portal anchored at a segment the Wall lacks lies past every segment, so it moves one on
    // like a Portal on a later segment, and the new segment never becomes its own: it keeps
    // standing where it was saved.
    let lost = lost_in(world, element);
    let renumbered =
        anchor_portals_through(&before, PointEdit::Added { segment, t }, &settings(&lost));
    for ((portal, anchor, _), place) in lost.iter().zip(renumbered) {
        if let Some(place) = place {
            moves.push(moved(*portal, *anchor, place.segment, place.t)?);
        }
    }
    record_together(
        world,
        Vec::new(),
        Reshape {
            element,
            wall,
            previous: None,
        },
        moves,
    )
}

/// Removes a point of a Wall as one history step: the two segments at an inner point join into
/// one straight segment, and an end point takes its segment with it. A Wall of two points is
/// removed whole, in a group of its own. The Portals set into the part of the Wall that goes
/// are removed in the same step, before the point, and the other Portals set into the Wall move
/// to the segment and parameter that keep them on their part of it; the authoring Manager
/// answers with [`PortalsRemoved`] when any went.
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
    let before = wall_of(world, element)?;
    let points = before.points.len();
    if index >= points {
        return Err(AuthoringError::NoPoint { index, points });
    }
    if points <= 2 {
        return remove_with_portals(world, element);
    }
    let mut wall = before.clone();
    wall.points.remove(index);
    if index == 0 {
        wall.segments.remove(0);
    } else if index == points - 1 {
        wall.segments.pop();
    } else {
        wall.segments.remove(index);
        wall.segments[index - 1].control = None;
    }
    let portals = set_into(world, element);
    let places = anchor_portals_through(&before, PointEdit::Removed { index }, &settings(&portals));
    let mut gone = Vec::new();
    let mut moves = Vec::new();
    for ((portal, anchor, _), place) in portals.iter().zip(places) {
        match place {
            Some(place) => moves.push(moved(*portal, *anchor, place.segment, place.t)?),
            None => gone.push(*portal),
        }
    }
    record_together(
        world,
        gone.clone(),
        Reshape {
            element,
            wall,
            previous: None,
        },
        moves,
    )?;
    tell_removed(world, element, gone);
    Ok(())
}

/// Removes a Wall and every Portal set into it as one history step, the Portals first, so undo
/// restores the Wall and then its Portals; the authoring Manager answers with
/// [`PortalsRemoved`] when any went.
///
/// # Errors
///
/// [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn remove_with_portals(
    world: &mut World,
    element: ElementId,
) -> Result<(), AuthoringError> {
    let gone: Vec<ElementId> = set_into(world, element)
        .into_iter()
        .map(|(portal, ..)| portal)
        .collect();
    crate::history(world)?.begin_group();
    let mut outcome = Ok(());
    for portal in &gone {
        outcome = outcome.and_then(|()| crate::record(world, Remove::of(*portal)));
    }
    outcome = outcome.and_then(|()| crate::record(world, Remove::of(element)));
    crate::history(world)?.end_group();
    outcome?;
    tell_removed(world, element, gone);
    Ok(())
}

/// What `AnchorPortals` is told about the Portals set into a Wall.
fn settings(portals: &[(ElementId, PortalAnchor, f32)]) -> Vec<PortalSetting> {
    portals
        .iter()
        .map(|(_, anchor, width)| PortalSetting {
            segment: anchor.index,
            t: anchor.t,
            width: *width,
        })
        .collect()
}

/// The field command that moves a Portal's anchor to another segment and parameter, its side
/// kept.
///
/// # Errors
///
/// [`AuthoringError::History`] when the anchor cannot be addressed.
fn moved(
    portal: ElementId,
    anchor: PortalAnchor,
    segment: usize,
    t: f32,
) -> Result<SetField<ElementId>, AuthoringError> {
    SetField::<ElementId>::new::<Portal>(
        portal,
        "anchor",
        Some(PortalAnchor {
            index: segment,
            t,
            ..anchor
        }),
    )
    .map_err(|error| AuthoringError::History(error.to_string()))
}

/// Records the removal of the Portals `gone`, a reshape of their Wall, and the moves of the
/// Portals that stay as one history step, in that order, so undo restores the Wall's points
/// before its Portals. With no Portal involved the reshape is a step of its own.
///
/// # Errors
///
/// [`AuthoringError::History`] when a command could not be applied; what was applied before it
/// stays in the step.
fn record_together(
    world: &mut World,
    gone: Vec<ElementId>,
    reshape: Reshape,
    moves: Vec<SetField<ElementId>>,
) -> Result<(), AuthoringError> {
    if gone.is_empty() && moves.is_empty() {
        return crate::record_step(world, reshape);
    }
    crate::history(world)?.begin_group();
    let mut outcome = Ok(());
    for portal in gone {
        outcome = outcome.and_then(|()| crate::record(world, Remove::of(portal)));
    }
    outcome = outcome.and_then(|()| crate::record(world, reshape));
    for field in moves {
        outcome = outcome.and_then(|()| crate::record(world, field));
    }
    crate::history(world)?.end_group();
    outcome
}

/// Tells the Editor which Portals set into `host` a Command removed, when it removed any.
fn tell_removed(world: &mut World, host: ElementId, portals: Vec<ElementId>) {
    if !portals.is_empty() {
        world.write_message(PortalsRemoved { host, portals });
    }
}

/// What a Wall's shape was last derived from: its points, segments, and thickness, and where
/// the Portals set into it are and how wide. A change that leaves them as they were, a new
/// colour, keeps the shape.
#[derive(Component, Debug, Clone, PartialEq)]
pub(crate) struct DerivedFrom {
    /// The points the shape was derived from.
    points: Vec<Vec2>,
    /// The segments it was derived from.
    segments: Vec<Segment>,
    /// The thickness it was derived at.
    thickness: f32,
    /// The Portals set into the Wall, in the order of their identities.
    portals: Vec<PortalSetting>,
}

impl DerivedFrom {
    /// What `wall`'s shape is derived from, with the Portals set into it.
    fn of(wall: &Wall, portals: &[(ElementId, PortalAnchor, f32)]) -> Self {
        Self {
            points: wall.points.clone(),
            segments: wall.segments.clone(),
            thickness: wall.thickness,
            portals: settings(portals),
        }
    }
}

/// The nearest ancestor of `entity` that `found` accepts.
fn ancestor(
    entity: Entity,
    parents: &Query<&ChildOf>,
    found: impl Fn(Entity) -> bool,
) -> Option<Entity> {
    let mut current = entity;
    while let Ok(parent) = parents.get(current).map(ChildOf::parent) {
        if found(parent) {
            return Some(parent);
        }
        current = parent;
    }
    None
}

/// The Walls as deriving reads and writes them.
type Walls<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ElementId,
        &'static Wall,
        &'static mut Element,
        Option<&'static mut WallShape>,
        Option<&'static DerivedFrom>,
    ),
    Without<Portal>,
>;

/// The Portals as deriving reads and writes them.
type Portals<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ElementId,
        &'static mut Portal,
        &'static mut Element,
    ),
    Without<Wall>,
>;

/// Derives the shape of every Wall whose points, segments, or thickness changed since the last
/// frame, or whose Portals were placed, edited, set, freed, or removed, through the shape
/// Engine, and sets its Element's box around its points; moves each Portal set into such a Wall
/// to where its anchor puts it, turned to the Wall's direction there and mirrored when it faces
/// the right; and sets every changed Portal's size from its width and its image's recorded pixel
/// size.
///
/// A Portal whose anchor names no Wall of its Level, or a segment its Wall lacks, is left as it
/// is and leaves no gap.
#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system is spelled out by the components it reads and writes"
)]
pub(crate) fn derive_shapes(
    mut commands: Commands,
    changed_walls: Query<(), Changed<Wall>>,
    mut removed_portals: RemovedComponents<Portal>,
    mut walls: Walls,
    mut portals: Portals,
    parents: Query<&ChildOf>,
    levels: Query<(), With<Level>>,
    references: Query<&AssetReferences>,
) {
    let removed = removed_portals.read().count() > 0;
    // Looking at whether a Portal changed through the query that writes them marks nothing.
    let portal_changed = portals
        .iter_mut()
        .any(|(_, _, portal, _)| portal.is_changed());
    if changed_walls.is_empty() && !portal_changed && !removed {
        return;
    }
    let level_of = |entity: Entity| ancestor(entity, &parents, |parent| levels.contains(parent));
    let set = set_by_host(&walls, &portals, level_of);
    let sides: BTreeMap<ElementId, bool> = set
        .values()
        .flatten()
        .map(|(id, anchor, _)| (*id, anchor.side.mirrors()))
        .collect();

    let mut standings = BTreeMap::new();
    for (entity, id, wall, mut element, shape, derived_from) in &mut walls {
        let into = set.get(id).map_or(&[][..], Vec::as_slice);
        let geometry = DerivedFrom::of(wall, into);
        if shape.is_some() && derived_from == Some(&geometry) {
            continue;
        }
        let placed = anchor_portals(wall, &geometry.portals);
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
        match shape {
            Some(mut shape) => *shape = generate_walls(wall, &stretches),
            None => {
                commands
                    .entity(entity)
                    .insert(generate_walls(wall, &stretches));
            }
        }
        commands.entity(entity).insert(geometry);
        let footprint = wall.element_box();
        if element.position != footprint.center() {
            element.position = footprint.center();
        }
        if element.size != footprint.size() {
            element.size = footprint.size();
        }
    }

    for (entity, id, mut portal, mut element) in &mut portals {
        let changed = portal.is_changed();
        if let Some(standing) = standings.get(id) {
            if element.position != standing.centre {
                element.position = standing.centre;
            }
            if let Some(direction) = standing.direction
                && portal.rotation.to_bits() != direction.to_bits()
            {
                portal.rotation = direction;
            }
        }
        // The side changes nothing about the Wall's shape, so it is followed for every Portal set
        // into a Wall, whether or not the Wall was derived again.
        if let Some(mirrored) = sides.get(id)
            && portal.mirrored != *mirrored
        {
            portal.mirrored = *mirrored;
        }
        if changed {
            let pixels = ancestor(entity, &parents, |parent| references.contains(parent))
                .and_then(|project| references.get(project).ok())
                .and_then(|table| table.get(portal.asset))
                .and_then(|reference| reference.pixel_size);
            let size = natural_size(portal.width, pixels, element.size);
            if element.size != size {
                element.size = size;
            }
        }
    }
}

/// The Portals set into each Wall, by the Wall's identity and in the order of their own, with
/// their anchors and widths: those whose anchor names a Wall on their Level and a segment it
/// has.
fn set_by_host(
    walls: &Walls,
    portals: &Portals,
    level_of: impl Fn(Entity) -> Option<Entity>,
) -> BTreeMap<ElementId, Vec<(ElementId, PortalAnchor, f32)>> {
    let hosts: BTreeMap<ElementId, (Entity, usize)> = walls
        .iter()
        .map(|(entity, id, wall, ..)| (*id, (entity, wall.segments.len())))
        .collect();
    let mut set: BTreeMap<ElementId, Vec<(ElementId, PortalAnchor, f32)>> = BTreeMap::new();
    for (entity, id, portal, _) in portals {
        let Some(anchor) = portal.anchor else {
            continue;
        };
        let Some(&(host, segments)) = hosts.get(&anchor.host) else {
            continue;
        };
        if anchor.index < segments
            && (0.0..=1.0).contains(&anchor.t)
            && level_of(entity) == level_of(host)
        {
            set.entry(anchor.host)
                .or_default()
                .push((*id, anchor, portal.width));
        }
    }
    for portals in set.values_mut() {
        portals.sort_by_key(|(id, ..)| *id);
    }
    set
}

/// The size of a Portal `width` cells wide: its image's proportions when its pixel size is
/// recorded, and otherwise the proportions of `current`.
#[expect(
    clippy::cast_precision_loss,
    reason = "image sides are far below where f32 loses whole numbers"
)]
fn natural_size(width: f32, pixels: Option<bevy_math::UVec2>, current: Vec2) -> Vec2 {
    let height = match pixels {
        Some(pixels) if pixels.x > 0 => width * pixels.y as f32 / pixels.x as f32,
        _ if current.x > 0.0 => width * current.y / current.x,
        _ => width,
    };
    Vec2::new(width, height)
}
