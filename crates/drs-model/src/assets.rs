//! Asset Folders as the editor knows them on this device, and Asset References as a Project
//! records them.

use bevy_ecs::component::Component;
use bevy_ecs::reflect::ReflectComponent;
use bevy_math::UVec2;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;

/// The name an Asset Folder is known by in Projects, the same on every device.
#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CanonicalName(pub String);

impl CanonicalName {
    /// The name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CanonicalName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A device-local identifier of an added Asset Folder.
///
/// It names the folder's Manifest, its index cache, and its `lib://` source entry; it is never
/// written into a Project, which knows a folder by its [`CanonicalName`].
#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FolderKey(pub String);

impl FolderKey {
    /// The key as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FolderKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// What an Asset is: an image, a Material, and so on. Plugins can add kinds.
#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AssetKind(Cow<'static, str>);

impl AssetKind {
    /// The image kind.
    pub const IMAGE: Self = Self(Cow::Borrowed("image"));

    /// A kind by name.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(Cow::Borrowed(name))
    }

    /// The kind's name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Enough about an Asset's content to recognise it elsewhere: `blake3:` followed by the hex digest.
#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Fingerprint(String);

impl Fingerprint {
    /// The fingerprint of content whose BLAKE3 digest is `hex`.
    #[must_use]
    pub fn blake3(hex: &str) -> Self {
        Self(format!("blake3:{hex}"))
    }

    /// The fingerprint as text, recipe included.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One Asset indexed in an Asset Folder on this device.
#[derive(Reflect, Debug, Clone, PartialEq, Eq)]
pub struct IndexedAsset {
    /// The file name without its extension.
    pub name: String,
    /// Where the Asset sits in the folder: its relative path with `/` separators.
    pub place: String,
    /// What the Asset is.
    pub kind: AssetKind,
    /// The size of the file in bytes.
    pub byte_size: u64,
    /// When the file was last modified, as time since the Unix epoch.
    pub modified: Duration,
}

/// What indexing an Asset Folder left out, and how much.
#[derive(Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScanSkips {
    /// Entries whose name is not valid Unicode.
    pub non_unicode_names: usize,
    /// Folders inside the Asset Folder that could not be listed.
    pub unlisted_folders: usize,
    /// Entries whose metadata could not be read.
    pub unreadable_entries: usize,
}

/// An Asset Folder added on this device: a folder of Assets used as-is, in place.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Eq)]
#[reflect(Component)]
pub struct AssetFolder {
    /// The name Projects know the folder by.
    pub name: CanonicalName,
    /// The device-local key of the folder.
    pub key: FolderKey,
    /// The folder's path as the Author gave it.
    pub path: PathBuf,
    /// The Assets in the folder, ordered by place.
    pub assets: Vec<IndexedAsset>,
    /// What the last indexing of the folder skipped.
    pub skips: ScanSkips,
}

/// What a Project records about one Asset it uses.
#[derive(Reflect, Debug, Clone, PartialEq, Eq)]
pub struct AssetReference {
    /// The Canonical Name of the Asset Folder the Asset came from.
    pub folder: CanonicalName,
    /// The place in that folder: the relative path, Unicode-normalised (NFC) with `/` separators.
    pub place: String,
    /// The Asset's name as shown, in its original spelling.
    pub name: String,
    /// What the Asset is.
    pub kind: AssetKind,
    /// The fingerprint of the Asset's content.
    pub fingerprint: Fingerprint,
    /// The size of the Asset's file in bytes.
    pub byte_size: u64,
    /// The pixel size of an image Asset, so that a placeholder keeps its layout.
    pub pixel_size: Option<UVec2>,
}

/// What a Project records about one Asset Folder its Assets come from.
#[derive(Reflect, Debug, Clone, PartialEq, Eq)]
pub struct AssetFolderReference {
    /// The folder's Canonical Name.
    pub name: CanonicalName,
    /// The folder's version as its Manifest recorded it when the Project last saw it.
    pub version: String,
}

/// A row of the [`AssetReferences`] table, as Elements refer to their Asset.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetReferenceRow(pub u32);

/// The [`AssetReferences`] table has no row left for another Asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the Project's Asset Reference table is full")]
pub struct AssetReferencesFull;

/// The Project's table of Asset References: one row per distinct Asset and one per Asset Folder.
#[derive(Component, Reflect, Debug, Clone, Default, PartialEq, Eq)]
#[reflect(Component)]
pub struct AssetReferences {
    /// The Assets the Project uses, addressed by row.
    pub assets: Vec<AssetReference>,
    /// The Asset Folders those Assets come from.
    pub folders: Vec<AssetFolderReference>,
}

impl AssetReferences {
    /// The row that holds the Asset at `place` in the folder named `folder`, if any.
    #[must_use]
    pub fn row_of(&self, folder: &CanonicalName, place: &str) -> Option<AssetReferenceRow> {
        self.assets
            .iter()
            .position(|asset| &asset.folder == folder && asset.place == place)
            .and_then(|index| u32::try_from(index).ok())
            .map(AssetReferenceRow)
    }

    /// The Asset Reference in a row, if the row exists.
    #[must_use]
    pub fn get(&self, row: AssetReferenceRow) -> Option<&AssetReference> {
        self.assets.get(row.0 as usize)
    }

    /// Adds an Asset Reference, or returns the row of the one already recorded for the same Asset.
    ///
    /// # Errors
    ///
    /// [`AssetReferencesFull`] when the table has no row left, in which case nothing is added.
    pub fn record(
        &mut self,
        reference: AssetReference,
    ) -> Result<AssetReferenceRow, AssetReferencesFull> {
        if let Some(row) = self.row_of(&reference.folder, &reference.place) {
            return Ok(row);
        }
        let row = u32::try_from(self.assets.len()).map_err(|_| AssetReferencesFull)?;
        self.assets.push(reference);
        Ok(AssetReferenceRow(row))
    }
}
