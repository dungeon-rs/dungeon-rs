//! The shapes derived from every Wall and Room and from the Portals set into them: the line, the
//! stroke, and a Room's floor through the shape Engine, the Rooms of a Layer combined together,
//! the Element's box around its points, and where each Portal stands, whether it follows its
//! host, and how large it is.

use crate::ancestor;
use crate::combined::{Batch, combination_of};
use crate::outline::OutlineHost;
use crate::portal::{Anchored, sets_into, setting_at, stood};
use bevy_ecs::change_detection::{DetectChanges, DetectChangesMut, Mut};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::lifecycle::RemovedComponents;
use bevy_ecs::query::{Changed, Or, With, Without};
use bevy_ecs::system::{Commands, Query, SystemParam};
use bevy_math::Vec2;
use drs_model::{
    Anchoring, AssetReferences, Element, ElementId, Layer, Level, Portal, PortalAnchor, Room,
    Stretch, Wall,
};
use drs_shape_engine::{
    Combination, Path, PortalSetting, Standing, anchor_portals, generate_walls,
};
use std::collections::BTreeMap;
use std::marker::PhantomData;

/// What the shapes of a batch of Walls or Rooms were last derived from: a Wall alone, or the
/// Rooms of a Layer together, each one's identity, outline, thickness, and whether it cuts, in
/// stacking order, and where the Portals set into them are and how wide. A change that leaves
/// them as they were, a new colour, keeps the shapes. It is kept on the Wall, or on the Layer of
/// the Rooms.
#[derive(Component, Debug, Clone, PartialEq)]
pub(crate) struct DerivedFrom<H: OutlineHost> {
    /// The outlines the shapes were derived from, in stacking order.
    outlines: Vec<(ElementId, Path, f32, bool)>,
    /// The Portals set into them, each with the outline's number in the batch.
    portals: Vec<(ElementId, PortalSetting)>,
    /// The kind of the outlines.
    kind: PhantomData<fn() -> H>,
}

impl<H: OutlineHost> DerivedFrom<H> {
    /// What `members`' shapes are derived from, with the Portals set into them.
    fn of(members: &[(Entity, ElementId, H)], set: &BTreeMap<ElementId, Vec<Anchored>>) -> Self {
        let mut portals = Vec::new();
        for (index, (_, id, _)) in members.iter().enumerate() {
            for (portal, anchor, width) in set.get(id).map_or(&[][..], Vec::as_slice) {
                portals.push((*portal, setting_at(index, anchor, *width)));
            }
        }
        Self {
            outlines: members
                .iter()
                .map(|(_, id, outline)| (*id, outline.path(), outline.thickness(), outline.cuts()))
                .collect(),
            portals,
            kind: PhantomData,
        }
    }
}

/// The Walls or the Rooms as deriving reads and writes them.
type Outlines<'w, 's, H> = Query<
    'w,
    's,
    (
        Entity,
        &'static ElementId,
        &'static H,
        Option<&'static mut <H as OutlineHost>::Shape>,
        Option<&'static ChildOf>,
    ),
>;

/// The boxes of the Elements that are not Portals, which deriving sets around the points of
/// Walls and Rooms.
type Boxes<'w, 's> = Query<'w, 's, &'static mut Element, Without<Portal>>;

/// The Portals as deriving reads and writes them.
type Portals<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ElementId,
        &'static mut Portal,
        &'static mut Element,
        Option<&'static mut Anchoring>,
    ),
>;

/// What tells deriving that the Walls or the Rooms changed: one placed, edited, or removed, and
/// a Layer's stacking order.
#[expect(
    clippy::type_complexity,
    reason = "a Bevy query filter is spelled out by the components it watches"
)]
#[derive(SystemParam)]
pub(crate) struct OutlineChanges<'w, 's> {
    /// The Walls and Rooms placed or edited.
    changed: Query<'w, 's, (), Or<(Changed<Wall>, Changed<Room>)>>,
    /// The Walls removed.
    removed_walls: RemovedComponents<'w, 's, Wall>,
    /// The Rooms removed.
    removed_rooms: RemovedComponents<'w, 's, Room>,
    /// The Layers whose order changed.
    orders: Query<'w, 's, (), (Changed<Children>, With<Layer>)>,
}

impl OutlineChanges<'_, '_> {
    /// Whether any Wall or Room changed, came, or went, or any Layer's order changed.
    fn any(&mut self) -> bool {
        let removed = self.removed_walls.read().count() + self.removed_rooms.read().count();
        !self.changed.is_empty() || !self.orders.is_empty() || removed > 0
    }
}

/// Derives the shapes of the Walls and the Rooms whose outlines, thicknesses, or cuts changed
/// since the last frame, or whose Portals were placed, edited, set, freed, or removed, a Room
/// together with every Room of its Layer: the shape Engine combines the Layer's Rooms in
/// stacking order into their floors and Walls, or a Wall alone into its line, says where each
/// Portal set into them stands and which Walls it leaves out, and strokes each one's Walls; each
/// Element's box is set around its points. Moves each Portal set into such a Wall or Room, where a
/// Wall runs at its centre, to where its anchor puts it, turned to the line's direction there
/// and mirrored when it faces the right; says of every Portal whether it follows its host; and
/// sets every changed Portal's size from its width and its image's recorded pixel size.
///
/// A Portal whose anchor names no Wall or Room of its Level, or a part its host lacks, or a place
/// on a Room's edge where no Wall runs, is lost: it is left standing as it is and leaves no gap.
#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system is spelled out by the components it reads and writes"
)]
pub(crate) fn derive_shapes(
    mut commands: Commands,
    mut edits: OutlineChanges,
    mut removed_portals: RemovedComponents<Portal>,
    mut walls: Outlines<Wall>,
    mut rooms: Outlines<Room>,
    derived: (Query<&DerivedFrom<Wall>>, Query<&DerivedFrom<Room>>),
    layers: Query<&Children, With<Layer>>,
    mut boxes: Boxes,
    mut portals: Portals,
    parents: Query<&ChildOf>,
    levels: Query<(), With<Level>>,
    references: Query<&AssetReferences>,
) {
    let removed = removed_portals.read().count() > 0;
    // Looking at whether a Portal changed through the query that writes them marks nothing.
    let portal_changed = portals
        .iter_mut()
        .any(|(_, _, portal, ..)| portal.is_changed());
    if !edits.any() && !portal_changed && !removed {
        return;
    }
    let parent_of = |child: Entity| parents.get(child).ok().map(ChildOf::parent);
    let level_of = |entity: Entity| ancestor(entity, parent_of, |parent| levels.contains(parent));
    let mut hosts = parts_of(&walls);
    hosts.append(&mut parts_of(&rooms));
    let set = set_by_host(&hosts, &portals, level_of);
    let mut standings = reshape(
        &mut commands,
        &mut walls,
        &derived.0,
        &layers,
        &mut boxes,
        &set,
    );
    standings.append(&mut reshape(
        &mut commands,
        &mut rooms,
        &derived.1,
        &layers,
        &mut boxes,
        &set,
    ));
    let anchors: BTreeMap<ElementId, PortalAnchor> = set
        .values()
        .flatten()
        .map(|(id, anchor, _)| (*id, *anchor))
        .collect();

    for (entity, id, mut portal, mut element, anchoring) in &mut portals {
        let changed = portal.is_changed();
        let had = anchoring.as_deref().copied();
        let derived = match (anchors.get(id), portal.anchor) {
            (Some(anchor), _) => match standings.get(id) {
                Some(Some(standing)) if standing.stretches.is_some() => {
                    follow(&mut portal, &mut element, anchor, Some(standing));
                    Anchoring::Set
                }
                // No Wall runs at its centre: it stands where it stood.
                Some(_) => Anchoring::Lost,
                // Its host was not derived again: it stands as it last did.
                None => match had {
                    Some(Anchoring::Lost) => Anchoring::Lost,
                    Some(Anchoring::Set | Anchoring::Freestanding) | None => {
                        follow(&mut portal, &mut element, anchor, None);
                        Anchoring::Set
                    }
                },
            },
            (None, Some(_)) => Anchoring::Lost,
            (None, None) => Anchoring::Freestanding,
        };
        match anchoring {
            Some(mut anchoring) => {
                anchoring.set_if_neq(derived);
            }
            None => {
                commands.entity(entity).insert(derived);
            }
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

/// Every Wall or every Room by its identity, with its entity and how many segments or edges it
/// has.
fn parts_of<H: OutlineHost>(outlines: &Outlines<H>) -> BTreeMap<ElementId, (Entity, usize)> {
    outlines
        .iter()
        .map(|(entity, id, outline, ..)| (*id, (entity, outline.path().parts())))
        .collect()
}

/// The Walls or the Rooms in the batches they are derived in, each batch with the entity its
/// [`DerivedFrom`] is kept on and its members in stacking order: the Rooms of a Layer together,
/// as a kind that combines is, and a Wall, or anything on no Layer, alone.
fn batches<H: OutlineHost>(
    outlines: &Outlines<H>,
    layers: &Query<&Children, With<Layer>>,
) -> BTreeMap<Entity, Batch<H>> {
    let mut batches: BTreeMap<Entity, Batch<H>> = BTreeMap::new();
    for (entity, id, outline, _, parent) in outlines {
        let key = parent
            .map(ChildOf::parent)
            .filter(|parent| H::COMBINES && layers.contains(*parent))
            .unwrap_or(entity);
        batches
            .entry(key)
            .or_default()
            .push((entity, *id, outline.clone()));
    }
    for (key, members) in &mut batches {
        if let Ok(children) = layers.get(*key) {
            let place = |entity: Entity| children.iter().position(|child| *child == entity);
            members.sort_by_key(|(entity, ..)| place(*entity));
        }
    }
    batches
}

/// Derives again the shapes of every batch of Walls or Rooms whose outlines, thicknesses, cuts,
/// or set Portals differ from what its shapes were last derived from: `CombineOutlines` combines
/// the batch, `AnchorPortals` places the Portals set into it and says which Walls each leaves
/// out, and `GenerateWalls` strokes each one's Walls, leaving those out. A shape is written only
/// where it differs. Sets each Element's box around its points. Returns where each Portal set
/// into those outlines stands, `None` for one on a part or parameter its outline lacks.
fn reshape<H: OutlineHost>(
    commands: &mut Commands,
    outlines: &mut Outlines<H>,
    derived: &Query<&DerivedFrom<H>>,
    layers: &Query<&Children, With<Layer>>,
    boxes: &mut Boxes,
    set: &BTreeMap<ElementId, Vec<Anchored>>,
) -> BTreeMap<ElementId, Option<Standing>> {
    let mut standings = BTreeMap::new();
    for (key, members) in batches(outlines, layers) {
        let geometry = DerivedFrom::of(&members, set);
        let shaped = members.iter().all(|(entity, ..)| {
            outlines
                .get(*entity)
                .is_ok_and(|(_, _, _, shape, _)| shape.is_some())
        });
        if shaped && derived.get(key).is_ok_and(|last| *last == geometry) {
            continue;
        }
        let combination = combination_of(&members);
        let stretches = stand_portals(&combination, &geometry, members.len(), &mut standings);
        write_shapes(
            commands,
            outlines,
            boxes,
            &members,
            &combination,
            &stretches,
        );
        commands.entity(key).insert(geometry);
    }
    standings
}

/// Where the Portals of `geometry` stand in `combination`, recorded in `standings`, and the
/// stretches of each of the `outlines` outlines' Walls they cover.
fn stand_portals<H: OutlineHost>(
    combination: &Combination,
    geometry: &DerivedFrom<H>,
    outlines: usize,
    standings: &mut BTreeMap<ElementId, Option<Standing>>,
) -> Vec<Vec<Stretch>> {
    let settings: Vec<PortalSetting> = geometry
        .portals
        .iter()
        .map(|(_, setting)| *setting)
        .collect();
    let mut stretches: Vec<Vec<Stretch>> = vec![Vec::new(); outlines];
    for ((portal, _), standing) in geometry
        .portals
        .iter()
        .zip(anchor_portals(combination, &settings))
    {
        if let Some(covered) = standing
            .as_ref()
            .and_then(|standing| standing.stretches.as_ref())
        {
            for (outline, stretch) in covered {
                if let Some(own) = stretches.get_mut(*outline) {
                    own.push(*stretch);
                }
            }
        }
        standings.insert(*portal, standing);
    }
    stretches
}

/// Strokes each member's Walls from its combined outline, leaving out its `stretches`, writes
/// the shape where it differs, and sets the member's box around its points.
fn write_shapes<H: OutlineHost>(
    commands: &mut Commands,
    outlines: &mut Outlines<H>,
    boxes: &mut Boxes,
    members: &Batch<H>,
    combination: &Combination,
    stretches: &[Vec<Stretch>],
) {
    for (((entity, id, outline), combined), own) in
        members.iter().zip(&combination.outlines).zip(stretches)
    {
        let mesh = generate_walls(combined, outline.thickness(), own);
        let drawn_at = members.get(combined.last).map_or(*id, |last| last.1);
        let shape = H::shape(combined, mesh, own.clone(), drawn_at);
        if let Ok((_, _, _, current, _)) = outlines.get_mut(*entity) {
            match current {
                Some(mut current) => {
                    current.set_if_neq(shape);
                }
                None => {
                    commands.entity(*entity).insert(shape);
                }
            }
        }
        let footprint = outline.element_box();
        if let Ok(mut element) = boxes.get_mut(*entity) {
            if element.position != footprint.center() {
                element.position = footprint.center();
            }
            if element.size != footprint.size() {
                element.size = footprint.size();
            }
        }
    }
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

/// The Portals set into each Wall and each Room of `hosts`, by the host's identity and in the
/// order of their own, with their anchors and widths: those whose anchor names a Wall or a Room
/// on their Level and a segment or edge it has.
fn set_by_host(
    hosts: &BTreeMap<ElementId, (Entity, usize)>,
    portals: &Portals,
    level_of: impl Fn(Entity) -> Option<Entity>,
) -> BTreeMap<ElementId, Vec<Anchored>> {
    let mut set: BTreeMap<ElementId, Vec<Anchored>> = BTreeMap::new();
    for (entity, id, portal, ..) in portals {
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
