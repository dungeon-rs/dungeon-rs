//! The shared shape of a failed check, and how it is reported.
use anyhow::Result;
use cli_colors::Colorizer;
use cli_table::{Cell, Style, Table, print_stderr};

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

/// Prints every violation, naming the crate and the rule it broke.
pub fn report(colorizer: &Colorizer, violations: &[Violation]) -> Result<()> {
    let mut sorted = violations.to_vec();
    sorted.sort();

    eprintln!("{}", colorizer.red("\nThe workspace breaks its rules:"));
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
        ])
        .bold(true);
    print_stderr(table)?;
    Ok(())
}
