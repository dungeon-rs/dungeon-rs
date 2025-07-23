//! Contains the commands available in the CLI.
//! Implementations of all commands are available in submodules.

pub mod assets;

use clap::Subcommand;

/// All commands available in the CLI.
#[derive(Debug, Subcommand)]
#[non_exhaustive]
pub enum Commands {
    /// Manage asset library and packs
    Assets(assets::Args),
}
