mod validate_documented_features;
mod validate_required_features;

use anyhow::Result;
use clap::{Parser, Subcommand};

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

    match cli.command {
        Commands::ValidateDocumentedFeatures => validate_documented_features::execute(),
        Commands::ValidateRequiredFeatures => validate_required_features::execute(),
    }
}
