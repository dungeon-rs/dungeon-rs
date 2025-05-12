use crate::export::ongoing::OngoingExport;
use bevy::prelude::ResMut;

/// Waits for [crate::export::tasks::process_image_data] to finish processing the received data.
///
/// It monitors for updates and completion of the processing and moves that information back into ECS.
pub fn wait_for_image_processing(_ongoing_export: ResMut<OngoingExport>) {
    // TODO: poll for progress

    // TODO: poll for completion
}
