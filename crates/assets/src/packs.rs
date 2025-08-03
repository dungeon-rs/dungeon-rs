//! An asset pack is a single root folder that contains asset and subfolders.

mod index;
mod thumbnails;

use crate::packs::index::{AssetPackIndex, AssetPackIndexError};
use crate::packs::thumbnails::{AssetPackThumbnailError, AssetPackThumbnails};
use bevy::prelude::{Asset, AssetServer, Handle, debug, info, trace};
use serialization::{Deserialize, SerializationFormat, Serialize, deserialize, serialize_to};
use std::fs::{File, create_dir_all};
use std::io::read_to_string;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use utils::file_name;

/// The filename of the asset pack manifests.
const MANIFEST_FILE_NAME: &str = "asset_pack.toml";

/// The directory name inside the `meta_dir` where the Tantivy index lives.
const INDEX_DIR_NAME: &str = "index";

/// The directory name inside the `meta_dir` where the thumbnails are generated into.
const THUMBNAIL_DIR_NAME: &str = "thumbnails";

/// An [`AssetPack`] is a single root folder that contains assets and subfolders.
///
/// The asset pack handles the indexing, categorising and loading the assets.
#[derive(Debug)]
pub struct AssetPack {
    /// The state the pack is currently in.
    ///
    /// This is used to track whether a pack needs to perform operations to be usable, whether some
    /// operations failed and so forth.
    pub state: AssetPackState,

    /// The identifier of this string, usually a hash or short ID defined by the creator of the asset
    /// pack represented.
    ///
    /// This ID is used when referring to files under [`AssetPack::root`].
    pub id: String,

    /// The human-readable name of this [`AssetPack`].
    ///
    /// This is not guaranteed to be unique! If you need to identify this pack, please use [`AssetPack::id`].
    pub name: String,

    /// The "root" directory under which the assets live for this pack.
    ///
    /// This is used internally to generate relative paths (that are portable) from absolute paths
    /// used in the asset loader.
    pub root: PathBuf,

    /// The directory in which metadata about the [`AssetPack`] is kept.
    /// This ranges from index metadata, scripts to thumbnails, this directory is not guaranteed to
    /// exist between runs and may be cleaned to recover disk space. The operations in this directory
    /// should be ephemeral by design.
    pub meta_dir: PathBuf,

    /// Contains the actual indexation logic for this `AssetPack`.
    index: AssetPackIndex,

    /// Contains the actual thumbnail generation and resolve logic for this [`AssetPack`].
    thumbnails: AssetPackThumbnails,

    /// A [Rhai](https://rhai.rs/) script that is used during indexing operations to filter whether
    /// an asset should be included in the pack or not.
    ///
    /// When set to `None` the pack will use the embedded script for filtering.
    filter_script: Option<String>,

    /// A [Rhai](https://rhai.rs/) script that is used during indexing operations to assist in categorising
    /// the assets in the pack.
    ///
    /// When set to `None` the pack will use the embedded script for indexing.
    index_script: Option<String>,
}

/// Internal "copy" of the [`AssetPack`] struct intended for saving/loading to disk.
#[derive(Serialize, Deserialize, Debug)]
#[allow(
    clippy::missing_docs_in_private_items,
    reason = "Copied from the original struct"
)]
struct _AssetPack {
    pub id: String,
    pub name: String,
    filter_script: Option<String>,
    index_script: Option<String>,
}

/// Describes the current state of an [`AssetPack`].
#[derive(Default, Debug, Serialize, Deserialize)]
pub enum AssetPackState {
    /// The asset pack was just created, no validation or checks to its current state have been made.
    /// Additional processing is required to validate the pack's state before it can be used.
    #[default]
    Created,

    /// The asset pack is currently (re)indexing its contents.
    Indexing,

    /// Something went wrong during processing, leaving this pack in an invalid state.
    Invalid(String),

    /// The pack is ready to use.
    Ready,
}

/// Describes the errors that can occur when working with [`AssetPack`]s
#[derive(Error, Debug)]
pub enum AssetPackError {
    /// Thrown when creating/opening the asset pack manifest fails.
    #[error("An IO error occurred while reading/writing the asset pack manifest")]
    ManifestFile(#[from] std::io::Error),

    /// Thrown when the serialisation of an asset pack manifest fails.
    #[error("An error occurred while serialising the asset pack manifest")]
    Serialisation(#[from] serialization::SerializationError),

    /// Thrown when Tantivy throws an error, usually during indexing or reading.
    #[error("An error occurred while indexing the asset pack")]
    Indexing(#[from] AssetPackIndexError),

    /// Thrown when thumbnail generation or resolution throws an error.
    #[error("An error occurred while generating thumbnails")]
    Thumbnails(#[from] AssetPackThumbnailError),
}

impl AssetPack {
    /// Generate a new [`AssetPack`] in the [`AssetPackState::Created`] state.
    ///
    /// # Errors
    /// This method may return an error if it fails to [canonicalize](https://doc.rust-lang.org/std/fs/fn.canonicalize.html)
    /// the root path.
    pub fn new(root: &Path, meta_dir: &Path, name: Option<String>) -> Result<Self, AssetPackError> {
        trace!("Creating new asset pack from {root}", root = root.display());
        let root = root.canonicalize()?;
        let id = blake3::hash(root.as_os_str().as_encoded_bytes()).to_string();
        let meta_dir = meta_dir.join(id.clone());
        let index_dir = meta_dir.join(INDEX_DIR_NAME);
        let thumbnails_dir = meta_dir.join(THUMBNAIL_DIR_NAME);
        let name = name
            .or_else(|| file_name(&root))
            .unwrap_or_else(|| id.clone());

        // Tantivy requires that the directory exists already.
        trace!(
            "Creating meta directory {meta_dir}",
            meta_dir = meta_dir.display()
        );
        create_dir_all(&index_dir)?;
        create_dir_all(&thumbnails_dir)?;

        info!("Created new asset pack with ID: {}", id);
        Ok(Self {
            state: AssetPackState::Created,
            id,
            name,
            root,
            meta_dir,
            index: AssetPackIndex::new(index_dir)?,
            thumbnails: AssetPackThumbnails::new(thumbnails_dir, None, None)?,
            filter_script: None,
            index_script: None,
        })
    }

    /// Deletes all cache and config for this [`AssetPack`].
    ///
    /// # Errors
    /// Can return [`AssetPackError::ManifestFile`] when it fails to clean up any files.
    pub(crate) fn delete(self) -> Result<(), AssetPackError> {
        info!("Deleting asset pack: {}", self.id);
        let config_file = self.root.join(MANIFEST_FILE_NAME);

        std::fs::remove_file(config_file)?;
        std::fs::remove_dir_all(self.meta_dir.clone())?;
        Ok(())
    }

    /// Attempts to save the manifest for this [`AssetPack`] to disk.
    /// The resulting file will be written under [`AssetPack::root`].
    ///
    /// # Errors
    /// - [`AssetPackError::ManifestFile`] when the file/folder for the manifest couldn't be created.
    /// - [`AssetPackError::Serialisation`] when serialising the manifest fails.
    pub fn save_manifest(&self) -> Result<(), AssetPackError> {
        debug!("Saving manifest for {}", self.id);
        let config = _AssetPack::from(self);
        let manifest = self.root.join(MANIFEST_FILE_NAME);
        let manifest = File::create(manifest).map_err(AssetPackError::ManifestFile)?;

        serialize_to(&config, &SerializationFormat::Toml, manifest)
            .map_err(AssetPackError::Serialisation)
    }

    /// Attempts to load an [`AssetPack`] from its manifest in the `root` folder.
    /// The resulting [`AssetPack`] will always be in [`AssetPackState::Crated`].
    ///
    /// # Errors
    /// - [`AssetPackError::ManifestFile`] when the file/folder for the manifest couldn't be opened.
    /// - [`AssetPackError::Serialisation`] when serialising the manifest fails.
    pub fn load_manifest(root: &Path, meta_dir: &Path) -> Result<Self, AssetPackError> {
        let manifest = root.join(MANIFEST_FILE_NAME);
        debug!("Loading manifest for {}", manifest.display());
        let manifest = File::open(manifest).map_err(AssetPackError::ManifestFile)?;
        let manifest = read_to_string(manifest).map_err(AssetPackError::ManifestFile)?;

        let manifest: _AssetPack = deserialize(manifest.as_bytes(), &SerializationFormat::Toml)?;
        debug!("Loaded manifest for {}", manifest.id);

        let meta_dir = meta_dir.join(manifest.id.clone());
        let index_dir = meta_dir.join(INDEX_DIR_NAME);
        let thumbnails_dir = meta_dir.join(THUMBNAIL_DIR_NAME);
        Ok(Self {
            state: AssetPackState::Created,
            id: manifest.id,
            name: manifest.name,
            root: root.to_path_buf(),
            meta_dir,
            index: AssetPackIndex::open(index_dir)?,
            thumbnails: AssetPackThumbnails::new(thumbnails_dir, None, None)?,
            filter_script: manifest.filter_script,
            index_script: manifest.index_script,
        })
    }

    /// This forces the underlying index for this [`AssetPack`] to be rebuilt from scratch.
    /// If `generate_thumbnails` is set to `true`, indexing will also generate thumbnails.
    ///
    /// Note that this is an expensive operation that may take several seconds to minutes to complete
    /// and will use a lot of CPU (indexing, hashing and thumbnail generation).
    ///
    /// # Errors
    /// For specific details on what can cause indexing to fail, see [`AssetPackIndex::index`].
    #[inline(always)]
    #[allow(
        clippy::inline_always,
        reason = "Wrapper function for AssetPackIndex::index"
    )]
    pub fn index(&self, generate_thumbnails: bool) -> Result<(), AssetPackError> {
        self.index
            .index(
                &self.root,
                if generate_thumbnails {
                    Some(&self.thumbnails)
                } else {
                    None
                },
                self.index_script.as_ref(),
                self.filter_script.as_ref(),
            )
            .map_err(AssetPackError::Indexing)
    }

    /// Attempts to resolve the given identifier into a [`PathBuf`].
    #[must_use]
    pub fn resolve(&self, id: &str) -> Option<PathBuf> {
        trace!("{} is resolving asset {}", self.id, id);

        self.index.find_by_id(id)
    }

    /// Attempts to load the asset associated with the given path.
    #[must_use = "Unused asset handle would be dropped immediately"]
    pub fn load<T>(&self, asset_server: &AssetServer, id: impl AsRef<str>) -> Option<Handle<T>>
    where
        T: Asset,
    {
        if let Some(path) = self.resolve(id.as_ref()) {
            return Some(asset_server.load::<T>(path));
        }

        None
    }
}

impl From<&AssetPack> for _AssetPack {
    fn from(pack: &AssetPack) -> Self {
        Self {
            id: pack.id.clone(),
            name: pack.name.clone(),
            filter_script: pack.filter_script.clone(),
            index_script: pack.index_script.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc)]
    #![allow(clippy::missing_errors_doc)]

    use super::*;
    use tempfile::tempdir;

    #[test]
    fn new_asset_pack_id_is_stable() -> anyhow::Result<()> {
        let path = tempdir()?;
        let subpath = path.path().join("new_asset_pack_id_is_stable");
        create_dir_all(&subpath)?;

        let pack = AssetPack::new(&subpath, &subpath, None)?;
        let pack_id = pack.id.clone();
        pack.save_manifest()?;
        pack.delete()?;
        create_dir_all(&subpath)?;

        let pack2 = AssetPack::new(&subpath, &subpath, None)?;

        assert_eq!(pack_id, pack2.id);
        Ok(())
    }

    #[test]
    fn new_asset_pack_id_unique() -> anyhow::Result<()> {
        let path1 = tempdir()?;
        let path2 = tempdir()?;
        let pack1 = AssetPack::new(path1.path(), path1.path(), None)?;
        let pack2 = AssetPack::new(path2.path(), path2.path(), None)?;

        assert_ne!(pack1.id, pack2.id);
        Ok(())
    }

    #[test]
    #[should_panic = "Should fail to create asset pack"]
    fn new_asset_error_on_invalid_path() {
        let path = Path::new("./does/not/exist");
        AssetPack::new(path, path, None).expect("Should fail to create asset pack");
    }

    #[test]
    #[should_panic = "IndexAlreadyExists"]
    fn new_asset_pack_error_on_existing() {
        let path = tempdir().unwrap();
        AssetPack::new(path.path(), path.path(), None).unwrap();
        AssetPack::new(path.path(), path.path(), None).unwrap();
    }
}
