use std::path::PathBuf;
use crate::export::ExportCompleted;

/// This method is intended to be running as an asynchronous task in the background.
/// See [bevy::tasks::AsyncComputeTaskPool] for how to run this.
pub async fn process_image_data() -> ExportCompleted {
    // TODO: fill out export completed data.
    ExportCompleted {
        path: PathBuf::new()
    }
}