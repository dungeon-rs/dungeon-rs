//! Open: a saved Project replacing the current one.

use crate::ProjectManagerError;
use crate::resolve::resolve_project;
use crate::save::history_position;
use crate::snapshot::{materialise, registry};
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::world::World;
use drs_history::History;
use drs_model::{
    AssetReferenceRow, AssetReferences, Element, ElementKindName, ElementKindRegistry, Layer,
    Level, MissingAsset, OpenReport, Portal, ProjectOpened, Prop, Resolution, ResolutionTable,
    SavedMark, UnknownKind, project_name_of,
};
use drs_project_access::read_project;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Open: reads the file at `path` and, once the whole Project in it stands in the World,
/// despawns the current Project, clears the history, records the file and the history's
/// position as the saved mark, resolves every Asset Reference, and abandons any Export of the
/// Project that was replaced.
///
/// # Errors
///
/// The error of reading or materialising the file, in which case nothing changes.
pub(crate) fn open_project(
    world: &mut World,
    path: PathBuf,
) -> Result<ProjectOpened, ProjectManagerError> {
    let registry = registry(world)?;
    let snapshot = read_project(&path, &registry)?;
    let current = crate::projects(world);
    let project = materialise(world, &path, &snapshot, project_name_of(&path))?;
    for old in current {
        world.despawn(old);
    }
    if let Some(mut history) = world.get_resource_mut::<History>() {
        history.clear();
    }
    let position = history_position(world);
    world.insert_resource(SavedMark {
        file: Some(path.clone()),
        position,
    });
    resolve_project(world, project);
    crate::export::abandon_exports(world);
    let report = report(world, project);
    Ok(ProjectOpened { path, report })
}

/// Every Element of the Project, Level by Level and Layer by Layer.
fn elements_of(world: &World, project: Entity) -> Vec<Entity> {
    let children = |entity: Entity| -> Vec<Entity> {
        world
            .get::<Children>(entity)
            .map(|children| children.iter().copied().collect())
            .unwrap_or_default()
    };
    let mut elements = Vec::new();
    for level in children(project) {
        if world.get::<Level>(level).is_none() {
            continue;
        }
        for layer in children(level) {
            if world.get::<Layer>(layer).is_none() {
                continue;
            }
            elements.extend(
                children(layer)
                    .into_iter()
                    .filter(|element| world.get::<Element>(*element).is_some()),
            );
        }
    }
    elements
}

/// What the Author is told about the Project at `project`: each Missing Asset with how many
/// Elements use it, and each Element kind this editor does not know.
pub(crate) fn report(world: &World, project: Entity) -> OpenReport {
    let mut uses: BTreeMap<AssetReferenceRow, usize> = BTreeMap::new();
    let mut unknown: BTreeMap<ElementKindName, usize> = BTreeMap::new();
    let kinds = world.get_resource::<ElementKindRegistry>();
    for element in elements_of(world, project) {
        let row = world
            .get::<Prop>(element)
            .map(|prop| prop.asset)
            .or_else(|| world.get::<Portal>(element).map(|portal| portal.asset));
        if let Some(row) = row {
            *uses.entry(row).or_default() += 1;
        }
        if let Some(common) = world.get::<Element>(element)
            && kinds.is_none_or(|kinds| kinds.get(&common.kind).is_none())
        {
            *unknown.entry(common.kind.clone()).or_default() += 1;
        }
    }
    let references = world.get::<AssetReferences>(project);
    let resolutions = world.get::<ResolutionTable>(project);
    let mut missing_assets = Vec::new();
    if let (Some(references), Some(resolutions)) = (references, resolutions) {
        for (index, reference) in references.assets.iter().enumerate() {
            let row = u32::try_from(index).map(AssetReferenceRow).ok();
            let Some(Resolution::Missing(reason)) = row.and_then(|row| resolutions.get(row)) else {
                continue;
            };
            missing_assets.push(MissingAsset {
                name: reference.name.clone(),
                folder: reference.folder.clone(),
                recorded_version: references
                    .folders
                    .iter()
                    .find(|folder| folder.name == reference.folder)
                    .map(|folder| folder.version.clone())
                    .unwrap_or_default(),
                elements: row.and_then(|row| uses.get(&row)).copied().unwrap_or(0),
                reason: reason.clone(),
            });
        }
    }
    OpenReport {
        missing_assets,
        unknown_kinds: unknown
            .into_iter()
            .map(|(kind, elements)| UnknownKind { kind, elements })
            .collect(),
    }
}
