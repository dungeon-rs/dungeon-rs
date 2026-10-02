//! Loading one Asset: what a Project needs to record about it.

use crate::LibraryError;
use bevy_asset::AssetPath;
use bevy_math::UVec2;
use drs_model::{Fingerprint, FolderKey};
use std::path::{Component, Path, PathBuf};

/// What loading an Asset yields besides the image itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedAsset {
    /// The size of the file in bytes.
    pub byte_size: u64,
    /// The width and height of the image in pixels.
    pub pixel_size: UVec2,
    /// The BLAKE3 fingerprint of the file's content.
    pub fingerprint: Fingerprint,
}

/// The `lib://` asset path of the Asset at `place` in the folder with `key`.
///
/// The path is assembled from its parts rather than parsed from text, so a `#` or `?` in a file
/// name stays part of the name instead of being read as an asset label or query.
#[must_use]
pub fn asset_path(key: &FolderKey, place: &str) -> AssetPath<'static> {
    AssetPath::from_path_buf(Path::new(key.as_str()).join(place)).with_source(crate::LIBRARY_SOURCE)
}

/// The file at `place` inside `folder`, refusing a place that climbs out of the folder.
///
/// # Errors
///
/// [`LibraryError::EscapesFolder`] when `place` is absolute or holds a `..` component.
pub(crate) fn file_in_folder(folder: &Path, place: &str) -> Result<PathBuf, LibraryError> {
    let relative = Path::new(place);
    let inside = relative.components().all(|component| match component {
        Component::Normal(_) | Component::CurDir => true,
        Component::ParentDir | Component::RootDir | Component::Prefix(_) => false,
    });
    if !inside || relative.as_os_str().is_empty() {
        return Err(LibraryError::EscapesFolder {
            place: place.to_owned(),
        });
    }
    Ok(folder.join(relative))
}

/// `LoadAsset`: reads the Asset at `place` in the folder at `folder` and yields its byte size,
/// pixel size, and fingerprint.
///
/// # Errors
///
/// [`LibraryError::EscapesFolder`] when `place` is not inside the folder, [`LibraryError::Io`]
/// when the file cannot be read, or [`LibraryError::BadImage`] when its header is not an image.
pub fn load_asset(folder: &Path, place: &str) -> Result<LoadedAsset, LibraryError> {
    let file = file_in_folder(folder, place)?;
    let bytes = std::fs::read(&file).map_err(|source| LibraryError::Io {
        action: "read",
        path: file.clone(),
        source,
    })?;
    let (width, height) = image::ImageReader::new(std::io::Cursor::new(&bytes))
        .with_guessed_format()
        .map_err(|error| LibraryError::BadImage {
            path: file.clone(),
            reason: error.to_string(),
        })?
        .into_dimensions()
        .map_err(|error| LibraryError::BadImage {
            path: file.clone(),
            reason: error.to_string(),
        })?;
    let digest = blake3::hash(&bytes);
    Ok(LoadedAsset {
        byte_size: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        pixel_size: UVec2::new(width, height),
        fingerprint: Fingerprint::blake3(&digest.to_hex()),
    })
}
