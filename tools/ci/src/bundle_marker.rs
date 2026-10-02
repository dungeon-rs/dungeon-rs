//! This module contains the command to validate that the bundle marker names the workspace's version.
use crate::violation::Violation;
use cargo_metadata::Metadata;
use std::fs;

const RULE: &str = "Bundle marker";

/// The file that marks the workspace's bundle directory as the editor's, relative to the
/// workspace root; it holds the editor's version.
pub const MARKER: &str = "bundle/dungeon-rs.bundle";

/// The marker in the workspace's bundle directory names the version every workspace crate has.
pub fn check(metadata: &Metadata) -> Vec<Violation> {
    let path = metadata.workspace_root.join(MARKER);
    let Ok(marker) = fs::read_to_string(&path) else {
        return vec![Violation::new(
            "workspace",
            RULE,
            format!("`{MARKER}` is missing or cannot be read"),
        )];
    };
    let marker = marker.trim();

    let mut versions: Vec<String> = metadata
        .workspace_packages()
        .iter()
        .map(|package| package.version.to_string())
        .collect();
    versions.sort();
    versions.dedup();

    versions
        .into_iter()
        .filter(|version| version != marker)
        .map(|version| {
            Violation::new(
                "workspace",
                RULE,
                format!(
                    "`{MARKER}` names version `{marker}` while the workspace is at `{version}`"
                ),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Crate, workspace};
    use std::path::Path;

    /// Writes the marker holding `version` into the fixture workspace at `root`.
    fn marker(root: &Path, version: &str) {
        let path = root.join(MARKER);
        fs::create_dir_all(path.parent().expect("the bundle directory")).expect("create it");
        fs::write(path, format!("{version}\n")).expect("write the marker");
    }

    #[test]
    fn a_marker_naming_the_workspace_version_passes() {
        let (dir, metadata) = workspace(&[Crate::new("drs-model"), Crate::new("drs-app")]);
        marker(dir.path(), "0.0.0");

        assert!(check(&metadata).is_empty());
    }

    #[test]
    fn a_marker_naming_another_version_is_named_with_both() {
        let (dir, metadata) = workspace(&[Crate::new("drs-model")]);
        marker(dir.path(), "0.0.1");

        let violations = check(&metadata);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "Bundle marker");
        assert!(violations[0].detail.contains("`0.0.1`"));
        assert!(violations[0].detail.contains("`0.0.0`"));
    }

    #[test]
    fn a_missing_marker_is_reported() {
        let (_dir, metadata) = workspace(&[Crate::new("drs-model")]);

        let violations = check(&metadata);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].detail.contains(MARKER));
    }
}
