use bevy::prelude::Event;

/// A Bevy event that's emitted when the export is progressing.
///
/// This event allows tracking progress on an [`crate::export::ExportRequest`] event.
/// Given the export process tends to export in "frames", this usually means a frame started exporting.
#[derive(Debug, Event)]
pub struct ExportProgress {
    /// Pre-calculated progress ranging from `0.0` to `100.0`.
    /// This is calculated from the [`Self::total`] and [`Self::current`] properties.
    pub progress: f32,
    /// The total number of "steps" the export process will take.
    /// This usually correlates to the number of frames needed to export + processing.
    pub total: u64,
    /// The current step the export process finished, see [`Self::total`].
    pub current: u64,
}
