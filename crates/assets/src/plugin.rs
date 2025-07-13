//! The [`AssetPlugin`] is responsible for loading the required information into Bevy.

use crate::library::AssetLibrary;
use crate::reader::DrsAssetReader;
use bevy::app::App;
use bevy::asset::io::AssetSourceBuilder;
use bevy::prelude::{AssetApp, Plugin};

/// Handles registering the required resources and functionality for the asset system.
#[derive(Default)]
pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        let library = AssetLibrary::load_or_default(None).expect("Failed to load asset library");
        app.insert_resource(library);

        app.register_asset_source(
            "drs",
            AssetSourceBuilder::default().with_reader(|| Box::new(DrsAssetReader)),
        );
    }
}
