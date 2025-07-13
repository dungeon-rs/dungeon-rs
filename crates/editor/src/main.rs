#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "no_console", windows_subsystem = "windows")]

mod panic;

use assets::AssetPlugin;

use bevy::prelude::*;
use config::Configuration;
use i18n::I18nPlugin;
use io::IOPlugin;
use logging::log_plugin;
use ui::UIPlugin;

/// Main entry point for the editor.
///
/// # Panics
/// The application will panic when a configuration error occurs, or when Bevy panics, specific
/// circumstances for when Bevy panics can be found in Bevy's documentation.
fn main() -> AppExit {
    panic::register_panic_handler();
    let config = match Configuration::load() {
        Ok(cfg) => cfg,
        Err(err) => panic!("Failed to load configuration: {err:?}"),
    };

    let resource_path = utils::resource_path().expect("Failed to get resource path");
    let plugin_builder = DefaultPlugins
        .build()
        .add(I18nPlugin::new(&config.language))
        .add(IOPlugin)
        .add(UIPlugin)
        .add_before::<bevy::prelude::AssetPlugin>(AssetPlugin)
        .set(log_plugin(&config.logging))
        .set(bevy::asset::AssetPlugin {
            file_path: utils::to_string(&resource_path.join("assets")),
            ..default()
        });

    App::new()
        .add_plugins(plugin_builder)
        .insert_resource(config)
        .run()
}
