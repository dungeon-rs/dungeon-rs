//! What the browser's search text matches, as the library Manager answers it.

use bevy_ecs::entity::Entity;
use bevy_ecs::resource::Resource;

/// One Asset that matches the browser's search text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssetMatch {
    /// The entity of the Asset Folder it sits in.
    pub folder: Entity,
    /// Its position in that folder's index of Assets.
    pub position: usize,
}

/// The answer to the search text the browser last sent with [`crate::Browse`]: what matches it
/// over every added Asset Folder, kept current as folders are added, undone, redone, and
/// refreshed. Written only by the library Manager.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchMatches {
    /// The text answered, as it was sent.
    pub text: String,
    /// How many Assets match: with a text of no words, every Asset of every folder.
    pub total: usize,
    /// How many Assets of each added folder match, by folder entity, in no particular order.
    pub counts: Vec<(Entity, usize)>,
    /// The Assets that match, in order: those in which every word begins a word of the name
    /// first, then the others, each by name, then by Canonical Name, then by place.
    pub assets: Vec<AssetMatch>,
}

impl SearchMatches {
    /// How many Assets of the folder `folder` match; none for a folder it does not know.
    #[must_use]
    pub fn count(&self, folder: Entity) -> usize {
        self.counts
            .iter()
            .find_map(|(counted, count)| (*counted == folder).then_some(*count))
            .unwrap_or_default()
    }
}
