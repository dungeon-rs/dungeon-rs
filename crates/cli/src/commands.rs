//! Contains the commands available in the CLI.
//! Implementations of all commands are available in submodules.

pub mod assets;

use crate::commands::assets::AssetsArgs;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Commands {
    Assets(AssetsArgs),
}
