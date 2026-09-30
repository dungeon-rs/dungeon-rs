//! This module contains the command to validate that all required features in the workspace are present.
use crate::violation::Violation;
use cargo_metadata::Metadata;

/// The names of features we require to be present in each sub-crate.
pub const REQUIRED_FEATURES: [&str; 2] = ["default", "dev"];

const RULE: &str = "Required features";

/// Every workspace crate must declare each of [`REQUIRED_FEATURES`].
pub fn check(metadata: &Metadata) -> Vec<Violation> {
    let mut violations = Vec::new();

    for package in metadata.workspace_packages() {
        for required in REQUIRED_FEATURES {
            if !package.features.contains_key(required) {
                violations.push(Violation::new(
                    package.name.to_string(),
                    RULE,
                    format!("missing the `{required}` feature"),
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
    fn a_crate_with_both_required_features_passes() {
        let (_dir, metadata) = workspace(&[Crate::new("drs-model")]);

        assert!(check(&metadata).is_empty());
    }

    #[test]
    fn a_crate_without_dev_is_named_with_the_missing_feature() {
        let (_dir, metadata) =
            workspace(&[Crate::new("drs-model").with_features(&[("default", &[])])]);

        let violations = check(&metadata);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].package, "drs-model");
        assert_eq!(violations[0].rule, "Required features");
        assert!(violations[0].detail.contains("`dev`"));
    }

    #[test]
    fn a_crate_without_default_is_reported() {
        let (_dir, metadata) = workspace(&[Crate::new("drs-model").with_features(&[("dev", &[])])]);

        assert_eq!(check(&metadata).len(), 1);
    }
}
