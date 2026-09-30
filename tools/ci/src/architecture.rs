//! This module contains the command that enforces the Dependencies tables of `ARCHITECTURE.md`.
use crate::violation::Violation;
use anyhow::{Context, Result};
use cargo_metadata::Metadata;
use std::collections::BTreeMap;

/// A component (crate) and what the architecture allows it to depend on.
pub struct Component {
    /// The component type, e.g. `Engine`.
    pub kind: String,
    /// The workspace crates it may depend on.
    pub may_depend_on: Vec<String>,
}

/// An external crate and the crates or component types allowed to use it.
pub struct Restriction {
    pub external: String,
    pub allowed_for: Vec<String>,
}

/// The enforceable tables of `ARCHITECTURE.md`.
pub struct Architecture {
    pub components: BTreeMap<String, Component>,
    pub restrictions: Vec<Restriction>,
}

/// Component types that may never depend on a component of their own type.
const ISOLATED_KINDS: [&str; 3] = ["Engine", "ResourceAccess", "Manager"];

impl Architecture {
    /// Reads the Dependencies and Restricted external dependencies tables.
    pub fn parse(markdown: &str) -> Result<Self> {
        let components = table_rows(markdown, "Dependencies")
            .context("`ARCHITECTURE.md` has no Dependencies table")?
            .into_iter()
            .map(|row| {
                let component = Component {
                    kind: row.get(1).cloned().unwrap_or_default(),
                    may_depend_on: split_list(row.get(2)),
                };
                (row[0].clone(), component)
            })
            .collect();

        let restrictions = table_rows(markdown, "Restricted external dependencies")
            .context("`ARCHITECTURE.md` has no Restricted external dependencies table")?
            .into_iter()
            .map(|row| Restriction {
                external: row[0].clone(),
                allowed_for: split_list(row.get(1)),
            })
            .collect();

        Ok(Self {
            components,
            restrictions,
        })
    }

    /// Compares the workspace against the tables.
    pub fn check(&self, metadata: &Metadata) -> Vec<Violation> {
        let mut violations = Vec::new();
        let packages = metadata.workspace_packages();

        for name in self.components.keys() {
            if !packages.iter().any(|pkg| pkg.name == name.as_str()) {
                violations.push(Violation::new(
                    name,
                    "Every component is a crate",
                    "listed in the Dependencies table but missing from the workspace",
                ));
            }
        }

        for package in packages {
            let name = package.name.to_string();
            let Some(component) = self.components.get(&name) else {
                violations.push(Violation::new(
                    name,
                    "Known crates only",
                    "has no row in the Dependencies table",
                ));
                continue;
            };

            for dependency in &package.dependencies {
                let dependency = dependency.name.as_str();
                if let Some(target) = self.components.get(dependency) {
                    if !component.may_depend_on.iter().any(|d| d == dependency) {
                        violations.push(Violation::new(
                            &name,
                            "Allowed dependencies",
                            format!("may not depend on `{dependency}`"),
                        ));
                    }
                    if target.kind == component.kind
                        && ISOLATED_KINDS.contains(&component.kind.as_str())
                    {
                        violations.push(Violation::new(
                            &name,
                            "Same-kind isolation",
                            format!(
                                "{} `{dependency}` may not be depended on by a {}",
                                target.kind, component.kind
                            ),
                        ));
                    }
                }

                if let Some(restriction) =
                    self.restrictions.iter().find(|r| r.external == dependency)
                    && !restriction
                        .allowed_for
                        .iter()
                        .any(|allowed| *allowed == name || *allowed == component.kind)
                {
                    violations.push(Violation::new(
                        &name,
                        "Restricted externals",
                        format!(
                            "may not use `{dependency}`; allowed for {}",
                            restriction.allowed_for.join(", ")
                        ),
                    ));
                }
            }
        }

        violations
    }
}

/// The body rows of the first table under the `##` heading with the given title.
fn table_rows(markdown: &str, heading: &str) -> Option<Vec<Vec<String>>> {
    let mut lines = markdown.lines();
    lines.find(|line| line.trim() == format!("## {heading}"))?;

    let rows: Vec<Vec<String>> = lines
        .skip_while(|line| !line.trim_start().starts_with('|'))
        .take_while(|line| line.trim_start().starts_with('|'))
        .skip(2) // header and separator
        .map(|line| {
            let line = line.trim().trim_start_matches('|').trim_end_matches('|');
            line.split('|')
                .map(|cell| cell.trim().to_string())
                .collect()
        })
        .collect();

    (!rows.is_empty()).then_some(rows)
}

fn split_list(cell: Option<&String>) -> Vec<String> {
    cell.map(|cell| {
        cell.split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect()
    })
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Crate, workspace};

    const TABLES: &str = "\
# Architecture

## Dependencies

| Unit (crate) | Type | May depend on |
|---|---|---|
| drs-app | Host | drs-editor, drs-shape-engine, drs-model |
| drs-editor | Client | drs-model |
| drs-authoring-manager | Manager | drs-shape-engine, drs-model |
| drs-project-manager | Manager | drs-model |
| drs-shape-engine | Engine | drs-model |
| drs-paint-engine | Engine | drs-model |
| drs-project-access | ResourceAccess | drs-model |
| drs-library-access | ResourceAccess | drs-model |
| drs-model | Model | |

Prose after the table.

## Restricted external dependencies

| External crate | Allowed for |
|---|---|
| bevy | Client, Host |
| kurbo | drs-shape-engine |

## Call chains
";

    /// The components in [`TABLES`], each with its allowed dependencies and `dev` propagation.
    fn valid_workspace() -> Vec<Crate> {
        vec![
            Crate::new("drs-model"),
            Crate::new("drs-shape-engine").depends_on("drs-model"),
            Crate::new("drs-paint-engine").depends_on("drs-model"),
            Crate::new("drs-project-access").depends_on("drs-model"),
            Crate::new("drs-library-access").depends_on("drs-model"),
            Crate::new("drs-authoring-manager")
                .depends_on("drs-shape-engine")
                .depends_on("drs-model"),
            Crate::new("drs-project-manager").depends_on("drs-model"),
            Crate::new("drs-editor").depends_on("drs-model"),
            Crate::new("drs-app")
                .depends_on("drs-editor")
                .depends_on("drs-shape-engine")
                .depends_on("drs-model"),
        ]
    }

    /// Replaces one crate of [`valid_workspace`].
    fn with(replacement: Crate) -> Vec<Crate> {
        let name = replacement.name();
        let mut crates: Vec<Crate> = valid_workspace()
            .into_iter()
            .filter(|c| c.name() != name)
            .collect();
        crates.push(replacement);
        crates
    }

    fn violations_for(crates: &[Crate]) -> Vec<Violation> {
        let architecture = Architecture::parse(TABLES).expect("tables parse");
        let (_dir, metadata) = workspace(crates);
        architecture.check(&metadata)
    }

    #[test]
    fn the_tables_are_read_from_the_markdown() {
        let architecture = Architecture::parse(TABLES).expect("tables parse");

        assert_eq!(architecture.components.len(), 9);
        let shape = &architecture.components["drs-shape-engine"];
        assert_eq!(shape.kind, "Engine");
        assert_eq!(shape.may_depend_on, ["drs-model"]);
        assert!(
            architecture.components["drs-model"]
                .may_depend_on
                .is_empty()
        );
        assert_eq!(architecture.restrictions.len(), 2);
        assert_eq!(architecture.restrictions[0].external, "bevy");
        assert_eq!(architecture.restrictions[0].allowed_for, ["Client", "Host"]);
    }

    #[test]
    fn a_document_without_the_tables_is_an_error() {
        assert!(Architecture::parse("# nothing here").is_err());
    }

    #[test]
    fn a_workspace_that_follows_the_tables_passes() {
        assert_eq!(violations_for(&valid_workspace()), []);
    }

    #[test]
    fn a_missing_component_is_reported() {
        let crates: Vec<Crate> = valid_workspace()
            .into_iter()
            .filter(|c| c.name() != "drs-paint-engine")
            .collect();

        let violations = violations_for(&crates);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].package, "drs-paint-engine");
        assert_eq!(violations[0].rule, "Every component is a crate");
    }

    #[test]
    fn a_crate_without_a_row_is_reported() {
        let mut crates = valid_workspace();
        crates.push(Crate::new("drs-mystery"));

        let violations = violations_for(&crates);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].package, "drs-mystery");
        assert_eq!(violations[0].rule, "Known crates only");
    }

    #[test]
    fn a_dependency_the_row_does_not_allow_is_named() {
        let violations = violations_for(&with(
            Crate::new("drs-editor")
                .depends_on("drs-model")
                .depends_on("drs-shape-engine"),
        ));

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].package, "drs-editor");
        assert_eq!(violations[0].rule, "Allowed dependencies");
        assert!(violations[0].detail.contains("drs-shape-engine"));
    }

    #[test]
    fn an_engine_depending_on_an_engine_breaks_same_kind_isolation() {
        let violations = violations_for(&with(
            Crate::new("drs-shape-engine")
                .depends_on("drs-model")
                .depends_on("drs-paint-engine"),
        ));

        assert!(violations.iter().any(|v| v.package == "drs-shape-engine"
            && v.rule == "Same-kind isolation"
            && v.detail.contains("drs-paint-engine")));
    }

    #[test]
    fn a_resource_access_depending_on_a_resource_access_breaks_same_kind_isolation() {
        let violations = violations_for(&with(
            Crate::new("drs-project-access")
                .depends_on("drs-model")
                .depends_on("drs-library-access"),
        ));

        assert!(
            violations
                .iter()
                .any(|v| v.package == "drs-project-access" && v.rule == "Same-kind isolation")
        );
    }

    #[test]
    fn a_manager_depending_on_a_manager_breaks_same_kind_isolation() {
        let violations = violations_for(&with(
            Crate::new("drs-authoring-manager")
                .depends_on("drs-shape-engine")
                .depends_on("drs-model")
                .depends_on("drs-project-manager"),
        ));

        assert!(
            violations
                .iter()
                .any(|v| v.package == "drs-authoring-manager" && v.rule == "Same-kind isolation")
        );
    }

    #[test]
    fn a_restricted_external_in_an_allowed_component_type_passes() {
        let violations = violations_for(&with(
            Crate::new("drs-editor")
                .depends_on("drs-model")
                .depends_on_external("bevy"),
        ));

        assert_eq!(violations, []);
    }

    #[test]
    fn a_restricted_external_in_an_allowed_crate_passes() {
        let violations = violations_for(&with(
            Crate::new("drs-shape-engine")
                .depends_on("drs-model")
                .depends_on_external("kurbo"),
        ));

        assert_eq!(violations, []);
    }

    #[test]
    fn a_restricted_external_outside_its_allowed_components_is_named() {
        let violations = violations_for(&with(Crate::new("drs-model").depends_on_external("bevy")));

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].package, "drs-model");
        assert_eq!(violations[0].rule, "Restricted externals");
        assert!(violations[0].detail.contains("bevy"));
    }

    #[test]
    fn an_unrestricted_external_is_free_to_use() {
        let violations =
            violations_for(&with(Crate::new("drs-model").depends_on_external("serde")));

        assert_eq!(violations, []);
    }
}
