mod architecture;
mod bevy_crates;
mod bundle_marker;
mod documented_features;
mod engine_events;
mod guideline_examples;
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
    /// Validates that Engine crates define and use no messages, events, or observers.
    #[clap(name = "engine-events")]
    ValidateEngineEvents,
    /// Validates that the bundle marker names the workspace's version.
    #[clap(name = "bundle-marker")]
    ValidateBundleMarker,
    /// Validates that every guideline's Example is taken, line by line, from its Exemplar file.
    #[clap(name = "guideline-examples")]
    ValidateGuidelineExamples,
    /// Validates that every Bevy crate in use is a narrow crate or restricted in `ARCHITECTURE.md`.
    #[clap(name = "bevy-crates")]
    ValidateBevyCrates,
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
        Commands::ValidateArchitecture => architecture(metadata)?.check(metadata),
        Commands::ValidateEngineEvents => engine_events::check(&architecture(metadata)?, metadata)?,
        Commands::ValidateBundleMarker => bundle_marker::check(metadata),
        Commands::ValidateGuidelineExamples => guideline_examples::check(metadata)?,
        Commands::ValidateBevyCrates => {
            bevy_crates::check(&architecture_markdown(metadata)?, metadata)?
        }
    })
}

/// The tables of `ARCHITECTURE.md`, read from the workspace.
fn architecture(metadata: &Metadata) -> Result<Architecture> {
    let path = metadata
        .workspace_root
        .join("docs/architecture/ARCHITECTURE.md");
    let markdown = std::fs::read_to_string(&path).with_context(|| format!("reading {path}"))?;
    Architecture::parse(&markdown)
}

/// The text of `ARCHITECTURE.md`, read from the workspace.
fn architecture_markdown(metadata: &Metadata) -> Result<String> {
    let path = metadata
        .workspace_root
        .join("docs/architecture/ARCHITECTURE.md");
    std::fs::read_to_string(&path).with_context(|| format!("reading {path}"))
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
