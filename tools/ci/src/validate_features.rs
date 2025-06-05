//! This module contains the command to validate that all features in the workspace have been documented.

use anyhow::{Context, Result, anyhow};
use cargo_metadata::MetadataCommand;
use cli_colors::Colorizer;
use cli_table::format::Justify;
use cli_table::{Cell, Style, Table, print_stdout};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn execute() -> Result<()> {
    let colorizer = Colorizer::new();
    let metadata = MetadataCommand::new()
        .manifest_path("../../Cargo.toml")
        .no_deps()
        .exec()
        .context("running `cargo metadata` failed")?;

    let mut errors = HashMap::new();

    for package in metadata
        .workspace_members
        .iter()
        .filter_map(|id| metadata.packages.iter().find(|pkg| &pkg.id == id))
    {
        validate_package_features(package, &mut errors)?;
    }

    if !errors.is_empty() {
        eprintln!(
            "{}",
            colorizer.red("\nSome crates have undocumented features:")
        );

        let table = errors
            .iter()
            .map(|(key, value)| vec![key.cell(), value.join(", ").cell().justify(Justify::Center)])
            .table()
            .title(vec![
                "Package".cell().bold(true),
                " Undocumented features".cell(),
            ])
            .bold(true);
        print_stdout(table)?;

        std::process::exit(1);
    }

    println!(
        "{}",
        colorizer.green("All crate READMEs contain their declared features. 🎉")
    );
    Ok(())
}

/// For one package, read its declared features (excluding "default"), then ensure each name
/// appears in its README.md (as a substring).
fn validate_package_features(
    pkg: &cargo_metadata::Package,
    errors: &mut HashMap<String, Vec<String>>,
) -> Result<()> {
    let manifest_path = Path::new(&pkg.manifest_path);
    let crate_dir = manifest_path
        .parent()
        .ok_or_else(|| anyhow!("could not get parent directory of {}", pkg.manifest_path))?;
    let readme_path = crate_dir.join("README.md");

    // 1. If there is no README.md, we can either treat that as an error or skip. Let's treat as error
    if !readme_path.exists() {
        return Err(anyhow!(
            "crate `{}`: missing README.md at {}",
            pkg.name,
            readme_path.display()
        ));
    }

    let readme_contents = fs::read_to_string(&readme_path).with_context(|| {
        format!(
            "failed to read README.md for crate `{}` at {}",
            pkg.name,
            readme_path.display()
        )
    })?;

    // We can skip the default feature.
    let declared_features: Vec<String> = pkg
        .features
        .keys()
        .filter(|&feat_name| feat_name != "default")
        .cloned()
        .collect();

    if declared_features.is_empty() {
        return Ok(());
    }

    // for each feature, ensure it appears in the README.md (case‐sensitive substring match)
    let mut missing = Vec::new();
    for feat in &declared_features {
        if !readme_contents.contains(feat) {
            missing.push(feat.clone());
        }
    }

    // if any are missing, return an error listing them
    if !missing.is_empty() {
        errors.insert(pkg.name.to_string(), missing);
    }

    Ok(())
}
