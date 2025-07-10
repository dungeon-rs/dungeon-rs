#![doc = include_str!("../README.md")]

use bevy::log::tracing_subscriber::Layer;
use bevy::log::tracing_subscriber::fmt::layer;
use bevy::log::{BoxedLayer, DEFAULT_FILTER, Level, LogPlugin};
use bevy::prelude::App;
use config::{Configuration, LogConfiguration};
use std::str::FromStr;
use tracing_appender::rolling::daily;

/// Builds the [`LogPlugin`] for the application.
#[must_use]
pub fn log_plugin(config: &LogConfiguration) -> LogPlugin {
    LogPlugin {
        filter: format!("{DEFAULT_FILTER},{}", config.filter),
        level: Level::from_str(config.level.as_str()).unwrap_or(Level::INFO),
        custom_layer,
    }
}

/// This method builds a custom logging layer for `trace` and `log`.
///
/// This allows us to configure how the software should log, where and so forth.
#[allow(
    clippy::unnecessary_wraps,
    reason = "Bevy's API requires wrapping in an Option<T>"
)]
fn custom_layer(app: &mut App) -> Option<BoxedLayer> {
    let output = app
        .world()
        .get_resource::<Configuration>()
        .and_then(|config| config.logging.output.clone())
        .unwrap_or(String::from("logs"));

    let mut layer = layer()
        .with_file(false)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_writer(daily(output, "dungeonrs"))
        .json();

    #[cfg(feature = "dev")]
    {
        layer = layer.with_ansi(true).with_file(true).with_line_number(true);
    }

    layer = layer.with_level(true);

    Some(Box::new(vec![layer.boxed()]))
}
