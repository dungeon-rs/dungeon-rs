mod architecture;
mod documented_features;
mod required_features;
#[cfg(test)]
mod testing;
mod violation;
mod workspace_features;

use anyhow::{Context, Result};
use architecture::Architecture;
use cargo_metadata::{Metadata, MetadataCommand};
use clap::{Parser, Subcommand};
use cli_colors::Colorizer;
use strum::{EnumIter, IntoEnumIterator};
use violation::Violation;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, EnumIter)]
pub enum Commands {
    /// Runs all other commands in this tool.
    #[clap(name = "all")]
    All,
    /// Validates that all features in each crate in the workspace have been documented.
    #[clap(name = "documented-features")]
    ValidateDocumentedFeatures,
    /// Validates that every crate declares the required features (`default` and `dev`).
    #[clap(name = "required-features")]
    ValidateRequiredFeatures,
    /// Validates that all features in the workspace are correctly propagated.
    #[clap(name = "workspace-features")]
    ValidateWorkspaceFeatures,
    /// Validates that the workspace follows the Dependencies tables of `ARCHITECTURE.md`.
    #[clap(name = "architecture")]
    ValidateArchitecture,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let colorizer = Colorizer::new();
    let metadata = MetadataCommand::new()
        .manifest_path(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.toml"))
        .no_deps()
        .exec()
        .context("running `cargo metadata` failed")?;

    let violations = run(cli.command, &metadata)?;

    if violations.is_empty() {
        println!("{}", colorizer.green("The workspace follows its rules."));
        return Ok(());
    }

    violation::report(&colorizer, &violations)?;
    std::process::exit(1);
}

/// Runs a single check against the workspace.
fn run(command: Commands, metadata: &Metadata) -> Result<Vec<Violation>> {
    Ok(match command {
        Commands::All => run_all(metadata)?,
        Commands::ValidateDocumentedFeatures => documented_features::check(metadata),
        Commands::ValidateRequiredFeatures => required_features::check(metadata),
        Commands::ValidateWorkspaceFeatures => workspace_features::check(metadata),
        Commands::ValidateArchitecture => {
            let path = metadata
                .workspace_root
                .join("docs/architecture/ARCHITECTURE.md");
            let markdown =
                std::fs::read_to_string(&path).with_context(|| format!("reading {path}"))?;
            Architecture::parse(&markdown)?.check(metadata)
        }
    })
}

/// Runs every check, so one run reports everything that is wrong.
fn run_all(metadata: &Metadata) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();

    for command in Commands::iter() {
        if matches!(command, Commands::All) {
            continue;
        }

        violations.extend(run(command, metadata)?);
    }

    Ok(violations)
}
