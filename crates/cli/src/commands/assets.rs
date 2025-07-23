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
    /// Asset command.
    #[command(subcommand)]
    command: AssetsCommands,
}

/// All commands available for assets.
#[derive(Debug, Subcommand)]
pub enum AssetsCommands {
    /// Add an asset pack to the asset library.
    Add {
        /// The library configuration file to add this asset pack to.
        /// If left empty, it uses the default asset library.
        #[arg(short, long)]
        library: Option<PathBuf>,
        /// The directory to add as an asset pack.
        path: PathBuf,
    },
    /// List all asset packs.
    List {
        /// An optional path to the asset library configuration.
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Cleans up the given asset library.
    CleanUp {
        /// An optional path to the asset library configuration.
        #[arg(short, long)]
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
        AssetsCommands::Add { library, path } => {
            let mut asset_library = AssetLibrary::load_or_default(library.clone())?;
            let added_pack = asset_library.add_pack(path.as_path(), None)?;
            asset_library.save(library)?;

            info!("Added {} as '{}' to library", path.display(), added_pack);
            Ok(())
        }
    }
}
