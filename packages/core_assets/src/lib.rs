#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::suspicious, clippy::complexity)]

mod asset_library;
mod asset_library_builder;
mod asset_pack;

pub use crate::asset_library_builder::AssetLibraryBuilder;
pub use crate::asset_pack::AssetPack;
use bevy::prelude::{App, Plugin};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, _app: &mut App) {
        //
    }
}

// fn setup(mut commands: Commands) -> Result {
// let mut library = AssetLibrary::create("default".into(), ".library")?;
// library.add_pack(AssetPack::new(String::from("asset pack 1"), "/some/path"))?;
// let library = AssetLibrary::open(".library")?;

// commands.insert_resource(library);
// Ok(())
// }
