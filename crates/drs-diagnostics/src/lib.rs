#![doc = include_str!("../README.md")]

mod bundle;
mod crash;
mod logging;
mod reveal;

pub use bundle::{
    BUNDLE_MARKER, BundledFileError, BundledFiles, BundledFilesNotFound, locate_bundled_files,
};
pub use crash::{
    CrashHandler, CrashReport, announce, install_crash_handler, run_guarded, take_pending_report,
};
pub use logging::{
    BoxedLayer, DEFAULT_LEVEL, KEPT_LOG_FILES, LOG_FILE_PREFIX, LOG_FILE_SUFFIX, LevelFilter,
    Logging, LoggingError, level_filter, log_directory, start_logging, take_layer,
};
pub use reveal::{RevealError, reveal_logs};

/// Logs what the start found: first the log file's path, then the bundle directory, or the
/// locations tried when none is marked, or a warning when its marker names another version.
///
/// Called once the subscriber is installed, since nothing logged before it is kept.
pub fn announce_start(logging: &Logging, bundle: &Result<BundledFiles, BundledFilesNotFound>) {
    logging.announce();
    match bundle {
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
}
