//! Opening the log directory in the platform's file manager.

use std::path::{Path, PathBuf};
use thiserror::Error;

/// Why the log directory could not be shown.
#[derive(Debug, Error)]
pub enum RevealError {
    /// The directory could not be created.
    #[error("the log directory {path} could not be created: {source}")]
    Directory {
        /// The directory.
        path: PathBuf,
        /// What the file system said.
        #[source]
        source: std::io::Error,
    },
    /// The platform's file manager could not be started.
    #[error("the file manager ({command}) could not be started: {source}")]
    FileManager {
        /// The command that was to open it.
        command: &'static str,
        /// What the operating system said.
        #[source]
        source: std::io::Error,
    },
}

/// Opens `directory` in the platform's file manager, creating it first when it does not exist.
///
/// # Errors
///
/// [`RevealError`] when the directory cannot be created or the file manager cannot be started.
pub fn reveal_logs(directory: &Path) -> Result<(), RevealError> {
    std::fs::create_dir_all(directory).map_err(|source| RevealError::Directory {
        path: directory.to_path_buf(),
        source,
    })?;
    let command = file_manager();
    std::process::Command::new(command)
        .arg(directory)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(drop)
        .map_err(|source| RevealError::FileManager { command, source })
}

/// The command that opens a folder on this platform.
fn file_manager() -> &'static str {
    if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    }
}
