#![doc = include_str!("../README.md")]

mod directories;
mod load;
mod manifest;
mod scan;
mod source;
mod thumbnail;

pub use directories::LibraryDirectories;
pub use load::{LoadedAsset, asset_path, load_asset};
pub use manifest::{
    Manifest, ManifestsRead, Rename, forget_manifest, read_manifests, write_manifest,
};
pub use scan::{Scan, ScanDiff, ScannedFile, scan_folder};
pub use source::{LIBRARY_SOURCE, LibraryTable, register_library_source};
pub use thumbnail::{
    ThumbnailCache, ThumbnailCompletion, ThumbnailGenerator, ThumbnailJob, ThumbnailKey,
    ThumbnailTable, register_thumbnail_source,
};

use bevy_app::{App, Plugin};
use std::path::PathBuf;

/// What can go wrong while reading Asset Folders and the editor's own files about them.
#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    /// The platform names no home directory, so no configuration or cache directory is known.
    #[error(transparent)]
    NoPlatformDirectories(#[from] drs_model::NoPlatformDirectories),
    /// A file or directory could not be read, written, or listed.
    #[error("cannot {action} {}: {source}", path.display())]
    Io {
        /// What was attempted, such as `list` or `write`.
        action: &'static str,
        /// The file or directory.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// A Manifest file does not hold a Manifest.
    #[error("the Manifest {} is not valid: {reason}", path.display())]
    BadManifest {
        /// The file.
        path: PathBuf,
        /// Why it is not valid.
        reason: String,
    },
    /// A Manifest cannot be written because its path is not valid Unicode.
    #[error("the path {} cannot be recorded in a Manifest: {reason}", path.display())]
    UnrecordablePath {
        /// The folder's path.
        path: PathBuf,
        /// Why it cannot be recorded.
        reason: String,
    },
    /// An index cache could not be encoded.
    #[error("the index cache {} cannot be encoded: {reason}", path.display())]
    UnencodableIndex {
        /// The cache file.
        path: PathBuf,
        /// Why it cannot be encoded.
        reason: String,
    },
    /// An image file's header could not be read.
    #[error("{} is not an image that can be read: {reason}", path.display())]
    BadImage {
        /// The file.
        path: PathBuf,
        /// Why it could not be read.
        reason: String,
    },
    /// A place climbs out of its folder or is otherwise not a relative path inside it.
    #[error("the place `{place}` is not inside the folder")]
    EscapesFolder {
        /// The place as given.
        place: String,
    },
}

/// Registers the [`LibraryTable`] and the [`ThumbnailTable`] when the Host has not already done
/// so through [`register_library_source`] and [`register_thumbnail_source`].
pub struct LibraryAccessPlugin;

impl Plugin for LibraryAccessPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LibraryTable>()
            .init_resource::<ThumbnailTable>();
    }
}
