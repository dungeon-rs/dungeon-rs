//! Terrain: Paint, which lays a stroke on a Layer's Terrain and makes the Terrain with the first
//! one, the Edit Element that changes the image a Terrain shows, and the coverage derived from
//! its strokes.

use crate::AuthoringError;
use crate::place::{Resolved, project_of, resolve, take_off};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Commands, Query};
use bevy_ecs::world::World;
use drs_history::{ReversibleCommand, Target};
use drs_model::{
    AssetAddress, AssetFolder, AssetFolderReference, AssetReference, AssetReferenceRow,
    AssetReferences, Element, ElementId, Paint, Stroke, TERRAIN, Terrain, TerrainCoverage,
};
use drs_paint_engine::{PaintCache, apply_stroke};
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
        let material = world
            .get_mut::<AssetReferences>(self.project)
            .ok_or(AuthoringError::NoProject)?
            .record(self.reference.clone(), self.folder.clone())?;
        let terrain = Terrain {
            material,
            strokes: vec![self.stroke.clone()],
        };
        let footprint = terrain.element_box();
        let entity = world
            .spawn((
                Element {
                    kind: TERRAIN,
                    position: footprint.center(),
                    size: footprint.size(),
                },
                terrain,
                self.element,
            ))
            .id();
        world
            .get_entity_mut(self.layer)
            .map_err(|_| AuthoringError::NotALayer)?
            .insert_children(0, &[entity]);
        Ok(())
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
        let previous = std::mem::replace(&mut terrain.material, row);
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
            .material = previous;
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
            Some((*world.get::<ElementId>(*child)?, terrain.material))
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
    let (folder, name) = {
        let folder = world
            .query::<&AssetFolder>()
            .iter(world)
            .find(|folder| folder.key == asset.folder)
            .ok_or_else(|| AuthoringError::UnknownFolder(asset.folder.clone()))?;
        let indexed = folder
            .assets
            .iter()
            .find(|indexed| indexed.place == asset.place)
            .ok_or_else(|| AuthoringError::UnknownAsset {
                folder: folder.name.clone(),
                place: asset.place.clone(),
            })?;
        (folder.name.clone(), indexed.name.clone())
    };
    let place: String = asset.place.nfc().collect();
    let row = world
        .get::<AssetReferences>(project)
        .and_then(|references| references.row_of(&folder, &place));
    Ok((name, row))
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
/// history step.
///
/// # Errors
///
/// [`AuthoringError::NotALayer`] or [`AuthoringError::NoProject`] for a Layer that is not one or
/// belongs to no Project, [`AuthoringError::MalformedStroke`] for a stroke that is not one,
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
        (Some((element, material)), asset) => {
            if let Some(asset) = asset {
                let (painted, row) = recorded_row(world, project, asset)?;
                if row != Some(material) {
                    return Err(AuthoringError::AnotherImage {
                        painted,
                        shown: name_of(world, project, material),
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
/// stroke kept, as one history step.
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
    if world.get::<Terrain>(entity).is_none() {
        return Err(AuthoringError::NotATerrain(element));
    }
    let layer = world
        .get::<bevy_ecs::hierarchy::ChildOf>(entity)
        .map(bevy_ecs::hierarchy::ChildOf::parent)
        .ok_or(AuthoringError::NotOnALayer(element))?;
    let project = project_of(world, layer)?;
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

/// A Terrain's tiled coverage as the paint Engine keeps it, private to the Manager: the cache the
/// published [`TerrainCoverage`] shares its tiles with.
#[derive(Component, Debug, Default)]
pub(crate) struct Painted(PaintCache);

/// Brings the coverage of every Terrain whose strokes may have changed since the last frame up
/// to its strokes through the paint Engine, publishes it when a tile changed, and sets the
/// Element's box around the strokes. A new image keeps the coverage.
#[expect(
    clippy::type_complexity,
    reason = "a Bevy query is spelled out by the components it reads and writes"
)]
pub(crate) fn derive_coverage(
    mut commands: Commands,
    mut terrains: Query<
        (
            Entity,
            &Terrain,
            &mut Element,
            Option<&mut TerrainCoverage>,
            Option<&mut Painted>,
        ),
        Changed<Terrain>,
    >,
) {
    for (entity, terrain, mut element, coverage, painted) in &mut terrains {
        match (coverage, painted) {
            (Some(mut coverage), Some(mut painted)) => {
                apply_stroke(&mut painted.0, &terrain.strokes);
                if !same_tiles(&coverage, &painted.0) {
                    *coverage = painted.0.coverage();
                }
            }
            (_, painted) => {
                let mut cache = painted
                    .map(|mut painted| std::mem::take(&mut painted.0))
                    .unwrap_or_default();
                apply_stroke(&mut cache, &terrain.strokes);
                commands
                    .entity(entity)
                    .insert((cache.coverage(), Painted(cache)));
            }
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

/// Whether the published coverage holds the cache's tiles at their revisions.
fn same_tiles(coverage: &TerrainCoverage, cache: &PaintCache) -> bool {
    coverage.tiles.len() == cache.tiles().len()
        && coverage
            .tiles
            .iter()
            .zip(cache.tiles())
            .all(|((key, tile), (cached_key, cached))| {
                key == cached_key && tile.revision == cached.revision
            })
}
