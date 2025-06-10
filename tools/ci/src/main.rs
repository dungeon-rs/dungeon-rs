mod documented_features;
mod required_features;
mod workspace_features;

use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use clap::{Parser, Subcommand};
use cli_colors::Colorizer;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Validates that all features in each crate in the workspace have been documented.
    #[clap(name = "documented-features")]
    ValidateDocumentedFeatures,
    /// Validates that all required features (such as the platform features) have been added
    #[clap(name = "required-features")]
    ValidateRequiredFeatures,
    #[clap(name = "workspace-features")]
    ValidateWorkspaceFeatures,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let colorizer = Colorizer::new();
    let metadata = MetadataCommand::new()
        .manifest_path("../../Cargo.toml")
        .no_deps()
        .exec()
        .context("running `cargo metadata` failed")?;

    match cli.command {
        Commands::ValidateDocumentedFeatures => documented_features::execute(colorizer, metadata),
        Commands::ValidateRequiredFeatures => required_features::execute(colorizer, metadata),
        Commands::ValidateWorkspaceFeatures => workspace_features::execute(colorizer, metadata),
    }
}
