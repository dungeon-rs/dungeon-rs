use bevy::prelude::Event;
use std::path::PathBuf;

/// A Bevy event emitted when the export has completed.
///
/// This is usually the final event for an [`crate::export::ExportRequest`].
#[derive(Debug, Event)]
pub struct ExportCompleted {
    /// The path to the final exported image.
    pub path: PathBuf,
}
