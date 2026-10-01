//! This module contains the command that enforces that Engine crates publish and subscribe to no
//! events: their sources define and use no messages, events, or observers.
use crate::architecture::Architecture;
use crate::violation::Violation;
use anyhow::{Context, Result};
use cargo_metadata::Metadata;
use std::fs;
use std::path::{Path, PathBuf};

const RULE: &str = "Engines publish no events";

/// The component type the rule applies to.
const ENGINE: &str = "Engine";

/// The derives that define a message or an event.
const DEFINING_DERIVES: [&str; 3] = ["Message", "Event", "EntityEvent"];

/// The names that read, write, or observe them.
const USING_NAMES: [&str; 5] = [
    "MessageReader",
    "MessageWriter",
    "Observer",
    "add_observer",
    "On<",
];

/// Every crate the architecture types as an Engine has sources that define and use no messages,
/// events, or observers.
///
/// Comments are not read: a doc comment may explain why an Engine has none.
pub fn check(architecture: &Architecture, metadata: &Metadata) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();

    for package in metadata.workspace_packages() {
        let name = package.name.to_string();
        let is_engine = architecture
            .components
            .get(&name)
            .is_some_and(|component| component.kind == ENGINE);
        if !is_engine {
            continue;
        }

        let sources = package
            .manifest_path
            .parent()
            .with_context(|| format!("`{name}` has a manifest without a directory"))?
            .join("src");
        for file in rust_files(sources.as_std_path())? {
            let text =
                fs::read_to_string(&file).with_context(|| format!("reading {}", file.display()))?;
            let shown = file
                .strip_prefix(metadata.workspace_root.as_std_path())
                .unwrap_or(&file)
                .display();
            for (index, line) in text.lines().enumerate() {
                if let Some(found) = forbidden_in(line) {
                    violations.push(Violation::new(
                        name.clone(),
                        RULE,
                        format!("{shown}:{} uses `{found}`", index + 1),
                    ));
                }
            }
        }
    }

    Ok(violations)
}

/// What a line of source uses that an Engine may not, if anything.
fn forbidden_in(line: &str) -> Option<&'static str> {
    let code = line.trim_start();
    if code.starts_with("//") {
        return None;
    }
    if let Some(start) = code.find("derive(") {
        let list = &code[start + "derive(".len()..];
        let list = list.split(')').next().unwrap_or_default();
        if let Some(found) = list
            .split(',')
            .map(str::trim)
            .find_map(|derived| DEFINING_DERIVES.into_iter().find(|name| *name == derived))
        {
            return Some(found);
        }
    }
    USING_NAMES.into_iter().find(|name| mentions(code, name))
}

/// Whether `code` holds `name` as a whole name, not as part of a longer one.
fn mentions(code: &str, name: &str) -> bool {
    let is_name_char = |c: char| c.is_alphanumeric() || c == '_';
    code.match_indices(name).any(|(start, _)| {
        let before = code[..start].chars().next_back();
        let after = code[start + name.len()..].chars().next();
        let bounded_before = before.is_none_or(|c| !is_name_char(c));
        let bounded_after = name.ends_with('<') || after.is_none_or(|c| !is_name_char(c));
        bounded_before && bounded_after
    })
}

/// Every `.rs` file under `root`, recursively, in a stable order.
fn rust_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return Ok(files);
    };
    for entry in entries {
        let entry = entry.with_context(|| format!("listing {}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(rust_files(&path)?);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Crate, workspace};

    const TABLES: &str = "\
## Dependencies

| Unit (crate) | Type | May depend on |
|---|---|---|
| drs-authoring-manager | Manager | drs-shape-engine, drs-model |
| drs-shape-engine | Engine | drs-model |
| drs-model | Model | |

## Restricted external dependencies

| External crate | Allowed for |
|---|---|
| bevy | Client, Host |
";

    fn violations_for(crates: &[Crate]) -> Vec<Violation> {
        let architecture = Architecture::parse(TABLES).expect("tables parse");
        let (_dir, metadata) = workspace(crates);
        check(&architecture, &metadata).expect("sources are read")
    }

    fn engine_with(source: &str) -> Vec<Crate> {
        vec![
            Crate::new("drs-model"),
            Crate::new("drs-authoring-manager").depends_on("drs-model"),
            Crate::new("drs-shape-engine")
                .depends_on("drs-model")
                .with_source(source),
        ]
    }

    #[test]
    fn an_engine_of_plain_systems_passes() {
        let violations = violations_for(&engine_with(
            "pub fn snap(query: Query<&Point>, mut commands: Commands) -> Option<Vec2> { None }\n",
        ));

        assert_eq!(violations, []);
    }

    #[test]
    fn an_engine_defining_a_message_is_named_with_the_line() {
        let violations = violations_for(&engine_with(
            "use bevy_ecs::message::Message;\n\n#[derive(Message, Debug, Clone)]\npub struct Snapped;\n",
        ));

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].package, "drs-shape-engine");
        assert_eq!(violations[0].rule, "Engines publish no events");
        assert!(
            violations[0].detail.contains("src/lib.rs:3"),
            "{}",
            violations[0].detail
        );
        assert!(
            violations[0].detail.contains("`Message`"),
            "{}",
            violations[0].detail
        );
    }

    #[test]
    fn an_engine_defining_an_event_is_named() {
        let violations = violations_for(&engine_with("#[derive(EntityEvent)]\npub struct Hit;\n"));

        assert_eq!(violations.len(), 1);
        assert!(violations[0].detail.contains("`EntityEvent`"));
    }

    #[test]
    fn an_engine_reading_or_writing_messages_is_named() {
        let violations = violations_for(&engine_with(
            "fn a(mut reader: MessageReader<X>) {}\nfn b(mut writer: MessageWriter<X>) {}\n",
        ));

        assert_eq!(violations.len(), 2);
        assert!(violations[0].detail.contains("`MessageReader`"));
        assert!(violations[1].detail.contains("`MessageWriter`"));
    }

    #[test]
    fn an_engine_observing_is_named() {
        let violations = violations_for(&engine_with(
            "fn hit(on: On<Hit>) {}\nfn setup(mut commands: Commands) { commands.add_observer(hit); }\n",
        ));

        assert_eq!(violations.len(), 2);
        assert!(violations[0].detail.contains("`On<`"));
        assert!(violations[1].detail.contains("`add_observer`"));
    }

    #[test]
    fn a_mention_in_a_comment_is_not_a_use() {
        let violations = violations_for(&engine_with(
            "//! No Observer here: an Engine is called, never notified.\n/// Nor a MessageReader.\n",
        ));

        assert_eq!(violations, []);
    }

    #[test]
    fn a_longer_name_sharing_the_letters_is_not_a_use() {
        let violations = violations_for(&engine_with(
            "pub struct EventuallyOn<T>(T);\npub fn observer_count(x: Option<u8>) {}\n",
        ));

        assert_eq!(violations, []);
    }

    #[test]
    fn a_manager_may_use_messages() {
        let violations = violations_for(&[
            Crate::new("drs-model"),
            Crate::new("drs-shape-engine").depends_on("drs-model"),
            Crate::new("drs-authoring-manager")
                .depends_on("drs-model")
                .with_source("fn apply(mut requests: MessageReader<Apply>) {}\n"),
        ]);

        assert_eq!(violations, []);
    }
}
