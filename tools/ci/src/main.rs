mod validate_features;

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
    /// Validates that all features in each crate in the workspace have been validated.
    #[clap(name = "validate-features")]
    ValidateFeatures,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::ValidateFeatures => validate_features::execute(),
    }
}
