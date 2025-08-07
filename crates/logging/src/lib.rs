#![doc = include_str!("../README.md")]

use bevy::log::tracing_subscriber::Layer;
use bevy::log::tracing_subscriber::fmt::layer;
use bevy::log::{DEFAULT_FILTER, Level, LogPlugin};
use std::str::FromStr;

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

/// Initialises logging for console applications.
///
/// It specifically configures a layer optimised for human reading and enables `tracing-indicatif`
/// integration for tracing.
///
/// # Errors
/// This method may return errors when initialising the global tracing subscriber fails.
#[cfg(feature = "console")]
pub fn console_logging(
    level: tracing::metadata::LevelFilter,
) -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    use bevy::log::BoxedLayer;
    use tracing_subscriber::Registry;
    use tracing_subscriber::layer::SubscriberExt;

    let indicatif_layer = tracing_indicatif::IndicatifLayer::default();
    let layer = layer()
        .with_file(false)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_level(true)
        .with_writer(indicatif_layer.get_stdout_writer())
        .compact()
        .with_filter(level);

    let layer: BoxedLayer = Box::new(vec![layer.boxed(), indicatif_layer.boxed()]);

    let subscriber = Registry::default();
    tracing::dispatcher::set_global_default(subscriber.with(layer).into())
}
