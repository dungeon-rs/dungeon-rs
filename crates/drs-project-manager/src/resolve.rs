//! Resolution: where each Asset Reference of a Project loads from on this device.

use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageReader;
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, SystemState};
use bevy_ecs::world::World;
use drs_catalog_engine::{resolve, same_name};
use drs_model::{AssetFolder, AssetFolderChanged, AssetReferences, CanonicalName, ResolutionTable};

/// The Asset Folders added on this device, ordered by key so that the outcome never depends on
/// the order entities happen to be stored in.
fn folders(world: &mut World) -> Vec<AssetFolder> {
    let mut folders: Vec<AssetFolder> =
        world.query::<&AssetFolder>().iter(world).cloned().collect();
    folders.sort_by(|a, b| a.key.cmp(&b.key));
    folders
}

/// Writes `rows` as the resolution table of `project`, when they differ from what it holds.
fn write_table(world: &mut World, project: Entity, rows: Vec<drs_model::Resolution>) {
    let Some(mut table) = world.get_mut::<ResolutionTable>(project) else {
        world.entity_mut(project).insert(ResolutionTable { rows });
        return;
    };
    if table.rows != rows {
        table.rows = rows;
    }
}

/// Resolves every Asset Reference of the Project at `project`.
pub(crate) fn resolve_project(world: &mut World, project: Entity) {
    let Some(references) = world.get::<AssetReferences>(project).cloned() else {
        return;
    };
    let folders = folders(world);
    let rows = references
        .assets
        .iter()
        .map(|reference| resolve(reference, &folders))
        .collect();
    write_table(world, project, rows);
}

/// Resolves again the Asset References of every Project recorded against the Asset Folder known
/// by `name`; the rest keep their resolution.
fn resolve_folder(world: &mut World, name: &CanonicalName) {
    let folders = folders(world);
    for project in crate::projects(world) {
        let Some(references) = world.get::<AssetReferences>(project).cloned() else {
            continue;
        };
        let current = world
            .get::<ResolutionTable>(project)
            .map(|table| table.rows.clone())
            .unwrap_or_default();
        let rows = references
            .assets
            .iter()
            .enumerate()
            .map(|(index, reference)| match current.get(index) {
                Some(kept) if !same_name(&reference.folder, name) => kept.clone(),
                Some(_) | None => resolve(reference, &folders),
            })
            .collect();
        write_table(world, project, rows);
    }
}

/// Resolves again whatever is recorded against each [`AssetFolderChanged`] folder's name.
pub(crate) fn handle_folder_changed(
    world: &mut World,
    changes: &mut SystemState<MessageReader<AssetFolderChanged>>,
) {
    let changes: Vec<AssetFolderChanged> = match changes.get_mut(world) {
        Ok(mut reader) => reader.read().cloned().collect(),
        Err(_) => return,
    };
    for AssetFolderChanged { name } in changes {
        resolve_folder(world, &name);
    }
}

/// Resolves every Asset Reference of each Project whose Asset Reference table changed, so a row a
/// placement has just added has its resolution before anything draws it.
pub(crate) fn resolve_changed_references(
    world: &mut World,
    changed: &mut SystemState<Query<Entity, Changed<AssetReferences>>>,
) {
    let projects: Vec<Entity> = match changed.get(world) {
        Ok(query) => query.iter().collect(),
        Err(_) => return,
    };
    for project in projects {
        resolve_project(world, project);
    }
}
