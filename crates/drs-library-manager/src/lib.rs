#![doc = include_str!("../README.md")]

mod add_folder;
mod index;
mod thumbnails;

pub use index::refresh;

use add_folder::add_folder;

use bevy_app::{App, Last, Plugin, Startup, Update};
use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageReader;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::system::SystemState;
use bevy_ecs::world::World;
use drs_library_access::LibraryError;
use drs_model::{
    AddFolder, AssetFolder, AssetFolderChanged, Browse, EditorDirectories, FolderRefused,
    FolderUnavailable, ManagerSystems,
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

/// Handles [`AddFolder`] and [`Browse`], restores the remembered Asset Folders at
/// startup, announcing each folder that arrives or goes with [`AssetFolderChanged`], and keeps
/// the thumbnails of every indexed folder generated in the background.
pub struct LibraryManagerPlugin;

impl Plugin for LibraryManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (thumbnails::open, restore_folders).chain())
            .add_systems(
                Update,
                (thumbnails::drain, handle_browse, handle_add_folder)
                    .chain()
                    .in_set(ManagerSystems::Commands),
            )
            .add_systems(Last, thumbnails::stop_on_exit);
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

/// Carries out the latest [`Browse`] request of the frame; an earlier one names a set of Assets
/// the browser no longer shows, so it is read and passed over. Nothing comes back.
fn handle_browse(world: &mut World, requests: &mut SystemState<MessageReader<Browse>>) {
    let mut requests: Vec<Browse> = match requests.get_mut(world) {
        Ok(mut reader) => reader.read().cloned().collect(),
        Err(_) => return,
    };
    if let Some(Browse { wanted }) = requests.pop() {
        thumbnails::browse(world, wanted);
    }
}

/// Reads every Manifest and refreshes each remembered folder, announcing each with
/// [`AssetFolderChanged`].
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
                name: manifest.name.clone(),
                path: manifest.path,
                reason: error.to_string(),
            });
        }
        world.write_message(AssetFolderChanged {
            name: manifest.name,
        });
    }
}
