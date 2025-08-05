//! # `DungeonRS` scripting
//!
//! `DungeonRS` provides a way to index your assets packs using custom logic in case the default script doesn't properly
//! index the asset structure of your files.

use bevy::prelude::debug;
use rhai::{Array, CustomType, Engine, ImmutableString, TypeBuilder};

/// This builds a Rhai engine with all functions in this module registered so they are available to
/// the scripts.
pub fn build_engine() -> Engine {
    let mut engine = Engine::new();
    engine.on_print(|message| debug!(message));
    engine.build_type::<IndexEntry>();
    engine.register_fn("hash", hash);

    engine
}

/// Generates a hash from the given `value`.
///
/// This implementation currently uses Blake3, but callers should not rely on this.
fn hash(value: &mut ImmutableString) -> ImmutableString {
    utils::hash_string(value.as_str()).into()
}

/// Represents the result of a Rhai indexing script.
///
/// This type exposes Rhai specific types on purpose to minimise marshalling overhead.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    /// The human-readable name of this asset as it will appear in the asset browser.
    pub name: ImmutableString,

    /// A list of categories under which this asset can be classified.
    /// Categories are essentially alternative keywords by which this asset can be discovered.
    ///
    /// Examples would be "Aquatic", "Mountain", "Water", "Foam", ...
    pub categories: Array,

    /// An internal (unique) identifier that indicates the thumbnail.
    ///
    /// Note that this thumbnail file may or may not exist.
    pub thumbnail: ImmutableString,
}

impl CustomType for IndexEntry {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("IndexEntry")
            .with_fn("index_entry", |name, categories, thumbnail| IndexEntry {
                name,
                categories,
                thumbnail,
            });
    }
}
