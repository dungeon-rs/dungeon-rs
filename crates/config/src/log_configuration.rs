//! Contains the configuration structures for logging.

use serialization::{Deserialize, Serialize};

/// A configuration block to control how the application should handle logging.
///
/// It allows configuring the filter (which parts log at which level) and the minimum level of the logging system.
#[derive(Debug, Serialize, Deserialize)]
pub struct LogConfiguration {
    /// A filter that indicates how to filter logging from various parts of the software.
    ///
    /// For syntax and information, see [`EnvFilter`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)
    pub filter: String,

    /// The minimum log level for logs to be emitted.
    ///
    /// The following values are valid:
    /// - `trace`
    /// - `debug`
    /// - `info`
    /// - `warn`
    /// - `error`
    ///
    /// Invalid or missing values will fall back to `info`.
    /// The `level` values are case-insensitive.
    pub level: String,

    /// Optional: the path where the logfiles should be written to, software will attempt to create
    /// this path if it doesn't exist.
    ///
    /// Defaults to the current working directory of the application if not set (or if it's relative,
    /// will be relative to the working directory).
    pub output: Option<String>,
}

impl Default for LogConfiguration {
    fn default() -> Self {
        Self {
            filter: String::from("io=trace"),
            level: String::from("info"),
            output: None,
        }
    }
}
