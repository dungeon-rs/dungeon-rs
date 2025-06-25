use std::collections::{HashMap, HashSet};

use anyhow::Result;
use cargo_metadata::{Dependency, Metadata, Package};
use cli_colors::Colorizer;
use cli_table::{Cell, Style, Table, print_stderr};

const PROPAGATED_FEATURES: &[&str] = &["dev"]; // `default` only needs to exist.

pub fn execute(colorizer: Colorizer, metadata: Metadata) -> Result<()> {
    let workspace_names: HashSet<_> = metadata
        .packages
        .iter()
        .filter(|pkg| {
            metadata.workspace_members.contains(&pkg.id) && !pkg.name.ends_with("_macros")
        })
        .map(|pkg| pkg.name.to_string().clone())
        .collect();

    let mut errors: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

    // Iterate over each crate in the workspace.
    for package in metadata
        .packages
        .iter()
        .filter(|pkg| metadata.workspace_members.contains(&pkg.id))
    {
        let mut missing = HashMap::new();
        check_dependency_feature_propagation(package, &workspace_names, &mut missing);

        if !missing.is_empty() {
            errors.insert(package.name.to_string(), missing);
        }
    }

    if errors.is_empty() {
        println!(
            "{}",
            colorizer.green("All crates propagate all required features 🎉")
        );
    } else {
        eprintln!(
            "❌ {}",
            colorizer.red("some packages are missing required features:")
        );
        for (package, errors) in errors {
            eprintln!(
                "{}[{}]",
                colorizer.red("package "),
                colorizer.bold(colorizer.red(&package)),
            );

            let table = errors
                .iter()
                .map(|(feature, missing)| vec![feature.cell(), missing.join(", ").cell()])
                .table()
                .title(vec!["feature".cell(), "missing".cell()])
                .bold(true);

            print_stderr(table)?;
        }

        std::process::exit(1);
    }

    Ok(())
}

/// Rule #2 – each workspace dependency must be re-exported in the crate's own feature list.
fn check_dependency_feature_propagation(
    package: &Package,
    workspace_names: &HashSet<String>,
    errors: &mut HashMap<String, Vec<String>>,
) {
    // Collect the names of *local* dependencies (i.e. other workspace members).
    let local_deps: Vec<&Dependency> = package
        .dependencies
        .iter()
        .filter(|dep| workspace_names.contains(&dep.name))
        .collect();

    if local_deps.is_empty() {
        return; // Nothing to verify.
    }

    for &feature in PROPAGATED_FEATURES {
        match package.features.get(feature) {
            Some(list) => {
                let mut missing: Vec<String> = vec![];
                for dep in &local_deps {
                    let required_token = format!("{}/{}", dep.name, feature);
                    if !list.contains(&required_token) {
                        missing.push(required_token.clone());
                    }
                }

                if !missing.is_empty() {
                    errors.insert(feature.to_string(), missing);
                }
            }
            None => {
                panic!(
                    "{}/Cargo.toml is missing the `[features].{}` table required for dependency propagation",
                    package.name, feature
                );
            }
        }
    }
}
