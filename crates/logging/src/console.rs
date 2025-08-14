#![cfg(feature = "console")]
//! This module contains all functionality for logging integration when running as a console application.
//!
//! This ranges from setting up `tracing` to ensuring progress isn't clobbered by logging.

use bevy::log::BoxedLayer;
pub use indicatif::MultiProgress;
use std::io;
use tracing_subscriber::fmt::{MakeWriter, layer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};

/// This represents the integration between `tracing` and `indicatif`.
///
/// This is largely lifted from `tracing-indicatif` but doesn't integrate span tracking.
#[derive(Clone, Default)]
struct IndicatifWriter {
    /// The `MultiProgress` instance used to display progress.
    /// All progress instances added to this will automatically be suspended when `tracing` writes
    /// to stdout.
    pub progress: MultiProgress,
}

/// Initialises logging for console applications.
///
/// # Errors
/// This method may return errors when initialising the global tracing subscriber fails.
#[cfg(feature = "console")]
pub fn console_logging(
    level: tracing::metadata::LevelFilter,
) -> Result<MultiProgress, tracing::subscriber::SetGlobalDefaultError> {
    let console = IndicatifWriter::default();

    let layer = layer()
        .with_file(false)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_level(true)
        .compact()
        .with_writer(console.clone())
        .with_filter(level);

    let layer: BoxedLayer = Box::new(vec![layer.boxed()]);

    let subscriber = Registry::default();
    tracing::dispatcher::set_global_default(subscriber.with(layer).into())?;

    Ok(console.progress)
}

impl<'a> MakeWriter<'a> for IndicatifWriter {
    type Writer = IndicatifWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

impl io::Write for IndicatifWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.progress.suspend(|| io::stdout().write(buf))
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.progress.suspend(|| io::stdout().write_vectored(bufs))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.progress.suspend(|| io::stdout().flush())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.progress.suspend(|| io::stdout().write_all(buf))
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()> {
        self.progress.suspend(|| io::stdout().write_fmt(fmt))
    }
}
