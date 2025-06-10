mod validate_documented_features;
mod validate_required_features;

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
        Commands::ValidateDocumentedFeatures => {
            validate_documented_features::execute(colorizer, metadata)
        }
        Commands::ValidateRequiredFeatures => {
            validate_required_features::execute(colorizer, metadata)
        }
    }
}
