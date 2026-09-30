//! The shared shape of a failed check, and how it is reported.
use anyhow::Result;
use cli_colors::Colorizer;
use cli_table::{Cell, Style, Table};

/// One crate breaking one rule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Violation {
    /// The crate that broke the rule.
    pub package: String,
    /// The short name of the rule that was broken.
    pub rule: &'static str,
    /// What exactly is wrong.
    pub detail: String,
}

impl Violation {
    pub fn new(package: impl Into<String>, rule: &'static str, detail: impl Into<String>) -> Self {
        Self {
            package: package.into(),
            rule,
            detail: detail.into(),
        }
    }
}

/// Renders every violation as a table naming the crate, the rule it broke, and the problem.
pub fn render(violations: &[Violation]) -> Result<String> {
    let mut sorted = violations.to_vec();
    sorted.sort();

    let table = sorted
        .iter()
        .map(|v| {
            vec![
                v.package.as_str().cell(),
                v.rule.cell(),
                v.detail.as_str().cell(),
            ]
        })
        .table()
        .title(vec![
            "Crate".cell().bold(true),
            "Rule".cell().bold(true),
            "Problem".cell().bold(true),
        ]);

    Ok(table.display()?.to_string())
}

/// Prints every violation to stderr.
pub fn report(colorizer: &Colorizer, violations: &[Violation]) -> Result<()> {
    eprintln!("{}", colorizer.red("\nThe workspace breaks its rules:"));
    eprintln!("{}", render(violations)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_violation_names_its_crate_rule_and_problem() {
        let output = render(&[
            Violation::new(
                "drs-editor",
                "Allowed dependencies",
                "may not depend on `drs-x`",
            ),
            Violation::new("drs-model", "Restricted externals", "may not use `bevy`"),
        ])
        .expect("renders");

        for expected in [
            "drs-editor",
            "Allowed dependencies",
            "may not depend on `drs-x`",
            "drs-model",
            "Restricted externals",
            "may not use `bevy`",
        ] {
            assert!(
                output.contains(expected),
                "missing `{expected}` in:\n{output}"
            );
        }
    }
}
