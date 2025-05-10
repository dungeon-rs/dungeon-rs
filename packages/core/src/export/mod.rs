pub mod events;
mod export_frame;
mod screenshot;

use bevy::app::App;
use bevy::prelude::{
    Assets, Commands, EventReader, FixedPostUpdate, Image, Plugin, PostUpdate, Res, ResMut,
};

pub use crate::export::events::export_completed::ExportCompleted;
pub use crate::export::events::export_progress::ExportProgress;
pub use crate::export::events::export_progress::ExportStatus;
pub use crate::export::events::export_request::ExportRequest;

use crate::export::screenshot::Screenshot;

#[derive(Default)]
pub struct ExportPlugin;

impl Plugin for ExportPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExportRequest>()
            .add_event::<ExportProgress>()
            .add_event::<ExportCompleted>();

        app.add_systems(PostUpdate, export_frame::export_frame);
        app.add_systems(FixedPostUpdate, on_export_request);
    }
}

/// Simple system that generates a [Screenshot] resource in response to a [ExportRequest].
fn on_export_request(
    mut commands: Commands,
    images: ResMut<Assets<Image>>,
    mut requests: EventReader<ExportRequest>,
    screenshot: Option<Res<Screenshot>>,
) {
    if screenshot.is_some() {
        return;
    }

    if let Some(event) = requests.read().next() {
        commands.insert_resource(Screenshot::new(event, images));
    }
}
