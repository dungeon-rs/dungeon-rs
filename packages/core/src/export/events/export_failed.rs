use bevy::prelude::{BevyError, Event};

/// When a [create::export::ExportRequest] fails, this event explains why.
#[derive(Debug, Event)]
pub struct ExportFailed {
    /// The error that caused the export to fail.
    pub error: BevyError,
}
