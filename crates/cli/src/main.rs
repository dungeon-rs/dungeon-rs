#![doc = include_str!("../README.md")]

mod commands;

use crate::commands::Commands;
use assets::AssetPlugin;
use bevy::DefaultPlugins;
use bevy::prelude::{App, PluginGroup};
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use config::LogConfiguration;
use logging::log_plugin;
use utils::CorePlugin;

/// A command line interface with `DungeonRS`.
#[derive(Debug, Parser)]
#[command(name = "drs-cli")]
#[command(about, long_about = None)]
struct Cli {
    /// Verbosity of the command line.
    #[command(flatten)]
    verbosity: Verbosity<InfoLevel>,
    #[command(subcommand)]
    /// Commands comment
    command: Commands,
}

#[allow(missing_docs)]
#[allow(clippy::missing_panics_doc)]
#[allow(clippy::missing_errors_doc)]
#[allow(clippy::missing_docs_in_private_items)]
fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    // Some commands require a `World` entry, so we build an app that can provide said world.
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(log_plugin(&LogConfiguration {
        filter: String::from("tantivy=warn,bevy_app=warn"),
        output: None,
        level: args.verbosity.to_string(),
        write_file: false,
    })))
    .add_plugins(CorePlugin)
    .add_plugins(AssetPlugin);

    match args.command {
        Commands::Assets(args) => commands::assets::execute(args, app.world_mut())?,
    }

    Ok(())
}
