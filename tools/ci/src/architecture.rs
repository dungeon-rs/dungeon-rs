//! This module contains the command that enforces the Dependencies tables of `ARCHITECTURE.md`.
use crate::violation::Violation;
use anyhow::{Context, Result, bail, ensure};
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

const EVERY_COMPONENT_IS_A_CRATE: &str = "Every component is a crate";
const KNOWN_CRATES_ONLY: &str = "Known crates only";
const ALLOWED_DEPENDENCIES: &str = "Allowed dependencies";
const SAME_KIND_ISOLATION: &str = "Same-kind isolation";
const RESTRICTED_EXTERNALS: &str = "Restricted externals";

/// The component types of the architecture.
const KINDS: [&str; 7] = [
    "Host",
    "Client",
    "Manager",
    "Engine",
    "ResourceAccess",
    "Utility",
    "Model",
];

/// Component types that may never depend on a component of their own type.
const ISOLATED_KINDS: [&str; 3] = ["Engine", "ResourceAccess", "Manager"];

impl Architecture {
    /// Reads the Dependencies and Restricted external dependencies tables.
    ///
    /// A malformed table is an error, never a silently weaker check: a typo in a component type or a
    /// crate name would otherwise make a rule stop matching.
    pub fn parse(markdown: &str) -> Result<Self> {
        let mut components = BTreeMap::new();
        for row in table_rows(markdown, "Dependencies")
            .context("`ARCHITECTURE.md` has no Dependencies table")?
        {
            let [name, kind, may_depend_on] = row.as_slice() else {
                bail!(
                    "Dependencies row has {} cells, expected 3: {row:?}",
                    row.len()
                );
            };
            ensure!(
                KINDS.contains(&kind.as_str()),
                "`{name}` has unknown type `{kind}`; expected one of {KINDS:?}"
            );
            let component = Component {
                kind: kind.clone(),
                may_depend_on: split_list(may_depend_on),
            };
            ensure!(
                components.insert(name.clone(), component).is_none(),
                "`{name}` has more than one row in the Dependencies table"
            );
        }

        for (name, component) in &components {
            for dependency in &component.may_depend_on {
                ensure!(
                    components.contains_key(dependency),
                    "`{name}` may depend on `{dependency}`, which has no row in the Dependencies table"
                );
            }
        }

        let mut restrictions = Vec::new();
        for row in table_rows(markdown, "Restricted external dependencies")
            .context("`ARCHITECTURE.md` has no Restricted external dependencies table")?
        {
            let [external, allowed_for] = row.as_slice() else {
                bail!(
                    "Restricted external dependencies row has {} cells, expected 2: {row:?}",
                    row.len()
                );
            };
            let allowed_for = split_list(allowed_for);
            for allowed in &allowed_for {
                ensure!(
                    components.contains_key(allowed) || KINDS.contains(&allowed.as_str()),
                    "`{external}` is allowed for `{allowed}`, which is neither a component nor a component type"
                );
            }
            restrictions.push(Restriction {
                external: external.clone(),
                allowed_for,
            });
        }

        Ok(Self {
            components,
            restrictions,
        })
    }

    /// Compares the workspace against the tables.
    pub fn check(&self, metadata: &Metadata) -> Vec<Violation> {
        let mut violations = self.missing_components(metadata);

        for package in metadata.workspace_packages() {
            let name = package.name.to_string();
            let Some(component) = self.components.get(&name) else {
                violations.push(Violation::new(
                    name,
                    KNOWN_CRATES_ONLY,
                    "has no row in the Dependencies table",
                ));
                continue;
            };

            for dependency in &package.dependencies {
                let dependency = dependency.name.as_str();
                violations.extend(self.check_workspace_dependency(&name, component, dependency));
                violations.extend(self.check_restricted_external(&name, component, dependency));
            }
        }

        violations
    }

    /// Components the table lists that the workspace lacks.
    fn missing_components(&self, metadata: &Metadata) -> Vec<Violation> {
        let packages = metadata.workspace_packages();
        self.components
            .keys()
            .filter(|name| !packages.iter().any(|pkg| pkg.name == name.as_str()))
            .map(|name| {
                Violation::new(
                    name,
                    EVERY_COMPONENT_IS_A_CRATE,
                    "listed in the Dependencies table but missing from the workspace",
                )
            })
            .collect()
    }

    /// A dependency on another workspace crate must be allowed by the table.
    ///
    /// Same-kind isolation follows from a correct table; it is checked on its own as well, so a table
    /// that wrongly allowed an Engine to depend on an Engine would still be caught.
    fn check_workspace_dependency(
        &self,
        name: &str,
        component: &Component,
        dependency: &str,
    ) -> Vec<Violation> {
        let Some(target) = self.components.get(dependency) else {
            return Vec::new();
        };

        let mut violations = Vec::new();
        if !component.may_depend_on.iter().any(|d| d == dependency) {
            violations.push(Violation::new(
                name,
                ALLOWED_DEPENDENCIES,
                format!("may not depend on `{dependency}`"),
            ));
        }
        if target.kind == component.kind && ISOLATED_KINDS.contains(&component.kind.as_str()) {
            violations.push(Violation::new(
                name,
                SAME_KIND_ISOLATION,
                format!(
                    "`{dependency}` is a {} and {} crates may not depend on each other",
                    target.kind, component.kind
                ),
            ));
        }
        violations
    }

    /// A restricted external crate is only used by the crates or component types listed for it.
    fn check_restricted_external(
        &self,
        name: &str,
        component: &Component,
        dependency: &str,
    ) -> Option<Violation> {
        let restriction = self
            .restrictions
            .iter()
            .find(|r| r.external == dependency)?;
        let allowed = restriction
            .allowed_for
            .iter()
            .any(|allowed| allowed == name || *allowed == component.kind);

        (!allowed).then(|| {
            Violation::new(
                name,
                RESTRICTED_EXTERNALS,
                format!(
                    "may not use `{dependency}`; allowed for {}",
                    restriction.allowed_for.join(", ")
                ),
            )
        })
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

fn split_list(cell: &str) -> Vec<String> {
    cell.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
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
        assert_eq!(violations[0].rule, EVERY_COMPONENT_IS_A_CRATE);
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

    #[test]
    fn the_host_type_resolves_a_restricted_external() {
        let violations = violations_for(&with(
            Crate::new("drs-app")
                .depends_on("drs-editor")
                .depends_on("drs-shape-engine")
                .depends_on("drs-model")
                .depends_on_external("bevy"),
        ));

        assert_eq!(violations, []);
    }

    #[test]
    fn an_unknown_component_type_is_an_error() {
        let markdown = TABLES.replace("| Engine |", "| Engines |");

        assert!(Architecture::parse(&markdown).is_err());
    }

    #[test]
    fn a_duplicate_row_is_an_error() {
        let markdown = TABLES.replace(
            "| drs-model | Model | |",
            "| drs-model | Model | |\n| drs-model | Model | |",
        );

        assert!(Architecture::parse(&markdown).is_err());
    }

    #[test]
    fn a_dependency_on_an_unlisted_component_is_an_error() {
        let markdown = TABLES.replace(
            "| drs-editor | Client | drs-model |",
            "| drs-editor | Client | drs-nope |",
        );

        assert!(Architecture::parse(&markdown).is_err());
    }

    #[test]
    fn a_row_with_the_wrong_number_of_cells_is_an_error() {
        let markdown = TABLES.replace("| drs-model | Model | |", "| drs-model | Model |");

        assert!(Architecture::parse(&markdown).is_err());
    }

    #[test]
    fn a_restriction_naming_an_unknown_type_is_an_error() {
        let markdown = TABLES.replace("| bevy | Client, Host |", "| bevy | Clients, Host |");

        assert!(Architecture::parse(&markdown).is_err());
    }
}
