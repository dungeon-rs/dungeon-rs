//! Where each Asset Reference of a Project loads from on this device, or why it cannot.

use crate::{AssetReferenceRow, FolderKey};
use bevy_ecs::component::Component;
use bevy_ecs::reflect::ReflectComponent;
use bevy_reflect::Reflect;

/// Why an Asset Reference is a Missing Asset on this device.
#[derive(Reflect, Debug, Clone, PartialEq, Eq)]
pub enum MissingReason {
    /// No Asset Folder on this device has the recorded Canonical Name.
    FolderAbsent,
    /// The folder is here, at the named version, but holds no Asset at any recorded place.
    AssetAbsent {
        /// The folder's version on this device.
        version: String,
    },
    /// The folder is here, at the named version, and more than one of its Assets differs from a
    /// recorded place only in letter case or Unicode normalisation, so none is chosen.
    Ambiguous {
        /// The folder's version on this device.
        version: String,
        /// The places that would each fit.
        candidates: Vec<String>,
    },
}

/// What an Asset Reference resolves to on this device.
#[derive(Reflect, Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// The Asset at `place` in the added folder with `folder`.
    Resolved {
        /// The key of the Asset Folder on this device.
        folder: FolderKey,
        /// The Asset's place in that folder, spelled as on disk.
        place: String,
    },
    /// A Missing Asset.
    Missing(MissingReason),
}

/// The Project's resolution table: one row per Asset Reference, written only by the project
/// Manager and read by whoever loads or explains an Asset.
#[derive(Component, Reflect, Debug, Clone, Default, PartialEq, Eq)]
#[reflect(Component)]
pub struct ResolutionTable {
    /// The resolutions, in the order of the Asset Reference table's rows.
    pub rows: Vec<Resolution>,
}

impl ResolutionTable {
    /// The resolution of a row, if the row has been resolved.
    #[must_use]
    pub fn get(&self, row: AssetReferenceRow) -> Option<&Resolution> {
        self.rows.get(row.0 as usize)
    }
}
