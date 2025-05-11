use crate::export::screenshot::{Screenshot, ScreenshotStatus};
use crate::export::systems;
use bevy::prelude::{Camera, Commands, Entity, ResMut, Single, With};
use bevy::render::gpu_readback::Readback;

/// System that runs when the [Screenshot] is in [ScreenshotStatus::Preparing] to initialize readback.
pub fn attach_readback(
    mut commands: Commands,
    camera: Single<Entity, With<Camera>>,
    mut screenshot: ResMut<Screenshot>,
) {
    if !matches!(screenshot.status, ScreenshotStatus::Preparing) {
        return;
    }

    commands.entity(*camera).with_children(|parent| {
        parent
            .spawn(Readback::texture(screenshot.render_texture.clone_weak()))
            .observe(systems::read_frame);
    });

    screenshot.status = ScreenshotStatus::Capturing;
}
