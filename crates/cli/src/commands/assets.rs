//! Implementation of the `assets` subcommand.

use anyhow::{Context, bail};
use assets::{AssetLibrary, AssetPackIndexCompletedEvent, AssetPackIndexProgressEvent};
use bevy::ecs::world::CommandQueue;
use bevy::prelude::{Events, World, debug, info};
use clap::Subcommand;
use logging::{MultiProgress, console_progress};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

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
        #[arg(long)]
        no_index: bool,

        /// If passed, thumbnail generation will be skipped.
        ///
        /// Ignored if `--no-index` if passed.
        #[arg(long)]
        no_thumbnail: bool,

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
        /// If omitted, all asset packs will be indexed.
        pack: Option<String>,

        /// If passed, thumbnails will be generated.
        #[arg(long)]
        thumbnails: bool,
    },
    /// Executes a query against the asset library.
    Query {
        /// The query to execute against the library, for the syntax see
        /// <https://docs.rs/tantivy/0.24.2/tantivy/query/struct.QueryParser.html>
        query: String,

        /// The (maximum) number of results to return for this query.
        #[arg(long, default_value_t = 100)]
        max_amount: usize,
    },
}

/// Executes the asset commands in the correct way.
///
/// # Errors
/// See the implementations of the command implementations for respective errors.
pub fn execute(
    Args { command, library }: Args,
    _world: &mut World,
    multi_progress: MultiProgress,
) -> anyhow::Result<()> {
    match command {
        Commands::List => execute_list(library),
        Commands::CleanUp => execute_cleanup(library),
        Commands::Add {
            path,
            name,
            no_index,
            no_thumbnail,
        } => execute_add(library, &path, name, no_index, no_thumbnail),
        Commands::Remove { id } => execute_remove(library, &id),
        Commands::Index { pack, thumbnails } => {
            execute_index(library, pack, multi_progress, thumbnails)
        }
        Commands::Query { query, max_amount } => execute_query(library, query, max_amount),
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
    no_thumbnail: bool,
) -> anyhow::Result<()> {
    debug!("Attempting to load asset library");
    let mut asset_library =
        AssetLibrary::load_or_default(library.clone()).context("Failed to load asset library")?;

    let added_pack = asset_library
        .add_pack(path, None, name.clone())
        .context("Failed to add asset pack to asset library")?;

    let (sender, _receiver) = utils::command_queue();
    if !no_index && let Some(pack) = asset_library.get_pack_mut(&added_pack) {
        pack.index(sender, !no_thumbnail)?;
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
fn execute_index(
    library: Option<PathBuf>,
    id: Option<String>,
    multi_progress: MultiProgress,
    generate_thumbnails: bool,
) -> anyhow::Result<()> {
    let mut asset_library = AssetLibrary::load(library).context("Failed to load asset library")?;
    if let Some(id) = id.clone() {
        asset_library
            .load_pack(&id)
            .with_context(|| format!("Failed to get pack with id '{id}'"))?;
    } else {
        asset_library
            .load_all()
            .context("Failed to load asset packs")?;
    }

    let mut progresses = HashMap::new();
    for (id, _) in asset_library.iter() {
        progresses.insert(id.clone(), console_progress(&multi_progress));
    }

    let (sender, receiver) = utils::command_queue();
    let thread = std::thread::spawn(move || {
        let mut world = World::default();
        world.init_resource::<Events<AssetPackIndexProgressEvent>>();
        world.init_resource::<Events<AssetPackIndexCompletedEvent>>();

        loop {
            let mut queue = match receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(queue) => queue,
                Err(error) => bail!("Timeout while waiting for index events {}", error),
            };

            queue.apply(&mut world);
            let mut progress_events = world
                .get_resource_mut::<Events<AssetPackIndexProgressEvent>>()
                .expect("Failed to get progress events");

            for event in progress_events.drain() {
                let progress = &progresses[&event.id];
                progress.set_length(event.total as u64);
                progress.set_position(event.current as u64);
            }

            queue.apply(&mut world);
            let mut completed_events = world
                .get_resource_mut::<Events<AssetPackIndexCompletedEvent>>()
                .expect("Failed to get completed events");
            for event in completed_events.drain() {
                progresses[&event.id].finish();

                return Ok(());
            }
        }
    });

    if let Some(id) = id {
        asset_library
            .get_pack_mut(&id)
            .with_context(|| format!("Failed to get pack with id '{id}'"))?
            .index(sender, generate_thumbnails)?;
    } else {
        asset_library
            .index(&sender, generate_thumbnails)
            .context("Failed to index asset packs")?;
    }

    thread
        .join()
        .expect("Failed to join progress reporting thread")
        .expect("Progress reporting thread failed");
    Ok(())
}

/// Executes a query on the library.
///
/// # Errors
/// Return an error when the asset library fails to load.
fn execute_query(library: Option<PathBuf>, query: String, max_amount: usize) -> anyhow::Result<()> {
    let mut asset_library = AssetLibrary::load(library).context("Failed to load asset library")?;
    asset_library.load_all()?;

    if max_amount == 0 {
        bail!("The maximum amount of entries returned cannot be 0");
    }

    let results = asset_library.search(query, max_amount)?;
    if results.is_empty() {
        info!("No results found");
        return Ok(());
    }

    info!("Found {amount} results:", amount = results.len());
    for result in results {
        info!("{}", result);
    }

    Ok(())
}
