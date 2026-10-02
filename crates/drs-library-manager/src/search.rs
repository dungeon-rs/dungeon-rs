//! Answering the browser's search: one search per Asset Folder, kept on the folder's entity and
//! built whenever its index is written, and the text last sent answered over all of them.

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;
use drs_catalog_engine::{FolderSearch, SearchOrder, search};
use drs_model::{AssetFolder, AssetMatch, SearchMatches};

/// The search of the Asset Folder on the same entity, built from its index. It is never written
/// to disk: it is built again at every start, so it can never disagree with the index.
#[derive(Component)]
pub(crate) struct Searchable(FolderSearch);

/// The text the browser last sent and what answering it needs.
#[derive(Resource, Default)]
pub(crate) struct Searching {
    /// The text last sent.
    text: String,
    /// Whether the text changed since it was last answered.
    asked: bool,
    /// Whether a folder's search was built or dropped since the text was last answered.
    changed: bool,
    /// The folders whose searches `order` orders, in the order it was worked out over.
    folders: Vec<Entity>,
    /// The order of every folder's Assets taken together.
    order: SearchOrder,
}

/// Builds the search of the Asset Folder on `folder` from its index as it is now, in place of
/// the one it had.
pub(crate) fn build(world: &mut World, folder: Entity) {
    let Some(added) = world.get::<AssetFolder>(folder) else {
        return;
    };
    let built = FolderSearch::build(&added.name, &added.assets);
    world.entity_mut(folder).insert(Searchable(built));
    dropped(world);
}

/// Notes that a folder's search was dropped with its entity, so the text is answered again.
pub(crate) fn dropped(world: &mut World) {
    world.get_resource_or_init::<Searching>().changed = true;
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
    let mut query = world.query::<(Entity, &Searchable)>();
    let mut folders: Vec<(Entity, &FolderSearch)> = query
        .iter(world)
        .map(|(folder, Searchable(built))| (folder, built))
        .collect();
    folders.sort_by_key(|(folder, _)| *folder);
    let entities: Vec<Entity> = folders.iter().map(|(folder, _)| *folder).collect();
    let searches: Vec<&FolderSearch> = folders.iter().map(|(_, built)| *built).collect();
    if searching.changed || searching.folders != entities {
        searching.order = SearchOrder::of(&searches);
        searching.folders.clone_from(&entities);
    }
    let found = search(&searching.text, &searches, &searching.order);
    let matches = SearchMatches {
        text: searching.text.clone(),
        total: found.assets.len(),
        counts: entities.iter().copied().zip(found.counts).collect(),
        assets: found
            .assets
            .iter()
            .map(|found| AssetMatch {
                folder: entities[found.folder],
                position: found.position,
            })
            .collect(),
    };
    searching.asked = false;
    searching.changed = false;
    world.insert_resource(matches);
    world.insert_resource(searching);
}
