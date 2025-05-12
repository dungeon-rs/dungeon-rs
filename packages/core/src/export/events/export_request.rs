use crate::export::size_2d::Size2D;
use bevy::prelude::Event;
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
    /// The Pixel Per Inch of the final image, this determines the size of the image.
    pub(crate) ppi: u32,
    /// The size of each frame the export takes, expressed in pixels.
    /// Larger frame sizes can reduce export time (as there are fewer frames required) but increase memory consumption.
    /// Each dimension needs to be an increment of `256` and no larger than `4096`.
    pub(crate) frame_size: Size2D,
}

impl ExportRequest {
    /// Creates a new export request with the specified parameters.
    /// See [`ExportRequest`] for details on expected parameters.
    ///
    /// # Returns
    /// * `Ok(ExportRequest)` - If frame sizes are valid multiples of 256
    /// * `Err(String)` - If frame sizes are not valid multiples of 256
    pub fn new(output: PathBuf, ppi: u32, frame_size: Size2D) -> Result<Self, String> {
        let Size2D {
            width: x,
            height: y,
        } = frame_size;

        if ppi == 0 {
            return Err(String::from("ppi must be greater than 0"));
        }
        if x % 256 != 0 || x > 4096 || x == 0 {
            return Err(format!(
                "frame size must be a multiple of 256 (up to 4096), got invalid X value {}",
                x
            ));
        }
        if y % 256 != 0 || y > 4096 || y == 0 {
            return Err(format!(
                "frame size must be a multiple of 256 (up to 4096), got invalid Y value {}",
                y
            ));
        }

        Ok(ExportRequest {
            output,
            ppi,
            frame_size,
        })
    }
}
