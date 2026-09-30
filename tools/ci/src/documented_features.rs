//! This module contains the command to validate that all features in the workspace have been documented.
use crate::required_features::REQUIRED_FEATURES;
use crate::violation::Violation;
use cargo_metadata::Metadata;
use std::fs;

const RULE: &str = "Documented features";

/// Every workspace crate has a README that mentions each feature other than the required ones.
pub fn check(metadata: &Metadata) -> Vec<Violation> {
    let mut violations = Vec::new();

    for package in metadata.workspace_packages() {
        let Some(crate_dir) = package.manifest_path.parent() else {
            continue;
        };

        let Ok(readme) = fs::read_to_string(crate_dir.join("README.md")) else {
            violations.push(Violation::new(
                package.name.to_string(),
                RULE,
                "missing README.md",
            ));
            continue;
        };

        for feature in package.features.keys() {
            if !REQUIRED_FEATURES.contains(&feature.as_str()) && !readme.contains(feature) {
                violations.push(Violation::new(
                    package.name.to_string(),
                    RULE,
                    format!("README.md does not document the `{feature}` feature"),
                ));
            }
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Crate, workspace};

    #[test]
    fn a_readme_that_mentions_every_extra_feature_passes() {
        let (_dir, metadata) = workspace(&[Crate::new("drs-model")
            .with_features(&[("default", &[]), ("dev", &[]), ("serde", &[])])
            .with_readme("Features: `serde`.")]);

        assert!(check(&metadata).is_empty());
    }

    #[test]
    fn an_undocumented_feature_is_named() {
        let (_dir, metadata) = workspace(&[Crate::new("drs-model")
            .with_features(&[("default", &[]), ("dev", &[]), ("serde", &[])])
            .with_readme("Nothing here.")]);

        let violations = check(&metadata);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].package, "drs-model");
        assert_eq!(violations[0].rule, "Documented features");
        assert!(violations[0].detail.contains("`serde`"));
    }

    #[test]
    fn a_crate_without_a_readme_is_reported() {
        let (_dir, metadata) = workspace(&[Crate::new("drs-model").without_readme()]);

        let violations = check(&metadata);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].detail.contains("README"));
    }
}
