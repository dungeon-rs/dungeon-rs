//! Answering the browser's search: the search of every Asset Folder, each built whenever its
//! index is written and dropped with the folder, and the text last sent answered over them.

use bevy_ecs::entity::Entity;
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;
use drs_catalog_engine::LibrarySearch;
use drs_model::{AssetFolder, AssetMatch, FolderKey, IndexedAsset, SearchMatches};
use std::collections::BTreeMap;

/// The text the browser last sent and the search it is answered over. The search is never
/// written to disk: it is built again at every start, so it can never disagree with the index.
#[derive(Resource, Default)]
pub(crate) struct Searching {
    /// The text last sent.
    text: String,
    /// Whether the text changed since it was last answered.
    asked: bool,
    /// Whether a folder's search was built or dropped since the text was last answered.
    changed: bool,
    /// The search of every added folder.
    library: LibrarySearch,
}

/// Builds the search of the Asset Folder on `folder` from its index as it is now, in place of
/// the one it had.
pub(crate) fn build(world: &mut World, folder: Entity) {
    let mut searching = world.remove_resource::<Searching>().unwrap_or_default();
    if let Some(added) = world.get::<AssetFolder>(folder) {
        searching
            .library
            .build(&added.key, &added.name, &added.assets);
        searching.changed = true;
    }
    world.insert_resource(searching);
}

/// Drops the search of the Asset Folder with `key`, which is no longer added.
pub(crate) fn remove(world: &mut World, key: &FolderKey) {
    let mut searching = world.get_resource_or_init::<Searching>();
    searching.library.remove(key);
    searching.changed = true;
}

/// Takes the browser's search text, to be answered at the end of the frame if it changed.
pub(crate) fn ask(world: &mut World, text: String) {
    let mut searching = world.get_resource_or_init::<Searching>();
    if searching.text != text {
        searching.text = text;
        searching.asked = true;
    }
}

/// Answers the text last sent into [`SearchMatches`] whenever it changed or a folder's search
/// was built or dropped in this frame.
pub(crate) fn answer(world: &mut World) {
    let Some(mut searching) = world.remove_resource::<Searching>() else {
        return;
    };
    if !searching.asked && !searching.changed {
        world.insert_resource(searching);
        return;
    }
    let mut query = world.query::<(Entity, &AssetFolder)>();
    let folders: BTreeMap<&FolderKey, (Entity, &[IndexedAsset])> = query
        .iter(world)
        .map(|(entity, folder)| (&folder.key, (entity, folder.assets.as_slice())))
        .collect();
    let text = searching.text.clone();
    let found = searching
        .library
        .search(&text, |key| folders.get(key).map(|(_, assets)| *assets));
    let entities: Vec<Option<Entity>> = found
        .folders
        .iter()
        .map(|(key, _)| folders.get(key).map(|(entity, _)| *entity))
        .collect();
    let counts: Vec<(Entity, usize)> = entities
        .iter()
        .zip(&found.folders)
        .filter_map(|(entity, (_, count))| entity.map(|entity| (entity, *count)))
        .collect();
    let assets: Vec<AssetMatch> = found
        .assets
        .iter()
        .filter_map(|found| {
            let folder = (*entities.get(found.folder as usize)?)?;
            Some(AssetMatch {
                folder,
                position: found.position,
            })
        })
        .collect();
    let matches = SearchMatches {
        text,
        has_words: found.has_words,
        total: if found.has_words {
            assets.len()
        } else {
            counts.iter().map(|(_, count)| count).sum()
        },
        counts,
        assets,
    };
    searching.asked = false;
    searching.changed = false;
    world.insert_resource(matches);
    world.insert_resource(searching);
}
