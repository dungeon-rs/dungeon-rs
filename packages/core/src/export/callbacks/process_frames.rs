use std::path::PathBuf;
use bevy::math::Vec2;
use crossbeam_channel::Sender;
use crate::export::{ExportCompleted, ExportProgress, ExportStatus};

/// Called as a task from [crate::export::systems::read_frame] when all frames have been received.
/// This system asynchronously processes the frames into an image.
pub fn process_frames(mut buffer: Vec<(Vec2, Vec<u8>)>, mut sender: Sender<ExportProgress>) -> ExportCompleted {
    for (coords, data) in buffer {
        sender.send(ExportProgress {
            status: ExportStatus::Processing,
            total: 64,
            current: 0,
            progress: 0.
        }).expect("Failed to report progress");
    }

    // Return the export completed event to dispatch.
    ExportCompleted {
        path: PathBuf::new(),
    }
}