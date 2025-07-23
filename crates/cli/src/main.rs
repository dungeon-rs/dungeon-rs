#![doc = include_str!("../README.md")]

mod commands;

use crate::commands::Commands;
use clap::Parser;

/// A command line interface with `DungeonRS`.
#[derive(Debug, Parser)]
#[command(name = "drs-cli")]
#[command(about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    /// Commands comment
    command: Commands,
}

#[allow(missing_docs)]
#[allow(clippy::missing_panics_doc)]
#[allow(clippy::missing_errors_doc)]
#[allow(clippy::missing_docs_in_private_items)]
fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .env()
        .without_timestamps()
        .init()?;

    let args = Cli::parse();
    match args.command {
        Commands::Assets(args) => commands::assets::execute(args)?,
    }

    Ok(())
}
