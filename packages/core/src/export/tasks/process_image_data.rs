use crate::export::{ExportCompleted, ExportProgress};
use bevy::math::UVec2;
use bevy::prelude::BevyError;
use crossbeam_channel::Sender;
use image::RgbaImage;
use std::path::PathBuf;

/// This method is intended to be running as an asynchronous task in the background.
/// See [bevy::tasks::AsyncComputeTaskPool] for how to run this.
///
/// ### Parameters:
/// - `output`: The file the resulting image should be written to.
/// - `width`: The width of the resulting image in pixels.
/// - `height`: The height of the resulting image in pixels.
/// - `frame_px_width`: The width of a single frame in pixels.
/// - `frame_px_height`: The height of a single frame in pixels.
/// - `progress`: The [`Sender<ExportProgress>`] used to report progress back to the main loop.
/// - `image_data`: The image data (and associated coordinates) for each frame.
pub async fn process_image_data(
    output: PathBuf,
    width: u32,
    height: u32,
    frame_px_width: u32,
    frame_px_height: u32,
    progress: Sender<ExportProgress>,
    image_data: Vec<(UVec2, Vec<u8>)>,
) -> Result<ExportCompleted, BevyError> {
    let mut image = RgbaImage::new(width, height);

    for (pos, data) in image_data {
        let Some(buffer) = RgbaImage::from_raw(frame_px_width, frame_px_height, data) else {
            return Err(BevyError::from("Failed to create image buffer from data"));
        };

        let (buffer_width, buffer_height) = buffer.dimensions();
        for dy in 0..buffer_height {
            for dx in 0..buffer_width {
                let pixel = buffer.get_pixel(dx, dy);

                let x = pos.x + dx;
                let y = pos.y + dy;
                image.put_pixel(x, y, *pixel);
            }
        }

        progress.send(ExportProgress {
            progress: 0.,
            total: 0,
            current: 0,
        })?;
    }

    // TODO: write finalized image to output based on the requested output.
    image.save_with_format(output.clone(), image::ImageFormat::Png)?;

    // TODO: fill out export completed data.
    Ok(ExportCompleted { path: output })
}
