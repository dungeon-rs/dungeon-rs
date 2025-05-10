use bevy::prelude::Event;

/// A Bevy event that's emitted when the export is progressing.
///
/// This event allows tracking progress on an [`ExportRequest`] event.
/// Given the export process tends to export in "tiles", this usually means a tile finished exporting.
#[derive(Debug, Event)]
pub struct ExportProgress {
    /// Pre-calculated progress ranging from `0.0` to `100.0`.
    /// This is calculated from the [`total`] and [`current`] properties.
    pub progress: f32,
    /// The total number of "steps" the export process will take.
    /// This usually correlates to the number of tiles needed to export + processing.
    pub total: u64,
    /// The current step the export process finished, see [`total`].
    pub current: u64,
    /// The status of the progress, providing additional information on what the process is executing.
    /// See [`ExportStatus`] for more details.
    pub status: ExportStatus,
}

/// Describes the current status of an export.
/// Used as metadata for a [`ExportProgress`] event.
#[derive(Debug, Clone, Copy)]
pub enum ExportStatus {
    /// The export is being prepared but hasn't started yet.
    /// This may mean a map is being loaded, temporary outputs are being set up or memory is being allocated.
    Preparing,
    /// The export is executing, this is the status for as long as tiles are being extracted.
    Exporting,
    /// All tiles have been extracted and are being compiled into a single output image.
    Processing,
}
