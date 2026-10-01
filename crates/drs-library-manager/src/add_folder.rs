//! Add Asset Folder: the checks, the step, and its undo.

use crate::index::{index_folder, library};
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::world::World;
use drs_catalog_engine::same_name;
use drs_history::ReversibleCommand;
use drs_library_access::{Manifest, forget_manifest, write_manifest};
use drs_model::{AssetFolder, AssetFolderChanged, CanonicalName, FolderAdded, FolderRefusal};
use std::path::{Path, PathBuf};

/// The folder's path with every symbolic link, `.`, `..`, and case variation resolved, so two
/// spellings of one folder compare equal.
///
/// # Errors
///
/// [`FolderRefusal::Unreadable`] when the folder does not exist or cannot be reached.
fn resolved(path: &Path) -> Result<PathBuf, FolderRefusal> {
    std::fs::canonicalize(path).map_err(|error| FolderRefusal::Unreadable {
        reason: format!("{}: {error}", path.display()),
    })
}

/// Everything that can refuse a folder before anything is recorded.
///
/// # Errors
///
/// The [`FolderRefusal`] that applies: a blank name first, then the folder itself, then a name
/// another folder already holds, so re-adding a folder under its own name says which folder it is.
fn check(world: &mut World, path: &Path, name: &CanonicalName) -> Result<(), FolderRefusal> {
    if name.as_str().is_empty() {
        return Err(FolderRefusal::BlankName);
    }
    let candidate = resolved(path)?;
    std::fs::read_dir(&candidate).map_err(|error| FolderRefusal::Unreadable {
        reason: format!("{}: {error}", path.display()),
    })?;

    let mut added: Vec<AssetFolder> = world.query::<&AssetFolder>().iter(world).cloned().collect();
    added.sort_by(|a, b| a.key.cmp(&b.key));
    for folder in &added {
        let existing = resolved(&folder.path).unwrap_or_else(|_| folder.path.clone());
        if existing == candidate {
            return Err(FolderRefusal::AlreadyAdded {
                name: folder.name.clone(),
            });
        }
        if candidate.starts_with(&existing) {
            return Err(FolderRefusal::InsideAdded {
                name: folder.name.clone(),
                path: folder.path.clone(),
            });
        }
        if existing.starts_with(&candidate) {
            return Err(FolderRefusal::ContainsAdded {
                name: folder.name.clone(),
                path: folder.path.clone(),
            });
        }
    }
    for folder in &added {
        if same_name(&folder.name, name) {
            return Err(FolderRefusal::NameInUse {
                name: folder.name.clone(),
                path: folder.path.clone(),
            });
        }
    }
    Ok(())
}

/// The recorded step: writing the Manifest, indexing the folder, and writing it into the World.
///
/// Applying and reverting both announce Asset Folder Changed for the Canonical Name, so that
/// whatever refers to the folder's Assets by that name resolves them again.
struct AddAssetFolder {
    /// What the folder is remembered by.
    manifest: Manifest,
    /// The entity carrying the folder while it is added.
    folder: Option<Entity>,
}

impl ReversibleCommand for AddAssetFolder {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        let (directories, table) = library(world)?;
        write_manifest(&directories, &self.manifest)?;
        let indexed = index_folder(
            &directories,
            &table,
            &self.manifest.key,
            &self.manifest.path,
        );
        let (assets, scan) = match indexed {
            Ok(indexed) => indexed,
            Err(error) => {
                forget_manifest(&directories, &table, &self.manifest.key)?;
                return Err(error.into());
            }
        };
        let folder = world
            .spawn(AssetFolder {
                name: self.manifest.name.clone(),
                key: self.manifest.key.clone(),
                path: self.manifest.path.clone(),
                version: self.manifest.version.clone(),
                assets,
                skips: scan.skips,
            })
            .id();
        self.folder = Some(folder);
        world.write_message(AssetFolderChanged {
            name: self.manifest.name.clone(),
        });
        Ok(())
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        let (directories, table) = library(world)?;
        forget_manifest(&directories, &table, &self.manifest.key)?;
        if let Some(folder) = self.folder.take() {
            world.despawn(folder);
        }
        world.write_message(AssetFolderChanged {
            name: self.manifest.name.clone(),
        });
        Ok(())
    }
}

/// `AddFolder`: makes the folder at `path` available under `name`, recording the step in the
/// history. The name is trimmed first.
///
/// # Errors
///
/// A [`FolderRefusal`] with the reason, in which case nothing is recorded.
pub(crate) fn add_folder(
    world: &mut World,
    path: PathBuf,
    name: &CanonicalName,
) -> Result<FolderAdded, FolderRefusal> {
    let name = CanonicalName(name.as_str().trim().to_owned());
    check(world, &path, &name)?;
    let manifest = Manifest::new(path, name);
    let key = manifest.key.clone();
    drs_history::apply_step(
        world,
        AddAssetFolder {
            manifest,
            folder: None,
        },
    )
    .map_err(|error| FolderRefusal::Failed {
        reason: error.to_string(),
    })?;
    world
        .query::<(Entity, &AssetFolder)>()
        .iter(world)
        .find(|(_, folder)| folder.key == key)
        .map(|(folder, added)| FolderAdded {
            folder,
            name: added.name.clone(),
            key: added.key.clone(),
            skips: added.skips,
        })
        .ok_or(FolderRefusal::Failed {
            reason: "the folder was indexed but is not in the World".to_owned(),
        })
}
