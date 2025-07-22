use anyhow::Context;
use assets::AssetLibrary;
use clap::{Args, Subcommand};
use log::info;
use std::path::PathBuf;

/// Manage asset library and packs
#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct AssetsArgs {
    #[command(subcommand)]
    command: AssetsCommands,
}

/// All commands available for assets.
#[derive(Debug, Subcommand)]
pub enum AssetsCommands {
    /// List all asset packs.
    List {
        path: Option<PathBuf>,
    },
    CleanUp {
        path: Option<PathBuf>,
    },
}

/// Executes the asset commands in the correct way.
pub fn execute(AssetsArgs { command }: AssetsArgs) -> anyhow::Result<()> {
    match command {
        AssetsCommands::List { path } => {
            info!("Loading asset library");
            let library = AssetLibrary::load_or_default(path)?;
            for (name, path) in library.iter() {
                println!("{}: {}", name, path.display());
            }

            Ok(())
        }
        AssetsCommands::CleanUp { path } => {
            let library = AssetLibrary::load(path)?;

            library.delete().context("Failed to delete asset library")
        }
    }
}
