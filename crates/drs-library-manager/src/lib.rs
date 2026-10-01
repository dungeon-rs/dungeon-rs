#![doc = include_str!("../README.md")]

mod add_folder;
mod index;

pub use index::refresh;

use add_folder::add_folder;

use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageReader;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::system::SystemState;
use bevy_ecs::world::World;
use drs_library_access::LibraryError;
use drs_model::{
    AddFolder, AssetFolder, EditorDirectories, FolderRefused, FolderUnavailable, ManagerSystems,
};

/// What can go wrong inside the Manager, beyond a refusal.
#[derive(Debug, thiserror::Error)]
pub enum LibraryManagerError {
    /// Reading or writing the editor's files, or scanning a folder, failed.
    #[error(transparent)]
    Library(#[from] LibraryError),
    /// The entity is not an Asset Folder.
    #[error("the entity is not an Asset Folder")]
    NotAFolder,
}

/// Handles [`AddFolder`] and restores the remembered Asset Folders at startup.
pub struct LibraryManagerPlugin;

impl Plugin for LibraryManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, restore_folders)
            .add_systems(Update, handle_add_folder.in_set(ManagerSystems::Commands));
    }
}

/// Carries out every [`AddFolder`] request, answering each with [`drs_model::FolderAdded`] or
/// [`FolderRefused`].
fn handle_add_folder(world: &mut World, requests: &mut SystemState<MessageReader<AddFolder>>) {
    let requests: Vec<AddFolder> = match requests.get_mut(world) {
        Ok(mut reader) => reader.read().cloned().collect(),
        Err(_) => return,
    };
    for AddFolder { path, name } in requests {
        match add_folder(world, path.clone(), &name) {
            Ok(added) => {
                world.write_message(added);
            }
            Err(reason) => {
                world.write_message(FolderRefused { path, name, reason });
            }
        }
    }
}

/// Reads every Manifest and refreshes each remembered folder.
///
/// A folder that cannot be indexed stays known with no Assets and is reported as
/// [`FolderUnavailable`]; a file that is not a Manifest is logged and skipped.
fn restore_folders(world: &mut World) {
    let overrides = world
        .get_resource::<EditorDirectories>()
        .cloned()
        .unwrap_or_default();
    let directories = match drs_library_access::LibraryDirectories::resolve(&overrides) {
        Ok(directories) => directories,
        Err(error) => {
            log::warn!("no Asset Folders are remembered: {error}");
            return;
        }
    };
    let read = match drs_library_access::read_manifests(&directories) {
        Ok(read) => read,
        Err(error) => {
            log::warn!("the remembered Asset Folders cannot be listed: {error}");
            return;
        }
    };
    for error in &read.skipped {
        log::warn!("a file among the Manifests was skipped: {error}");
    }
    for manifest in read.manifests {
        let folder: Entity = world
            .spawn(AssetFolder {
                name: manifest.name.clone(),
                key: manifest.key.clone(),
                path: manifest.path.clone(),
                version: manifest.version.clone(),
                assets: Vec::new(),
                skips: drs_model::ScanSkips::default(),
            })
            .id();
        if let Err(error) = refresh(world, folder) {
            world.write_message(FolderUnavailable {
                folder,
                name: manifest.name,
                path: manifest.path,
                reason: error.to_string(),
            });
        }
    }
}
