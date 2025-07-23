//! Implementation of the `assets` subcommand.

use anyhow::Context;
use assets::AssetLibrary;
use clap::Subcommand;
use log::{debug, info};
use std::path::{Path, PathBuf};

/// Manage asset library and packs
#[derive(Debug, clap::Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct Args {
    /// Asset command.
    #[command(subcommand)]
    command: Commands,
}

/// All commands available for assets.
#[derive(Debug, Subcommand)]
pub enum Commands {
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
///
/// # Errors
/// See the implementations of the command implementations for respective errors.
pub fn execute(Args { command }: Args) -> anyhow::Result<()> {
    match command {
        Commands::List { path } => execute_list(path),
        Commands::CleanUp { path } => execute_cleanup(path),
        Commands::Add { library, path } => execute_add(library, &path),
    }
}

/// Lists all asset packs in the given asset library.
///
/// # Errors
/// Return an error when the asset library fails to load.
fn execute_list(path: Option<PathBuf>) -> anyhow::Result<()> {
    debug!("Attempting to load asset library");
    let library = AssetLibrary::load_or_default(path).context("Failed to load asset library")?;
    for (name, path) in library.iter() {
        println!("{}: {}", name, path.display());
    }

    Ok(())
}

/// Cleans up the given asset library.
///
/// # Errors
/// Return an error when the asset library fails to load.
fn execute_cleanup(path: Option<PathBuf>) -> anyhow::Result<()> {
    debug!("Attempting to load asset library");
    let library = AssetLibrary::load(path).context("Failed to load asset library")?;

    library.delete().context("Failed to delete asset library")
}

/// Add a new asset pack to the library.
///
/// # Errors
/// Return an error when the asset library fails to load.
fn execute_add(library: Option<PathBuf>, path: &Path) -> anyhow::Result<()> {
    debug!("Attempting to load asset library");
    let mut asset_library =
        AssetLibrary::load_or_default(library.clone()).context("Failed to load asset library")?;

    let added_pack = asset_library
        .add_pack(path, None)
        .context("Failed to add asset pack to asset library")?;

    debug!("Attempting to save asset library");
    asset_library
        .save(library)
        .context("Failed to save asset library")?;

    info!("Added {} as '{}' to library", path.display(), added_pack);
    Ok(())
}
