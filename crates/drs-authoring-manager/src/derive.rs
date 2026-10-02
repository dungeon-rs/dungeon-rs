//! The shapes derived from every Wall and Room and from the Portals set into them: the line, the
//! stroke, and a Room's floor through the shape Engine, the Element's box around its points, and
//! where each Portal stands and how large it is.

use crate::ancestor;
use crate::outline::{OutlineHost, settings};
use crate::portal::{Anchored, sets_into, stood};
use bevy_ecs::change_detection::{DetectChanges, Mut};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::lifecycle::RemovedComponents;
use bevy_ecs::query::{Changed, Or, With, Without};
use bevy_ecs::system::{Commands, Query};
use bevy_math::Vec2;
use drs_model::{AssetReferences, Element, ElementId, Level, Portal, PortalAnchor, Room, Wall};
use drs_shape_engine::{
    Path, PortalSetting, Standing, anchor_portals, combine_outlines, generate_walls,
};
use std::collections::BTreeMap;

/// What a Wall's or a Room's shape was last derived from: its outline, its thickness, and where
/// the Portals set into it are and how wide. A change that leaves them as they were, a new
/// colour, keeps the shape.
#[derive(Component, Debug, Clone, PartialEq)]
pub(crate) struct DerivedFrom {
    /// The outline the shape was derived from.
    path: Path,
    /// The thickness it was derived at.
    thickness: f32,
    /// The Portals set into it, in the order of their identities.
    portals: Vec<PortalSetting>,
}

impl DerivedFrom {
    /// What `outline`'s shape is derived from, with the Portals set into it.
    fn of<H: OutlineHost>(outline: &H, portals: &[Anchored]) -> Self {
        Self {
            path: outline.path(),
            thickness: outline.thickness(),
            portals: settings(portals),
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
        Option<&'static DerivedFrom>,
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
    ),
>;

/// Derives the shape of every Wall and every Room whose outline or thickness changed since the
/// last frame, or whose Portals were placed, edited, set, freed, or removed: the shape Engine
/// combines its outline into its line and floor and strokes the line, and its Element's box is
/// set around its points. Moves each Portal set into such a Wall or Room to where its anchor puts
/// it, turned to the line's direction there and mirrored when it faces the right; and sets every
/// changed Portal's size from its width and its image's recorded pixel size.
///
/// A Portal whose anchor names no Wall or Room of its Level, or a part its host lacks, is lost:
/// it is left standing as it is and leaves no gap.
#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "a Bevy system is spelled out by the components it reads and writes"
)]
pub(crate) fn derive_shapes(
    mut commands: Commands,
    changed_outlines: Query<(), Or<(Changed<Wall>, Changed<Room>)>>,
    mut removed_portals: RemovedComponents<Portal>,
    mut walls: Outlines<Wall>,
    mut rooms: Outlines<Room>,
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
    if changed_outlines.is_empty() && !portal_changed && !removed {
        return;
    }
    let parent_of = |child: Entity| parents.get(child).ok().map(ChildOf::parent);
    let level_of = |entity: Entity| ancestor(entity, parent_of, |parent| levels.contains(parent));
    let mut hosts = parts_of(&walls);
    hosts.append(&mut parts_of(&rooms));
    let set = set_by_host(&hosts, &portals, level_of);
    let mut standings = reshape(&mut commands, &mut walls, &mut boxes, &set);
    standings.append(&mut reshape(&mut commands, &mut rooms, &mut boxes, &set));
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

/// Every Wall or every Room by its identity, with its entity and how many segments or edges it
/// has.
fn parts_of<H: OutlineHost>(outlines: &Outlines<H>) -> BTreeMap<ElementId, (Entity, usize)> {
    outlines
        .iter()
        .map(|(entity, id, outline, ..)| (*id, (entity, outline.path().parts())))
        .collect()
}

/// Derives again the shape of every Wall or every Room whose outline, thickness, or set Portals
/// differ from what its shape was last derived from: `CombineOutlines` gives its line and
/// floor, and `GenerateWalls` strokes the line, leaving out the stretches those Portals cover.
/// Sets its Element's box around its points. Returns where each Portal set into those outlines
/// stands.
fn reshape<H: OutlineHost>(
    commands: &mut Commands,
    outlines: &mut Outlines<H>,
    boxes: &mut Boxes,
    set: &BTreeMap<ElementId, Vec<Anchored>>,
) -> BTreeMap<ElementId, Standing> {
    let mut standings = BTreeMap::new();
    for (entity, id, outline, shape, derived_from) in outlines {
        let into = set.get(id).map_or(&[][..], Vec::as_slice);
        let geometry = DerivedFrom::of(outline, into);
        if shape.is_some() && derived_from == Some(&geometry) {
            continue;
        }
        let placed = anchor_portals(&geometry.path, &geometry.portals);
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
        let combined = combine_outlines(&geometry.path);
        let walls = generate_walls(&combined, outline.thickness(), &stretches);
        let derived = H::shape(walls, combined.floor);
        match shape {
            Some(mut shape) => *shape = derived,
            None => {
                commands.entity(entity).insert(derived);
            }
        }
        commands.entity(entity).insert(geometry);
        let footprint = outline.element_box();
        if let Ok(mut element) = boxes.get_mut(entity) {
            if element.position != footprint.center() {
                element.position = footprint.center();
            }
            if element.size != footprint.size() {
                element.size = footprint.size();
            }
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
