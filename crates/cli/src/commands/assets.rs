use clap::{Args, Subcommand};

/// Manage asset library and packs
#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct AssetsArgs {
    #[command(subcommand)]
    command: AssetsCommands,
}

/// All commands available for assets.
#[derive(Debug, Subcommand)]
pub enum AssetsCommands {
    /// List all asset packs.
    List,
}

/// Executes the asset commands in the correct way.
pub fn execute(AssetsArgs { command }: AssetsArgs) -> anyhow::Result<()> {
    match command {
        AssetsCommands::List => Ok(())
    }
}
