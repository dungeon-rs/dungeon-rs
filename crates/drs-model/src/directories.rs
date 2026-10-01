//! Where the editor keeps its own files.

use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;
use bevy_reflect::Reflect;
use std::path::PathBuf;

/// Overrides for the directories the editor writes its own files into.
///
/// `None` means the platform's directory. Tests point both at temporary directories; the Host
/// leaves them unset.
#[derive(Resource, Reflect, Debug, Clone, Default, PartialEq, Eq)]
#[reflect(Resource)]
pub struct EditorDirectories {
    /// Where Manifests live.
    pub configuration: Option<PathBuf>,
    /// Where index caches live.
    pub cache: Option<PathBuf>,
}

impl EditorDirectories {
    /// Both directories under one root, as tests use.
    #[must_use]
    pub fn under(root: &std::path::Path) -> Self {
        Self {
            configuration: Some(root.join("configuration")),
            cache: Some(root.join("cache")),
        }
    }
}
