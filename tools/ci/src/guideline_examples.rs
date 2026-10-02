//! This module contains the command to validate that every guideline's Example is taken from its
//! Exemplar, so an Example cannot drift from the code it was trimmed from.
use crate::violation::Violation;
use anyhow::{Context, Result};
use cargo_metadata::Metadata;
use std::fs;
use std::path::Path;

const RULE: &str = "Guideline example";

/// Where the guidelines live, relative to the workspace root.
pub const GUIDELINES: &str = "docs/guidelines";

/// The guidelines' index, which is no guideline.
const INDEX: &str = "INDEX.md";

/// What names a guideline's Exemplar: the path follows in backticks.
const EXEMPLAR: &str = "**Exemplar**: `";

/// The heading of a guideline's Example.
const EXAMPLE: &str = "## Example";

/// Every guideline names its Exemplar and has an Example whose lines all appear in the Exemplar
/// file in the same order, each compared without its indentation; blank lines are not compared,
/// and the Exemplar may hold lines in between, which the Example trimmed away.
///
/// A workspace without a guidelines directory has nothing to check.
pub fn check(metadata: &Metadata) -> Result<Vec<Violation>> {
    let root = metadata.workspace_root.as_std_path();
    let directory = root.join(GUIDELINES);
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut guidelines: Vec<_> = fs::read_dir(&directory)
        .with_context(|| format!("listing {}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<_, _>>()
        .with_context(|| format!("listing {}", directory.display()))?;
    guidelines.retain(|path| {
        path.extension().is_some_and(|extension| extension == "md")
            && path.file_name().is_some_and(|name| name != INDEX)
    });
    guidelines.sort();

    let mut violations = Vec::new();
    for path in guidelines {
        let text =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        let name = format!(
            "{GUIDELINES}/{}",
            path.file_name().unwrap_or_default().to_string_lossy()
        );
        if let Some(problem) = problem(root, &text) {
            violations.push(Violation::new(name, RULE, problem));
        }
    }
    Ok(violations)
}

/// What is wrong with the guideline `text`, if anything.
fn problem(root: &Path, text: &str) -> Option<String> {
    let Some(exemplar) = exemplar(text) else {
        return Some("names no Exemplar".to_owned());
    };
    let Some(example) = example(text) else {
        return Some("has no Example".to_owned());
    };
    let Ok(source) = fs::read_to_string(root.join(exemplar)) else {
        return Some(format!(
            "its Exemplar `{exemplar}` is missing or cannot be read"
        ));
    };
    let mut lines = source.lines().map(str::trim);
    for (number, line) in example.iter().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !lines.any(|found| found == line) {
            return Some(format!(
                "line {} of its Example is not in `{exemplar}`, or not in that order: `{line}`",
                number + 1
            ));
        }
    }
    None
}

/// The path the guideline names as its Exemplar.
fn exemplar(text: &str) -> Option<&str> {
    let start = text.find(EXEMPLAR)? + EXEMPLAR.len();
    let length = text[start..].find('`')?;
    Some(&text[start..start + length])
}

/// The lines of the first fenced block under the guideline's Example heading.
fn example(text: &str) -> Option<Vec<&str>> {
    let mut lines = text
        .lines()
        .skip_while(|line| line.trim_end() != EXAMPLE)
        .skip(1)
        .skip_while(|line| !line.starts_with("```"))
        .skip(1);
    let mut block = Vec::new();
    loop {
        let line = lines.next()?;
        if line.starts_with("```") {
            return Some(block);
        }
        block.push(line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Crate, workspace};

    /// A guideline naming `exemplar` with `example` as its Example.
    fn guideline(exemplar: &str, example: &str) -> String {
        format!(
            "# A building block\n\n**Use when**: always.\n**Exemplar**: `{exemplar}`\n\n## Rules\n\n- One.\n\n## Example\n\n```rust\n{example}```\n\n## Pitfalls\n\n- None.\n"
        )
    }

    /// Writes `text` as the guideline `name` and `source` as `crates/drs-model/src/lib.rs` into
    /// the fixture workspace at `root`.
    fn write(root: &Path, name: &str, text: &str, source: &str) {
        let directory = root.join(GUIDELINES);
        fs::create_dir_all(&directory).expect("create the guidelines");
        fs::write(directory.join(name), text).expect("write the guideline");
        fs::write(root.join("crates/drs-model/src/lib.rs"), source).expect("write the exemplar");
    }

    /// The exemplar every test trims its example from.
    const SOURCE: &str =
        "pub fn first() {\n    let a = 1;\n    let b = 2;\n    a + b\n}\n\npub fn second() {}\n";

    #[test]
    fn an_example_trimmed_from_its_exemplar_passes() {
        let (dir, metadata) = workspace(&[Crate::new("drs-model")]);
        write(
            dir.path(),
            "block.md",
            &guideline(
                "crates/drs-model/src/lib.rs",
                "pub fn first() {\n  let a = 1;\n\n  a + b\n}\npub fn second() {}\n",
            ),
            SOURCE,
        );
        fs::write(dir.path().join(GUIDELINES).join(INDEX), "# Guidelines\n").expect("the index");

        assert!(check(&metadata).expect("checks").is_empty());
    }

    #[test]
    fn a_line_not_in_the_exemplar_is_named() {
        let (dir, metadata) = workspace(&[Crate::new("drs-model")]);
        write(
            dir.path(),
            "block.md",
            &guideline(
                "crates/drs-model/src/lib.rs",
                "pub fn first() {\n    let a = 3;\n}\n",
            ),
            SOURCE,
        );

        let violations = check(&metadata).expect("checks");

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].package, "docs/guidelines/block.md");
        assert_eq!(violations[0].rule, "Guideline example");
        assert!(violations[0].detail.contains("line 2"));
        assert!(violations[0].detail.contains("`let a = 3;`"));
    }

    #[test]
    fn lines_in_another_order_are_named() {
        let (dir, metadata) = workspace(&[Crate::new("drs-model")]);
        write(
            dir.path(),
            "block.md",
            &guideline(
                "crates/drs-model/src/lib.rs",
                "pub fn second() {}\npub fn first() {\n",
            ),
            SOURCE,
        );

        let violations = check(&metadata).expect("checks");

        assert_eq!(violations.len(), 1);
        assert!(violations[0].detail.contains("`pub fn first() {`"));
    }

    #[test]
    fn a_missing_exemplar_is_reported() {
        let (dir, metadata) = workspace(&[Crate::new("drs-model")]);
        write(
            dir.path(),
            "block.md",
            &guideline("crates/drs-gone/src/lib.rs", "pub fn first() {\n"),
            SOURCE,
        );

        let violations = check(&metadata).expect("checks");

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .detail
                .contains("`crates/drs-gone/src/lib.rs`")
        );
    }

    #[test]
    fn a_guideline_without_an_exemplar_or_an_example_is_reported() {
        let (dir, metadata) = workspace(&[Crate::new("drs-model")]);
        write(
            dir.path(),
            "nameless.md",
            "# A building block\n\n## Example\n\n```rust\npub fn first() {\n```\n",
            SOURCE,
        );
        fs::write(
            dir.path().join(GUIDELINES).join("bare.md"),
            "# A building block\n\n**Exemplar**: `crates/drs-model/src/lib.rs`\n",
        )
        .expect("write the guideline");

        let violations = check(&metadata).expect("checks");

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].package, "docs/guidelines/bare.md");
        assert_eq!(violations[0].detail, "has no Example");
        assert_eq!(violations[1].package, "docs/guidelines/nameless.md");
        assert_eq!(violations[1].detail, "names no Exemplar");
    }

    #[test]
    fn a_workspace_without_guidelines_passes() {
        let (_dir, metadata) = workspace(&[Crate::new("drs-model")]);

        assert!(check(&metadata).expect("checks").is_empty());
    }
}
