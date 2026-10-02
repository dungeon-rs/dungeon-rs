#![doc = include_str!("../README.md")]

mod bundle;
mod crash;
mod logging;
mod reveal;

pub use bundle::{
    BUNDLE_MARKER, BundledFileError, BundledFiles, BundledFilesNotFound, locate_bundled_files,
    locate_bundled_files_of_this_executable,
};
pub use crash::{
    CrashHandler, CrashReport, announce, dialogs_possible, install_crash_handler, run_guarded,
    take_pending_report,
};
pub use logging::{
    BoxedLayer, DEFAULT_LEVEL, KEPT_LOG_FILES, LOG_FILE_PREFIX, LOG_FILE_SUFFIX, LogDirectives,
    Logging, LoggingError, log_directives, log_directory, start_logging, take_layer,
};
pub use reveal::{RevealError, reveal_logs};

use std::path::PathBuf;

/// The editor, as the crash report, the dialog, and the bundle marker name it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Product {
    /// The editor's name, as the Author knows it.
    pub name: &'static str,
    /// The editor's version.
    pub version: &'static str,
}

/// What the start of diagnostics set up and found, for the Editor to keep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Started {
    /// The log directory, where log files and crash reports are.
    pub logs: PathBuf,
    /// The current log file; `None` when logging goes to the terminal only.
    pub log_file: Option<PathBuf>,
    /// The bundle directory, or the locations tried when none is marked as the editor's.
    pub bundled_files: Result<BundledFiles, BundledFilesNotFound>,
}

impl Started {
    /// The directory the default asset source is rooted at, absolute, so that the engine's own
    /// guess at an asset root plays no part: the bundle directory, or, when none is marked, the
    /// first location tried, which holds no marker and so nothing the engine should read.
    #[must_use]
    pub fn asset_root(&self) -> PathBuf {
        match &self.bundled_files {
            Ok(found) => found.root().to_path_buf(),
            Err(missing) => missing.asset_root(),
        }
    }
}

/// Logs what the start found: first the log file's path, or why there is none; then the bundle
/// directory, or the locations tried when none is marked, or a warning when its marker names
/// another version. What was found is returned for the Editor.
///
/// Called once the subscriber is installed, since nothing logged before it is kept.
pub fn log_start(
    logging: &Logging,
    bundled_files: Result<BundledFiles, BundledFilesNotFound>,
) -> Started {
    logging.log_location();
    match &bundled_files {
        Ok(found) => {
            if found.version_differs() {
                tracing::warn!(
                    "the bundle directory {} is marked for version {}, not this editor's",
                    found.root().display(),
                    found.version().unwrap_or_default()
                );
            } else {
                tracing::info!("Bundled Files in {}", found.root().display());
            }
        }
        Err(missing) => tracing::error!("{missing}"),
    }
    Started {
        logs: logging.directory.clone(),
        log_file: logging.file.clone(),
        bundled_files,
    }
}
