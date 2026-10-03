//! What a Wall and a Room share as Elements drawn from an outline: placing one, the edits of its
//! points, control points, thickness, and colours, the edits that add and remove a point and
//! carry the Portals set into it, and removing it with them.
//!
//! Every operation is written once over [`OutlineHost`], which a kind implements by saying how its
//! component reads as the shape Engine's [`Path`] and back, so a kind gains them all through one
//! implementation.

use crate::combined::{Walled, layer_of, take_walls_away, walled};
use crate::place::{spawn_on_top, take_off};
use crate::portal::{Anchored, anchored_to};
use crate::remove::Remove;
use crate::{AuthoringError, OutlineKind};
use bevy_ecs::component::{Component, Mutable};
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::world::World;
use bevy_math::{Rect, Vec2};
use bevy_reflect::{Reflect, TypePath};
use drs_history::{ReversibleCommand, SetField, Target};
use drs_model::{
    Element, ElementChange, ElementId, ElementKindName, LinePlace, Portal, PortalAnchor,
    PortalsRemoved, Stretch, StrokeMesh,
};
use drs_shape_engine::{
    CombinedOutline, Path, PointEdit, PortalSetting, anchor_portals_through, split_wall,
};

/// A kind of Element drawn from an editable outline that Portals are set into: a Wall along its
/// open line, a Room round its closed outline. Its points and control points are read and written
/// as the shape Engine's [`Path`], and its derived shape is the Engine's stroke, with the floor a
/// closed outline winds around.
pub(crate) trait OutlineHost:
    Component<Mutability = Mutable> + Reflect + TypePath + Clone + PartialEq
{
    /// The kind's name, which its Element carries.
    const KIND: ElementKindName;
    /// What the kind is called when a point or a part it lacks is named.
    const OUTLINE: OutlineKind;
    /// The fewest points it has: removing a point from one of this many removes it.
    const FEWEST_POINTS: usize;
    /// The reflect path of the colour its Walls are drawn in.
    const COLOUR: &'static str;
    /// The reflect path of the colour its floor is drawn in, when it has one.
    const FLOOR_COLOUR: Option<&'static str>;
    /// The reflect path of whether it cuts, when it can.
    const CUTS: Option<&'static str>;
    /// Whether it combines with the others of its kind on its Layer, so an edit of one can take a
    /// Wall away from a Portal set into another.
    const COMBINES: bool;

    /// The shape derived from it, which it is drawn and picked by.
    type Shape: Component<Mutability = Mutable> + PartialEq;

    /// The reflect path of the control point of part `part`, its segment or edge.
    fn control_path(part: usize) -> String;

    /// Its points and control points as an outline.
    fn path(&self) -> Path;

    /// It with the points and control points of `path`, everything else kept.
    #[must_use]
    fn with_path(&self, path: Path) -> Self;

    /// How thick its Walls are drawn, in Grid cells.
    fn thickness(&self) -> f32;

    /// It with its Walls drawn `thickness` thick.
    #[must_use]
    fn with_thickness(&self, thickness: f32) -> Self;

    /// The box its Element has.
    fn element_box(&self) -> Rect;

    /// Why it is not one, if it is not.
    fn malformation(&self) -> Option<String>;

    /// Whether it takes floor away from those before it rather than adding its own.
    fn cuts(&self) -> bool;

    /// Its derived shape from what `CombineOutlines` gave it, the stroke of its Walls, the
    /// stretches its Walls give way along, and the one at whose place its Walls are drawn.
    fn shape(
        combined: &CombinedOutline,
        mesh: StrokeMesh,
        stretches: Vec<Stretch>,
        drawn_at: ElementId,
    ) -> Self::Shape;
}

/// The recorded step of placing a Wall or a Room: the Element spawned on top of its Layer,
/// keeping its identity so that redo puts it back exactly.
struct PlaceOutline<H> {
    /// The Layer to place on.
    layer: Entity,
    /// The Wall or the Room as placed.
    outline: H,
    /// The identity the Element keeps through undo and redo.
    element: ElementId,
}

impl<H: OutlineHost> ReversibleCommand for PlaceOutline<H> {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        let footprint = self.outline.element_box();
        spawn_on_top(
            world,
            self.layer,
            (
                Element {
                    kind: H::KIND,
                    position: footprint.center(),
                    size: footprint.size(),
                },
                self.outline.clone(),
                self.element,
            ),
        )
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        take_off(world, self.element)
    }
}

/// Places a Wall or a Room on top of the Layer as one history step, with the removal of every
/// Portal set into another of its kind on the Layer that it leaves with no Wall at its centre.
/// Returns the answers naming the Portals that went.
///
/// # Errors
///
/// [`AuthoringError::MalformedWall`] or [`AuthoringError::MalformedRoom`] for one that would not
/// be one, or [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn place_outline<H: OutlineHost>(
    world: &mut World,
    layer: Entity,
    outline: H,
) -> Result<Vec<PortalsRemoved>, AuthoringError> {
    well_formed(&outline)?;
    let place = PlaceOutline {
        layer,
        outline,
        element: ElementId::new(),
    };
    if !H::COMBINES {
        return crate::record_step(world, place).map(|()| Vec::new());
    }
    let before = walled::<H>(world, layer);
    crate::history(world)?.begin_group();
    let mut removed = Vec::new();
    let outcome = crate::record(world, place)
        .and_then(|()| take_walls_away::<H>(world, layer, &before).map(|taken| removed = taken));
    crate::close_group(world, outcome)?;
    Ok(removed)
}

/// The recorded step of adding or removing a point: the Wall or the Room as it becomes and as it
/// was, the control points of the parts it joins or splits included, so that undo restores it
/// exactly.
///
/// These are the only steps that renumber a Wall's segments or a Room's edges.
pub(crate) struct Reshape<H> {
    /// The identity of the Wall or the Room.
    pub(crate) element: ElementId,
    /// The Wall or the Room after the step.
    pub(crate) outline: H,
    /// The Wall or the Room before the step, once applied.
    pub(crate) previous: Option<H>,
}

impl<H: OutlineHost> ReversibleCommand for Reshape<H> {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        let entity = self.element.entity(world)?;
        let mut outline = world
            .get_mut::<H>(entity)
            .ok_or(AuthoringError::NotAnOutline(self.element))?;
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
            .get_mut::<H>(entity)
            .ok_or(AuthoringError::NotAnOutline(self.element))? = previous.clone();
        Ok(())
    }
}

/// Refuses a Wall or a Room that would not be one, for the reason its own check gives.
///
/// # Errors
///
/// [`AuthoringError::MalformedWall`] or [`AuthoringError::MalformedRoom`] with that reason.
fn well_formed<H: OutlineHost>(outline: &H) -> Result<(), AuthoringError> {
    outline
        .malformation()
        .map_or(Ok(()), |reason| Err(H::OUTLINE.malformed(reason)))
}

/// The Wall or the Room an Element is.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] when no Element carries the identity, or
/// [`AuthoringError::NotAnOutline`] when it is not one of the kind.
fn outline_of<H: OutlineHost>(world: &mut World, element: ElementId) -> Result<H, AuthoringError> {
    let entity = element
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(element))?;
    world
        .get::<H>(entity)
        .cloned()
        .ok_or(AuthoringError::NotAnOutline(element))
}

/// What an Edit Element of a Wall or a Room comes to: a field command for the gesture handling
/// to record, or a step of its own already recorded, with the answer naming the Portals it
/// removed.
pub(crate) enum OutlineEdit {
    /// The field command to record.
    Field(SetField<ElementId>),
    /// The step was recorded on its own.
    Recorded(Vec<PortalsRemoved>),
}

/// An Edit Element of the Wall or the Room `id`, checked against it as it is: its position, a
/// point, a part's control point, its thickness, or a colour as a field command, or a point
/// added or removed as a step of its own that carries the Portals set into it. Moving it, to a
/// position or by an amount, moves every point and control point by the same amount; a move by
/// an amount counts from `from`, the outline as the gesture it belongs to began, when there is
/// one, so a drag's every step is the travel since its press.
///
/// # Errors
///
/// [`AuthoringError::NoPoint`] or [`AuthoringError::NoPart`] for a point or part it does not
/// have, [`AuthoringError::Shape`] for a point added where the part cannot be split,
/// [`AuthoringError::MalformedWall`] or [`AuthoringError::MalformedRoom`] for a thickness not
/// above zero or a point that is not finite, [`AuthoringError::NotARoom`] for a floor colour of
/// a kind without a floor, [`AuthoringError::NotAPortal`] for a change only a Portal has,
/// [`AuthoringError::NotATerrain`] for a change only a Terrain has, or
/// [`AuthoringError::History`] when the change cannot be addressed or recorded.
pub(crate) fn outline_edit<H: OutlineHost>(
    world: &mut World,
    id: ElementId,
    change: &ElementChange,
    from: Option<&H>,
) -> Result<OutlineEdit, AuthoringError> {
    let history = |error: drs_history::HistoryError| AuthoringError::History(error.to_string());
    let outline = outline_of::<H>(world, id)?;
    let mut path = outline.path();
    let field = match change {
        ElementChange::Position(position) => {
            let offset = *position - outline.element_box().center();
            Ok(translated(id, &outline, path, offset)?)
        }
        ElementChange::MoveBy(amount) => {
            let from = from.map_or(path, OutlineHost::path);
            Ok(translated(id, &outline, from, *amount)?)
        }
        ElementChange::Point { index, position } => {
            let points = path.points.len();
            *path.points.get_mut(*index).ok_or(AuthoringError::NoPoint {
                outline: H::OUTLINE,
                index: *index,
                points,
            })? = *position;
            well_formed(&outline.with_path(path))?;
            SetField::<ElementId>::new::<H>(id, &format!("points[{index}]"), *position)
        }
        ElementChange::Control { segment, position } => {
            let parts = path.parts();
            *path
                .controls
                .get_mut(*segment)
                .ok_or(AuthoringError::NoPart {
                    outline: H::OUTLINE,
                    part: *segment,
                    parts,
                })? = *position;
            well_formed(&outline.with_path(path))?;
            SetField::<ElementId>::new::<H>(id, &H::control_path(*segment), *position)
        }
        ElementChange::Thickness(thickness) => {
            well_formed(&outline.with_thickness(*thickness))?;
            SetField::<ElementId>::new::<H>(id, "thickness", *thickness)
        }
        ElementChange::Colour(colour) => SetField::<ElementId>::new::<H>(id, H::COLOUR, *colour),
        ElementChange::FloorColour(colour) => {
            let floor = H::FLOOR_COLOUR.ok_or(AuthoringError::NotARoom(id))?;
            SetField::<ElementId>::new::<H>(id, floor, *colour)
        }
        ElementChange::Cuts(cuts) => {
            let field = H::CUTS.ok_or(AuthoringError::NotARoom(id))?;
            SetField::<ElementId>::new::<H>(id, field, *cuts)
        }
        ElementChange::AddPoint { segment, t } => {
            return add_point(world, id, &outline, *segment, *t)
                .map(|()| OutlineEdit::Recorded(Vec::new()));
        }
        ElementChange::RemovePoint { index } => {
            return remove_point(world, id, &outline, *index).map(OutlineEdit::Recorded);
        }
        ElementChange::Width(_)
        | ElementChange::Rotation(_)
        | ElementChange::Mirrored(_)
        | ElementChange::Side(_)
        | ElementChange::Along { .. } => return Err(AuthoringError::NotAPortal(id)),
        ElementChange::Material(_) | ElementChange::Stroke { .. } => {
            return Err(AuthoringError::NotATerrain(id));
        }
    };
    field.map(OutlineEdit::Field).map_err(history)
}

/// The field command that gives a Wall or a Room the points and control points of `path` moved
/// by `offset`, each by exactly what single-precision addition gives, so a whole number of cells
/// added to a point on a Grid corner lands on a Grid corner.
///
/// # Errors
///
/// [`AuthoringError::MalformedWall`] or [`AuthoringError::MalformedRoom`] for a point the move
/// takes past what is finite, or [`AuthoringError::History`] when the Wall or the Room cannot be
/// addressed.
fn translated<H: OutlineHost>(
    id: ElementId,
    outline: &H,
    mut path: Path,
    offset: Vec2,
) -> Result<SetField<ElementId>, AuthoringError> {
    for point in path
        .points
        .iter_mut()
        .chain(path.controls.iter_mut().flatten())
    {
        *point += offset;
    }
    let moved = outline.with_path(path);
    well_formed(&moved)?;
    SetField::<ElementId>::new::<H>(id, "", moved)
        .map_err(|error| AuthoringError::History(error.to_string()))
}

/// Adds a point on a part of a Wall or a Room, splitting it into two parts of the shape it had,
/// as one history step that also moves each Portal set into it to the part and parameter that
/// keep it where it was, and each Portal anchored at a part it lacks one part on, so it still
/// names none.
///
/// # Errors
///
/// [`AuthoringError::Shape`] when the part or the parameter is not on the outline, or
/// [`AuthoringError::History`] when the step could not be recorded.
fn add_point<H: OutlineHost>(
    world: &mut World,
    element: ElementId,
    before: &H,
    part: usize,
    t: f32,
) -> Result<(), AuthoringError> {
    let path = before.path();
    let outline = before.with_path(split_wall(&path, part, t)?);
    // A Portal anchored at a part the host lacks lies past every part, so it moves one on like a
    // Portal on a later part and never comes to name the new one: it keeps standing where it was
    // saved.
    let (set, lost) = anchored_to(world, element);
    let anchored: Vec<Anchored> = set.into_iter().chain(lost).collect();
    let places = anchor_portals_through(
        &path,
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
    record_together::<H>(
        world,
        Vec::new(),
        Reshape {
            element,
            outline,
            previous: None,
        },
        moves,
        None,
    )
    .map(|_| ())
}

/// The outline with the point `index` removed: on an open line the two segments at an inner
/// point join into one straight segment and an end point takes its segment with it; on a closed
/// outline the edge that ends at the point and the edge that starts there join into one straight
/// edge in the place of the first, the joined edge of the first point being the last.
fn without_point(path: &Path, index: usize) -> Path {
    let mut joined = path.clone();
    let points = path.points.len();
    joined.points.remove(index);
    if path.closed {
        joined.controls.remove(index);
        let into = if index == 0 { points - 2 } else { index - 1 };
        joined.controls[into] = None;
    } else if index == 0 {
        joined.controls.remove(0);
    } else if index == points - 1 {
        joined.controls.pop();
    } else {
        joined.controls.remove(index);
        joined.controls[index - 1] = None;
    }
    joined
}

/// Removes a point of a Wall or a Room as one history step, joining its outline straight there.
/// One of the fewest points it may have is removed whole, with its Portals, in a group of its
/// own. The Portals set into the part of it that goes are removed in the same step, before the
/// point, the other Portals set into it move to the part and parameter that keep them on their
/// part of it, and the Portals set into others of its kind on its Layer that the step leaves with
/// no Wall at their centre are removed after it. Returns the answers naming the Portals that
/// went.
///
/// # Errors
///
/// [`AuthoringError::NoPoint`] when it has no such point, or [`AuthoringError::History`] when
/// the step could not be recorded.
fn remove_point<H: OutlineHost>(
    world: &mut World,
    element: ElementId,
    before: &H,
    index: usize,
) -> Result<Vec<PortalsRemoved>, AuthoringError> {
    let path = before.path();
    let points = path.points.len();
    if index >= points {
        return Err(AuthoringError::NoPoint {
            outline: H::OUTLINE,
            index,
            points,
        });
    }
    if points <= H::FEWEST_POINTS {
        return remove_with_portals::<H>(world, element);
    }
    let outline = before.with_path(without_point(&path, index));
    let walls = walls_before::<H>(world, element);
    let (portals, _) = anchored_to(world, element);
    let places = anchor_portals_through(&path, PointEdit::Removed { index }, &settings(&portals));
    let mut gone = Vec::new();
    let mut moves = Vec::new();
    for ((portal, anchor, _), place) in portals.iter().zip(places) {
        match place {
            Some(place) => moves.extend(moved(*portal, *anchor, place)?),
            None => gone.push(*portal),
        }
    }
    let taken = record_together::<H>(
        world,
        gone.clone(),
        Reshape {
            element,
            outline,
            previous: None,
        },
        moves,
        walls,
    )?;
    Ok(removed(element, gone).into_iter().chain(taken).collect())
}

/// Removes a Wall or a Room and every Portal set into it as one history step, the Portals first,
/// so undo restores the host and then its Portals, and after them every Portal set into another
/// of its kind on its Layer that the removal leaves with no Wall at its centre. Returns the
/// answers naming the Portals that went.
///
/// # Errors
///
/// [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn remove_with_portals<H: OutlineHost>(
    world: &mut World,
    element: ElementId,
) -> Result<Vec<PortalsRemoved>, AuthoringError> {
    let gone: Vec<ElementId> = anchored_to(world, element)
        .0
        .into_iter()
        .map(|(portal, ..)| portal)
        .collect();
    let walls = walls_before::<H>(world, element);
    crate::history(world)?.begin_group();
    let mut outcome = Ok(());
    for portal in &gone {
        outcome = outcome.and_then(|()| crate::record(world, Remove::of(*portal)));
    }
    outcome = outcome.and_then(|()| crate::record(world, Remove::of(element)));
    let mut taken = Vec::new();
    if let Some((layer, before)) = &walls {
        outcome = outcome.and_then(|()| {
            take_walls_away::<H>(world, *layer, before).map(|removed| taken = removed)
        });
    }
    crate::close_group(world, outcome)?;
    Ok(removed(element, gone).into_iter().chain(taken).collect())
}

/// The Layer of the Wall or the Room `element` and the Portals set into the others of its kind
/// there that have a Wall at their centre, when the kind combines.
pub(crate) fn walls_before<H: OutlineHost>(
    world: &mut World,
    element: ElementId,
) -> Option<(Entity, Walled)> {
    if !H::COMBINES {
        return None;
    }
    let entity = element.entity(world).ok()?;
    let layer = layer_of(world, entity)?;
    Some((layer, walled::<H>(world, layer)))
}

/// What `AnchorPortals` is told about the Portals set into a Wall or a Room.
pub(crate) fn settings(portals: &[Anchored]) -> Vec<PortalSetting> {
    portals
        .iter()
        .map(|(_, anchor, width)| PortalSetting {
            outline: 0,
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
fn moved(
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

/// Records the removal of the Portals `gone`, a reshape of their Wall or Room, the moves of the
/// Portals that stay, and the removal of the Portals of `walls`, those set into others of its
/// kind on its Layer, that the reshape leaves with no Wall at their centre, as one history step,
/// in that order, so undo restores the host's points before its Portals. With no Portal involved
/// the reshape is a step of its own. Returns the answers naming the Portals of `walls` that went.
///
/// # Errors
///
/// [`AuthoringError::History`] when a command could not be applied; what was applied before it
/// is taken back, so nothing of the step is left.
fn record_together<H: OutlineHost>(
    world: &mut World,
    gone: Vec<ElementId>,
    reshape: impl ReversibleCommand,
    moves: Vec<SetField<ElementId>>,
    walls: Option<(Entity, Walled)>,
) -> Result<Vec<PortalsRemoved>, AuthoringError> {
    let walls = walls.filter(|(_, before)| !before.is_empty());
    if gone.is_empty() && moves.is_empty() && walls.is_none() {
        return crate::record_step(world, reshape).map(|()| Vec::new());
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
    let mut taken = Vec::new();
    if let Some((layer, before)) = &walls {
        outcome = outcome.and_then(|()| {
            take_walls_away::<H>(world, *layer, before).map(|removed| taken = removed)
        });
    }
    crate::close_group(world, outcome)?;
    Ok(taken)
}

/// The answer naming the Portals set into `host` that a Command removed, when it removed any.
fn removed(host: ElementId, portals: Vec<ElementId>) -> Option<PortalsRemoved> {
    (!portals.is_empty()).then_some(PortalsRemoved { host, portals })
}

/// The outline of the Wall or the Room of kind `H` an entity carries, if it carries one, with
/// what errors call it.
pub(crate) fn path_of<H: OutlineHost>(
    world: &World,
    entity: Entity,
) -> Option<(Path, OutlineKind)> {
    world
        .get::<H>(entity)
        .map(|outline| (outline.path(), H::OUTLINE))
}
