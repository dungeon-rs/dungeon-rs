//! Where the editor keeps its own files.

use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;
use bevy_reflect::Reflect;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Overrides for the directories the editor writes its own files into.
///
/// `None` means the platform's directory. Tests point them at temporary directories; the Host
/// leaves them unset outside development.
#[derive(Resource, Reflect, Debug, Clone, Default, PartialEq, Eq)]
#[reflect(Resource)]
pub struct EditorDirectories {
    /// Where Manifests live.
    pub configuration: Option<PathBuf>,
    /// Where index caches live.
    pub cache: Option<PathBuf>,
    /// Where log files and crash reports live; `logs` under the cache directory by default.
    pub logs: Option<PathBuf>,
}

/// The directories the editor writes its own files into, every one of them known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDirectories {
    /// Where Manifests live.
    pub configuration: PathBuf,
    /// Where index caches live.
    pub cache: PathBuf,
    /// Where log files and crash reports live.
    pub logs: PathBuf,
}

/// The platform names no home directory, so no configuration or cache directory is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("no configuration or cache directory is known on this platform")]
pub struct NoPlatformDirectories;

impl EditorDirectories {
    /// Every directory under one root, as tests and development runs use.
    #[must_use]
    pub fn under(root: &Path) -> Self {
        Self {
            configuration: Some(root.join("configuration")),
            cache: Some(root.join("cache")),
            logs: Some(root.join("logs")),
        }
    }

    /// The platform's directories for the editor, except where this resource names another;
    /// the log directory is `logs` under the cache directory unless named.
    ///
    /// # Errors
    ///
    /// [`NoPlatformDirectories`] when a directory is not overridden and the platform names none.
    pub fn resolve(&self) -> Result<ResolvedDirectories, NoPlatformDirectories> {
        let platform =
            || directories::ProjectDirs::from("", "", "dungeon-rs").ok_or(NoPlatformDirectories);
        let configuration = match &self.configuration {
            Some(directory) => directory.clone(),
            None => platform()?.config_dir().to_path_buf(),
        };
        let cache = match &self.cache {
            Some(directory) => directory.clone(),
            None => platform()?.cache_dir().to_path_buf(),
        };
        let logs = match &self.logs {
            Some(directory) => directory.clone(),
            None => cache.join("logs"),
        };
        Ok(ResolvedDirectories {
            configuration,
            cache,
            logs,
        })
    }
}
