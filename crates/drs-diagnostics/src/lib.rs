#![doc = include_str!("../README.md")]

mod crash;
mod logging;
mod resources;
mod reveal;

pub use crash::{
    CrashHandler, CrashReport, announce, install_crash_handler, run_guarded, take_pending_report,
};
pub use logging::{
    BoxedLayer, DEFAULT_LEVEL, KEPT_LOG_FILES, LOG_FILE_PREFIX, LOG_FILE_SUFFIX, LevelFilter,
    Logging, LoggingError, level_filter, log_directory, start_logging, take_layer,
};
pub use resources::{
    RESOURCES_MARKER, ResourceError, Resources, ResourcesNotFound, locate_resources,
};
pub use reveal::{RevealError, reveal_logs};

/// Logs what the start found: first the log file's path, then the resource directory, or the
/// locations tried when none is marked, or a warning when its marker names another version.
///
/// Called once the subscriber is installed, since nothing logged before it is kept.
pub fn announce_start(logging: &Logging, resources: &Result<Resources, ResourcesNotFound>) {
    logging.announce();
    match resources {
        Ok(found) => {
            if found.version_differs() {
                tracing::warn!(
                    "the resource directory {} is marked for version {}, not this editor's",
                    found.root().display(),
                    found.version().unwrap_or_default()
                );
            } else {
                tracing::info!("bundled resources in {}", found.root().display());
            }
        }
        Err(missing) => tracing::error!("{missing}"),
    }
}
