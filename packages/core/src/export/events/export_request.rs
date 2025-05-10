use bevy::prelude::Event;
use std::path::PathBuf;

/// A Bevy event that is dispatched when the user requests an export.
/// Usually this is done through the editor when selecting "export"
/// or by the headless exporter.
///
/// The request contains the information about how to construct the export (screenshot).
#[derive(Debug, Event)]
#[doc(alias = "export")]
pub struct ExportRequest {
    /// The path the final image should be saved to.
    pub output: PathBuf,
    /// The Pixel Per Inch of the final image, this determines the size of the image.
    pub ppi: u32,
    /// The size of each tile the export takes, expressed in pixels.
    /// Larger tile sizes can reduce export time (as there's less tiles required) but increases memory consumption.
    /// Each dimension needs to be an increment of `256` and no larger than `4096`.
    pub tile_size: (u16, u16),
}
