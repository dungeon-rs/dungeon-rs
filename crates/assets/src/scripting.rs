//! # DungeonRS scripting
//!
//! DungeonRS provides a way to index your assets packs using custom logic in case the default script doesn't properly
//! index the asset structure of your files.

use rhai::{CustomType, Engine, TypeBuilder};

/// This builds a Rhai engine with all functions in this module registered so they are available to
/// the scripts.
pub fn build_engine() -> Engine {
    let mut engine = Engine::new();
    engine.build_type::<IndexEntry>();
    engine.register_fn("hash", hash);

    engine
}

/// Generates a hash from the given `value`.
///
/// This implementation currently uses Blake3, but callers should not rely on this.
fn hash(value: String) -> String {
    blake3::hash(value.as_bytes()).to_string()
}

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub name: String,
}

impl CustomType for IndexEntry {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("IndexEntry")
            .with_fn("index_entry", |name| IndexEntry { name });
    }
}
