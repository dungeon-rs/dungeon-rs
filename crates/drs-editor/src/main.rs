#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "no_console", windows_subsystem = "windows")]

mod panic;

use clap::Parser;
use drs_assets::AssetPlugin;
use std::path::PathBuf;

use bevy::prelude::*;
use drs_config::Configuration;
use drs_i18n::I18nPlugin;
use drs_io::IOPlugin;
use logging::log_plugin;
use ui::UIPlugin;
use utils::CorePlugin;

/// Arguments for running the editor.
#[derive(Debug, Parser)]
struct Args {
    /// Optionally, specify a configuration file to use.
    #[clap(short, long)]
    config_file: Option<PathBuf>,
}

/// Main entry point for the editor.
///
/// # Panics
/// The application will panic when a configuration error occurs, or when Bevy panics, specific
/// circumstances for when Bevy panics can be found in Bevy's documentation.
fn main() -> AppExit {
    panic::register_panic_handler();

    let args = Args::parse();
    let config = match Configuration::load(args.config_file) {
        Ok(cfg) => cfg,
        Err(err) => panic!("Failed to load configuration: {err:?}"),
    };

    let resource_path = utils::resource_path().expect("Failed to get resource path");
    let plugin_builder = DefaultPlugins
        .build()
        .add(CorePlugin)
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
