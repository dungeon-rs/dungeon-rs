//! Driving thumbnail generation: the cache opened at startup, every indexed folder's Assets
//! served from it and the missing ones enqueued, the finished ones written into the World, and the
//! browser's wanted Assets put first.

use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageReader;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::SystemState;
use bevy_ecs::world::World;
use drs_library_access::{
    LibraryDirectories, ThumbnailCache, ThumbnailCompletion, ThumbnailGenerator, ThumbnailJob,
    ThumbnailKey, ThumbnailTable,
};
use drs_model::{
    AssetAddress, AssetFolder, CaughtPanics, EditorDirectories, FolderKey, IndexedAsset,
    ThumbnailState, Thumbnails, ThumbnailsUnavailable,
};
use std::collections::BTreeMap;

/// The thumbnail cache and its generator, while they work.
#[derive(Resource, Default)]
pub(crate) struct ThumbnailWork {
    /// The cache, once opened.
    cache: Option<ThumbnailCache>,
    /// The generator, until the cache fails or the editor quits.
    generator: Option<ThumbnailGenerator>,
}

/// Opens the thumbnail cache and starts its generator, or says once why thumbnails cannot be
/// kept.
pub(crate) fn open(world: &mut World) {
    let overrides = world
        .get_resource::<EditorDirectories>()
        .cloned()
        .unwrap_or_default();
    let table = world.get_resource_or_init::<ThumbnailTable>().clone();
    let caught = world
        .get_resource::<CaughtPanics>()
        .copied()
        .unwrap_or_default();
    let opened = LibraryDirectories::resolve(&overrides)
        .and_then(|directories| ThumbnailCache::open(&directories, &table))
        .and_then(|cache| {
            ThumbnailGenerator::start(&cache, caught).map(|generator| (cache, generator))
        });
    match opened {
        Ok((cache, generator)) => {
            world.insert_resource(ThumbnailWork {
                cache: Some(cache),
                generator: Some(generator),
            });
        }
        Err(error) => {
            world.insert_resource(ThumbnailWork::default());
            world.write_message(ThumbnailsUnavailable {
                reason: error.to_string(),
            });
        }
    }
}

/// The key an Asset of the folder with `key` is kept under.
fn key_of(key: &FolderKey, asset: &IndexedAsset) -> ThumbnailKey {
    ThumbnailKey {
        folder: key.clone(),
        place: asset.place.clone(),
        byte_size: asset.byte_size,
        modified: asset.modified,
    }
}

/// Serves the thumbnail of every Asset of a folder that was just indexed, writes their states
/// into the World, and enqueues the missing ones in place order.
pub(crate) fn track(world: &mut World, folder: Entity) {
    let Some(added) = world.get::<AssetFolder>(folder) else {
        return;
    };
    let work = world.get_resource::<ThumbnailWork>();
    let mut states = Vec::with_capacity(added.assets.len());
    let mut jobs = Vec::new();
    for asset in &added.assets {
        let key = key_of(&added.key, asset);
        let state = work
            .and_then(|work| work.cache.as_ref())
            .map_or(ThumbnailState::Pending, |cache| cache.serve(&key));
        if state == ThumbnailState::Pending {
            jobs.push(ThumbnailJob {
                key,
                file: added.path.join(&asset.place),
            });
        }
        states.push(state);
    }
    if let Some(generator) = work.and_then(|work| work.generator.as_ref()) {
        generator.enqueue(jobs);
    }
    world.entity_mut(folder).insert(Thumbnails { states });
}

/// Drops the waiting Assets of a folder that is no longer added; its thumbnails stay kept.
pub(crate) fn withdraw(world: &mut World, key: &FolderKey) {
    if let Some(generator) = world
        .get_resource::<ThumbnailWork>()
        .and_then(|work| work.generator.as_ref())
    {
        generator.withdraw(key);
    }
}

/// The added folders by key.
fn folders_by_key(world: &mut World) -> BTreeMap<FolderKey, Entity> {
    world
        .query::<(Entity, &AssetFolder)>()
        .iter(world)
        .map(|(entity, folder)| (folder.key.clone(), entity))
        .collect()
}

/// Where the Asset at `place` sits in a folder's index, ordered by place.
fn position(folder: &AssetFolder, place: &str) -> Option<usize> {
    folder
        .assets
        .binary_search_by(|asset| asset.place.as_str().cmp(place))
        .ok()
}

/// Writes what the generator finished into the World; a cache that failed is reported once and
/// its generator stopped.
pub(crate) fn drain(world: &mut World) {
    let completions = match world
        .get_resource::<ThumbnailWork>()
        .and_then(|work| work.generator.as_ref())
    {
        Some(generator) => generator.completions(),
        None => return,
    };
    if completions.is_empty() {
        return;
    }
    let folders = folders_by_key(world);
    for completion in completions {
        match completion {
            ThumbnailCompletion::Finished { key, state } => {
                if let Some(&folder) = folders.get(&key.folder) {
                    settle(world, folder, &key, state);
                }
            }
            ThumbnailCompletion::CacheFailed(reason) => {
                if let Some(mut work) = world.get_resource_mut::<ThumbnailWork>() {
                    work.generator = None;
                }
                world.write_message(ThumbnailsUnavailable { reason });
                return;
            }
        }
    }
}

/// Sets the state of the Asset under `key` in `folder`, if the folder still indexes it as it was
/// when its thumbnail was asked for.
fn settle(world: &mut World, folder: Entity, key: &ThumbnailKey, state: ThumbnailState) {
    let Some(index) = world.get::<AssetFolder>(folder).and_then(|added| {
        let index = position(added, &key.place)?;
        let asset = &added.assets[index];
        (asset.byte_size == key.byte_size && asset.modified == key.modified).then_some(index)
    }) else {
        return;
    };
    if let Some(mut thumbnails) = world.get_mut::<Thumbnails>(folder)
        && let Some(slot) = thumbnails.states.get_mut(index)
    {
        *slot = state;
    }
}

/// Browse: puts the `wanted` Assets that are still pending at the front of the generator's
/// queue, in place of those named before.
pub(crate) fn browse(world: &mut World, wanted: Vec<AssetAddress>) {
    let folders = folders_by_key(world);
    let mut jobs = Vec::new();
    for AssetAddress { folder: key, place } in wanted {
        let Some(&folder) = folders.get(&key) else {
            continue;
        };
        let Some(added) = world.get::<AssetFolder>(folder) else {
            continue;
        };
        let Some(index) = position(added, &place) else {
            continue;
        };
        let pending = world
            .get::<Thumbnails>(folder)
            .and_then(|thumbnails| thumbnails.states.get(index))
            .is_some_and(|state| *state == ThumbnailState::Pending);
        if pending {
            let asset = &added.assets[index];
            jobs.push(ThumbnailJob {
                key: key_of(&key, asset),
                file: added.path.join(&asset.place),
            });
        }
    }
    if let Some(generator) = world
        .get_resource::<ThumbnailWork>()
        .and_then(|work| work.generator.as_ref())
    {
        generator.want(jobs);
    }
}

/// Stops generating as soon as the editor is asked to quit, keeping what was finished.
pub(crate) fn stop_on_exit(
    world: &mut World,
    exits: &mut SystemState<MessageReader<bevy_app::AppExit>>,
) {
    let quitting = match exits.get_mut(world) {
        Ok(mut reader) => reader.read().count() > 0,
        Err(_) => return,
    };
    if quitting && let Some(mut work) = world.get_resource_mut::<ThumbnailWork>() {
        work.generator = None;
    }
}
