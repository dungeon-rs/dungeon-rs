//! A library serves as a device-wide registry of asset packs.

use crate::{AssetPack, AssetPackError};
use bevy::ecs::world::CommandQueue;
use bevy::prelude::{Resource, debug, debug_span, info, info_span, trace, trace_span};
use drs_serialization::{Deserialize, SerializationFormat, Serialize, deserialize, serialize_to};
use semver::Version;
use std::collections::HashMap;
use std::fs::{File, create_dir_all};
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;
use utils::{DirectoryError, Sender, cache_path, config_path};

/// The name of the library configuration file.
const LIBRARY_FILE_NAME: &str = "library.toml";

/// An [`AssetLibrary`] is a device-wide registry of packs that save files can refer to.
/// It handles as the bridge between relative paths within an asset pack and the actual paths on
/// a user's device.
#[derive(Resource, Default, Debug)]
pub struct AssetLibrary {
    /// A map of asset packs, keyed by their (public) identifiers.
    registered_packs: HashMap<String, AssetLibraryEntry>,
    /// A map of currently loaded asset packs, keyed by their (public) identifiers.
    ///
    /// Given that this map is only persisted for the runtime of the application,
    /// it's possible that an [`AssetPack`] is 'known' but not loaded.
    loaded_packs: HashMap<String, AssetPack>,
}

/// Represents an entry in the library, containing additional metadata about an asset pack.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct AssetLibraryEntry {
    /// Location of the asset pack on disk.
    ///
    /// Currently only filesystem packs are supported (e.g., no network-backed protocols like HTTP, FTP, ...)
    root: PathBuf,
    /// The location of the asset pack's cache.
    /// The asset pack itself determines what is stored here; this includes the index.
    cache: PathBuf,
}

/// Internal "copy" of the [`AssetLibrary`] struct intended for saving/loading to disk.
#[derive(Debug, Serialize, Deserialize)]
struct _AssetLibrary {
    /// The version of the software that last touched the library, used to help with future migrations.
    version: Version,

    /// All packs that have been registered in the [`AssetLibrary`].
    packs: HashMap<String, AssetLibraryEntry>,
}

/// The errors that can occur when loading or saving the [`AssetLibrary`].
#[derive(Error, Debug)]
pub enum AssetLibraryError {
    /// An error occurred while locating the library configuration folder.
    #[error("failed to locate library configuration")]
    LocateConfigFolder(#[from] DirectoryError),
    /// An error occurred reading the configuration file itself.
    #[error("failed to read library configuration")]
    ReadFile(#[from] std::io::Error),
    /// An error occurred while (de)serialising the library configuration.
    #[error("failed to (de)serialize library configuration")]
    Serialisation(#[from] drs_serialization::SerializationError),
    /// Wrapper for the [`AssetPackError`].
    #[error(transparent)]
    OpenAssetPack(#[from] AssetPackError),
    /// The requested asset pack was not loaded or registered, depending on the operation.
    #[error("Could not resolve AssetPack with ID '{0}'")]
    NotFound(String),
}

impl AssetLibrary {
    /// Attempts to [`AssetLibrary::load`] the library from `path`, and returns the default value on [`AssetLibraryError::ReadFile`].
    ///
    /// Any other error will be propagated by this method.
    ///
    /// # Errors
    /// See [`AssetLibrary::load`].
    pub fn load_or_default(path: Option<PathBuf>) -> Result<Self, AssetLibraryError> {
        match Self::load(path) {
            Err(AssetLibraryError::ReadFile(_)) => {
                debug!("Failed to load library, using default");
                Ok(Self::default())
            }
            result => result,
        }
    }

    /// Attempts to load the asset library from `path`, where `path` is the configuration directory.
    /// If `None` is passed, the [`utils::config_path`] method is used instead.
    ///
    /// # Errors
    /// An error can be returned for the following situations:
    /// - The configuration folder could not be retrieved: [`AssetLibraryError::LocateConfigFolder`]
    /// - An error occurs while trying to read the config file (doesn't exist, permissions, ...):
    ///   [`AssetLibraryError::LocateConfigFolder`]
    /// - The file was found, could be read but failed to deserialize: [`AssetLibraryError::Serialisation`].
    pub fn load(path: Option<PathBuf>) -> Result<Self, AssetLibraryError> {
        let path = Self::get_path(path)?.join(LIBRARY_FILE_NAME);

        debug!("Attempting to load library at {}", path.display());
        let mut file = File::open(path).map_err(AssetLibraryError::ReadFile)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(AssetLibraryError::ReadFile)?;

        let library: _AssetLibrary = deserialize(contents.as_bytes(), &SerializationFormat::Toml)
            .map_err(AssetLibraryError::Serialisation)?;

        // TODO: validate `library`'s version for compatibility.

        Ok(Self {
            registered_packs: library.packs,
            loaded_packs: HashMap::new(),
        })
    }

    /// Recursively removes all cache and config of all packs and the library itself.
    /// To clean up a specific [`AssetPack`], use [`AssetLibrary::delete_pack`].
    ///
    /// # Errors
    /// See errors returned by [`AssetLibrary::delete_pack`].
    pub fn delete(mut self) -> Result<(), AssetLibraryError> {
        let _ = info_span!("delete_library").entered();
        info!("Running delete on asset library");
        let ids = self.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>();
        for id in ids {
            self.delete_pack(&id)?;
        }

        let path = Self::get_path(None)?.join(LIBRARY_FILE_NAME);
        let _ = std::fs::remove_file(path);
        Ok(())
    }

    /// Attempts to delete cache and config files of a given [`AssetPack`] and then unregisters it
    /// from the library.
    ///
    /// # Errors
    /// If any IO-related errors occur while removing the pack, this method can return a
    /// [`AssetLibraryError::OpenAssetPack`].
    pub fn delete_pack(&mut self, id: &String) -> Result<(), AssetLibraryError> {
        let _ = debug_span!("delete_pack", ?id).entered();
        if !self.is_pack_loaded(id) {
            self.load_pack(id)?;
        }

        if let Some(pack) = self.loaded_packs.remove(id) {
            pack.delete()?;

            self.loaded_packs.remove(id);
            self.registered_packs.remove(id);
        }

        Ok(())
    }

    /// Saves the asset library and all loaded asset packs.
    ///
    /// # Errors
    /// An error can be returned for the following situations:
    /// - The configuration folder could not be retrieved: [`AssetLibraryError::LocateConfigFolder`]
    /// - An error occurs while trying to read the config file (doesn't exist, permissions, ...):
    ///   [`AssetLibraryError::LocateConfigFolder`]
    /// - The file was found, could be read but failed to deserialize: [`AssetLibraryError::Serialisation`].
    pub fn save(&self, path: Option<PathBuf>) -> Result<(), AssetLibraryError> {
        let _ = debug_span!("saving-library").entered();
        let path = Self::get_path(path)?;

        for (id, pack) in &self.loaded_packs {
            trace!("Saving AssetPack {id}");

            pack.save_manifest()?;
        }

        debug!("Saving library to {}", path.display());
        create_dir_all(&path)?; // Ensure the directory exists.
        let file =
            File::create(path.join(LIBRARY_FILE_NAME)).map_err(AssetLibraryError::ReadFile)?;
        let library: _AssetLibrary = self.into();
        serialize_to(&library, &SerializationFormat::Toml, &file)?;
        Ok(())
    }

    /// Registers a new [`AssetPack`] in the library and returns the [`AssetPack`]'s ID.
    /// If `meta_dir` is passed as `None`, the cache directory will be used (namespaced to the pack).
    /// You usually won't pass a specific path for `meta_dir`, but it's there for debugging and tests.
    ///
    /// Note that you should only use this to create a *new* pack, not to load an existing one.
    ///
    /// # Errors
    /// - The configuration folder could not be retrieved: [`AssetLibraryError::LocateConfigFolder`]
    /// - An error occurs while trying to read the config file (doesn't exist, permissions, ...):
    ///   [`AssetLibraryError::LocateConfigFolder`]
    /// - The file was found, could be read but failed to deserialize: [`AssetLibraryError::Serialisation`].
    pub fn add_pack(
        &mut self,
        root: &Path,
        meta_dir: Option<PathBuf>,
        name: Option<String>,
    ) -> Result<String, AssetLibraryError> {
        let _ = info_span!("add_pack", ?name).entered();

        let id = utils::hash_path(root);
        let meta_dir = meta_dir.unwrap_or(cache_path()?.join(id.clone()));

        let pack = AssetPack::new(id.clone(), root, meta_dir.as_path(), name)?;
        let entry = AssetLibraryEntry {
            root: root.to_path_buf(),
            cache: pack.meta_dir.clone(),
        };

        self.registered_packs.insert(id.clone(), entry);
        self.loaded_packs.insert(id.clone(), pack);

        info!("Registered pack {}", id);
        Ok(id)
    }

    /// Attempt to load a previously registered [`AssetPack`].
    ///
    /// # Errors
    /// - If the asset pack isn't previously registered using [`AssetLibrary::add_pack`] this method
    ///   returns [`AssetLibraryError::NotFound`].
    /// - If an error occurs while loading the [`AssetPack`], it returns [`AssetLibraryError::OpenAssetPack`].
    pub fn load_pack(&mut self, id: &String) -> Result<&mut AssetPack, AssetLibraryError> {
        let Some(entry) = self.registered_packs.get(id) else {
            return Err(AssetLibraryError::NotFound(id.clone()));
        };

        let pack = AssetPack::load_manifest(entry.root.as_path(), entry.cache.as_path())?;
        self.loaded_packs.insert(id.clone(), pack);

        debug!("Loaded pack {}", id);
        self.loaded_packs
            .get_mut(id)
            .ok_or(AssetLibraryError::NotFound(id.clone()))
    }

    /// Attempts to load all packs that are currently not loaded.
    /// This loops over all registered packs and calls [`AssetLibrary::load_pack`] on each of them.
    ///
    /// # Errors
    /// This method propagates the errors returned from [`AssetLibrary::load_pack`].
    pub fn load_all(&mut self) -> Result<(), AssetLibraryError> {
        // We take an owned clone of the keys, otherwise we run into a double borrow later on.
        let keys = self.registered_packs.keys().cloned().collect::<Vec<_>>();

        for id in keys {
            if !self.is_pack_loaded(&id) {
                self.load_pack(&id)?;
            }
        }

        Ok(())
    }

    /// Adds a method to check if a given [`AssetPack`] is already loaded in the current library.
    #[inline]
    #[must_use]
    pub fn is_pack_loaded(&self, id: &String) -> bool {
        self.loaded_packs.contains_key(id)
    }

    /// Checks whether an asset pack is "known" (registered) within the current asset library.
    ///
    /// You can register new asset packs using [`AssetLibrary::add_pack`].
    #[inline]
    #[must_use]
    pub fn is_pack_registered(&self, id: &String) -> bool {
        self.registered_packs.contains_key(id)
    }

    /// Attempts to retrieve an previously loaded [`AssetPack`] from the current library.
    ///
    /// If `None` is returned the pack either doesn't exist or hasn't been loaded yet (see [`AssetLibrary::load_pack`]).
    #[inline]
    #[must_use]
    pub fn get_pack(&self, id: &String) -> Option<&AssetPack> {
        self.loaded_packs.get(id)
    }

    /// Attempts to retrieve an previously loaded [`AssetPack`] from the current library.
    ///
    /// If `None` is returned the pack either doesn't exist or hasn't been loaded yet (see [`AssetLibrary::load_pack`]).
    #[inline]
    #[must_use]
    pub fn get_pack_mut(&mut self, id: &String) -> Option<&mut AssetPack> {
        self.loaded_packs.get_mut(id)
    }

    /// Iterator over all registered packs.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&String, &PathBuf)> {
        self.registered_packs
            .iter()
            .map(|(key, value)| (key, &value.root))
    }

    /// Attempts to resolve a given `id` from all currently loaded asset packs.
    ///
    /// If no asset packs are loaded, or the ID is not known to a loaded pack, this method returns `None`.
    #[must_use]
    pub fn resolve(&self, id: impl AsRef<str>) -> Option<PathBuf> {
        for pack in self.loaded_packs.values() {
            if let Some(result) = pack.resolve(id.as_ref()) {
                return Some(result);
            }
        }

        None
    }

    /// Attempts to execute `query` for all currently loaded `AssetPack`s.
    ///
    /// If no asset packs are loaded, or the query does not resolve to any results, this method returns
    /// an empty list.
    ///
    /// # Errors
    /// for the errors that can be returned for this method, see [`AssetPack::search`].
    pub fn search(
        &self,
        query: impl AsRef<str>,
        max_amount: usize,
    ) -> Result<Vec<crate::packs::AssetPackSearchResult>, crate::packs::AssetPackSearchError> {
        let mut results = vec![];
        let query = query.as_ref();

        let _ = trace_span!("search", query = query).entered();
        for asset_pack in self.loaded_packs.values() {
            let mut result = asset_pack.search(query, max_amount)?;
            results.append(&mut result);
        }

        Ok(results)
    }

    /// Will index all currently loaded asset packs.
    ///
    /// # Errors
    /// This method will forward any errors thrown by [`AssetPack::index`].
    pub fn index(
        &self,
        sender: &Sender<CommandQueue>,
        generate_thumbnails: bool,
    ) -> Result<(), AssetLibraryError> {
        let _ = info_span!("index_library").entered();

        for pack in self.loaded_packs.values() {
            info!("Starting indexation of {}", pack.id);

            pack.index(sender, generate_thumbnails)?;
        }

        Ok(())
    }

    /// Either returns `path` or `config_path()` if `path` is `None`.
    ///
    /// # Errors
    /// Returns an error if the configuration folder cannot be found.
    fn get_path(path: Option<PathBuf>) -> Result<PathBuf, AssetLibraryError> {
        let path = if let Some(path) = path {
            path
        } else {
            config_path().map_err(AssetLibraryError::LocateConfigFolder)?
        };

        Ok(path)
    }
}

impl From<&AssetLibrary> for _AssetLibrary {
    fn from(value: &AssetLibrary) -> Self {
        Self {
            version: utils::version().clone(),
            packs: value.registered_packs.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc)]
    #![allow(clippy::missing_errors_doc)]

    use super::*;
    use tempfile::TempDir;

    #[test]
    fn add_pack_creates_asset_pack() -> anyhow::Result<()> {
        let tmp = TempDir::with_prefix("dungeon_rs-assets")?;
        let cache = TempDir::with_prefix("dungeon_rs-assets")?;
        let mut library = AssetLibrary::default();
        let pack_id = library.add_pack(tmp.path(), Some(cache.path().to_path_buf()), None)?;

        assert_eq!(library.registered_packs.len(), 1);
        assert!(library.registered_packs.contains_key(&pack_id));
        Ok(())
    }

    #[test]
    fn save_and_load_library() -> anyhow::Result<()> {
        let tmp = TempDir::with_prefix("dungeon_rs-assets")?;
        let cache = TempDir::with_prefix("dungeon_rs-assets")?;

        let mut library = AssetLibrary::default();
        let pack_id = library.add_pack(tmp.path(), Some(cache.path().to_path_buf()), None)?;

        library.save(Some(tmp.path().to_path_buf()))?;
        let library = AssetLibrary::load_or_default(Some(tmp.path().to_path_buf()))?;

        assert_eq!(library.registered_packs.len(), 1);
        assert!(library.registered_packs.contains_key(&pack_id));
        Ok(())
    }

    #[test]
    fn load_asset_pack_requires_registration() -> anyhow::Result<()> {
        let tmp = TempDir::with_prefix("dungeon_rs-assets")?;
        let mut library = AssetLibrary::default();
        let id = utils::hash_path(tmp.path());
        let pack = AssetPack::new(id, tmp.path(), tmp.path(), None)?;

        library
            .load_pack(&pack.id)
            .expect_err("Asset pack should not be registered");
        assert!(library.loaded_packs.is_empty());
        assert!(library.registered_packs.is_empty());
        Ok(())
    }

    #[test]
    fn iterate_registered_packs() -> anyhow::Result<()> {
        let tmp = tempfile::tempdir()?;
        let cache = tempfile::tempdir()?;
        let mut library = AssetLibrary::default();
        let pack_id = library.add_pack(tmp.path(), Some(cache.path().to_path_buf()), None)?;
        library.loaded_packs.clear();

        for (id, entry) in library.iter() {
            assert_eq!(id, &pack_id);
            assert_eq!(entry, &tmp.path().to_path_buf());
        }
        Ok(())
    }
}
