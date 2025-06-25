//! This module contains the command to validate that all required features in the workspace are present.
use anyhow::Result;
use cargo_metadata::Metadata;
use cli_colors::Colorizer;
use cli_table::format::Justify;
use cli_table::{Cell, Style, Table, print_stdout};
use std::collections::HashMap;

/// The names of features we require to be present in each sub-crate.
pub const REQUIRED_FEATURES: [&str; 2] = ["default", "dev"];

pub fn execute(colorizer: Colorizer, metadata: Metadata) -> Result<()> {
    let mut errors = HashMap::new();

    for package in metadata
        .workspace_members
        .iter()
        .filter_map(|id| metadata.packages.iter().find(|pkg| &pkg.id == id))
    {
        let missing = validate_package_features(package);
        if !missing.is_empty() {
            errors.insert(package.name.clone(), missing);
        }
    }

    if !errors.is_empty() {
        eprintln!(
            "{}",
            colorizer.red("\nSome crates are missing required features:")
        );

        let table = errors
            .iter()
            .map(|(key, value)| vec![key.cell(), value.join(", ").cell().justify(Justify::Center)])
            .table()
            .title(vec!["Package".cell().bold(true), "Missing features".cell()])
            .bold(true);
        print_stdout(table)?;

        std::process::exit(1);
    }

    println!(
        "{}",
        colorizer.green("All crates contain their required features. 🎉")
    );
    Ok(())
}

/// For one package, read its declared features (excluding "default"), then ensure each name
/// appears in its README.md (as a substring).
fn validate_package_features(pkg: &cargo_metadata::Package) -> Vec<&str> {
    // Check which features are missing for this package:
    let mut missing_features: Vec<_> = Vec::from(REQUIRED_FEATURES);
    for declared in pkg.features.keys() {
        if let Some(index) = missing_features
            .iter()
            .position(|feature| declared.eq(feature))
        {
            missing_features.remove(index);
        }
    }

    missing_features
}
