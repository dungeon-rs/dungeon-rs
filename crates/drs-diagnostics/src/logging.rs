//! Logging to daily files in the editor's own directories.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use thiserror::Error;
use tracing_subscriber::Layer;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::registry::Registry;

/// A boxed layer over the registry: the type Bevy's log plugin takes as its custom layer.
pub type BoxedLayer = Box<dyn Layer<Registry> + Send + Sync + 'static>;

/// The part of a log file's name before its date.
const LOG_FILE_PREFIX: &str = "dungeon-rs";

/// The extension of a log file.
const LOG_FILE_SUFFIX: &str = "log";

/// How many daily log files are kept.
pub const KEPT_LOG_FILES: usize = 7;

/// The level without `RUST_LOG`: the editor's own events, without the engine's noise.
pub const DEFAULT_LEVEL: &str = "info,wgpu=error,naga=warn";

/// The layer built by [`start_logging`], until the Host takes it.
static LAYER: Mutex<Option<BoxedLayer>> = Mutex::new(None);

/// The log directory and whether a file is written in it, for the crash handler.
static STATE: Mutex<Option<State>> = Mutex::new(None);

/// Where logging goes.
#[derive(Debug, Clone)]
struct State {
    /// The log directory.
    directory: PathBuf,
    /// Whether the daily file is written, or the terminal alone.
    file: bool,
}

/// Why logging goes to the terminal only.
#[derive(Debug, Error)]
pub enum LoggingError {
    /// The log directory could not be created.
    #[error("the log directory {path} could not be created: {source}")]
    Directory {
        /// The directory.
        path: PathBuf,
        /// What the file system said.
        #[source]
        source: std::io::Error,
    },
    /// The log file could not be opened.
    #[error("the log file in {path} could not be opened: {source}")]
    File {
        /// The directory the file was to be in.
        path: PathBuf,
        /// What the appender said.
        #[source]
        source: tracing_appender::rolling::InitError,
    },
}

/// What [`start_logging`] set up.
#[derive(Debug)]
pub struct Logging {
    /// The log directory, whether or not a file could be written in it.
    pub directory: PathBuf,
    /// The current log file; `None` when logging goes to the terminal only.
    pub file: Option<PathBuf>,
    /// Why logging goes to the terminal only, when it does.
    pub failure: Option<LoggingError>,
}

impl Logging {
    /// Logs the first entry of the run: where the log file is, or why there is none.
    pub(crate) fn log_location(&self) {
        match (&self.file, &self.failure) {
            (Some(file), _) => tracing::info!("logging to {}", file.display()),
            (None, Some(failure)) => {
                tracing::warn!("logging to the terminal only: {failure}");
            }
            (None, None) => tracing::warn!("logging to the terminal only"),
        }
    }
}

/// Which entries are logged, derived from `RUST_LOG`.
#[derive(Debug)]
pub struct LogDirectives {
    /// The filter the file layer applies.
    pub filter: EnvFilter,
    /// What is said on the terminal when `RUST_LOG` was malformed and the default applies.
    pub fallback: Option<String>,
}

impl LogDirectives {
    /// The directives derived from the `RUST_LOG` variable of this process.
    #[must_use]
    pub fn from_environment() -> Self {
        log_directives(std::env::var(EnvFilter::DEFAULT_ENV).ok().as_deref())
    }
}

/// Derives the filter from a `RUST_LOG` value: the value's directives alone when it is
/// well-formed, as Bevy's log plugin reads it for the terminal, so the file and the terminal
/// agree; the default directives otherwise, with a malformed value said in the fallback note.
#[must_use]
pub fn log_directives(rust_log: Option<&str>) -> LogDirectives {
    let defaults = || EnvFilter::builder().parse_lossy(DEFAULT_LEVEL);
    let Some(value) = rust_log.map(str::trim).filter(|value| !value.is_empty()) else {
        return LogDirectives {
            filter: defaults(),
            fallback: None,
        };
    };
    match EnvFilter::builder().parse(value) {
        Ok(filter) => LogDirectives {
            filter,
            fallback: None,
        },
        Err(error) => LogDirectives {
            filter: defaults(),
            fallback: Some(format!(
                "RUST_LOG is malformed ({error}); logging at the default level {DEFAULT_LEVEL}"
            )),
        },
    }
}

/// The log directory: the one the editor's directories resolved to, or `logs` under the
/// platform's temporary directory when they resolved to none, so that logging has somewhere to
/// go on a platform that names no cache directory.
#[must_use]
pub fn log_directory(resolved: Option<PathBuf>) -> PathBuf {
    resolved.unwrap_or_else(|| std::env::temp_dir().join("dungeon-rs").join("logs"))
}

/// Starts logging to a daily file in `directory`, keeping the last seven, with entries written
/// as they happen.
///
/// The directory is created first. The layer that writes the file waits for [`take_layer`];
/// until the Host composes it into a subscriber, nothing is written. When the directory or the
/// file cannot be created, the reason is printed to the terminal and logging goes there only,
/// so the editor starts regardless. A malformed `RUST_LOG` is said on the terminal after that,
/// so the first line on the terminal is about where the log went.
#[must_use]
pub fn start_logging(directory: &Path, directives: LogDirectives) -> Logging {
    let LogDirectives { filter, fallback } = directives;
    let logging = match appender(directory) {
        Ok(appender) => {
            let layer = tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(appender)
                .with_filter(filter)
                .boxed();
            *LAYER.lock().unwrap_or_else(PoisonError::into_inner) = Some(layer);
            *STATE.lock().unwrap_or_else(PoisonError::into_inner) = Some(State {
                directory: directory.to_path_buf(),
                file: true,
            });
            Logging {
                directory: directory.to_path_buf(),
                file: Some(directory.join(log_file_name(time::OffsetDateTime::now_utc()))),
                failure: None,
            }
        }
        Err(failure) => {
            eprintln!("logging to the terminal only: {failure}");
            *STATE.lock().unwrap_or_else(PoisonError::into_inner) = Some(State {
                directory: directory.to_path_buf(),
                file: false,
            });
            Logging {
                directory: directory.to_path_buf(),
                file: None,
                failure: Some(failure),
            }
        }
    };
    if let Some(note) = fallback {
        eprintln!("{note}");
    }
    logging
}

/// The layer [`start_logging`] built, once; `None` before logging started, after the layer was
/// taken, or when logging goes to the terminal only.
///
/// Bevy's log plugin takes a plain function for its custom layer, so the Host hands it
/// `|_| drs_diagnostics::take_layer()`.
#[must_use]
pub fn take_layer() -> Option<BoxedLayer> {
    LAYER.lock().unwrap_or_else(PoisonError::into_inner).take()
}

/// The log file entries go to at `now`, or `None` while logging goes to the terminal only or
/// has not started.
pub(crate) fn current_log_file(now: time::OffsetDateTime) -> Option<PathBuf> {
    let state = STATE.lock().unwrap_or_else(PoisonError::into_inner);
    let state = state.as_ref()?;
    state.file.then(|| state.directory.join(log_file_name(now)))
}

/// Deletes the oldest daily log files beyond the kept count, oldest by the date in the name,
/// leaving room for the file of the day of `now`; crash reports and anything else in the
/// directory are never touched.
fn prune_by_date(directory: &Path, now: time::OffsetDateTime) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let today = log_file_name(now);
    let prefix = format!("{LOG_FILE_PREFIX}.");
    let suffix = format!(".{LOG_FILE_SUFFIX}");
    let mut dated: Vec<(String, PathBuf)> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_owned();
            let date = name.strip_prefix(&prefix)?.strip_suffix(&suffix)?;
            (name != today).then(|| (date.to_owned(), entry.path()))
        })
        .collect();
    dated.sort();
    let room_for_today = KEPT_LOG_FILES.saturating_sub(1);
    for (_, path) in dated
        .iter()
        .take(dated.len().saturating_sub(room_for_today))
    {
        // A file that cannot be deleted stays; the next start tries again.
        let _ = std::fs::remove_file(path);
    }
}

/// The name of the log file for the UTC day of `now`.
fn log_file_name(now: time::OffsetDateTime) -> String {
    let mut name = String::new();
    // Writing to a `String` cannot fail.
    let _ = write!(name, "{LOG_FILE_PREFIX}.");
    let date = time::macros::format_description!("[year]-[month]-[day]");
    match now.date().format(date) {
        Ok(formatted) => name.push_str(&formatted),
        Err(_) => name.push_str("undated"),
    }
    let _ = write!(name, ".{LOG_FILE_SUFFIX}");
    name
}

/// The daily appender over `directory`, created first so the appender finds the directory it
/// prunes, and pruned by the dates in the file names before the appender prunes by creation
/// time at midnight, so a start deletes the oldest files whatever the file system remembers.
///
/// # Errors
///
/// [`LoggingError`] when the directory or the file cannot be created.
fn appender(
    directory: &Path,
) -> Result<tracing_appender::rolling::RollingFileAppender, LoggingError> {
    std::fs::create_dir_all(directory).map_err(|source| LoggingError::Directory {
        path: directory.to_path_buf(),
        source,
    })?;
    prune_by_date(directory, time::OffsetDateTime::now_utc());
    tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix(LOG_FILE_PREFIX)
        .filename_suffix(LOG_FILE_SUFFIX)
        .max_log_files(KEPT_LOG_FILES)
        .build(directory)
        .map_err(|source| LoggingError::File {
            path: directory.to_path_buf(),
            source,
        })
}
