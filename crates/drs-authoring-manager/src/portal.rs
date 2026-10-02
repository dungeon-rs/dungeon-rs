//! Portals: placing one set into a Wall or a Room or freestanding, Set Portal into Wall and Free
//! Portal, the edits only a Portal has, and finding the Portals set into a Wall or a Room.

use crate::AuthoringError;
use crate::place::{Place, Spawned, resolve};
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::world::World;
use bevy_math::Vec2;
use drs_history::{ReversibleCommand, SetField, Target};
use drs_model::{
    AssetAddress, AssetReferenceRow, Element, ElementChange, ElementId, FreePortal, Level, Portal,
    PortalAnchor, Room, SetPortalIntoWall, Wall,
};
use drs_shape_engine::{Path, PortalSetting, Standing, anchor_portals};

/// The outline of the Wall or the Room an entity carries, if it carries either: what a Portal is
/// set into, its parts being the Wall's segments or the Room's edges. The kind of the Element an
/// anchor names says how its `index` is read.
pub(crate) fn host_path(world: &World, entity: Entity) -> Option<Path> {
    world
        .get::<Wall>(entity)
        .map(Path::of_wall)
        .or_else(|| world.get::<Room>(entity).map(Path::of_room))
}

/// The Level an Element lies on: its nearest ancestor carrying [`Level`].
pub(crate) fn level_of(world: &World, entity: Entity) -> Option<Entity> {
    crate::ancestor(
        entity,
        |child| world.get::<ChildOf>(child).map(ChildOf::parent),
        |parent| world.get::<Level>(parent).is_some(),
    )
}

/// Whether a Portal anchored at `anchor` is set into the Wall or the Room it names, of `parts`
/// segments or edges: the anchor names a part the host has and a parameter from zero to one, and
/// the Portal lies on the host's Level, `levels` being the Portal's and the host's.
pub(crate) fn sets_into(
    anchor: &PortalAnchor,
    parts: usize,
    levels: (Option<Entity>, Option<Entity>),
) -> bool {
    anchor.index < parts && (0.0..=1.0).contains(&anchor.t) && levels.0 == levels.1
}

/// The Wall or the Room an anchor names, checked for a Portal on `level`: the host must be a
/// Wall or a Room on that Level with the segment or edge named. Whether the parameter is one is
/// the Portal's own check.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] or [`AuthoringError::NotAHost`] when the host is neither a
/// Wall nor a Room, [`AuthoringError::OnAnotherLevel`] when it lies on another Level, or
/// [`AuthoringError::NoSegment`] or [`AuthoringError::NoEdge`] when it has no such part.
pub(crate) fn host_of(
    world: &mut World,
    anchor: &PortalAnchor,
    level: Option<Entity>,
) -> Result<Path, AuthoringError> {
    let entity = anchor
        .host
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(anchor.host))?;
    let host = host_path(world, entity).ok_or(AuthoringError::NotAHost(anchor.host))?;
    if level_of(world, entity) != level {
        return Err(AuthoringError::OnAnotherLevel(anchor.host));
    }
    let parts = host.parts();
    if anchor.index >= parts {
        return Err(if host.closed {
            AuthoringError::NoEdge {
                edge: anchor.index,
                edges: parts,
            }
        } else {
            AuthoringError::NoSegment {
                segment: anchor.index,
                segments: parts,
            }
        });
    }
    Ok(host)
}

/// Whether the Portal `id` follows a host: it is anchored, and its anchor names a Wall or a Room
/// on the Portal's Level and a segment or edge it has. A Portal whose anchor names none, as an editor
/// that does not know Portals may leave it, is lost: it keeps its anchor but stands, turns, and
/// mirrors as a freestanding one.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] or [`AuthoringError::NotAPortal`] when the Element is no
/// Portal.
pub(crate) fn follows_host(world: &mut World, id: ElementId) -> Result<bool, AuthoringError> {
    let Some(anchor) = portal_of(world, id)?.anchor else {
        return Ok(false);
    };
    let entity = id
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(id))?;
    let level = level_of(world, entity);
    Ok(host_of(world, &anchor, level).is_ok())
}

/// Refuses a Portal that would not be one, for the reason [`Portal::malformation`] gives.
///
/// # Errors
///
/// [`AuthoringError::MalformedPortal`] with that reason.
pub(crate) fn well_formed(portal: &Portal) -> Result<(), AuthoringError> {
    portal.malformation().map_or(Ok(()), |reason| {
        Err(AuthoringError::MalformedPortal(reason))
    })
}

/// Where a Portal set at `anchor` stands where `AnchorPortals` places it: its centre, its
/// rotation, which stays `rotation` where the segment has no direction, and its mirroring, by the
/// side it faces.
pub(crate) fn stood(
    standing: &Standing,
    anchor: &PortalAnchor,
    rotation: f32,
) -> (Vec2, f32, bool) {
    (
        standing.centre,
        standing.direction.unwrap_or(rotation),
        anchor.side.mirrors(),
    )
}

/// Where a Portal of `width` set at `anchor` into the outline `host` stands, as [`stood`] says.
fn standing_in(
    host: &Path,
    anchor: &PortalAnchor,
    width: f32,
    rotation: f32,
) -> Option<(Vec2, f32, bool)> {
    let setting = PortalSetting {
        segment: anchor.index,
        t: anchor.t,
        width,
    };
    let standing = anchor_portals(host, &[setting])
        .into_iter()
        .next()
        .flatten()?;
    Some(stood(&standing, anchor, rotation))
}

/// Places a Portal of the chosen Asset at its image's natural size on top of the Layer, set
/// into the Wall or the Room the anchor names or freestanding, unturned, and centred on `position`, as one
/// history step.
///
/// # Errors
///
/// [`AuthoringError::MalformedPortal`] for a position that is not finite or a Portal that would
/// not be one, what [`host_of`] reports for an anchor it refuses, what resolving the Asset
/// reports, or [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn place_portal(
    world: &mut World,
    layer: Entity,
    position: Vec2,
    asset: &AssetAddress,
    anchor: Option<PortalAnchor>,
) -> Result<(), AuthoringError> {
    if !position.is_finite() {
        return Err(AuthoringError::MalformedPortal(
            "a Portal's position must be finite".to_owned(),
        ));
    }
    let host = match &anchor {
        Some(anchor) => {
            let level = level_of(world, layer);
            Some(host_of(world, anchor, level)?)
        }
        None => None,
    };
    let resolved = resolve(world, layer, asset)?;
    let mut portal = Portal {
        // The row is the one the Project records the Asset in when the step is applied; which
        // row it is changes nothing about whether the Portal is one.
        asset: AssetReferenceRow(0),
        width: resolved.size.x,
        rotation: 0.0,
        mirrored: false,
        anchor,
    };
    well_formed(&portal)?;
    let mut position = position;
    if let (Some(host), Some(anchor)) = (&host, &anchor)
        && let Some((centre, rotation, mirrored)) =
            standing_in(host, anchor, portal.width, portal.rotation)
    {
        position = centre;
        portal.rotation = rotation;
        portal.mirrored = mirrored;
    }
    crate::record_step(
        world,
        Place {
            layer,
            position,
            resolved,
            spawned: Spawned::Portal(portal),
            element: ElementId::new(),
        },
    )
}

/// The Portal an Element is.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] when no Element carries the identity, or
/// [`AuthoringError::NotAPortal`] when it is no Portal.
pub(crate) fn portal_of(world: &mut World, element: ElementId) -> Result<Portal, AuthoringError> {
    let entity = element
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(element))?;
    world
        .get::<Portal>(entity)
        .cloned()
        .ok_or(AuthoringError::NotAPortal(element))
}

/// How a Portal stood before a Set Portal into Wall: its anchor, position, rotation, and
/// mirroring.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Stood {
    /// Its anchor.
    anchor: Option<PortalAnchor>,
    /// Its centre.
    position: Vec2,
    /// Its rotation.
    rotation: f32,
    /// Its mirroring.
    mirrored: bool,
}

/// The recorded step of setting a Portal into a Wall: the anchor it is given, and how it stood
/// before, so that undo returns a freestanding Portal to the angle it stood at rather than the
/// Wall's.
struct SetIntoWall {
    /// The Portal.
    portal: ElementId,
    /// Where it is set.
    anchor: PortalAnchor,
    /// How it stood before, once applied.
    before: Option<Stood>,
}

impl ReversibleCommand for SetIntoWall {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        let entity = self.portal.entity(world)?;
        let position = world
            .get::<Element>(entity)
            .map(|element| element.position)
            .ok_or(AuthoringError::NotAPortal(self.portal))?;
        let mut portal = world
            .get_mut::<Portal>(entity)
            .ok_or(AuthoringError::NotAPortal(self.portal))?;
        let stood = Stood {
            anchor: portal.anchor,
            position,
            rotation: portal.rotation,
            mirrored: portal.mirrored,
        };
        portal.anchor = Some(self.anchor);
        if self.before.is_none() {
            self.before = Some(stood);
        }
        Ok(())
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        let Some(before) = self.before else {
            return Ok(());
        };
        let entity = self.portal.entity(world)?;
        let mut portal = world
            .get_mut::<Portal>(entity)
            .ok_or(AuthoringError::NotAPortal(self.portal))?;
        portal.anchor = before.anchor;
        portal.rotation = before.rotation;
        portal.mirrored = before.mirrored;
        world
            .get_mut::<Element>(entity)
            .ok_or(AuthoringError::NotAPortal(self.portal))?
            .position = before.position;
        Ok(())
    }
}

/// Set Portal into Wall: anchors a Portal, freestanding or set into any Wall, to the given
/// Wall, segment, parameter, and side, as one history step.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] or [`AuthoringError::NotAPortal`] when the Element is no
/// Portal, what [`host_of`] reports for an anchor it refuses,
/// [`AuthoringError::MalformedPortal`] for a parameter outside zero to one, or
/// [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn set_portal_into_wall(
    world: &mut World,
    command: &SetPortalIntoWall,
) -> Result<(), AuthoringError> {
    let portal = portal_of(world, command.portal)?;
    let entity = command
        .portal
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(command.portal))?;
    let level = level_of(world, entity);
    host_of(world, &command.anchor, level)?;
    well_formed(&Portal {
        anchor: Some(command.anchor),
        ..portal
    })?;
    crate::record_step(
        world,
        SetIntoWall {
            portal: command.portal,
            anchor: command.anchor,
            before: None,
        },
    )
}

/// Free Portal: makes a Portal set into a Wall freestanding with the position, rotation, and
/// mirroring it has, as one history step. Those already hold where it stands, so freeing is
/// clearing the anchor.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] or [`AuthoringError::NotAPortal`] when the Element is no
/// Portal, [`AuthoringError::Freestanding`] when it is set into no Wall, or
/// [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn free_portal(world: &mut World, command: &FreePortal) -> Result<(), AuthoringError> {
    let portal = portal_of(world, command.portal)?;
    if portal.anchor.is_none() {
        return Err(AuthoringError::Freestanding);
    }
    let free = SetField::<ElementId>::new::<Portal>(command.portal, "anchor", None::<PortalAnchor>)
        .map_err(|error| AuthoringError::History(error.to_string()))?;
    crate::record_step(world, free)
}

/// The field command of an Edit Element change only a Portal has, checked against the Portal
/// as it is: its width, the rotation and mirroring of a Portal that follows no Wall, and a set
/// Portal's side and place along its Wall. `None` for a change that is not one of those.
///
/// # Errors
///
/// [`AuthoringError::NotAPortal`] when the Element is no Portal,
/// [`AuthoringError::MalformedPortal`] for a width not above zero, a rotation that is not
/// finite, or a parameter outside zero to one, [`AuthoringError::FollowsItsWall`] for the
/// rotation or mirroring of a Portal that follows its Wall,
/// [`AuthoringError::Freestanding`] for the side or place of a freestanding one, what
/// [`host_of`] reports for a place its Wall does not have, or [`AuthoringError::History`] when
/// the field cannot be addressed.
pub(crate) fn portal_change(
    world: &mut World,
    id: ElementId,
    change: &ElementChange,
) -> Result<Option<SetField<ElementId>>, AuthoringError> {
    let history = |error: drs_history::HistoryError| AuthoringError::History(error.to_string());
    let field = match change {
        ElementChange::Width(width) => {
            let mut portal = portal_of(world, id)?;
            portal.width = *width;
            well_formed(&portal)?;
            SetField::<ElementId>::new::<Portal>(id, "width", *width)
        }
        ElementChange::Rotation(rotation) => {
            let mut portal = portal_of(world, id)?;
            if follows_host(world, id)? {
                return Err(AuthoringError::FollowsItsWall);
            }
            portal.rotation = *rotation;
            well_formed(&portal)?;
            SetField::<ElementId>::new::<Portal>(id, "rotation", *rotation)
        }
        ElementChange::Mirrored(mirrored) => {
            if follows_host(world, id)? {
                return Err(AuthoringError::FollowsItsWall);
            }
            SetField::<ElementId>::new::<Portal>(id, "mirrored", *mirrored)
        }
        ElementChange::Side(side) => {
            let mut anchor = portal_of(world, id)?
                .anchor
                .ok_or(AuthoringError::Freestanding)?;
            anchor.side = *side;
            SetField::<ElementId>::new::<Portal>(id, "anchor", Some(anchor))
        }
        ElementChange::Along { segment, t } => {
            let mut portal = portal_of(world, id)?;
            let anchor = portal.anchor.as_mut().ok_or(AuthoringError::Freestanding)?;
            anchor.index = *segment;
            anchor.t = *t;
            let anchor = *anchor;
            let entity = id
                .entity(world)
                .map_err(|_| AuthoringError::UnknownElement(id))?;
            let level = level_of(world, entity);
            host_of(world, &anchor, level)?;
            well_formed(&portal)?;
            SetField::<ElementId>::new::<Portal>(id, "anchor", Some(anchor))
        }
        ElementChange::Position(_)
        | ElementChange::Point { .. }
        | ElementChange::Control { .. }
        | ElementChange::AddPoint { .. }
        | ElementChange::RemovePoint { .. }
        | ElementChange::Thickness(_)
        | ElementChange::Colour(_)
        | ElementChange::FloorColour(_)
        | ElementChange::Material(_) => return Ok(None),
    };
    field.map(Some).map_err(history)
}

/// A Portal anchored to a Wall or a Room, with its anchor and its width.
pub(crate) type Anchored = (ElementId, PortalAnchor, f32);

/// The Portals whose anchor names the Wall or the Room `host`, in the order of their identities
/// with their anchors and widths: first those set into it, then those on its Level it does not
/// hold, whose anchor names a part it lacks, as an editor that does not know Portals leaves them.
pub(crate) fn anchored_to(world: &mut World, host: ElementId) -> (Vec<Anchored>, Vec<Anchored>) {
    let Ok(host_entity) = host.entity(world) else {
        return (Vec::new(), Vec::new());
    };
    let parts = host_path(world, host_entity).map_or(0, |host| host.parts());
    let level = level_of(world, host_entity);
    let mut portals: Vec<(Entity, ElementId, PortalAnchor, f32)> = world
        .query::<(Entity, &ElementId, &Portal)>()
        .iter(world)
        .filter_map(|(entity, id, portal)| {
            let anchor = portal.anchor?;
            (anchor.host == host).then_some((entity, *id, anchor, portal.width))
        })
        .collect();
    portals.sort_by_key(|(_, id, ..)| *id);
    let mut set = Vec::new();
    let mut lost = Vec::new();
    for (entity, id, anchor, width) in portals {
        let levels = (level_of(world, entity), level);
        if sets_into(&anchor, parts, levels) {
            set.push((id, anchor, width));
        } else if levels.0 == levels.1 {
            lost.push((id, anchor, width));
        }
    }
    (set, lost)
}
