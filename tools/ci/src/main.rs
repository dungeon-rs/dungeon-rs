mod documented_features;
mod missing_translations;
mod required_features;
mod translations;
mod workspace_features;

use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use clap::{Parser, Subcommand};
use cli_colors::Colorizer;
use strum::{EnumIter, IntoEnumIterator};

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
    /// Validates that all required features (such as the platform features) have been added
    #[clap(name = "required-features")]
    ValidateRequiredFeatures,
    /// Validates that all features in the workspace are correctly propagated.
    #[clap(name = "workspace-features")]
    ValidateWorkspaceFeatures,
    /// Validates that all translation keys (tracked with the `t!` macro) are translated.
    #[clap(name = "translations")]
    ValidateTranslations,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    execute(cli.command)
}

/// Executes the given command.
fn execute(command: Commands) -> Result<()> {
    let colorizer = Colorizer::new();
    let metadata = MetadataCommand::new()
        .manifest_path("../../Cargo.toml")
        .no_deps()
        .exec()
        .context("running `cargo metadata` failed")?;

    match command {
        Commands::All => run_all(),
        Commands::ValidateDocumentedFeatures => documented_features::execute(colorizer, metadata),
        Commands::ValidateRequiredFeatures => required_features::execute(colorizer, metadata),
        Commands::ValidateWorkspaceFeatures => workspace_features::execute(colorizer, metadata),
        Commands::ValidateTranslations => translations::execute(colorizer, metadata), // missing_translations::execute(colorizer, metadata),
    }
}

fn run_all() -> Result<()> {
    for command in Commands::iter() {
        if matches!(command, Commands::All) {
            continue;
        }

        execute(command)?;
    }

    Ok(())
}
