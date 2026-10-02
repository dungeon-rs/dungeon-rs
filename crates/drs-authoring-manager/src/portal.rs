//! Portals: placing one set into a Wall or freestanding, Set Portal into Wall and Free Portal,
//! the edits only a Portal has, and finding the Portals set into a Wall.

use crate::AuthoringError;
use crate::place::{Place, Shown, resolve};
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::world::World;
use bevy_math::Vec2;
use drs_history::{ReversibleCommand, SetField, Target};
use drs_model::{
    AssetAddress, Element, ElementChange, ElementId, FreePortal, Level, Portal, PortalAnchor,
    SetPortalIntoWall, Wall,
};
use drs_shape_engine::{PortalSetting, anchor_portals};

/// The Level an Element lies on: its nearest ancestor carrying [`Level`].
pub(crate) fn level_of(world: &World, entity: Entity) -> Option<Entity> {
    let mut current = entity;
    while let Some(parent) = world.get::<ChildOf>(current).map(ChildOf::parent) {
        if world.get::<Level>(parent).is_some() {
            return Some(parent);
        }
        current = parent;
    }
    None
}

/// The Wall an anchor names, checked for a Portal on `level`: the host must be a Wall on that
/// Level with the segment named, and the parameter between zero and one.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] or [`AuthoringError::NotAWall`] when the host is no Wall,
/// [`AuthoringError::OnAnotherLevel`] when it lies on another Level,
/// [`AuthoringError::NoSegment`] when it has no such segment, or
/// [`AuthoringError::OutsideSegment`] for a parameter outside zero to one.
pub(crate) fn host_of(
    world: &mut World,
    anchor: &PortalAnchor,
    level: Option<Entity>,
) -> Result<Wall, AuthoringError> {
    let entity = anchor
        .host
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(anchor.host))?;
    let wall = world
        .get::<Wall>(entity)
        .cloned()
        .ok_or(AuthoringError::NotAWall(anchor.host))?;
    if level_of(world, entity) != level {
        return Err(AuthoringError::OnAnotherLevel(anchor.host));
    }
    if anchor.segment >= wall.segments.len() {
        return Err(AuthoringError::NoSegment {
            segment: anchor.segment,
            segments: wall.segments.len(),
        });
    }
    if !(0.0..=1.0).contains(&anchor.t) {
        return Err(AuthoringError::OutsideSegment(anchor.t));
    }
    Ok(wall)
}

/// Where a Portal of `width` set at `anchor` into `wall` stands: its centre, its rotation, which
/// is `rotation` where the segment has no direction, and its mirroring.
fn standing_in(
    wall: &Wall,
    anchor: &PortalAnchor,
    width: f32,
    rotation: f32,
) -> Option<(Vec2, f32, bool)> {
    let standing = anchor_portals(
        wall,
        &[PortalSetting {
            segment: anchor.segment,
            t: anchor.t,
            width,
        }],
    )
    .into_iter()
    .next()
    .flatten()?;
    Some((
        standing.centre,
        standing.direction.unwrap_or(rotation),
        anchor.side.mirrors(),
    ))
}

/// Places a Portal of the chosen Asset at its image's natural size on top of the Layer, set
/// into the Wall the anchor names or freestanding, unturned, and centred on `position`, as one
/// history step.
///
/// # Errors
///
/// What [`host_of`] reports for an anchor it refuses, what resolving the Asset reports, or
/// [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn place_portal(
    world: &mut World,
    layer: Entity,
    position: Vec2,
    asset: &AssetAddress,
    anchor: Option<PortalAnchor>,
) -> Result<(), AuthoringError> {
    let host = match &anchor {
        Some(anchor) => {
            let level = level_of(world, layer);
            Some(host_of(world, anchor, level)?)
        }
        None => None,
    };
    if !position.is_finite() {
        return Err(AuthoringError::MalformedPortal(
            "a Portal's position must be finite".to_owned(),
        ));
    }
    let resolved = resolve(world, layer, asset)?;
    let (position, rotation, mirrored) = match (&host, &anchor) {
        (Some(wall), Some(anchor)) => {
            standing_in(wall, anchor, resolved.size.x, 0.0).unwrap_or((position, 0.0, false))
        }
        _ => (position, 0.0, false),
    };
    crate::record_step(
        world,
        Place {
            layer,
            position,
            resolved,
            shown: Shown::Portal {
                rotation,
                mirrored,
                anchor,
            },
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
/// Portal, what [`host_of`] reports for an anchor it refuses, or [`AuthoringError::History`]
/// when the step could not be recorded.
pub(crate) fn set_portal_into_wall(
    world: &mut World,
    command: &SetPortalIntoWall,
) -> Result<(), AuthoringError> {
    portal_of(world, command.portal)?;
    let entity = command
        .portal
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(command.portal))?;
    let level = level_of(world, entity);
    host_of(world, &command.anchor, level)?;
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
/// as it is: its width, a freestanding Portal's rotation and mirroring, and a set Portal's side
/// and place along its Wall. `None` for a change that is not one of those.
///
/// # Errors
///
/// [`AuthoringError::NotAPortal`] when the Element is no Portal,
/// [`AuthoringError::MalformedPortal`] for a width not above zero or a rotation that is not
/// finite, [`AuthoringError::FollowsItsWall`] for the rotation or mirroring of a set Portal,
/// [`AuthoringError::Freestanding`] for the side or place of a freestanding one, what
/// [`host_of`] reports for a place its Wall does not have, or [`AuthoringError::History`] when
/// the field cannot be addressed.
pub(crate) fn portal_change(
    world: &mut World,
    id: ElementId,
    change: &ElementChange,
) -> Result<Option<SetField<ElementId>>, AuthoringError> {
    let history = |error: drs_history::HistoryError| AuthoringError::History(error.to_string());
    let well_formed = |portal: &Portal| {
        portal.malformation().map_or(Ok(()), |reason| {
            Err(AuthoringError::MalformedPortal(reason))
        })
    };
    let field = match change {
        ElementChange::Width(width) => {
            let mut portal = portal_of(world, id)?;
            portal.width = *width;
            well_formed(&portal)?;
            SetField::<ElementId>::new::<Portal>(id, "width", *width)
        }
        ElementChange::Rotation(rotation) => {
            let mut portal = portal_of(world, id)?;
            if portal.anchor.is_some() {
                return Err(AuthoringError::FollowsItsWall);
            }
            portal.rotation = *rotation;
            well_formed(&portal)?;
            SetField::<ElementId>::new::<Portal>(id, "rotation", *rotation)
        }
        ElementChange::Mirrored(mirrored) => {
            if portal_of(world, id)?.anchor.is_some() {
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
            let mut anchor = portal_of(world, id)?
                .anchor
                .ok_or(AuthoringError::Freestanding)?;
            anchor.segment = *segment;
            anchor.t = *t;
            let entity = id
                .entity(world)
                .map_err(|_| AuthoringError::UnknownElement(id))?;
            let level = level_of(world, entity);
            host_of(world, &anchor, level)?;
            SetField::<ElementId>::new::<Portal>(id, "anchor", Some(anchor))
        }
        ElementChange::Position(_)
        | ElementChange::Point { .. }
        | ElementChange::Control { .. }
        | ElementChange::AddPoint { .. }
        | ElementChange::RemovePoint { .. }
        | ElementChange::Thickness(_)
        | ElementChange::Colour(_) => return Ok(None),
    };
    field.map(Some).map_err(history)
}

/// The Portals set into the Wall `host`, in the order of their identities, with their anchors
/// and widths: those whose anchor names the Wall, on its Level, at a segment it has.
pub(crate) fn set_into(world: &mut World, host: ElementId) -> Vec<(ElementId, PortalAnchor, f32)> {
    let Ok(wall_entity) = host.entity(world) else {
        return Vec::new();
    };
    let segments = world
        .get::<Wall>(wall_entity)
        .map_or(0, |wall| wall.segments.len());
    let level = level_of(world, wall_entity);
    let mut portals: Vec<(Entity, ElementId, PortalAnchor, f32)> = world
        .query::<(Entity, &ElementId, &Portal)>()
        .iter(world)
        .filter_map(|(entity, id, portal)| {
            let anchor = portal.anchor?;
            (anchor.host == host && anchor.segment < segments && (0.0..=1.0).contains(&anchor.t))
                .then_some((entity, *id, anchor, portal.width))
        })
        .collect();
    portals.retain(|(entity, ..)| level_of(world, *entity) == level);
    portals.sort_by_key(|(_, id, ..)| *id);
    portals
        .into_iter()
        .map(|(_, id, anchor, width)| (id, anchor, width))
        .collect()
}
