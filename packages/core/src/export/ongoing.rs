//! Holds the active export session state and associated data.
//!
//! This resource is created when an export process is initiated and persists until the export
//! is either completed or cancelled. It encapsulates the current state of the export lifecycle,
//! allowing systems to coordinate their behaviour accordingly. This resource is removed once
//! the export concludes.
//!
//! All sizes and coordinates expressed in [`Size2D`] and `Vec2` are in **world units**,
//! never pixels. The pixel size is calculated explicitly using the provided PPI and the known
//! grid cell size in world units. This ensures that logic and calculations remain consistent and
//! do not mix world units and pixel units inadvertently.

const GRID_CELL_UNITS: f32 = 100.0;
const MAX_TEXTURE_SIZE_PX: u32 = 4096;

use crate::export::size_2d::Size2D;
use crate::export::state::ExportState;
use crate::export::tasks::process_image_data;
use crate::export::{ExportCompleted, ExportRequest};
use bevy::asset::RenderAssetUsages;
use bevy::image::{BevyDefault, Image};
use bevy::prelude::{
    Assets, BevyError, Camera, Handle, OrthographicProjection, ResMut, Resource, Result, Vec2,
    default, info,
};
use bevy::render::camera::{RenderTarget, ScalingMode};
use bevy::render::gpu_readback::Readback;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use std::collections::VecDeque;

/// Tracks the state and internal data for an ongoing export operation.
///
/// This resource acts as the central coordinator for the export lifecycle, ensuring that all
/// participating systems are aware of the current phase and behave accordingly. It is created
/// upon export initiation and disposed of upon completion or cancellation.
#[derive(Resource, Debug)]
pub(super) struct OngoingExport {
    /// The current state of the export process.
    pub state: ExportState,
    /// Strong handle to keep the texture we render into alive.
    texture: Handle<Image>,
    /// The size of the frames, expressed in world units.
    frame_world_size: Size2D,
    /// The size of the frames, expressed in pixels.
    frame_px_size: (u32, u32),
    /// The queue of coordinates the camera needs to be moved to for a frame capture.
    /// Every time a movement completes, it moves to [OngoingExport::extracting].
    pending: VecDeque<Vec2>,
    /// The queue of coordinates the camera has been moved to and are awaiting GPU extraction.
    /// Once a frame is extracted, the coordinate gets popped, and they move to [OngoingExport::extracted].
    extracting: VecDeque<Vec2>,
    /// The frames that have been extracted from the GPU alongside the coordinates they were extracted from.
    extracted: Vec<(Vec2, Vec<u8>)>,
    /// If set contains the task processing the image data into a final export image.
    processing_task: Option<Task<ExportCompleted>>,
}

impl OngoingExport {
    /// Creates a new `OngoingExport` instance, initialised in the `PrepareTargetAndCamera` state.
    ///
    /// This ensures the export process begins in a consistent state, ready for systems to perform
    /// camera and render target preparation before proceeding to frame capture.
    pub fn new(request: &ExportRequest, images: &mut ResMut<Assets<Image>>) -> Self {
        let (frames, frame_world_size, frame_px_size) =
            Self::calculate_frames(Size2D::splat(600), request.ppi);
        let mut image = Image::new_fill(
            Extent3d {
                width: frame_px_size.0,
                height: frame_px_size.1,
                ..default()
            },
            TextureDimension::D2,
            &[0; 4],
            TextureFormat::bevy_default(),
            RenderAssetUsages::default(),
        );
        image.texture_descriptor.usage |=
            TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT;

        let frame_count = frames.len();
        OngoingExport {
            state: ExportState::PrepareTargetAndCamera,
            texture: images.add(image),
            frame_world_size,
            frame_px_size,
            pending: frames,
            extracting: VecDeque::with_capacity(frame_count),
            extracted: Vec::with_capacity(frame_count),
            processing_task: None,
        }
    }

    /// Attaches this export to the camera, ensuring we render into a buffer we can read.
    ///
    /// We then return a `Readback` that will handle reading the results back to the CPU.
    pub fn attach_to_camera(
        &self,
        camera: &mut Camera,
        projection: &mut OrthographicProjection,
    ) -> Readback {
        // We use a weak handle, so the lifetime is always determined by the lifetime of the export.
        camera.target = RenderTarget::Image(self.texture.clone_weak().into());
        // We adjust the projection of the camera to match the size of the frames we're about to use.
        info!(
            "Adjusting projection to {}x{} {:?}",
            self.frame_world_size.width, self.frame_world_size.height, self.frame_px_size
        );
        projection.scaling_mode = ScalingMode::Fixed {
            width: self.frame_world_size.width as f32,
            height: self.frame_world_size.height as f32,
        };

        Readback::Texture(self.texture.clone_weak())
    }

    /// Attempts to pop a camera movement coordinate set from the internal queue.
    pub fn pop_camera_movement(&mut self) -> Option<Vec2> {
        match self.pending.pop_front() {
            None => None,
            Some(coordinates) => {
                self.extracting.push_back(coordinates);

                Some(coordinates)
            }
        }
    }

    /// Associates the given raw image data with the next expected capture coordinate.
    ///
    /// The first frame returned by the GPU is always invalid and is silently discarded.
    /// Each later frame is matched to the next coordinate in the extraction queue.
    ///
    /// If there are no coordinates left to associate with image data, returns an error.
    /// This allows the export system to determine completion based solely on its internal state.
    pub fn push_image_data(&mut self, data: Vec<u8>) -> Result {
        // The very first read frame should be ignored as it's a stale frame (see `SkipFirstFrame`)
        if self.extracting.is_empty() && self.extracted.is_empty() {
            return Ok(());
        }

        match self.extracting.pop_front() {
            None => Err(BevyError::from(
                "No coordinates available to push image data to",
            )),
            Some(coordinates) => {
                image::RgbaImage::from_raw(
                    self.frame_px_size.0,
                    self.frame_px_size.1,
                    data.clone(),
                )
                .unwrap()
                .save_with_format(
                    format!("tile-{}x{}.png", coordinates.x, coordinates.y),
                    image::ImageFormat::Png,
                )
                .expect("TODO: panic message");
                self.extracted.push((coordinates, data));

                Ok(())
            }
        }
    }

    /// Starts asynchronous processing of the received image data.
    /// If attempting to process before all image data is received, this method will panic.
    pub fn process_async(&mut self) {
        if !self.pending.is_empty() || !self.extracting.is_empty() {
            panic!("Cannot process image data before all frames are extracted");
        }

        let task = AsyncComputeTaskPool::get().spawn(process_image_data());
        self.processing_task = Some(task);
    }

    /// Calculates a queue of coordinates at which the camera should capture a frame, along with the
    /// size of each frame in world units and its size in pixels.
    ///
    /// This method calculates the coordinates at which to capture and the size of all frames,
    /// attempting to minimise the number of frames needed without exceeding GPU memory limits.
    ///
    /// The calculation ensures the frame pixel size never exceeds the GPU texture limit and is
    /// aligned to 256 pixels as required by WGPU.
    ///
    /// Returns a tuple containing:
    /// - the list of capture coordinates,
    /// - the size of each frame in world units ([Size2D]),
    /// - the size of each frame in pixels as a tuple `(u32, u32)`.
    fn calculate_frames(map_size: Size2D, ppi: u32) -> (VecDeque<Vec2>, Size2D, (u32, u32)) {
        let pixels_per_world_unit = ppi as f32 / GRID_CELL_UNITS;

        let max_world_per_frame = (MAX_TEXTURE_SIZE_PX as f32 / pixels_per_world_unit).floor();

        let frame_count_x = (map_size.width as f32 / max_world_per_frame).ceil() as i32;
        let frame_count_y = (map_size.height as f32 / max_world_per_frame).ceil() as i32;

        let mut adjusted_world_per_frame_x = map_size.width as f32 / frame_count_x as f32;
        let mut adjusted_world_per_frame_y = map_size.height as f32 / frame_count_y as f32;

        let pixels_per_frame_x = adjusted_world_per_frame_x * pixels_per_world_unit;
        let pixels_per_frame_y = adjusted_world_per_frame_y * pixels_per_world_unit;

        // Align pixel size to nearest multiple of 256 and adjust units per frame accordingly
        let aligned_pixels_per_frame_x = (pixels_per_frame_x / 256.0).ceil() * 256.0;
        let aligned_pixels_per_frame_y = (pixels_per_frame_y / 256.0).ceil() * 256.0;

        assert!(aligned_pixels_per_frame_x <= MAX_TEXTURE_SIZE_PX as f32);
        assert!(aligned_pixels_per_frame_y <= MAX_TEXTURE_SIZE_PX as f32);

        adjusted_world_per_frame_x = aligned_pixels_per_frame_x / pixels_per_world_unit;
        adjusted_world_per_frame_y = aligned_pixels_per_frame_y / pixels_per_world_unit;

        let mut coordinates = VecDeque::with_capacity((frame_count_x * frame_count_y) as usize);

        for x in -(frame_count_x / 2)..(frame_count_x / 2) + 1 {
            for y in -(frame_count_y / 2)..(frame_count_y / 2) + 1 {
                coordinates.push_back(Vec2::new(
                    x as f32 * adjusted_world_per_frame_x,
                    y as f32 * adjusted_world_per_frame_y,
                ));
            }
        }
        //
        (
            coordinates,
            Size2D::new(
                adjusted_world_per_frame_x.round() as u32,
                adjusted_world_per_frame_y.round() as u32,
            ),
            (
                aligned_pixels_per_frame_x.round() as u32,
                aligned_pixels_per_frame_y.round() as u32,
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_grid_covers_map(
        coordinates: &VecDeque<Vec2>,
        frame_size: &Size2D,
        map_size: &Size2D,
    ) {
        // Calculate actual covered area
        let min_x = coordinates
            .iter()
            .map(|c| c.x)
            .fold(f32::INFINITY, f32::min);
        let min_y = coordinates
            .iter()
            .map(|c| c.y)
            .fold(f32::INFINITY, f32::min);
        let max_x = coordinates
            .iter()
            .map(|c| c.x + frame_size.width as f32)
            .fold(f32::NEG_INFINITY, f32::max);
        let max_y = coordinates
            .iter()
            .map(|c| c.y + frame_size.height as f32)
            .fold(f32::NEG_INFINITY, f32::max);

        assert!(min_x <= 0.0);
        assert!(min_y <= 0.0);
        assert!((max_x - min_x).ceil() >= map_size.width as f32);
        assert!((max_y - min_y).ceil() >= map_size.height as f32);

        // Check that all coordinates are aligned correctly (no gaps or jitter)
        for coord in coordinates {
            assert_eq!(coord.x % frame_size.width as f32, 0.0);
            assert_eq!(coord.y % frame_size.height as f32, 0.0);
        }
    }

    #[test]
    fn test_square_map_low_ppi() {
        let (coordinates, world_size, px_size) =
            OngoingExport::calculate_frames(Size2D::new(1000, 1000), 128);
        assert_eq!(coordinates.len(), 1);
        assert_eq!(px_size.0 % 256, 0);
        assert_eq!(px_size.1 % 256, 0);

        assert_grid_covers_map(&coordinates, &world_size, &Size2D::new(1000, 1000));
    }

    #[test]
    fn test_non_square_map() {
        let (coordinates, world_size, px_size) =
            OngoingExport::calculate_frames(Size2D::new(3000, 5000), 128);
        assert!(coordinates.len() > 1);
        assert_eq!(px_size.0 % 256, 0);
        assert_eq!(px_size.1 % 256, 0);

        assert_grid_covers_map(&coordinates, &world_size, &Size2D::new(3000, 5000));
    }

    #[test]
    fn test_min_ppi_tiny_map() {
        let (coordinates, world_size, px_size) =
            OngoingExport::calculate_frames(Size2D::new(100, 100), 16);
        assert_eq!(coordinates.len(), 1);
        assert!(px_size.0 >= 256);
        assert!(px_size.1 >= 256);

        assert_grid_covers_map(&coordinates, &world_size, &Size2D::new(100, 100));
    }

    #[test]
    fn test_max_ppi_tiny_map() {
        let (coordinates, world_size, px_size) =
            OngoingExport::calculate_frames(Size2D::new(100, 100), 512);
        assert_eq!(coordinates.len(), 1);
        assert!(px_size.0 <= MAX_TEXTURE_SIZE_PX);
        assert!(px_size.1 <= MAX_TEXTURE_SIZE_PX);

        assert_grid_covers_map(&coordinates, &world_size, &Size2D::new(100, 100));
    }

    #[test]
    fn test_wide_map_medium_ppi() {
        let (coordinates, world_size, px_size) =
            OngoingExport::calculate_frames(Size2D::new(5000, 1000), 256);
        assert!(coordinates.len() > 1);
        assert_eq!(px_size.0 % 256, 0);
        assert_eq!(px_size.1 % 256, 0);

        assert_grid_covers_map(&coordinates, &world_size, &Size2D::new(5000, 1000));
    }

    #[test]
    fn test_large_map_high_ppi() {
        let (coordinates, world_size, px_size) =
            OngoingExport::calculate_frames(Size2D::new(10000, 10000), 512);
        assert!(coordinates.len() > 10);
        assert!(px_size.0 <= MAX_TEXTURE_SIZE_PX);
        assert!(px_size.1 <= MAX_TEXTURE_SIZE_PX);
        assert_eq!(px_size.0 % 256, 0);
        assert_eq!(px_size.1 % 256, 0);

        assert_grid_covers_map(&coordinates, &world_size, &Size2D::new(10000, 10000));
    }
}
