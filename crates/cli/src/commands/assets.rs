//! Implementation of the `assets` subcommand.

use anyhow::Context;
use assets::AssetLibrary;
use clap::Subcommand;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Manage asset library and packs
#[derive(Debug, clap::Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct Args {
    /// The library configuration file to use.
    /// If left empty, it uses the default asset library.
    #[arg(long, short, global = true)]
    library: Option<PathBuf>,
    /// Asset command.
    #[command(subcommand)]
    command: Commands,
}

/// All commands available for assets.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Add an asset pack to the asset library.
    Add {
        /// If passed, the asset pack will not be indexed during addition.
        #[arg(short, long)]
        no_index: bool,
        /// The directory to add as an asset pack.
        path: PathBuf,
        /// Optional name for the asset pack to add.
        name: Option<String>,
    },
    /// Removes an asset pack from the library.
    Remove {
        /// The ID of the asset pack to remove.
        id: String,
    },
    /// List all asset packs.
    List,
    /// Cleans up the given asset library.
    CleanUp,
    /// Forces a re-index of the given asset pack.
    Index {
        /// The ID of the asset pack to index.
        pack: String,
    },
}

/// Executes the asset commands in the correct way.
///
/// # Errors
/// See the implementations of the command implementations for respective errors.
pub fn execute(Args { command, library }: Args) -> anyhow::Result<()> {
    match command {
        Commands::List => execute_list(library),
        Commands::CleanUp => execute_cleanup(library),
        Commands::Add {
            path,
            name,
            no_index,
        } => execute_add(library, &path, name, no_index),
        Commands::Remove { id } => execute_remove(library, &id),
        Commands::Index { pack } => execute_index(library, &pack),
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
        info!("{name}: {path}", name = name, path = path.display());
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
fn execute_add(
    library: Option<PathBuf>,
    path: &Path,
    name: Option<String>,
    no_index: bool,
) -> anyhow::Result<()> {
    debug!("Attempting to load asset library");
    let mut asset_library =
        AssetLibrary::load_or_default(library.clone()).context("Failed to load asset library")?;

    let added_pack = asset_library
        .add_pack(path, name.clone())
        .context("Failed to add asset pack to asset library")?;

    if !no_index && let Some(pack) = asset_library.get_pack_mut(&added_pack) {
        pack.index()?;
    }

    debug!("Attempting to save asset library");
    asset_library
        .save(library)
        .context("Failed to save asset library")?;

    let name = name.unwrap_or_else(|| path.to_string_lossy().to_string());
    info!("Added {name} as '{added_pack}' to library");
    Ok(())
}

/// Remove an existing asset pack from the library.
///
/// # Errors
/// Return an error when the asset library fails to load.
fn execute_remove(library: Option<PathBuf>, id: &String) -> anyhow::Result<()> {
    let mut asset_library = AssetLibrary::load(library).context("Failed to load asset library")?;
    asset_library
        .delete_pack(id)
        .context("Failed to delete asset pack")?;

    Ok(())
}

/// Re-indexes an asset pack from the library.
///
/// # Errors
/// Return an error when the asset library fails to load.
fn execute_index(library: Option<PathBuf>, id: &String) -> anyhow::Result<()> {
    let mut asset_library = AssetLibrary::load(library).context("Failed to load asset library")?;
    asset_library
        .load_pack(id)
        .with_context(|| format!("Failed to get pack with id '{}'", id))?;

    asset_library
        .get_pack_mut(id)
        .with_context(|| format!("Failed to get pack with id '{}'", id))?
        .index()?;

    Ok(())
}
