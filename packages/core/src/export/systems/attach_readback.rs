use crate::export::ongoing_export::{OngoingExport, ExportState};
use crate::export::systems;
use bevy::prelude::{Camera, Commands, Entity, ResMut, Single, With};
use bevy::render::gpu_readback::Readback;

/// System that runs when the [OngoingExport] is in [ExportState::Preparing] to initialize readback.
pub fn attach_readback(
    mut commands: Commands,
    camera: Single<Entity, With<Camera>>,
    mut export: ResMut<OngoingExport>,
) {
    if !matches!(export.state, ExportState::Preparing) {
        return;
    }

    commands.entity(*camera).with_children(|parent| {
        parent
            .spawn(Readback::texture(export.render_texture.clone_weak()))
            .observe(systems::read_frame);
    });

    export.state = ExportState::Capturing;
}
