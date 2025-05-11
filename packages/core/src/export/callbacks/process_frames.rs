use std::path::PathBuf;
use bevy::math::Vec2;
use crossbeam_channel::Sender;
use crate::export::{ExportCompleted, ExportProgress, ExportStatus};

/// Called as a task from [crate::export::systems::read_frame] when all frames have been received.
/// This system asynchronously processes the frames into an image.
pub fn process_frames(buffer: Vec<(Vec2, Vec<u8>)>, total: u64, current: u64, sender: Sender<ExportProgress>) -> ExportCompleted {
    let mut current = current;
    for (coords, data) in buffer {
        // TODO: process buffer

        current += 1;
        sender.send(ExportProgress {
            status: ExportStatus::Processing,
            total,
            current,
            progress: (current as f32 / total as f32) * 100.0,
        }).expect("Failed to report progress");
    }

    // Return the export-completed event to dispatch.
    ExportCompleted {
        path: PathBuf::new(),
    }
}