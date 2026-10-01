//! Loading one Asset: what a Project needs to record about it.

use crate::LibraryError;
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
    /// The `lib://` path that loads the image through the asset server.
    pub asset_path: String,
}

/// The `lib://` path of the Asset at `place` in the folder with `key`.
#[must_use]
pub fn asset_path(key: &FolderKey, place: &str) -> String {
    format!("{}://{}/{place}", crate::LIBRARY_SOURCE, key.as_str())
}

/// The file at `place` inside `folder`, refusing a place that climbs out of the folder.
///
/// # Errors
///
/// [`LibraryError::EscapesFolder`] when `place` is absolute or holds a `..` component.
pub fn file_in_folder(folder: &Path, place: &str) -> Result<PathBuf, LibraryError> {
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

/// `LoadAsset`: reads the Asset at `place` in the folder at `folder` (whose key is `key`) and
/// yields its byte size, pixel size, fingerprint, and `lib://` path.
///
/// # Errors
///
/// [`LibraryError::EscapesFolder`] when `place` is not inside the folder, [`LibraryError::Io`]
/// when the file cannot be read, or [`LibraryError::BadImage`] when its header is not an image.
pub fn load_asset(
    folder: &Path,
    key: &FolderKey,
    place: &str,
) -> Result<LoadedAsset, LibraryError> {
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
        asset_path: asset_path(key, place),
    })
}
