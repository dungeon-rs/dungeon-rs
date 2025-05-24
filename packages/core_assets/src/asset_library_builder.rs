use crate::asset_pack::AssetPack;
use bevy::prelude::{BevyError, Component, Resource};
use std::path::PathBuf;
use crate::asset_library::AssetLibrary;

/// Resource that can be used to construct a new [`AssetLibrary`].
#[derive(Resource, Component, Debug)]
pub struct AssetLibraryBuilder {
    pub name: String,
    pub root: PathBuf,
    pub packs: Vec<AssetPack>,
}

impl Default for AssetLibraryBuilder {
    fn default() -> Self {
        Self {
            name: String::new(),
            root: PathBuf::new(),
            packs: Vec::new(),
        }
    }
}

impl AssetLibraryBuilder {
    pub fn build(self) -> Result<AssetLibrary, BevyError> {
        if self.name.is_empty() {
            return Err(BevyError::from("AssetLibrary name cannot be empty."));
        }
        if self.packs.is_empty() {
            return Err(BevyError::from("AssetLibrary packs cannot be empty."));
        }

        AssetLibrary::create(self.name, self.root)
    }
}
