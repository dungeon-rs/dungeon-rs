//! Terrain: Paint, which lays a stroke on a Layer's Terrain and makes the Terrain with the first
//! one that paints, the Edit Elements that change the image a Terrain shows and each of its
//! strokes, and the coverage derived from its strokes.

use crate::AuthoringError;
use crate::place::{Resolved, indexed_asset, project_of, resolve, spawn_beneath, take_off};
use bevy_ecs::change_detection::{DetectChanges, Ref};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::system::{Commands, Query, Res};
use bevy_ecs::world::World;
use drs_history::{ReversibleCommand, SetField, Target};
use drs_model::{
    AssetAddress, AssetFolderReference, AssetReference, AssetReferenceRow, AssetReferences,
    Element, ElementChange, ElementId, Paint, Stroke, StrokeChange, TERRAIN, Terrain,
    TerrainCoverage, Viewport,
};
use drs_paint_engine::{PaintCache, StrokeRasterizer, apply_stroke};
use unicode_normalization::UnicodeNormalization;

/// The recorded step of the first stroke on a Layer: the Terrain spawned under every Element on
/// the Layer, holding the stroke, keeping its identity so that redo brings it back exactly.
///
/// The Asset Reference row of its image, and the Asset Folder row it comes from, are recorded on
/// the first application and never removed, as a Prop's are.
struct PaintTerrain {
    /// The Project whose Asset Reference table records the image.
    project: Entity,
    /// The Layer painted on.
    layer: Entity,
    /// What the Project records about the image.
    reference: AssetReference,
    /// What the Project records about the image's folder.
    folder: AssetFolderReference,
    /// The stroke.
    stroke: Stroke,
    /// The identity the Terrain keeps through undo and redo.
    element: ElementId,
}

impl ReversibleCommand for PaintTerrain {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        if world.get_entity(self.layer).is_err() {
            return Err(AuthoringError::NotALayer.into());
        }
        let image = world
            .get_mut::<AssetReferences>(self.project)
            .ok_or(AuthoringError::NoProject)?
            .record(self.reference.clone(), self.folder.clone())?;
        let terrain = Terrain {
            image,
            strokes: vec![self.stroke.clone()],
        };
        let footprint = terrain.element_box();
        spawn_beneath(
            world,
            self.layer,
            (
                Element {
                    kind: TERRAIN,
                    position: footprint.center(),
                    size: footprint.size(),
                },
                terrain,
                self.element,
            ),
        )
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        take_off(world, self.element)
    }
}

/// The recorded step of a stroke on a Terrain that has strokes: the stroke appended, and on
/// revert taken off the end again. It keeps only its own stroke.
struct AppendStroke {
    /// The Terrain.
    element: ElementId,
    /// The stroke.
    stroke: Stroke,
}

impl ReversibleCommand for AppendStroke {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        let entity = self.element.entity(world)?;
        world
            .get_mut::<Terrain>(entity)
            .ok_or(AuthoringError::NotATerrain(self.element))?
            .strokes
            .push(self.stroke.clone());
        Ok(())
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        let entity = self.element.entity(world)?;
        let mut terrain = world
            .get_mut::<Terrain>(entity)
            .ok_or(AuthoringError::NotATerrain(self.element))?;
        if terrain.strokes.last() != Some(&self.stroke) {
            return Err(AuthoringError::History(
                "the Terrain's last stroke is not the one to take back".to_owned(),
            )
            .into());
        }
        terrain.strokes.pop();
        Ok(())
    }
}

/// The recorded step of changing the image a Terrain shows: its Material's row swapped for the
/// chosen Asset's, every stroke untouched, and swapped back on revert.
struct SetMaterial {
    /// The Project whose Asset Reference table records the image.
    project: Entity,
    /// The Terrain.
    element: ElementId,
    /// What the Project records about the image.
    reference: AssetReference,
    /// What the Project records about the image's folder.
    folder: AssetFolderReference,
    /// The row the Terrain showed before, once applied.
    previous: Option<AssetReferenceRow>,
}

impl ReversibleCommand for SetMaterial {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        let entity = self.element.entity(world)?;
        if world.get::<Terrain>(entity).is_none() {
            return Err(AuthoringError::NotATerrain(self.element).into());
        }
        let row = world
            .get_mut::<AssetReferences>(self.project)
            .ok_or(AuthoringError::NoProject)?
            .record(self.reference.clone(), self.folder.clone())?;
        let mut terrain = world
            .get_mut::<Terrain>(entity)
            .ok_or(AuthoringError::NotATerrain(self.element))?;
        let previous = std::mem::replace(&mut terrain.image, row);
        if self.previous.is_none() {
            self.previous = Some(previous);
        }
        Ok(())
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        let Some(previous) = self.previous else {
            return Ok(());
        };
        let entity = self.element.entity(world)?;
        world
            .get_mut::<Terrain>(entity)
            .ok_or(AuthoringError::NotATerrain(self.element))?
            .image = previous;
        Ok(())
    }
}

/// The topmost Terrain on a Layer, with its identity and the row of its image.
fn topmost_terrain(world: &World, layer: Entity) -> Option<(ElementId, AssetReferenceRow)> {
    world
        .get::<Children>(layer)?
        .iter()
        .rev()
        .find_map(|child| {
            let terrain = world.get::<Terrain>(*child)?;
            Some((*world.get::<ElementId>(*child)?, terrain.image))
        })
}

/// The name a chosen Asset has, and the row of the Project's Asset Reference table that already
/// records it, if one does.
///
/// # Errors
///
/// [`AuthoringError::UnknownFolder`] or [`AuthoringError::UnknownAsset`] when the chosen Asset is
/// not indexed.
fn recorded_row(
    world: &mut World,
    project: Entity,
    asset: &AssetAddress,
) -> Result<(String, Option<AssetReferenceRow>), AuthoringError> {
    let indexed = indexed_asset(world, asset)?;
    let place: String = asset.place.nfc().collect();
    let row = world
        .get::<AssetReferences>(project)
        .and_then(|references| references.row_of(&indexed.folder.name, &place));
    Ok((indexed.asset.name, row))
}

/// The name a row of the Project's Asset Reference table records.
fn name_of(world: &World, project: Entity, row: AssetReferenceRow) -> String {
    world
        .get::<AssetReferences>(project)
        .and_then(|references| references.get(row))
        .map_or_else(
            || format!("row {}", row.0),
            |reference| reference.name.clone(),
        )
}

/// Refuses a stroke that would not be one, for the reason the Terrain's own check gives.
///
/// # Errors
///
/// [`AuthoringError::MalformedStroke`] with that reason.
fn well_formed(stroke: &Stroke) -> Result<(), AuthoringError> {
    stroke.malformation().map_or(Ok(()), |reason| {
        Err(AuthoringError::MalformedStroke(reason))
    })
}

/// Paint: adds the stroke to the topmost Terrain on the Layer, or, on a Layer with none, places
/// a Terrain of the chosen Asset's image holding it under every Element on the Layer, as one
/// history step. A stroke that erases is added to the topmost Terrain whatever image it names,
/// or none, and makes no Terrain.
///
/// # Errors
///
/// [`AuthoringError::NotALayer`] or [`AuthoringError::NoProject`] for a Layer that is not one or
/// belongs to no Project, [`AuthoringError::MalformedStroke`] for a stroke that is not one,
/// [`AuthoringError::NothingToErase`] for an erase on a Layer with no Terrain,
/// [`AuthoringError::NothingToPaintWith`] for a Paint naming no Asset on a Layer with no
/// Terrain, [`AuthoringError::AnotherImage`] for an Asset other than the one the Terrain shows,
/// the errors of resolving the Asset, or [`AuthoringError::History`] when the step could not be
/// recorded.
pub(crate) fn paint(world: &mut World, command: &Paint) -> Result<(), AuthoringError> {
    if world.get::<drs_model::Layer>(command.layer).is_none() {
        return Err(AuthoringError::NotALayer);
    }
    well_formed(&command.stroke)?;
    let project = project_of(world, command.layer)?;
    if command.stroke.erase {
        let (element, _) =
            topmost_terrain(world, command.layer).ok_or(AuthoringError::NothingToErase)?;
        return crate::record_step(
            world,
            AppendStroke {
                element,
                stroke: command.stroke.clone(),
            },
        );
    }
    match (topmost_terrain(world, command.layer), &command.asset) {
        (None, None) => Err(AuthoringError::NothingToPaintWith),
        (None, Some(asset)) => {
            let Resolved {
                reference, folder, ..
            } = resolve(world, command.layer, asset)?;
            crate::record_step(
                world,
                PaintTerrain {
                    project,
                    layer: command.layer,
                    reference,
                    folder,
                    stroke: command.stroke.clone(),
                    element: ElementId::new(),
                },
            )
        }
        (Some((element, image)), asset) => {
            if let Some(asset) = asset {
                let (painted, row) = recorded_row(world, project, asset)?;
                if row != Some(image) {
                    return Err(AuthoringError::AnotherImage {
                        painted,
                        shown: name_of(world, project, image),
                    });
                }
            }
            crate::record_step(
                world,
                AppendStroke {
                    element,
                    stroke: command.stroke.clone(),
                },
            )
        }
    }
}

/// Edit Element of a Terrain's Material: makes the Terrain show the chosen Asset's image, every
/// stroke kept, as one history step; naming the image it already shows changes nothing and
/// records no step.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] when no Element carries the identity,
/// [`AuthoringError::NotATerrain`] when it is no Terrain, [`AuthoringError::NotOnALayer`] or
/// [`AuthoringError::NoProject`] when it belongs to no Project, the errors of resolving the
/// Asset, or [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn set_material(
    world: &mut World,
    element: ElementId,
    asset: &AssetAddress,
) -> Result<(), AuthoringError> {
    let entity = element
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(element))?;
    let shown = world
        .get::<Terrain>(entity)
        .ok_or(AuthoringError::NotATerrain(element))?
        .image;
    let layer = world
        .get::<bevy_ecs::hierarchy::ChildOf>(entity)
        .map(bevy_ecs::hierarchy::ChildOf::parent)
        .ok_or(AuthoringError::NotOnALayer(element))?;
    let project = project_of(world, layer)?;
    if recorded_row(world, project, asset)?.1 == Some(shown) {
        return Ok(());
    }
    let Resolved {
        reference, folder, ..
    } = resolve(world, layer, asset)?;
    crate::record_step(
        world,
        SetMaterial {
            project,
            element,
            reference,
            folder,
            previous: None,
        },
    )
}

/// A copy of the Terrain an Element is, as it stands, to change and check before any change is
/// recorded.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`] when no Element carries the identity, or
/// [`AuthoringError::NotATerrain`] when it is no Terrain.
fn terrain_of(world: &mut World, element: ElementId) -> Result<Terrain, AuthoringError> {
    let entity = element
        .entity(world)
        .map_err(|_| AuthoringError::UnknownElement(element))?;
    world
        .get::<Terrain>(entity)
        .cloned()
        .ok_or(AuthoringError::NotATerrain(element))
}

/// The stroke of that number, to change in a copy of its Terrain.
///
/// # Errors
///
/// [`AuthoringError::NoStroke`] when the Terrain has no stroke of that number.
fn stroke_of(terrain: &mut Terrain, stroke: usize) -> Result<&mut Stroke, AuthoringError> {
    let strokes = terrain.strokes.len();
    terrain
        .strokes
        .get_mut(stroke)
        .ok_or(AuthoringError::NoStroke { stroke, strokes })
}

/// The field command of an Edit Element changing the stroke of number `stroke` of a Terrain: a
/// point of its path, its whole path, its Brush settings, or whether it erases, once the Terrain
/// it would leave is checked; `None` when the change names what the stroke already does, which
/// records nothing, or removes the stroke, which [`remove_stroke`] records as a step of its own.
///
/// # Errors
///
/// [`AuthoringError::UnknownElement`], [`AuthoringError::NotATerrain`],
/// [`AuthoringError::NoStroke`], or [`AuthoringError::NoStrokePoint`] for what the change names
/// and the Element lacks, [`AuthoringError::MalformedStroke`] for the reason the Terrain's own
/// check gives the Terrain it would leave, or [`AuthoringError::History`] when the field cannot
/// be named or the removal recorded.
pub(crate) fn stroke_change(
    world: &mut World,
    element: ElementId,
    stroke: usize,
    change: &StrokeChange,
) -> Result<Option<SetField<ElementId>>, AuthoringError> {
    let mut terrain = terrain_of(world, element)?;
    let history = |error: drs_history::HistoryError| AuthoringError::History(error.to_string());
    let field = match change {
        StrokeChange::Point { index, position } => {
            let points = &mut stroke_of(&mut terrain, stroke)?.points;
            let count = points.len();
            *points
                .get_mut(*index)
                .ok_or(AuthoringError::NoStrokePoint {
                    index: *index,
                    points: count,
                })? = *position;
            SetField::new::<Terrain>(
                element,
                &format!("strokes[{stroke}].points[{index}]"),
                *position,
            )
        }
        StrokeChange::Position(position) => {
            let moved = stroke_of(&mut terrain, stroke)?;
            let by = *position - moved.centre();
            for point in &mut moved.points {
                *point += by;
            }
            SetField::new::<Terrain>(
                element,
                &format!("strokes[{stroke}].points"),
                moved.points.clone(),
            )
        }
        StrokeChange::Brush(brush) => {
            stroke_of(&mut terrain, stroke)?.brush = *brush;
            SetField::new::<Terrain>(element, &format!("strokes[{stroke}].brush"), *brush)
        }
        StrokeChange::Erase(erase) => {
            let changed = stroke_of(&mut terrain, stroke)?;
            if changed.erase == *erase {
                return Ok(None);
            }
            changed.erase = *erase;
            SetField::new::<Terrain>(element, &format!("strokes[{stroke}].erase"), *erase)
        }
        StrokeChange::Remove => {
            return remove_stroke(world, element, terrain, stroke).map(|()| None);
        }
    };
    if let Some(reason) = terrain.malformation() {
        return Err(AuthoringError::MalformedStroke(reason));
    }
    field.map(Some).map_err(history)
}

/// Refuses an Edit Element of a Terrain that changes anything but its Material or one of its
/// strokes. Every change is named, so a new one is routed here on purpose.
///
/// # Errors
///
/// [`AuthoringError::TerrainChangesOnlyItsMaterialAndStrokes`] for such a change of a Terrain.
pub(crate) fn only_material_and_strokes(
    world: &World,
    entity: Entity,
    element: ElementId,
    change: &ElementChange,
) -> Result<(), AuthoringError> {
    let allowed = match change {
        ElementChange::Material(_) | ElementChange::Stroke { .. } => true,
        ElementChange::Position(_)
        | ElementChange::MoveBy(_)
        | ElementChange::Point { .. }
        | ElementChange::Control { .. }
        | ElementChange::AddPoint { .. }
        | ElementChange::RemovePoint { .. }
        | ElementChange::Thickness(_)
        | ElementChange::Colour(_)
        | ElementChange::FloorColour(_)
        | ElementChange::Cuts(_)
        | ElementChange::Width(_)
        | ElementChange::Rotation(_)
        | ElementChange::Mirrored(_)
        | ElementChange::Side(_)
        | ElementChange::Along { .. } => false,
    };
    if world.get::<Terrain>(entity).is_some() && !allowed {
        return Err(AuthoringError::TerrainChangesOnlyItsMaterialAndStrokes(
            element,
        ));
    }
    Ok(())
}

/// The recorded step of removing a stroke that is not its Terrain's only one: the stroke taken
/// out of the order, and on revert put back at its number exactly as it was. It is the only edit
/// that numbers strokes afresh.
struct RemoveStroke {
    /// The Terrain.
    element: ElementId,
    /// The stroke's number.
    index: usize,
    /// The stroke, once taken out.
    removed: Option<Stroke>,
}

impl ReversibleCommand for RemoveStroke {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        let entity = self.element.entity(world)?;
        let mut terrain = world
            .get_mut::<Terrain>(entity)
            .ok_or(AuthoringError::NotATerrain(self.element))?;
        let strokes = terrain.strokes.len();
        if self.index >= strokes {
            return Err(AuthoringError::NoStroke {
                stroke: self.index,
                strokes,
            }
            .into());
        }
        self.removed = Some(terrain.strokes.remove(self.index));
        Ok(())
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        let Some(stroke) = self.removed.take() else {
            return Ok(());
        };
        let entity = self.element.entity(world)?;
        let mut terrain = world
            .get_mut::<Terrain>(entity)
            .ok_or(AuthoringError::NotATerrain(self.element))?;
        let index = self.index.min(terrain.strokes.len());
        terrain.strokes.insert(index, stroke);
        Ok(())
    }
}

/// Edit Element removing a stroke of `terrain`, as a step of its own that closes any gesture
/// left open: every other stroke keeps its order, and the only stroke takes its Terrain with
/// it, as a Remove Element of the Terrain would.
///
/// # Errors
///
/// [`AuthoringError::NoStroke`] for a stroke the Terrain lacks, or [`AuthoringError::History`]
/// when the step could not be recorded.
fn remove_stroke(
    world: &mut World,
    element: ElementId,
    mut terrain: Terrain,
    stroke: usize,
) -> Result<(), AuthoringError> {
    stroke_of(&mut terrain, stroke)?;
    if terrain.strokes.len() == 1 {
        return crate::record_step(world, crate::remove::Remove::of(element));
    }
    crate::record_step(
        world,
        RemoveStroke {
            element,
            index: stroke,
            removed: None,
        },
    )
}

/// A Terrain's tiled coverage as the paint Engine keeps it, private to the Manager: the cache the
/// published [`TerrainCoverage`] shares its tiles with.
#[derive(Component, Debug)]
pub(crate) struct Painted(PaintCache);

/// Brings the coverage of every Terrain whose strokes may have changed since the last frame up
/// to its strokes, and of every Terrain when the view changed up to the view, through the paint
/// Engine, publishes it when something it holds changed, and sets the Element's box around the
/// strokes. A new image keeps the coverage.
#[expect(
    clippy::type_complexity,
    reason = "a Bevy query is spelled out by the components it reads and writes"
)]
pub(crate) fn derive_coverage(
    mut commands: Commands,
    mut terrains: Query<(
        Entity,
        Ref<Terrain>,
        &mut Element,
        Option<&mut TerrainCoverage>,
        Option<&mut Painted>,
    )>,
    viewport: Option<Res<Viewport>>,
    mut rasterizer: StrokeRasterizer,
) {
    let looked_elsewhere = viewport.as_ref().is_some_and(DetectChanges::is_changed);
    let viewport = viewport.map_or_else(Viewport::default, |viewport| *viewport);
    for (entity, terrain, mut element, coverage, painted) in &mut terrains {
        let repainted = terrain.is_changed();
        if !repainted && !looked_elsewhere {
            continue;
        }
        match (coverage, painted) {
            (Some(mut coverage), Some(mut painted)) => {
                if apply_stroke(&mut painted.0, &terrain.strokes, &viewport, &mut rasterizer) {
                    *coverage = painted.0.coverage();
                }
            }
            (_, painted) => {
                let fresh = PaintCache::new(&rasterizer);
                let mut cache = match painted {
                    Some(mut painted) => std::mem::replace(&mut painted.0, fresh),
                    None => fresh,
                };
                apply_stroke(&mut cache, &terrain.strokes, &viewport, &mut rasterizer);
                commands
                    .entity(entity)
                    .insert((cache.coverage(), Painted(cache)));
            }
        }
        if !repainted {
            continue;
        }
        let footprint = terrain.element_box();
        if element.position != footprint.center() {
            element.position = footprint.center();
        }
        if element.size != footprint.size() {
            element.size = footprint.size();
        }
    }
}
