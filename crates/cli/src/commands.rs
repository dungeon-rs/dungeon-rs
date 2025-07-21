//! Contains the commands available in the CLI.
//! Implementations of all commands are available in submodules.

pub mod assets;

use clap::Subcommand;
use crate::commands::assets::AssetsArgs;

#[derive(Debug, Subcommand)]
pub enum Commands {
    Assets(AssetsArgs)
}