use crate::export::ExportRequest;
use crate::export::ongoing::OngoingExport;
use bevy::prelude::{Assets, Commands, EventReader, Image, ResMut};

/// This system checks for incoming [ExportRequest]s and initializes the [OngoingExport].
pub fn check_for_requests(
    mut requests: EventReader<ExportRequest>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(request) = requests.read().next() else {
        return;
    };

    commands.insert_resource(OngoingExport::new(request, &mut images));
}
