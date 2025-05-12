use bevy::prelude::Vec2;
use crossbeam_channel::Sender;
use crate::export::ExportProgress;

/// Represents the collected image data for a [crate::export::OngoingExport],
/// intended to be passed to the background task responsible for assembling an image.
pub(crate) struct ExportData {
    /// The frames that have been extracted from the GPU alongside the coordinates they were extracted from.
    pub buffer: Vec<(Vec2, Vec<u8>)>,
    /// Used to communicate [ExportProgress] to the main thread.
    pub sender: Sender<ExportProgress>,
    /// The total number of steps needed to complete the export.
    /// This includes all camera movements, frame captures and processing.
    pub total_steps: u64,
    /// The number of steps that have been completed.
    pub current_step: u64,
    /// The *total* width of the export image in pixels.
    pub width: u32,
    /// The *total* height of the export image in pixels.
    pub height: u32,
    /// The width of a single frame in pixels.
    pub frame_width: u32,
    /// The height of a single frame in pixels.
    pub frame_height: u32,
}