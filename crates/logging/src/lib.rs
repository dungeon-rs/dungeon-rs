#![doc = include_str!("../README.md")]

mod console;

use bevy::log::tracing_subscriber::Layer;
use bevy::log::tracing_subscriber::fmt::layer;
use bevy::log::{DEFAULT_FILTER, Level, LogPlugin};
use std::str::FromStr;

#[cfg(feature = "console")]
pub use console::*;

/// Builds the [`LogPlugin`] for the application.
#[must_use]
pub fn log_plugin(config: &config::LogConfiguration) -> bevy::log::LogPlugin {
    LogPlugin {
        filter: format!("{DEFAULT_FILTER},{}", config.filter),
        level: Level::from_str(config.level.as_str()).unwrap_or(Level::INFO),
        custom_layer: |app| {
            let configuration = app
                .world()
                .get_resource::<config::Configuration>()
                .map(|config| &config.logging);

            if let Some(configuration) = configuration
                && configuration.write_file
            {
                let output = configuration.output.clone().unwrap_or(String::from("logs"));

                let layer = layer()
                    .with_file(false)
                    .with_thread_names(true)
                    .with_thread_ids(true)
                    .with_level(true)
                    .json()
                    .with_writer(tracing_appender::rolling::daily(output, "dungeonrs"));

                Some(layer.boxed())
            } else {
                None
            }
        },
    }
}
