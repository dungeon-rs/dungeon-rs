use crate::asset_pack::AssetPack;
use bevy::prelude::{Component, Resource};
use std::path::PathBuf;

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
