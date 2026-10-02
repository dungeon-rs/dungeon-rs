//! Indexing a folder: the scan classified into Assets, and Refresh.

use crate::LibraryManagerError;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use drs_catalog_engine::Classifier;
use drs_library_access::{
    LibraryDirectories, LibraryError, LibraryTable, Scan, ScanDiff, scan_folder,
};
use drs_model::{AssetFolder, EditorDirectories, FolderKey, IndexedAsset};
use std::path::Path;

/// Scans the folder at `path` under `key` and classifies what it finds into Assets.
///
/// # Errors
///
/// [`LibraryError`] when the folder cannot be listed or its index cache cannot be written.
pub(crate) fn index_folder(
    directories: &LibraryDirectories,
    table: &LibraryTable,
    key: &FolderKey,
    path: &Path,
) -> Result<(Vec<IndexedAsset>, Scan), LibraryError> {
    let scan = scan_folder(directories, table, key, path)?;
    let classifier = Classifier::built_in();
    let assets = scan
        .files
        .iter()
        .filter_map(|file| {
            let place = Path::new(&file.place);
            let kind = classifier.classify(place)?;
            let name = place.file_stem()?.to_str()?.to_owned();
            Some(IndexedAsset {
                name,
                place: file.place.clone(),
                kind,
                byte_size: file.byte_size,
                modified: file.modified,
            })
        })
        .collect();
    Ok((assets, scan))
}

/// The resolved directories and the `lib://` table, as every indexing step needs them.
///
/// # Errors
///
/// [`LibraryError::NoPlatformDirectories`] when no directory is known.
pub(crate) fn library(
    world: &mut World,
) -> Result<(LibraryDirectories, LibraryTable), LibraryError> {
    let overrides = world
        .get_resource::<EditorDirectories>()
        .cloned()
        .unwrap_or_default();
    let directories = LibraryDirectories::resolve(&overrides)?;
    let table = world.get_resource_or_init::<LibraryTable>().clone();
    Ok((directories, table))
}

/// Refresh: scans an added Asset Folder again, diffed against its index cache, rewrites its
/// index of Assets, and looks up their thumbnails, enqueuing the missing ones.
///
/// # Errors
///
/// [`LibraryManagerError::NotAFolder`] when `folder` carries no [`AssetFolder`], or the
/// [`LibraryError`] of a scan that failed, in which case the index is left as it was.
pub fn refresh(world: &mut World, folder: Entity) -> Result<ScanDiff, LibraryManagerError> {
    let (key, path) = world
        .get::<AssetFolder>(folder)
        .map(|added| (added.key.clone(), added.path.clone()))
        .ok_or(LibraryManagerError::NotAFolder)?;
    let (directories, table) = library(world)?;
    let (assets, scan) = index_folder(&directories, &table, &key, &path)?;
    if let Some(mut added) = world.get_mut::<AssetFolder>(folder) {
        added.assets = assets;
        added.skips = scan.skips;
    }
    crate::thumbnails::track(world, folder);
    Ok(scan.diff)
}
