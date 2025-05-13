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
use crate::export::{ExportCompleted, ExportProgress, ExportRequest};
use bevy::asset::RenderAssetUsages;
use bevy::image::{BevyDefault, Image};
use bevy::math::UVec2;
use bevy::prelude::{
    Assets, BevyError, Camera, Handle, OrthographicProjection, ResMut, Resource, Result, Vec2,
    default, info,
};
use bevy::render::camera::{RenderTarget, ScalingMode};
use bevy::render::gpu_readback::Readback;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use crossbeam_channel::Receiver;
use std::collections::VecDeque;
use std::mem;
use std::path::PathBuf;

/// Tracks the state and internal data for an ongoing export operation.
///
/// This resource acts as the central coordinator for the export lifecycle, ensuring that all
/// participating systems are aware of the current phase and behave accordingly. It is created
/// upon export initiation and disposed of upon completion or cancellation.
#[derive(Resource, Debug)]
pub(super) struct OngoingExport {
    /// The current state of the export process.
    pub state: ExportState,
    /// The file we're expected to write the result to.
    pub output: PathBuf,
    /// Strong handle to keep the texture we render into alive.
    texture: Handle<Image>,
    /// The size of the frames, expressed in world units.
    frame_world_size: Size2D,
    /// The size of the frames, expressed in pixels.
    frame_px_size: (u32, u32),
    /// The size of all frames combined, expressed in pixels.
    final_px_size: (u32, u32),
    /// The queue of coordinates the camera needs to be moved to for a frame capture along the coordinates
    /// expressed in pixels (used when stitching the frames together).
    /// Every time a movement completes, it moves to [OngoingExport::extracting].
    pending: VecDeque<(Vec2, UVec2)>,
    /// The coordinates in pixels where the camera has been moved to, used to stitch the frames together.
    /// Once a frame is extracted, the coordinates get popped, and they move to [OngoingExport::extracted].
    extracting: VecDeque<UVec2>,
    /// The frames that have been extracted from the GPU alongside the coordinates they were extracted from.
    extracted: Vec<(UVec2, Vec<u8>)>,
    /// The [`Receiver<ExportProgress>`] used to communicate [ExportProgress] from the async processing task
    /// back to the main thread.
    processing_receiver: Option<Receiver<ExportProgress>>,
    /// If set contains the task processing the image data into a final export image.
    processing_task: Option<Task<std::result::Result<ExportCompleted, BevyError>>>,
}

/// Contains the result of calculating the export frame grid and related pixel dimensions.
#[derive(Debug)]
struct FramesGrid {
    /// The queue of frame capture points, where each entry contains the camera's world-space centre and its corresponding position in the final output image in pixels.
    pub frames: VecDeque<(Vec2, UVec2)>,
    /// The size of each frame, expressed in world units.
    pub frame_world_size: Size2D,
    /// The size of each frame, expressed in pixels.
    pub frame_px_size: (u32, u32),
    /// The total size of the stitched image, expressed in pixels.
    pub final_px_size: (u32, u32),
}


impl OngoingExport {
    /// Creates a new `OngoingExport` instance, initialised in the `PrepareTargetAndCamera` state.
    ///
    /// This ensures the export process begins in a consistent state, ready for systems to perform
    /// camera and render target preparation before proceeding to frame capture.
    pub fn new(request: &ExportRequest, images: &mut ResMut<Assets<Image>>) -> Self {
        let FramesGrid {
            frames,
            frame_px_size,
            frame_world_size,
            final_px_size,
        } = Self::calculate_frames(Size2D::splat(4000), request.ppi);
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
            output: request.output.clone(),
            texture: images.add(image),
            frame_world_size,
            frame_px_size,
            final_px_size,
            pending: frames,
            extracting: VecDeque::with_capacity(frame_count),
            extracted: Vec::with_capacity(frame_count),
            processing_receiver: None,
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
            Some((world_coordinates, pixel_coordinates)) => {
                self.extracting.push_back(pixel_coordinates);

                Some(world_coordinates)
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

        let image_data = mem::take(&mut self.extracted);
        let (sender, receiver) = crossbeam_channel::unbounded();
        let task = AsyncComputeTaskPool::get().spawn(process_image_data(
            self.output.clone(),
            self.final_px_size.0,
            self.final_px_size.1,
            self.frame_px_size.0,
            self.frame_px_size.1,
            sender,
            image_data,
        ));

        self.processing_receiver = Some(receiver);
        self.processing_task = Some(task);
    }

    /// Calculates the grid of camera positions and corresponding pixel positions for the export.
    /// A best effort is made to minimise the number of frames needed for the export.
    ///
    /// The method calculates:
    /// - The coordinates at which the camera must capture the map, expressed as both world units and pixel coordinates relative to the stitched output image.
    /// - The size of each capture frame in world units and pixels.
    /// - The total size of the stitched output image in pixels.
    ///
    /// Ensures the frame pixel size does not exceed the GPU texture limit, aligning sizes to 256 pixels as required by WGPU.
    ///
    /// Returns a [FramesGrid] containing the calculated data.
    #[must_use]
    fn calculate_frames(map_size: Size2D, ppi: u32) -> FramesGrid {
        let pixels_per_world_unit = ppi as f32 / GRID_CELL_UNITS;

        let max_world_per_frame = (MAX_TEXTURE_SIZE_PX as f32 / pixels_per_world_unit).floor();

        let frame_count_x = (map_size.width as f32 / max_world_per_frame).ceil() as i32;
        let frame_count_y = (map_size.height as f32 / max_world_per_frame).ceil() as i32;

        let mut adjusted_world_per_frame_x = map_size.width as f32 / frame_count_x as f32;
        let mut adjusted_world_per_frame_y = map_size.height as f32 / frame_count_y as f32;

        let pixels_per_frame_x = adjusted_world_per_frame_x * pixels_per_world_unit;
        let pixels_per_frame_y = adjusted_world_per_frame_y * pixels_per_world_unit;

        // Align pixel size to nearest multiple of 256 and adjust units per frame accordingly
        let aligned_pixels_per_frame_x = ((pixels_per_frame_x / 256.0).ceil() * 256.0) as u32;
        let aligned_pixels_per_frame_y = ((pixels_per_frame_y / 256.0).ceil() * 256.0) as u32;

        assert!(aligned_pixels_per_frame_x <= MAX_TEXTURE_SIZE_PX);
        assert!(aligned_pixels_per_frame_y <= MAX_TEXTURE_SIZE_PX);

        adjusted_world_per_frame_x = aligned_pixels_per_frame_x as f32 / pixels_per_world_unit;
        adjusted_world_per_frame_y = aligned_pixels_per_frame_y as f32 / pixels_per_world_unit;

        let mut coordinates = VecDeque::with_capacity((frame_count_x * frame_count_y) as usize);

        // The most negative world coordinate is always:
        let min_world_x = -(map_size.width as f32) / 2.0;
        let min_world_y = -(map_size.height as f32) / 2.0;

        // While we're looping, calculate the total pixels outputted
        let mut max_pixel_x = 0;
        let mut max_pixel_y = 0;

        for x in -(frame_count_x / 2)..(frame_count_x / 2) + 1 {
            for y in -(frame_count_y / 2)..(frame_count_y / 2) + 1 {
                let origin_world_x = x as f32 * adjusted_world_per_frame_x;
                let origin_world_y = y as f32 * adjusted_world_per_frame_y;

                // Adjust camera center to top-left of the frame
                let frame_top_left_world_x = origin_world_x - (adjusted_world_per_frame_x / 2.0);
                let frame_top_left_world_y = origin_world_y - (adjusted_world_per_frame_y / 2.0);

                let pixel_x =
                    ((frame_top_left_world_x - min_world_x) * pixels_per_world_unit).round() as u32;
                let pixel_y =
                    ((frame_top_left_world_y - min_world_y) * pixels_per_world_unit).round() as u32;

                // Track max extents directly here
                max_pixel_x = max_pixel_x.max(pixel_x + aligned_pixels_per_frame_x);
                max_pixel_y = max_pixel_y.max(pixel_y + aligned_pixels_per_frame_y);

                coordinates.push_back((
                    Vec2::new(origin_world_x, origin_world_y),
                    UVec2::new(pixel_x, pixel_y),
                ));
            }
        }

        FramesGrid {
            frames: coordinates,
            frame_world_size: Size2D::new(
                adjusted_world_per_frame_x.round() as u32,
                adjusted_world_per_frame_y.round() as u32,
            ),
            frame_px_size: (aligned_pixels_per_frame_x, aligned_pixels_per_frame_y),
            final_px_size: (max_pixel_x, max_pixel_y),
        }
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
        let min_x = coordinates.iter().map(|c| c.x).fold(f32::INFINITY, f32::min);
        let min_y = coordinates.iter().map(|c| c.y).fold(f32::INFINITY, f32::min);
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

        for coord in coordinates {
            assert_eq!(coord.x % frame_size.width as f32, 0.0);
            assert_eq!(coord.y % frame_size.height as f32, 0.0);
        }
    }

    #[test]
    fn test_square_map_low_ppi() {
        let grid = OngoingExport::calculate_frames(Size2D::new(1000, 1000), 128);
        assert_eq!(grid.frames.len(), 1);
        assert_eq!(grid.frame_px_size.0 % 256, 0);
        assert_eq!(grid.frame_px_size.1 % 256, 0);

        let world_coords = grid.frames.iter().map(|(w, _)| *w).collect();
        assert_grid_covers_map(&world_coords, &grid.frame_world_size, &Size2D::new(1000, 1000));

        assert!(grid.final_px_size.0 >= grid.frame_px_size.0);
        assert!(grid.final_px_size.1 >= grid.frame_px_size.1);
    }

    #[test]
    fn test_non_square_map_higher() {
        let grid = OngoingExport::calculate_frames(Size2D::new(3000, 5000), 128);
        assert!(grid.frames.len() > 1);
        assert_eq!(grid.frame_px_size.0 % 256, 0);
        assert_eq!(grid.frame_px_size.1 % 256, 0);

        let world_coords = grid.frames.iter().map(|(w, _)| *w).collect();
        assert_grid_covers_map(&world_coords, &grid.frame_world_size, &Size2D::new(3000, 5000));

        assert!(grid.final_px_size.0 >= grid.frame_px_size.0);
        assert!(grid.final_px_size.1 > grid.frame_px_size.1);
    }

    #[test]
    fn test_non_square_map_wider() {
        let grid = OngoingExport::calculate_frames(Size2D::new(5000, 3000), 128);
        assert!(grid.frames.len() > 1);
        assert_eq!(grid.frame_px_size.0 % 256, 0);
        assert_eq!(grid.frame_px_size.1 % 256, 0);

        let world_coords = grid.frames.iter().map(|(w, _)| *w).collect();
        assert_grid_covers_map(&world_coords, &grid.frame_world_size, &Size2D::new(5000, 3000));

        assert!(grid.final_px_size.0 > grid.frame_px_size.0);
        assert!(grid.final_px_size.1 >= grid.frame_px_size.1);
    }


    #[test]
    fn test_min_ppi_tiny_map() {
        let grid = OngoingExport::calculate_frames(Size2D::new(100, 100), 16);
        assert_eq!(grid.frames.len(), 1);
        assert!(grid.frame_px_size.0 >= 256);
        assert!(grid.frame_px_size.1 >= 256);

        let world_coords = grid.frames.iter().map(|(w, _)| *w).collect();
        assert_grid_covers_map(&world_coords, &grid.frame_world_size, &Size2D::new(100, 100));
    }

    #[test]
    fn test_max_ppi_tiny_map() {
        let grid = OngoingExport::calculate_frames(Size2D::new(100, 100), 512);
        assert_eq!(grid.frames.len(), 1);
        assert!(grid.frame_px_size.0 <= MAX_TEXTURE_SIZE_PX);
        assert!(grid.frame_px_size.1 <= MAX_TEXTURE_SIZE_PX);

        let world_coords = grid.frames.iter().map(|(w, _)| *w).collect();
        assert_grid_covers_map(&world_coords, &grid.frame_world_size, &Size2D::new(100, 100));
    }

    #[test]
    fn test_wide_map_medium_ppi() {
        let grid = OngoingExport::calculate_frames(Size2D::new(5000, 1000), 256);
        assert!(grid.frames.len() > 1);
        assert_eq!(grid.frame_px_size.0 % 256, 0);
        assert_eq!(grid.frame_px_size.1 % 256, 0);

        let world_coords = grid.frames.iter().map(|(w, _)| *w).collect();
        assert_grid_covers_map(&world_coords, &grid.frame_world_size, &Size2D::new(5000, 1000));
    }

    #[test]
    fn test_large_map_high_ppi() {
        let grid = OngoingExport::calculate_frames(Size2D::new(10000, 10000), 512);
        assert!(grid.frames.len() > 10);
        assert!(grid.frame_px_size.0 <= MAX_TEXTURE_SIZE_PX);
        assert!(grid.frame_px_size.1 <= MAX_TEXTURE_SIZE_PX);
        assert_eq!(grid.frame_px_size.0 % 256, 0);
        assert_eq!(grid.frame_px_size.1 % 256, 0);

        let world_coords = grid.frames.iter().map(|(w, _)| *w).collect();
        assert_grid_covers_map(&world_coords, &grid.frame_world_size, &Size2D::new(10000, 10000));
    }

    #[test]
    fn test_pixel_coordinate_mapping() {
        let grid = OngoingExport::calculate_frames(Size2D::new(1000, 1000), 128);

        // Find the frame containing world center
        let center_world = Vec2::ZERO;
        let frame = grid.frames.iter().find(|(w, _)| {
            let half_frame_wu = grid.frame_world_size.width as f32 / 2.0;
            let half_frame_hu = grid.frame_world_size.height as f32 / 2.0;
            w.x - half_frame_wu <= center_world.x
                && w.x + half_frame_wu >= center_world.x
                && w.y - half_frame_hu <= center_world.y
                && w.y + half_frame_hu >= center_world.y
        });

        assert!(
            frame.is_some(),
            "Expected to find frame containing world (0,0)"
        );
    }
}
