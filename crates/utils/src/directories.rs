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

/// Attempts to retrieve the current platform's configuration directory.
///
/// # Errors
///
/// The underlying reason for this method failing depends on the platform, however it always boils down
/// to the base directory (`$HOME`, `%APPDATA%`, ...) not being found.
pub fn config_path() -> Result<PathBuf, DirectoryError> {
    #[cfg(target_os = "macos")]
    return config_path_macos();

    #[cfg(target_os = "linux")]
    return config_path_linux();

    #[cfg(target_os = "windows")]
    return config_path_windows();
}

/// Attempts to retrieve the current platform's cache directory.
///
/// # Errors
///
/// The underlying reason for this method failing depends on the platform, however it always boils down
/// to the base directory (`$HOME`, `%APPDATA%`, ...) not being found.
pub fn cache_path() -> Result<PathBuf, DirectoryError> {
    #[cfg(target_os = "macos")]
    return cache_path_macos();

    #[cfg(target_os = "linux")]
    return cache_path_linux();

    #[cfg(target_os = "windows")]
    return cache_path_windows();
}

/// Attempts to retrieve the path to the resource bundle of the application.
/// For most platforms this is the path where the executable is located, but under OSX this is a
/// separate `Resources` folder.
///
/// When running in the cargo workspace (typically in a development environment), this method will
/// return the path to the cargo root, as that's where all resources (should be) located.
///
/// Under all operating systems this folder should be considered read-only, even if under some it is not.
///
/// # Errors
///
/// This method may fail when it cannot determine the location of the current executable.
pub fn resource_path() -> Result<PathBuf, DirectoryError> {
    // When running in dev, and we have `CARGO_MANIFEST_DIR` set (auto set by Cargo),
    // we can attempt to resolve the workspace root, it's dirty, but it works (and won't make it to
    // a release build anyway).
    #[cfg(feature = "dev")]
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let start_path = PathBuf::from(manifest_dir);

        // Walk up the directory tree looking for the workspace root
        for ancestor in start_path.ancestors() {
            let cargo_toml = ancestor.join("Cargo.toml");

            if cargo_toml.exists() {
                // Check if this is a workspace root by reading the file
                if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                    if content.contains("[workspace]") {
                        return Ok(ancestor.to_path_buf());
                    }
                }
            }
        }

        // If no workspace found, return the original manifest dir
        return Ok(start_path);
    }

    // Production: Get an executable path
    let exe_path =
        std::env::current_exe().map_err(|_| DirectoryError::NotFound("executable path"))?;

    // Specifically for OSX: we'll fetch the `Resources` folder in the app bundle.
    #[cfg(target_os = "macos")]
    {
        exe_path // *.app/Contents/MacOS/binary
            .parent() // *.app/Contents/MacOS
            .and_then(std::path::Path::parent) // *.app/Contents
            .map(|path| path.join("Resources"))
            .ok_or(DirectoryError::NotFound("resources"))
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Default: Return executable directory
        exe_path
            .parent()
            .map(std::path::Path::to_path_buf)
            .ok_or(DirectoryError::NotFound("executable directory"))
    }
}

#[inline]
#[cfg(target_os = "macos")]
#[allow(clippy::missing_docs_in_private_items, clippy::missing_errors_doc)]
fn config_path_macos() -> Result<PathBuf, DirectoryError> {
    let home = std::env::home_dir().ok_or(DirectoryError::NotFound("home"))?;

    Ok(home.join("Library/Application Support/DungeonRS/config"))
}

#[inline]
#[cfg(target_os = "linux")]
#[allow(clippy::missing_docs_in_private_items, clippy::missing_errors_doc)]
fn config_path_linux() -> Result<PathBuf, DirectoryError> {
    let path = std::env::var("XDG_CONFIG_HOME")
        .map(|home| PathBuf::from(home).join("DungeonRS"))
        .map_err(|_| {
            std::env::var("HOME").map(|home| PathBuf::from(home).join(".config/DungeonRS"))
        })
        .map_err(|_| DirectoryError::NotFound("XDG_CONFIG_HOME"))?;

    Ok(path)
}

#[inline]
#[cfg(target_os = "windows")]
#[allow(clippy::missing_docs_in_private_items, clippy::missing_errors_doc)]
fn config_path_windows() -> Result<PathBuf, DirectoryError> {
    let home = known_folders::get_known_folder_path(known_folders::KnownFolder::RoamingAppData)
        .ok_or(DirectoryError::NotFound("RoamingAppData"))?;

    Ok(home.join("DungeonRS/config"))
}

#[inline]
#[cfg(target_os = "macos")]
#[allow(clippy::missing_docs_in_private_items, clippy::missing_errors_doc)]
fn cache_path_macos() -> Result<PathBuf, DirectoryError> {
    let home = std::env::home_dir().ok_or(DirectoryError::NotFound("home"))?;

    Ok(home.join("Library/Cache/DungeonRS"))
}

#[inline]
#[cfg(target_os = "linux")]
#[allow(clippy::missing_docs_in_private_items, clippy::missing_errors_doc)]
fn cache_path_linux() -> Result<PathBuf, DirectoryError> {
    let path = std::env::var("XDG_CACHE_HOME")
        .map(|home| PathBuf::from(home).join("DungeonRS"))
        .map_err(|_| std::env::var("HOME").map(|home| PathBuf::from(home).join(".cache/DungeonRS")))
        .map_err(|_| DirectoryError::NotFound("XDG_CACHE_HOME"))?;

    Ok(path)
}

#[inline]
#[cfg(target_os = "windows")]
#[allow(clippy::missing_docs_in_private_items, clippy::missing_errors_doc)]
fn cache_path_windows() -> Result<PathBuf, DirectoryError> {
    let home = known_folders::get_known_folder_path(known_folders::KnownFolder::LocalAppData)
        .ok_or(DirectoryError::NotFound("LocalAppData"))?;

    Ok(home.join("DungeonRS/cache"))
}
