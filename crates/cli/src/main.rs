#![doc = include_str!("../README.md")]

mod commands;

use crate::commands::Commands;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use tracing_subscriber::FmtSubscriber;

/// A command line interface with `DungeonRS`.
#[derive(Debug, Parser)]
#[command(name = "drs-cli")]
#[command(about, long_about = None)]
struct Cli {
    /// Verbosity of the command line.
    #[command(flatten)]
    verbosity: Verbosity<InfoLevel>,
    #[command(subcommand)]
    /// Commands comment
    command: Commands,
}

#[allow(missing_docs)]
#[allow(clippy::missing_panics_doc)]
#[allow(clippy::missing_errors_doc)]
#[allow(clippy::missing_docs_in_private_items)]
fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    FmtSubscriber::builder()
        .with_max_level(args.verbosity)
        .without_time()
        .compact()
        .init();

    match args.command {
        Commands::Assets(args) => commands::assets::execute(args)?,
    }

    Ok(())
}
