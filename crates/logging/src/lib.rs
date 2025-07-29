#![doc = include_str!("../README.md")]

use bevy::log::tracing_subscriber::Layer;
use bevy::log::tracing_subscriber::fmt::layer;
use bevy::log::{BoxedLayer, DEFAULT_FILTER, Level, LogPlugin};
use bevy::prelude::App;
use config::LogConfiguration;
use std::str::FromStr;

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
fn custom_layer(_app: &mut App) -> Option<BoxedLayer> {
    #[allow(unused_mut, reason = "Needs to be mutable in console + dev feature")]
    let mut layer = layer()
        .with_file(false)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_level(true);

    // Console feature takes precedence - return early
    #[cfg(feature = "console")]
    {
        let indicatif_layer = tracing_indicatif::IndicatifLayer::new();
        return Some(Box::new(vec![
            layer
                .with_writer(indicatif_layer.get_stdout_writer())
                .boxed(),
            indicatif_layer.boxed(),
        ]));
    }

    // Only execute when console feature is disabled
    #[cfg(not(feature = "console"))]
    {
        #[cfg(feature = "dev")]
        {
            layer = layer.with_ansi(true).with_file(true).with_line_number(true);
        }

        let configuration = _app.world().get_resource::<config::Configuration>();
        let output = configuration
            .and_then(|config| config.logging.output.clone())
            .unwrap_or(String::from("logs"));
        if let Some(configuration) = configuration
            && configuration.logging.write_file
        {
            Some(Box::new(vec![
                layer
                    .with_writer(tracing_appender::rolling::daily(output, "dungeonrs"))
                    .json(),
            ]))
        } else {
            Some(Box::new(vec![layer.boxed()]))
        }
    }
}
