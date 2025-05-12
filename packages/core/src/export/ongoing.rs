//! Holds the active export session state and associated data.
//!
//! This resource is created when an export process is initiated and persists until the export
//! is either completed or cancelled. It encapsulates the current state of the export lifecycle,
//! allowing systems to coordinate their behaviour accordingly. This resource is removed once
//! the export concludes.

use crate::export::{ExportCompleted, ExportRequest};
use crate::export::size_2d::Size2D;
use crate::export::state::ExportState;
use bevy::asset::RenderAssetUsages;
use bevy::image::{BevyDefault, Image};
use bevy::prelude::{Assets, BevyError, Camera, Handle, ResMut, Resource, Result, Vec2, default};
use bevy::render::camera::RenderTarget;
use bevy::render::gpu_readback::Readback;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use std::collections::VecDeque;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use crate::export::tasks::process_image_data;

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
    /// The queue of coordinates the camera needs to be moved to for a frame capture.
    /// Every time a movement completes, it moves to [self::extracting].
    pending: VecDeque<Vec2>,
    /// The queue of coordinates the camera has been moved to and are awaiting GPU extraction.
    /// Once a frame is extracted, the coordinate gets popped, and they move to [self::extracted].
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
        let frames = Self::calculate_frames(Size2D::splat(1024), request.frame_size);
        let mut image = Image::new_fill(
            Extent3d {
                width: request.frame_size.width * request.ppi,
                height: request.frame_size.height * request.ppi,
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
            pending: frames,
            extracting: VecDeque::with_capacity(frame_count),
            extracted: Vec::with_capacity(frame_count),
            processing_task: None,
        }
    }

    /// Attaches this export to the camera, ensuring we render into a buffer we can read.
    ///
    /// We then return a [Readback] that will handle reading the results back to the CPU.
    pub fn attach_to_camera(&self, camera: &mut Camera) -> Readback {
        // We use a weak handle, so the lifetime is always determined by the lifetime of the export.
        camera.target = RenderTarget::Image(self.texture.clone_weak().into());

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

    /// Generate a queue of coordinates at which a frame should be captured.
    /// This method assumes that *all* frames will be the same size (being `frame_width`x`frame_height`).
    fn calculate_frames(size: Size2D, frame_size: Size2D) -> VecDeque<Vec2> {
        let x_count = (size.width as f32 / frame_size.width as f32).ceil() as u32;
        let y_count = (size.height as f32 / frame_size.height as f32).ceil() as u32;

        let mut coordinates = VecDeque::with_capacity((x_count * y_count) as usize);
        for x in 0..x_count {
            for y in 0..y_count {
                coordinates.push_back(Vec2::new(
                    x as f32 * frame_size.width as f32,
                    y as f32 * frame_size.height as f32,
                ));
            }
        }

        coordinates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_frames_calculated() {
        let coordinates =
            OngoingExport::calculate_frames(Size2D::new(100, 100), Size2D::new(10, 10));
        assert_eq!(coordinates.len(), 100);

        let coordinates =
            OngoingExport::calculate_frames(Size2D::new(500, 500), Size2D::new(10, 10));
        assert_eq!(coordinates.len(), 2500);
    }

    #[test]
    fn test_frames_coordinates() {
        let coordinates = OngoingExport::calculate_frames(Size2D::new(2, 2), Size2D::new(1, 1));

        assert_eq!(coordinates.front(), Some(&Vec2::new(0.0, 0.0)));
        assert_eq!(coordinates.get(1), Some(&Vec2::new(0.0, 1.0)));
        assert_eq!(coordinates.get(2), Some(&Vec2::new(1.0, 0.0)));
        assert_eq!(coordinates.get(3), Some(&Vec2::new(1.0, 1.0)));
    }
}
