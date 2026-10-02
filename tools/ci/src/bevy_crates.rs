//! This module contains the command that enforces that every Bevy crate a workspace crate uses is
//! a deliberate choice of `ARCHITECTURE.md`: one of the narrow crates the Framework boundary bullet
//! lists, or a row of the Restricted external dependencies table.
use crate::architecture::Architecture;
use crate::violation::Violation;
use anyhow::{Context, Result, bail, ensure};
use cargo_metadata::Metadata;

const RULE: &str = "Known Bevy crates";

/// The bullet whose first parenthesised list names the narrow crates every crate may use.
const FRAMEWORK_BOUNDARY: &str = "- **Framework boundary**:";

/// Whether a crate is part of Bevy or built for it, by its name.
fn is_bevy(name: &str) -> bool {
    name == "bevy" || name.starts_with("bevy_")
}

/// The narrow crates the Framework boundary bullet lists: the first parenthesised list in the
/// bullet, each item a code-formatted Bevy crate name.
///
/// A missing bullet, list, or a malformed item is an error, never an empty list: a reworded
/// bullet would otherwise make every narrow crate a violation, or a typo would hide one.
pub fn narrow_crates(markdown: &str) -> Result<Vec<String>> {
    let bullet = markdown
        .lines()
        .find(|line| line.trim_start().starts_with(FRAMEWORK_BOUNDARY))
        .context("`ARCHITECTURE.md` has no Framework boundary bullet")?;
    let list = bullet
        .split_once('(')
        .and_then(|(_, rest)| rest.split_once(')'))
        .map(|(list, _)| list)
        .context("the Framework boundary bullet lists no narrow crates in parentheses")?;

    let mut crates = Vec::new();
    for item in list.split(',').map(str::trim) {
        let Some(name) = item
            .strip_prefix('`')
            .and_then(|item| item.strip_suffix('`'))
        else {
            bail!(
                "the Framework boundary bullet lists `{item}`, which is not a code-formatted crate name"
            );
        };
        ensure!(
            is_bevy(name),
            "the Framework boundary bullet lists `{name}`, which is not a Bevy crate"
        );
        crates.push(name.to_string());
    }
    Ok(crates)
}

/// Every Bevy crate a workspace crate depends on, dev-dependencies included, is a narrow crate
/// or has a row in the Restricted external dependencies table, which says who may use it.
pub fn check(markdown: &str, metadata: &Metadata) -> Result<Vec<Violation>> {
    let architecture = Architecture::parse(markdown)?;
    let narrow = narrow_crates(markdown)?;

    let mut violations = Vec::new();
    for package in metadata.workspace_packages() {
        for dependency in &package.dependencies {
            let name = dependency.name.as_str();
            let known = !is_bevy(name)
                || narrow.iter().any(|crate_name| crate_name == name)
                || architecture
                    .restrictions
                    .iter()
                    .any(|restriction| restriction.external == name);
            if !known {
                violations.push(Violation::new(
                    package.name.to_string(),
                    RULE,
                    format!(
                        "uses `{name}`, which is neither a narrow crate of the Framework boundary nor in the Restricted external dependencies table"
                    ),
                ));
            }
        }
    }
    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Crate, workspace};

    const MARKDOWN: &str = "\
# Architecture

## Technology

- **Framework boundary**: every crate may use the narrow ECS crates (`bevy_ecs`, `bevy_math`). Only the Client uses the umbrella `bevy` crate and `bevy_render`.

## Dependencies

| Unit (crate) | Type | May depend on |
|---|---|---|
| drs-editor | Client | drs-model |
| drs-model | Model | |

## Restricted external dependencies

| External crate | Allowed for |
|---|---|
| bevy | Client |
| bevy_render | Client |
";

    fn violations_for(crates: &[Crate]) -> Vec<Violation> {
        let (_dir, metadata) = workspace(crates);
        check(MARKDOWN, &metadata).expect("the document is read")
    }

    #[test]
    fn the_narrow_crates_are_read_from_the_bullet() {
        assert_eq!(
            narrow_crates(MARKDOWN).expect("the bullet is read"),
            ["bevy_ecs", "bevy_math"]
        );
    }

    #[test]
    fn narrow_and_restricted_bevy_crates_pass() {
        let violations = violations_for(&[
            Crate::new("drs-model")
                .depends_on_external("bevy_ecs")
                .depends_on_external("bevy_math"),
            Crate::new("drs-editor")
                .depends_on("drs-model")
                .depends_on_external("bevy")
                .depends_on_external("bevy_render"),
        ]);

        assert_eq!(violations, []);
    }

    #[test]
    fn a_bevy_crate_nobody_chose_is_named() {
        let violations = violations_for(&[
            Crate::new("drs-model").depends_on_external("bevy_ecs"),
            Crate::new("drs-editor")
                .depends_on("drs-model")
                .depends_on_external("bevy_mesh"),
        ]);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].package, "drs-editor");
        assert_eq!(violations[0].rule, RULE);
        assert!(violations[0].detail.contains("bevy_mesh"));
    }

    #[test]
    fn a_crate_outside_bevy_is_free_to_use() {
        let violations = violations_for(&[Crate::new("drs-model").depends_on_external("serde")]);

        assert_eq!(violations, []);
    }

    #[test]
    fn a_document_without_the_bullet_is_an_error() {
        let markdown = MARKDOWN.replace("**Framework boundary**", "**Framework**");

        assert!(narrow_crates(&markdown).is_err());
    }

    #[test]
    fn a_bullet_without_a_list_is_an_error() {
        let markdown = MARKDOWN.replace("(`bevy_ecs`, `bevy_math`)", "`bevy_ecs` and `bevy_math`");

        assert!(narrow_crates(&markdown).is_err());
    }

    #[test]
    fn a_list_item_that_is_not_a_crate_name_is_an_error() {
        let markdown = MARKDOWN.replace("(`bevy_ecs`, `bevy_math`)", "(`bevy_ecs`, maths)");

        assert!(narrow_crates(&markdown).is_err());
    }
}
