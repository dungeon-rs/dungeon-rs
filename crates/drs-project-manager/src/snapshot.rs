//! The Project between the World and its snapshot: gathering every component through the
//! serialisation registry, and materialising a snapshot into a Project tree.

use crate::ProjectManagerError;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;
use drs_model::{
    Bounds, Element, ElementId, Envelopes, Layer, LayerSnapshot, Level, LevelSnapshot, Project,
    ProjectSnapshot, SerialisationRegistry, Tier,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The serialisation registry.
///
/// # Errors
///
/// [`ProjectManagerError::NoRegistry`] when the World has none.
pub(crate) fn registry(world: &World) -> Result<SerialisationRegistry, ProjectManagerError> {
    world
        .get_resource::<SerialisationRegistry>()
        .cloned()
        .ok_or(ProjectManagerError::NoRegistry)
}

/// The children of `entity` that carry `C`, in order.
fn children_with<C: bevy_ecs::component::Component>(world: &World, entity: Entity) -> Vec<Entity> {
    world
        .get::<Children>(entity)
        .map(|children| {
            children
                .iter()
                .copied()
                .filter(|child| world.get::<C>(*child).is_some())
                .collect()
        })
        .unwrap_or_default()
}

/// The envelopes of every registered component `entity` carries.
///
/// # Errors
///
/// [`ProjectManagerError::Gone`] when the entity is gone, or the registry's error.
fn envelopes_of(
    world: &World,
    registry: &SerialisationRegistry,
    entity: Entity,
) -> Result<Envelopes, ProjectManagerError> {
    let entity = world
        .get_entity(entity)
        .map_err(|_| ProjectManagerError::Gone)?;
    Ok(registry.write_all(entity)?)
}

/// Gathers the Project at `project` from the World into its snapshot.
///
/// # Errors
///
/// [`ProjectManagerError::NoRegistry`] without a registry, or the registry's error when a
/// component cannot be written.
pub(crate) fn gather(
    world: &World,
    project: Entity,
) -> Result<ProjectSnapshot, ProjectManagerError> {
    let registry = registry(world)?;
    let mut elements = BTreeMap::new();
    let mut levels = Vec::new();
    for level in children_with::<Level>(world, project) {
        let mut layers = Vec::new();
        for layer in children_with::<Layer>(world, level) {
            let mut order = Vec::new();
            for element in children_with::<ElementId>(world, layer) {
                let Some(id) = world.get::<ElementId>(element).copied() else {
                    continue;
                };
                elements.insert(id, envelopes_of(world, &registry, element)?);
                order.push(id);
            }
            layers.push(LayerSnapshot {
                components: envelopes_of(world, &registry, layer)?,
                elements: order,
            });
        }
        levels.push(LevelSnapshot {
            components: envelopes_of(world, &registry, level)?,
            layers,
        });
    }
    Ok(ProjectSnapshot {
        project: envelopes_of(world, &registry, project)?,
        levels,
        elements,
    })
}

/// Materialises `snapshot`, read from the file at `path`, as a new Project tree named `name`,
/// beside whatever the World holds.
///
/// Nothing of the current Project is touched: every entity of the tree is spawned as a child
/// first, so on any failure despawning the half-built Project takes the whole tree with it and
/// the error is returned for the caller to refuse the file.
///
/// # Errors
///
/// [`ProjectManagerError::NoRegistry`] without a registry, the registry's error when a
/// component cannot be read or sits on an entity of another tier than its own, or
/// [`ProjectManagerError::Malformed`] when the snapshot's parts do not fit together: a Level or
/// Layer without its component, an Element without its common component, an Element listed on
/// no Layer or on more than one, a listed identity the snapshot does not hold, or Bounds with a
/// width or a height of no cell.
pub(crate) fn materialise(
    world: &mut World,
    path: &Path,
    snapshot: &ProjectSnapshot,
    name: String,
) -> Result<Entity, ProjectManagerError> {
    let registry = registry(world)?;
    let project = world.spawn_empty().id();
    match build(world, &registry, project, path, snapshot, name) {
        Ok(()) => Ok(project),
        Err(error) => {
            world.despawn(project);
            Err(error)
        }
    }
}

/// Fills the tree under `project` from `snapshot`.
///
/// # Errors
///
/// As [`materialise`].
fn build(
    world: &mut World,
    registry: &SerialisationRegistry,
    project: Entity,
    path: &Path,
    snapshot: &ProjectSnapshot,
    name: String,
) -> Result<(), ProjectManagerError> {
    let malformed = |reason: String| ProjectManagerError::Malformed {
        path: path.to_path_buf(),
        reason,
    };
    {
        let mut entity = world.entity_mut(project);
        registry.read_all(&mut entity, &snapshot.project, Tier::Project)?;
        entity.insert(Project { name });
        // Bounds of no cell export nothing and no Command can make them; Bounds beyond what a
        // Resize Bounds may make are kept as they are, for an editor that allows them.
        if let Some(bounds) = entity.get::<Bounds>()
            && (bounds.size.x == 0 || bounds.size.y == 0)
        {
            return Err(malformed(format!(
                "the Bounds are {} by {} cells; they need a width and a height of at least one cell",
                bounds.size.x, bounds.size.y
            )));
        }
    }
    let mut placed: BTreeSet<ElementId> = BTreeSet::new();
    for (level_index, level_snapshot) in snapshot.levels.iter().enumerate() {
        let mut level = world.spawn(ChildOf(project));
        registry.read_all(&mut level, &level_snapshot.components, Tier::Level)?;
        if !level.contains::<Level>() {
            return Err(malformed(format!(
                "Level {level_index} has no `level` component"
            )));
        }
        let level = level.id();
        for (layer_index, layer_snapshot) in level_snapshot.layers.iter().enumerate() {
            let mut layer = world.spawn(ChildOf(level));
            registry.read_all(&mut layer, &layer_snapshot.components, Tier::Layer)?;
            if !layer.contains::<Layer>() {
                return Err(malformed(format!(
                    "Layer {layer_index} of Level {level_index} has no `layer` component"
                )));
            }
            let layer = layer.id();
            for id in &layer_snapshot.elements {
                let Some(envelopes) = snapshot.elements.get(id) else {
                    return Err(malformed(format!(
                        "the Element {} is listed on a Layer but not held",
                        id.as_raw()
                    )));
                };
                if !placed.insert(*id) {
                    return Err(malformed(format!(
                        "the Element {} is listed more than once",
                        id.as_raw()
                    )));
                }
                // The identity goes in before the envelopes, so the common component's
                // required identity is the recorded one and not a fresh one.
                let mut element = world.spawn((*id, ChildOf(layer)));
                registry.read_all(&mut element, envelopes, Tier::Element)?;
                if !element.contains::<Element>() {
                    return Err(malformed(format!(
                        "the Element {} has no `element` component",
                        id.as_raw()
                    )));
                }
            }
        }
    }
    if placed.len() != snapshot.elements.len() {
        return Err(malformed(
            "an Element is held but listed on no Layer".to_owned(),
        ));
    }
    Ok(())
}
