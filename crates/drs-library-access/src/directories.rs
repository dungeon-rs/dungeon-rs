//! Where the editor keeps Manifests and index caches.

use crate::LibraryError;
use drs_model::EditorDirectories;
use std::path::PathBuf;

/// The directories the editor writes its own files about Asset Folders into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryDirectories {
    /// Where Manifests live.
    pub configuration: PathBuf,
    /// Where index caches live.
    pub cache: PathBuf,
}

impl LibraryDirectories {
    /// The platform's directories, except where `overrides` names another.
    ///
    /// # Errors
    ///
    /// [`LibraryError::NoPlatformDirectories`] when a directory is not overridden and the
    /// platform names none.
    pub fn resolve(overrides: &EditorDirectories) -> Result<Self, LibraryError> {
        let platform = || {
            directories::ProjectDirs::from("", "", "dungeon-rs")
                .ok_or(LibraryError::NoPlatformDirectories)
        };
        let configuration = match &overrides.configuration {
            Some(directory) => directory.clone(),
            None => platform()?.config_dir().to_path_buf(),
        };
        let cache = match &overrides.cache {
            Some(directory) => directory.clone(),
            None => platform()?.cache_dir().to_path_buf(),
        };
        Ok(Self {
            configuration,
            cache,
        })
    }

    /// The file holding the Manifest of the folder with `key`.
    pub(crate) fn manifest_file(&self, key: &drs_model::FolderKey) -> PathBuf {
        self.configuration.join(format!("{}.json", key.as_str()))
    }

    /// The file holding the index cache of the folder with `key`.
    pub(crate) fn index_cache_file(&self, key: &drs_model::FolderKey) -> PathBuf {
        self.cache.join(format!("{}.index.json", key.as_str()))
    }
}
