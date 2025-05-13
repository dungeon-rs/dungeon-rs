use bevy::prelude::{Event, Rect};
use std::path::PathBuf;

/// A Bevy event that is dispatched when the user requests an export.
/// Usually this is done through the editor when selecting "export"
/// or by the headless exporter.
///
/// The request contains the information about how to construct the export.
#[derive(Debug, Event)]
#[doc(alias = "export")]
pub struct ExportRequest {
    /// The path the final image should be saved to.
    pub(crate) output: PathBuf,
    /// The region of the screen to export (in world units).
    pub(crate) region: Rect,
    /// The Pixel Per Inch of the final image, this determines the size of the image.
    pub(crate) ppi: u32,
}

impl ExportRequest {
    /// Creates a new export request with the specified parameters.
    /// See [`ExportRequest`] for details on expected parameters.
    ///
    /// # Returns
    /// * `Ok(ExportRequest)` - If frame sizes are valid multiples of 256
    /// * `Err(String)` - If frame sizes are not valid multiples of 256
    pub fn new(output: PathBuf, region: Rect, ppi: u32) -> Result<Self, String> {
        if ppi == 0 {
            return Err(String::from("ppi must be greater than 0"));
        }

        Ok(ExportRequest {
            output,
            region,
            ppi,
        })
    }
}
