use bevy::asset::AssetContainer;
use crate::export::ongoing_export::{ExportState, OngoingExport};
use crate::export::callbacks;
use bevy::prelude::{Camera, Commands, Entity, Query, ResMut, Single, With};
use bevy::render::camera::{ImageRenderTarget, RenderTarget};
use bevy::render::gpu_readback::Readback;

/// System that runs when the [OngoingExport] is in [ExportState::Preparing] to initialize readback.
pub fn attach_readback(
    mut commands: Commands,
    mut camera: Query<(&mut Camera, Entity), With<Camera>>,
    mut export: ResMut<OngoingExport>,
) {
    if !matches!(export.state, ExportState::Preparing) {
        return;
    }

    let Ok((camera, entity)) = &mut camera.single_mut() else {
        return;
    };

    camera.target = RenderTarget::Image(ImageRenderTarget::from(export.render_texture.clone_weak()));
    commands.entity(*entity).with_children(|parent| {
        parent
            .spawn(Readback::texture(export.render_texture.clone_weak()))
            .observe(callbacks::read_frame);
    });

    export.state = ExportState::Capturing;
}
