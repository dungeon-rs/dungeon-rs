use crate::export::export_data::ExportData;
use crate::export::{ExportCompleted, ExportProgress, ExportStatus};
use image::{ImageFormat, RgbaImage};
use std::path::PathBuf;

/// Called as a task from [crate::export::systems::read_frame] when all frames have been received.
/// This system asynchronously processes the frames into an image.
pub fn process_frames(mut export_data: ExportData) -> ExportCompleted {
    let mut buffer = RgbaImage::new(export_data.width, export_data.height);

    for (coords, data) in export_data.buffer {
        let x = coords.x as u32;
        let y = coords.y as u32;
        let tile_image =
            RgbaImage::from_raw(export_data.frame_width, export_data.frame_height, data)
                .expect("Invalid raw RGBA tile data");

        #[cfg(feature = "dev")]
        tile_image
            .save_with_format(format!("tile-{}x{}.png", x, y), ImageFormat::Png)
            .expect("Failed to save tile");

        let (tile_width, tile_height) = tile_image.dimensions();

        for dy in 0..tile_height {
            for dx in 0..tile_width {
                let pixel = tile_image.get_pixel(dx, dy);
                buffer.put_pixel(x + dx, y + dy, *pixel);
            }
        }

        export_data.current_step += 1;
        export_data
            .sender
            .send(ExportProgress {
                status: ExportStatus::Processing,
                total: export_data.total_steps,
                current: export_data.current_step,
                progress: (export_data.current_step as f32 / export_data.total_steps as f32)
                    * 100.0,
            })
            .expect("Failed to report progress");
    }

    buffer
        .save_with_format("output.png", ImageFormat::Png)
        .expect("Failed to save output image");
    // Return the export-completed event to dispatch.
    ExportCompleted {
        path: PathBuf::new(),
    }
}
