use crate::export::ExportRequest;
use crate::export::ongoing_export::OngoingExport;
use bevy::asset::Assets;
use bevy::image::Image;
use bevy::prelude::{Commands, EventReader, Res, ResMut};

/// Simple system that generates an [OngoingExport] resource in response to a [ExportRequest].
pub fn on_export_request(
    mut commands: Commands,
    images: ResMut<Assets<Image>>,
    mut requests: EventReader<ExportRequest>,
    export: Option<Res<OngoingExport>>,
) {
    if export.is_some() {
        return;
    }

    if let Some(event) = requests.read().next() {
        let export = OngoingExport::new(event, images);

        commands.insert_resource(export);
    }
}
