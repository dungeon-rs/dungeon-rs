//! Resolve: which Asset on this device an Asset Reference means.

use drs_model::{AssetFolder, AssetReference, CanonicalName, MissingReason, Resolution};
use unicode_normalization::UnicodeNormalization;

/// A Canonical Name as compared for uniqueness: Unicode-normalised and in lower case.
fn folded_name(name: &CanonicalName) -> String {
    name.as_str().nfc().collect::<String>().to_lowercase()
}

/// Whether two Canonical Names are the same name, compared as uniqueness compares them: ignoring
/// letter case and Unicode normalisation.
#[must_use]
pub fn same_name(a: &CanonicalName, b: &CanonicalName) -> bool {
    folded_name(a) == folded_name(b)
}

/// A place in its stored form: Unicode-normalised.
fn normalised(place: &str) -> String {
    place.nfc().collect()
}

/// A place as compared ignoring letter case and Unicode normalisation: normalised, then fully
/// case-folded, so that `Straße` and `STRASSE` compare equal where lower-casing would not.
fn folded_place(place: &str) -> String {
    caseless::default_case_fold_str(&normalised(place))
}

/// Resolve: the Asset on this device that `reference` means, among `folders`, the Asset Folders
/// added on this device.
///
/// The folder is the one whose Canonical Name is the reference's, compared as [`same_name`]
/// compares; without it the Asset is Missing because the folder is absent. In that folder, the
/// first recorded place at which an Asset sits resolves exactly; failing that, the first recorded
/// place that exactly one Asset's place matches ignoring letter case and Unicode normalisation
/// resolves to that Asset, while two or more such Assets make the reference Missing as ambiguous.
/// A folder holding neither makes it Missing with the folder's version on this device. Nothing
/// is read from disk: the folder's version and the recorded one are never compared, and neither
/// are the Asset's size or content.
pub fn resolve<'a>(
    reference: &AssetReference,
    folders: impl IntoIterator<Item = &'a AssetFolder>,
) -> Resolution {
    let Some(folder) = folders
        .into_iter()
        .find(|folder| same_name(&folder.name, &reference.folder))
    else {
        return Resolution::Missing(MissingReason::FolderAbsent);
    };
    let resolved = |place: &str| Resolution::Resolved {
        folder: folder.key.clone(),
        place: place.to_owned(),
    };

    for known in &reference.places {
        let wanted = normalised(known);
        if let Some(asset) = folder
            .assets
            .iter()
            .find(|asset| normalised(&asset.place) == wanted)
        {
            return resolved(&asset.place);
        }
    }
    for known in &reference.places {
        let wanted = folded_place(known);
        let candidates: Vec<&str> = folder
            .assets
            .iter()
            .map(|asset| asset.place.as_str())
            .filter(|place| folded_place(place) == wanted)
            .collect();
        match candidates.as_slice() {
            [] => {}
            [only] => return resolved(only),
            several => {
                return Resolution::Missing(MissingReason::Ambiguous {
                    version: folder.version.clone(),
                    candidates: several.iter().map(|place| (*place).to_owned()).collect(),
                });
            }
        }
    }
    Resolution::Missing(MissingReason::AssetAbsent {
        version: folder.version.clone(),
    })
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        reason = "a test stops at the first thing that is not as expected"
    )]
    use super::*;
    use drs_model::{AssetKind, Fingerprint, FolderKey, IndexedAsset, ScanSkips};
    use std::time::Duration;

    /// An Asset Folder named `Fixtures` holding an image at each of `places`.
    fn folder(places: &[&str]) -> AssetFolder {
        AssetFolder {
            name: CanonicalName("Fixtures".to_owned()),
            key: FolderKey("fixtures".to_owned()),
            path: "/library".into(),
            version: "2026-10-02".to_owned(),
            assets: places
                .iter()
                .map(|place| IndexedAsset {
                    name: "barrel".to_owned(),
                    place: (*place).to_owned(),
                    kind: AssetKind::IMAGE,
                    byte_size: 1,
                    modified: Duration::ZERO,
                })
                .collect(),
            skips: ScanSkips::default(),
        }
    }

    /// An Asset Reference into `Fixtures` recorded at `place`.
    fn reference(place: &str) -> AssetReference {
        AssetReference {
            folder: CanonicalName("fixtures".to_owned()),
            places: vec![place.to_owned()],
            name: "barrel".to_owned(),
            kind: AssetKind::IMAGE,
            fingerprint: Fingerprint::blake3("00"),
            byte_size: 1,
            pixel_size: None,
        }
    }

    /// Two Assets that each differ from the recorded place only in letter case or Unicode
    /// normalisation make the reference a Missing Asset as ambiguous, naming both, rather than
    /// resolving to either; a lone such Asset resolves.
    #[test]
    fn two_spellings_are_ambiguous() {
        let fixtures = folder(&["props/Barrel.png", "props/barrel.png"]);

        let resolution = resolve(&reference("PROPS/BARREL.png"), [&fixtures]);

        assert_eq!(
            resolution,
            Resolution::Missing(MissingReason::Ambiguous {
                version: "2026-10-02".to_owned(),
                candidates: vec!["props/Barrel.png".to_owned(), "props/barrel.png".to_owned()],
            })
        );
        assert_eq!(
            resolve(
                &reference("PROPS/BARREL.png"),
                [&folder(&["props/Barrel.png"])]
            ),
            Resolution::Resolved {
                folder: FolderKey("fixtures".to_owned()),
                place: "props/Barrel.png".to_owned(),
            }
        );
    }
}
