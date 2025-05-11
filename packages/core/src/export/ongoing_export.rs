use crate::export::{ExportCompleted, ExportProgress, ExportRequest};
use bevy::asset::RenderAssetUsages;
use bevy::image::BevyDefault;
use bevy::prelude::{Assets, Handle, Image, ResMut, Resource, Vec2, default};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use crossbeam_channel::{Receiver, Sender};
use std::collections::VecDeque;
use std::mem;
use bevy::tasks::Task;

#[derive(Debug, Resource)]
pub(crate) struct OngoingExport {
    /// What the export is currently processing.
    pub state: ExportState,
    /// The queue of coordinates the camera needs to be moved to for a frame capture.
    /// Every time a movement completes, it moves to [self::extracting].
    pending: VecDeque<Vec2>,
    /// The queue of coordinates the camera has been moved to and are awaiting GPU extraction.
    /// Once a frame is extracted, the coordinate gets popped and they move to [self::extracted].
    extracting: VecDeque<Vec2>,
    /// The frames that have been extracted from the GPU alongside the coordinates they were extracted from.
    extracted: Vec<(Vec2, Vec<u8>)>,
    /// The texture the camera renders into, used to copy from GPU to CPU.
    pub render_texture: Handle<Image>,
    /// The total number of steps needed to complete the export.
    /// This includes all camera movements, frame captures and processing.
    pub total_steps: u64,
    /// The number of steps that have been completed.
    pub current_step: u64,
    /// Once we reach the [ExportState::Processing] state, this will be set to the receiver of the
    /// asynchronous processing channel.
    pub(crate) receiver: Option<Receiver<ExportProgress>>,
    /// The task that is processing the received frames into an image.
    pub(crate) processing_task: Option<Task<ExportCompleted>>
}

/// A more specialized version of [ExportStatus] used for internal state tracking.
#[derive(Debug)]
pub enum ExportState {
    /// The export is initializing for capture.
    /// This is usually only for one frame, when the readback and camera position are set up.
    Preparing,
    /// The export is moving the camera around to capture each frame.
    Capturing,
    /// The export has finishing moving and is extracting frames from the GPU.
    Extracting,
    /// All frames have been extracted and are being converted to images.
    Processing,
}

impl OngoingExport {
    /// Generate a new [OngoingExport] resource.
    pub fn new(request: &ExportRequest, mut images: ResMut<Assets<Image>>) -> Self {
        // TODO: for now we'll assume the canvas is *always* 2048 pixels.
        let frames = OngoingExport::frames(2048, 2048, request.frame_size.0, request.frame_size.1);
        let texture = OngoingExport::render_image(request.frame_size.0, request.frame_size.1);

        let frame_count = frames.len();
        Self {
            state: ExportState::Preparing,
            pending: frames,
            extracting: VecDeque::with_capacity(frame_count),
            extracted: Vec::with_capacity(frame_count),
            render_texture: images.add(texture),
            total_steps: (frame_count as u64 * 3),
            current_step: 0,
            receiver: None,
            processing_task: None,
        }
    }

    /// Attempts to pop a pending frame.
    /// If one is available, it's automatically added to [Self::extracting]
    #[inline(always)]
    pub fn pop_pending(&mut self) -> Option<Vec2> {
        match self.pending.pop_front() {
            None => None,
            Some(pending) => {
                self.extracting.push_back(pending);
                self.current_step += 1;

                Some(pending)
            }
        }
    }

    /// Attempts to add an extracted frame and associate coordinates.
    #[inline(always)]
    pub fn push_extracted(&mut self, data: Vec<u8>) -> Result<(), ()> {
        match self.extracting.pop_front() {
            None => Err(()),
            Some(extracting) => {
                self.extracted.push((extracting, data));
                self.current_step += 1;

                Ok(())
            }
        }
    }

    /// Consumes the current export buffer, reading all extracted frames and setting a receiver.
    /// The returned [Sender<ExportProgress>] can be used to communicate progress of the processing.
    /// The task you process the given buffer on should be given to [Self::set_processing_task].
    ///
    /// <div class="warning">
    ///
    /// The internal frame buffer is emptied when this method is called
    ///
    /// </div>
    pub fn consume(&mut self) -> (Vec<(Vec2, Vec<u8>)>, Sender<ExportProgress>) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let buffer = mem::take(&mut self.extracted);
        self.receiver = Some(receiver);

        (buffer, sender)
    }

    pub fn set_processing_task(&mut self, task: Task<ExportCompleted>) {
        self.processing_task = Some(task);
    }

    /// Generates an [Image] that Bevy will render GPU images into.
    /// We can then use a [bevy::render::gpu_readback::Readback] to copy that image to the CPU.
    fn render_image(width: u32, height: u32) -> Image {
        let mut image = Image::new_fill(
            Extent3d {
                width,
                height,
                ..default()
            },
            TextureDimension::D2,
            &[0; 4],
            TextureFormat::bevy_default(),
            RenderAssetUsages::default(),
        );
        image.texture_descriptor.usage |= TextureUsages::COPY_SRC
            | TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::TEXTURE_BINDING;

        image
    }

    /// Builds a list of all coordinates the camera needs to be moved to for a export.
    /// This should be an exhaustive list that covers the entire given `width` and `height`.
    ///
    /// # Parameters
    /// * `width` - Total width of the area to capture
    /// * `height` - Total height of the area to capture
    /// * `frame_width` - Width of each frame
    /// * `frame_height` - Height of each frame
    fn frames(width: u32, height: u32, frame_width: u32, frame_height: u32) -> VecDeque<Vec2> {
        let x_count = (width as f32 / frame_width as f32).ceil() as u32;
        let y_count = (height as f32 / frame_height as f32).ceil() as u32;

        let mut coordinates = VecDeque::with_capacity((x_count * y_count) as usize);
        for x in 0..x_count {
            for y in 0..y_count {
                coordinates.push_back(Vec2::new(
                    x as f32 * frame_width as f32,
                    y as f32 * frame_height as f32,
                ));
            }
        }

        coordinates
    }
}
