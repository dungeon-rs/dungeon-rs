//! The Walls of a Layer as they run when a Command is handled, worked out through the shape
//! Engine for the Commands that must know them before deriving has caught up with the frame:
//! placing, setting, and sliding a Portal, and every Command that may take a Wall away from a
//! Portal set into another Room.
//!
//! Written once over [`OutlineHost`]: a kind that combines is worked out with every other of its
//! kind on its Layer, in stacking order, and one that does not on its own.

use crate::AuthoringError;
use crate::outline::OutlineHost;
use crate::portal::{level_of, sets_into};
use crate::remove::Remove;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;
use drs_history::Target;
use drs_model::{ElementId, Layer, Portal, PortalAnchor, PortalsRemoved};
use drs_shape_engine::{
    Combination, Outline, PortalSetting, Standing, anchor_portals, combine_outlines,
};
use std::collections::BTreeMap;

/// The Portals with a Wall at their centre, each with the host it is set into.
pub(crate) type Walled = BTreeMap<ElementId, ElementId>;

/// Hosts of one kind worked out together, in stacking order, each with its entity and identity.
type Batch<H> = Vec<(Entity, ElementId, H)>;

/// The Layer an Element lies on: its parent, when that is a Layer.
pub(crate) fn layer_of(world: &World, entity: Entity) -> Option<Entity> {
    let parent = world.get::<ChildOf>(entity)?.parent();
    world.get::<Layer>(parent).is_some().then_some(parent)
}

/// The hosts of kind `H` on `layer`, in stacking order.
fn on_layer<H: OutlineHost>(world: &World, layer: Entity) -> Batch<H> {
    world
        .get::<Children>(layer)
        .map(|children| {
            children
                .iter()
                .filter_map(|child| {
                    Some((
                        *child,
                        *world.get::<ElementId>(*child)?,
                        world.get::<H>(*child)?.clone(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The hosts the host of kind `H` on `entity` is worked out with, itself among them: every one of
/// its kind on its Layer when the kind combines, and it alone otherwise; with its number among
/// them.
fn batch_of<H: OutlineHost>(world: &World, entity: Entity) -> Option<(Batch<H>, usize)> {
    let own = world.get::<H>(entity)?.clone();
    if H::COMBINES
        && let Some(layer) = layer_of(world, entity)
    {
        let members = on_layer::<H>(world, layer);
        let index = members.iter().position(|(member, ..)| *member == entity)?;
        return Some((members, index));
    }
    let id = *world.get::<ElementId>(entity)?;
    Some((vec![(entity, id, own)], 0))
}

/// `CombineOutlines` over `members`, in the order given.
fn combined<H: OutlineHost>(members: &[(Entity, ElementId, H)]) -> Combination {
    let outlines: Vec<Outline> = members
        .iter()
        .map(|(_, _, host)| Outline {
            path: host.path(),
            cuts: host.cuts(),
        })
        .collect();
    combine_outlines(&outlines)
}

/// Where a Portal `width` wide would stand set at `anchor` into the host of kind `H` on
/// `entity`, as the Walls run now: `None` when the entity is no such host or it lacks the part or
/// the parameter; a standing with no stretches has no Wall at its centre.
pub(crate) fn standing_at<H: OutlineHost>(
    world: &World,
    entity: Entity,
    anchor: &PortalAnchor,
    width: f32,
) -> Option<Standing> {
    let (members, index) = batch_of::<H>(world, entity)?;
    let combination = combined(&members);
    let setting = PortalSetting {
        outline: index,
        segment: anchor.index,
        t: anchor.t,
        width,
    };
    anchor_portals(&combination, &[setting])
        .into_iter()
        .next()
        .flatten()
}

/// The Portals set into the hosts of kind `H` on `layer` that have a Wall at their centre, as
/// the Walls run now.
pub(crate) fn walled<H: OutlineHost>(world: &mut World, layer: Entity) -> Walled {
    let members = on_layer::<H>(world, layer);
    if members.is_empty() {
        return Walled::new();
    }
    let level = level_of(world, layer);
    let hosts: BTreeMap<ElementId, (usize, usize)> = members
        .iter()
        .enumerate()
        .map(|(index, (_, id, host))| (*id, (index, host.path().parts())))
        .collect();
    let mut set: Vec<(ElementId, PortalAnchor, f32, usize)> = world
        .query::<(Entity, &ElementId, &Portal)>()
        .iter(world)
        .filter_map(|(entity, id, portal)| {
            let anchor = portal.anchor?;
            let &(index, parts) = hosts.get(&anchor.host)?;
            sets_into(&anchor, parts, (level_of(world, entity), level)).then_some((
                *id,
                anchor,
                portal.width,
                index,
            ))
        })
        .collect();
    if set.is_empty() {
        return Walled::new();
    }
    set.sort_by_key(|(id, ..)| *id);
    let combination = combined(&members);
    let settings: Vec<PortalSetting> = set
        .iter()
        .map(|(_, anchor, width, index)| PortalSetting {
            outline: *index,
            segment: anchor.index,
            t: anchor.t,
            width: *width,
        })
        .collect();
    set.iter()
        .zip(anchor_portals(&combination, &settings))
        .filter(|(_, standing)| {
            standing
                .as_ref()
                .is_some_and(|standing| standing.stretches.is_some())
        })
        .map(|((id, anchor, ..), _)| (*id, anchor.host))
        .collect()
}

/// Records, in the group that is open, the removal of every Portal of `before` that no longer
/// has a Wall at its centre on `layer` and still exists, and returns the answers naming them, one
/// for each host they were set into.
///
/// # Errors
///
/// [`AuthoringError::History`] when a removal could not be recorded.
pub(crate) fn take_walls_away<H: OutlineHost>(
    world: &mut World,
    layer: Entity,
    before: &Walled,
) -> Result<Vec<PortalsRemoved>, AuthoringError> {
    if before.is_empty() {
        return Ok(Vec::new());
    }
    let after = walled::<H>(world, layer);
    let mut gone: BTreeMap<ElementId, Vec<ElementId>> = BTreeMap::new();
    for (portal, host) in before {
        if after.contains_key(portal) || portal_gone(world, *portal) {
            continue;
        }
        crate::record(world, Remove::of(*portal))?;
        gone.entry(*host).or_default().push(*portal);
    }
    Ok(gone
        .into_iter()
        .map(|(host, portals)| PortalsRemoved { host, portals })
        .collect())
}

/// Whether no Element carries the identity any more.
fn portal_gone(world: &mut World, portal: ElementId) -> bool {
    portal.entity(world).is_err()
}
