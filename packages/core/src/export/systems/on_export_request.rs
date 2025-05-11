use crate::export::ExportRequest;
use crate::export::screenshot::Screenshot;
use bevy::asset::Assets;
use bevy::image::Image;
use bevy::prelude::{Commands, EventReader, Res, ResMut};

/// Simple system that generates a [Screenshot] resource in response to a [ExportRequest].
pub fn on_export_request(
    mut commands: Commands,
    images: ResMut<Assets<Image>>,
    mut requests: EventReader<ExportRequest>,
    screenshot: Option<Res<Screenshot>>,
) {
    if screenshot.is_some() {
        return;
    }

    if let Some(event) = requests.read().next() {
        let screenshot = Screenshot::new(event, images);

        commands.insert_resource(screenshot);
    }
}
