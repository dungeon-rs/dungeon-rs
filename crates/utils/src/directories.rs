//! Contains methods for working with directories that have special meanings to the software.
//!
//! This module attempts to follow the conventions for each platform's guidelines, and as a result
//! the paths generated for each platform may differ.
//!
//! Inspiration of this library was taken from [directories](https://crates.io/crates/directories).
//! The library wasn't used due to licensing issues.
//!
//! Below are the conventions followed for each platform:
//!
//! - Windows: [Known Folder](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx) API
//! - Linux: [XDG Base Directory](https://specifications.freedesktop.org/basedir-spec/latest/) specifications.
//! - macOS: [Standard Directories](https://developer.apple.com/library/content/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/FileSystemOverview/FileSystemOverview.html#//apple_ref/doc/uid/TP40010672-CH2-SW6).

use std::path::PathBuf;
use thiserror::Error;

/// Contains the errors that can occur when attempting to retrieve one of the known directories.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum DirectoryError {
    /// The path could not be retrieved because it (or one if it's descendants) does not exist.
    #[error("Could not find the requested {0} directory on the system")]
    NotFound(&'static str),
}

/// The namespace for the application.
pub fn config_path() -> Result<PathBuf, DirectoryError> {
    #[cfg(target_os = "macos")]
    return config_path_macos();

    #[cfg(target_os = "linux")]
    return config_path_linux();

    #[cfg(target_os = "windows")]
    return config_path_windows();
}

#[inline]
#[cfg(target_os = "macos")]
#[allow(clippy::missing_docs_in_private_items, clippy::missing_errors_doc)]
fn config_path_macos() -> Result<PathBuf, DirectoryError> {
    let home = std::env::home_dir().ok_or(DirectoryError::NotFound("home"))?;

    Ok(home.join("Library/Application Support/DungeonRS/config.toml"))
}

#[inline]
#[cfg(target_os = "linux")]
#[allow(clippy::missing_docs_in_private_items, clippy::missing_errors_doc)]
fn config_path_linux() -> Result<PathBuf, DirectoryError> {
    let xdg = microxdg::XdgApp::new("DungeonRS").ok_or(DirectoryError::NotFound("home"))?;

    Ok(xdg.app_config().unwrap())
}

#[inline]
#[cfg(target_os = "windows")]
#[allow(clippy::missing_docs_in_private_items, clippy::missing_errors_doc)]
fn config_path_windows() -> Result<PathBuf, DirectoryError> {
    let home = known_folders::get_known_folder_path(known_folders::KnownFolder::RoamingAppData)
        .ok_or(DirectoryError::NotFound("RoamingAppData"))?;

    Ok(home.join("DungeonRS/config"))
}
