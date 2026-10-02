//! Walls: placing one, the edits that add and remove its points and carry the Portals set into
//! it, the steps those edits share with a Room's, and the shape derived from every Wall and Room
//! and from the Portals set into them.

use crate::AuthoringError;
use crate::ancestor;
use crate::place::{spawn_on_top, take_off};
use crate::portal::{Anchored, anchored_to, sets_into, stood};
use crate::remove::Remove;
use crate::room::{Rooms, reshape_rooms};
use bevy_ecs::change_detection::{DetectChanges, Mut};
use bevy_ecs::component::{Component, Mutable};
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::lifecycle::RemovedComponents;
use bevy_ecs::query::{Changed, Or, With, Without};
use bevy_ecs::system::{Commands, Query};
use bevy_ecs::world::World;
use bevy_math::Vec2;
use drs_history::{ReversibleCommand, SetField, Target};
use drs_model::{
    AssetReferences, Element, ElementId, Level, LinePlace, Portal, PortalAnchor, PortalsRemoved,
    Room, Segment, WALL, Wall, WallShape,
};
use drs_shape_engine::{
    Path, PointEdit, PortalSetting, Standing, anchor_portals, anchor_portals_through,
    combine_outlines, generate_walls, split_wall,
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

/// The recorded step of adding or removing a point: the Wall or the Room as it becomes and as it
/// was, the control points of the segments or edges it joins or splits included, so that undo
/// restores it exactly.
///
/// These are the only steps that renumber a Wall's segments or a Room's edges.
pub(crate) struct Reshape<C> {
    /// The identity of the Wall or the Room.
    pub(crate) element: ElementId,
    /// The Wall or the Room after the step.
    pub(crate) outline: C,
    /// The Wall or the Room before the step, once applied.
    pub(crate) previous: Option<C>,
}

impl<C: Component<Mutability = Mutable> + Clone> ReversibleCommand for Reshape<C> {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        let entity = self.element.entity(world)?;
        let mut outline = world
            .get_mut::<C>(entity)
            .ok_or(AuthoringError::NotAWall(self.element))?;
        let previous = std::mem::replace(&mut *outline, self.outline.clone());
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
            .get_mut::<C>(entity)
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
    let path = Path::of_wall(&before);
    let split = split_wall(&path, segment, t)?;
    let wall = Wall {
        points: split.points,
        segments: split
            .controls
            .into_iter()
            .map(|control| Segment { control })
            .collect(),
        ..before
    };
    let moves = carried_past(world, element, &path, segment, t)?;
    record_together(
        world,
        Vec::new(),
        Reshape {
            element,
            outline: wall,
            previous: None,
        },
        moves,
    )
}

/// The anchor moves that keep every Portal anchored to `host` where it was when a point is added
/// on `part` at `t`, `before` being the host's outline as it was.
///
/// # Errors
///
/// [`AuthoringError::History`] when an anchor cannot be addressed.
pub(crate) fn carried_past(
    world: &mut World,
    host: ElementId,
    before: &Path,
    part: usize,
    t: f32,
) -> Result<Vec<SetField<ElementId>>, AuthoringError> {
    // A Portal anchored at a part the host lacks lies past every part, so it moves one on like a
    // Portal on a later part and never comes to name the new one: it keeps standing where it was
    // saved.
    let (set, lost) = anchored_to(world, host);
    let anchored: Vec<Anchored> = set.into_iter().chain(lost).collect();
    let places = anchor_portals_through(
        before,
        PointEdit::Added { segment: part, t },
        &settings(&anchored),
    );
    let mut moves = Vec::new();
    for ((portal, anchor, _), place) in anchored.iter().zip(places) {
        if let Some(place) = place
            && let Some(field) = moved(*portal, *anchor, place)?
        {
            moves.push(field);
        }
    }
    Ok(moves)
}

/// Removes a point of a Wall as one history step: the two segments at an inner point join into
/// one straight segment, and an end point takes its segment with it. A Wall of two points is
/// removed whole, in a group of its own. The Portals set into the part of the Wall that goes
/// are removed in the same step, before the point, and the other Portals set into the Wall move
/// to the segment and parameter that keep them on their part of it. Returns the answer naming
/// the Portals that went, when any did.
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
) -> Result<Option<PortalsRemoved>, AuthoringError> {
    let before = wall_of(world, element)?;
    let points = before.points.len();
    if index >= points {
        return Err(AuthoringError::NoPoint {
            outline: "Wall",
            index,
            points,
        });
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
    let (portals, _) = anchored_to(world, element);
    let places = anchor_portals_through(
        &Path::of_wall(&before),
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
            outline: wall,
            previous: None,
        },
        moves,
    )?;
    Ok(removed(element, gone))
}

/// Removes a Wall or a Room and every Portal set into it as one history step, the Portals first,
/// so undo restores the host and then its Portals. Returns the answer naming the Portals that went, when
/// any did.
///
/// # Errors
///
/// [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn remove_with_portals(
    world: &mut World,
    element: ElementId,
) -> Result<Option<PortalsRemoved>, AuthoringError> {
    let gone: Vec<ElementId> = anchored_to(world, element)
        .0
        .into_iter()
        .map(|(portal, ..)| portal)
        .collect();
    crate::history(world)?.begin_group();
    let mut outcome = Ok(());
    for portal in &gone {
        outcome = outcome.and_then(|()| crate::record(world, Remove::of(*portal)));
    }
    outcome = outcome.and_then(|()| crate::record(world, Remove::of(element)));
    crate::close_group(world, outcome)?;
    Ok(removed(element, gone))
}

/// What `AnchorPortals` is told about the Portals set into a Wall or a Room.
pub(crate) fn settings(portals: &[Anchored]) -> Vec<PortalSetting> {
    portals
        .iter()
        .map(|(_, anchor, width)| PortalSetting {
            segment: anchor.index,
            t: anchor.t,
            width: *width,
        })
        .collect()
}

/// The field command that moves a Portal's anchor to another place along its Wall or Room, its
/// side kept, or `None` when the place is the one it has, so a step records no change that is
/// none.
///
/// # Errors
///
/// [`AuthoringError::History`] when the anchor cannot be addressed.
pub(crate) fn moved(
    portal: ElementId,
    anchor: PortalAnchor,
    place: LinePlace,
) -> Result<Option<SetField<ElementId>>, AuthoringError> {
    if (anchor.index, anchor.t.to_bits()) == (place.segment, place.t.to_bits()) {
        return Ok(None);
    }
    SetField::<ElementId>::new::<Portal>(
        portal,
        "anchor",
        Some(PortalAnchor {
            index: place.segment,
            t: place.t,
            ..anchor
        }),
    )
    .map(Some)
    .map_err(|error| AuthoringError::History(error.to_string()))
}

/// Records the removal of the Portals `gone`, a reshape of their Wall or Room, and the moves of
/// the Portals that stay as one history step, in that order, so undo restores the host's points
/// before its Portals. With no Portal involved the reshape is a step of its own.
///
/// # Errors
///
/// [`AuthoringError::History`] when a command could not be applied; what was applied before it
/// is taken back, so nothing of the step is left.
pub(crate) fn record_together(
    world: &mut World,
    gone: Vec<ElementId>,
    reshape: impl ReversibleCommand,
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
    crate::close_group(world, outcome)
}

/// The answer naming the Portals set into `host` that a Command removed, when it removed any.
pub(crate) fn removed(host: ElementId, portals: Vec<ElementId>) -> Option<PortalsRemoved> {
    (!portals.is_empty()).then_some(PortalsRemoved { host, portals })
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
    fn of(wall: &Wall, portals: &[Anchored]) -> Self {
        Self {
            points: wall.points.clone(),
            segments: wall.segments.clone(),
            thickness: wall.thickness,
            portals: settings(portals),
        }
    }
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
    (Without<Portal>, Without<Room>),
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
    (Without<Wall>, Without<Room>),
>;

/// Derives the shape of every Wall and every Room whose points, segments or edges, or thickness
/// changed since the last frame, or whose Portals were placed, edited, set, freed, or removed,
/// through the shape Engine, and sets its Element's box around its points; moves each Portal set
/// into such a Wall or Room to where its anchor puts it, turned to the line's direction there and
/// mirrored when it faces the right; and sets every changed Portal's size from its width and its
/// image's recorded pixel size.
///
/// A Portal whose anchor names no Wall or Room of its Level, or a part its host lacks, is left as
/// it is and leaves no gap.
#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "a Bevy system is spelled out by the components it reads and writes"
)]
pub(crate) fn derive_shapes(
    mut commands: Commands,
    changed_outlines: Query<(), Or<(Changed<Wall>, Changed<Room>)>>,
    mut removed_portals: RemovedComponents<Portal>,
    mut walls: Walls,
    mut rooms: Rooms,
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
    if changed_outlines.is_empty() && !portal_changed && !removed {
        return;
    }
    let parent_of = |child: Entity| parents.get(child).ok().map(ChildOf::parent);
    let level_of = |entity: Entity| ancestor(entity, parent_of, |parent| levels.contains(parent));
    let set = set_by_host(&walls, &rooms, &portals, level_of);
    let mut standings = reshape_walls(&mut commands, &mut walls, &set);
    standings.append(&mut reshape_rooms(&mut commands, &mut rooms, &set));
    let anchors: BTreeMap<ElementId, PortalAnchor> = set
        .values()
        .flatten()
        .map(|(id, anchor, _)| (*id, *anchor))
        .collect();

    for (entity, id, mut portal, mut element) in &mut portals {
        let changed = portal.is_changed();
        if let Some(anchor) = anchors.get(id) {
            follow(&mut portal, &mut element, anchor, standings.get(id));
        }
        if changed {
            let pixels = ancestor(entity, parent_of, |parent| references.contains(parent))
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

/// Derives again the shape of every Wall whose points, segments, or thickness, or whose set
/// Portals, differ from what its shape was last derived from, leaving out the stretches those
/// Portals cover, and sets its Element's box around its points. Returns where each Portal set
/// into those Walls stands.
fn reshape_walls(
    commands: &mut Commands,
    walls: &mut Walls,
    set: &BTreeMap<ElementId, Vec<Anchored>>,
) -> BTreeMap<ElementId, Standing> {
    let mut standings = BTreeMap::new();
    for (entity, id, wall, mut element, wall_shape, derived_from) in walls {
        let into = set.get(id).map_or(&[][..], Vec::as_slice);
        let geometry = DerivedFrom::of(wall, into);
        if wall_shape.is_some() && derived_from == Some(&geometry) {
            continue;
        }
        let path = Path::of_wall(wall);
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
        let shape = generate_walls(&combine_outlines(&path), wall.thickness, &stretches);
        match wall_shape {
            Some(mut wall_shape) => *wall_shape = shape,
            None => {
                commands.entity(entity).insert(shape);
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
    standings
}

/// Keeps a Portal set into a Wall or a Room where its anchor puts it: at `standing`, turned to
/// the line's direction there, when its host was derived again, and facing its side in any case,
/// since the side changes nothing about the host's shape. Writes only what differs.
fn follow(
    portal: &mut Mut<Portal>,
    element: &mut Mut<Element>,
    anchor: &PortalAnchor,
    standing: Option<&Standing>,
) {
    let (centre, rotation, mirrored) = standing.map_or(
        (element.position, portal.rotation, anchor.side.mirrors()),
        |standing| stood(standing, anchor, portal.rotation),
    );
    if element.position != centre {
        element.position = centre;
    }
    if portal.rotation.to_bits() != rotation.to_bits() {
        portal.rotation = rotation;
    }
    if portal.mirrored != mirrored {
        portal.mirrored = mirrored;
    }
}

/// The Portals set into each Wall and each Room, by the host's identity and in the order of
/// their own, with their anchors and widths: those whose anchor names a Wall or a Room on their
/// Level and a segment or edge it has.
fn set_by_host(
    walls: &Walls,
    rooms: &Rooms,
    portals: &Portals,
    level_of: impl Fn(Entity) -> Option<Entity>,
) -> BTreeMap<ElementId, Vec<Anchored>> {
    let hosts: BTreeMap<ElementId, (Entity, usize)> = walls
        .iter()
        .map(|(entity, id, wall, ..)| (*id, (entity, wall.segments.len())))
        .chain(
            rooms
                .iter()
                .map(|(entity, id, room, ..)| (*id, (entity, room.edges.len()))),
        )
        .collect();
    let mut set: BTreeMap<ElementId, Vec<Anchored>> = BTreeMap::new();
    for (entity, id, portal, _) in portals {
        let Some(anchor) = portal.anchor else {
            continue;
        };
        let Some(&(host, parts)) = hosts.get(&anchor.host) else {
            continue;
        };
        if sets_into(&anchor, parts, (level_of(entity), level_of(host))) {
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
