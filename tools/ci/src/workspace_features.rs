//! This module contains the command to validate that features are propagated to workspace dependencies.
use crate::violation::Violation;
use cargo_metadata::Metadata;
use std::collections::HashSet;

const RULE: &str = "Dev propagates";

/// Features that must be enabled on every workspace dependency; `default` only needs to exist.
const PROPAGATED_FEATURES: &[&str] = &["dev"];

/// Every workspace crate's `dev` feature must enable `dev` on each workspace crate it depends on.
pub fn check(metadata: &Metadata) -> Vec<Violation> {
    let workspace_names: HashSet<String> = metadata
        .workspace_packages()
        .iter()
        .map(|pkg| pkg.name.to_string())
        .collect();

    let mut violations = Vec::new();

    for package in metadata.workspace_packages() {
        let mut local_deps: Vec<&str> = package
            .dependencies
            .iter()
            .filter(|dep| workspace_names.contains(dep.name.as_str()))
            .map(|dep| dep.name.as_str())
            .collect();
        local_deps.sort_unstable();
        local_deps.dedup();

        for &feature in PROPAGATED_FEATURES {
            // A missing feature is reported by the required features check.
            let Some(enabled) = package.features.get(feature) else {
                continue;
            };

            for dep in &local_deps {
                let token = format!("{dep}/{feature}");
                if !enabled.contains(&token) {
                    violations.push(Violation::new(
                        package.name.to_string(),
                        RULE,
                        format!("`{feature}` does not enable `{token}`"),
                    ));
                }
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
    fn dev_enabling_dev_on_every_workspace_dependency_passes() {
        let (_dir, metadata) = workspace(&[
            Crate::new("drs-history"),
            Crate::new("drs-model").depends_on("drs-history"),
        ]);

        assert!(check(&metadata).is_empty());
    }

    #[test]
    fn dev_that_skips_a_workspace_dependency_is_named() {
        let (_dir, metadata) = workspace(&[
            Crate::new("drs-history"),
            Crate::new("drs-model").depends_on_without_dev("drs-history"),
        ]);

        let violations = check(&metadata);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].package, "drs-model");
        assert_eq!(violations[0].rule, "Dev propagates");
        assert!(violations[0].detail.contains("drs-history/dev"));
    }

    #[test]
    fn external_dependencies_need_no_propagation() {
        let (_dir, metadata) = workspace(&[Crate::new("drs-model").depends_on_external("serde")]);

        assert!(check(&metadata).is_empty());
    }
}
