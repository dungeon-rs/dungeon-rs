pub mod events;

use bevy::app::App;
use bevy::prelude::{EventReader, FixedPostUpdate, Plugin, info};

pub use crate::export::events::export_completed::ExportCompleted;
pub use crate::export::events::export_progress::ExportProgress;
pub use crate::export::events::export_request::ExportRequest;

#[derive(Default)]
pub struct ExportPlugin;

impl Plugin for ExportPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExportRequest>()
            .add_event::<ExportProgress>()
            .add_event::<ExportCompleted>();

        app.add_systems(FixedPostUpdate, on_export_request);
    }
}

fn on_export_request(mut event_reader: EventReader<ExportRequest>) {
    for event in event_reader.read() {
        info!("Received export request: {:?}", event);
    }
}
