use clap::{Args, Subcommand};

/// Manage asset library and packs
#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct AssetsArgs {
    #[command(subcommand)]
    command: Option<AssetsCommands>,
}

/// All commands available for assets.
#[derive(Debug, Subcommand)]
pub enum AssetsCommands {
    /// List all asset packs.
    List
}

pub fn execute(AssetsArgs { command }: AssetsArgs) -> anyhow::Result<()> {
    println!("{:?}", command);
    Ok(())
}
