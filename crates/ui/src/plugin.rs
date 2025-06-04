//! Defines the [`UIPlugin`] which inserts all UI related functionality into the bevy `App`.
use crate::camera::{UICamera, camera_control_system};
use bevy::app::App;
use bevy::prelude::{Commands, Plugin, PostUpdate, Startup};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

/// A [Bevy](https://bevyengine.org/) plugin that adds UI to the app it's added to.
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, camera_control_system);
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, setup_ui_camera);
    }
}

/// Sets up the [`UICamera`] bundle.
#[utils::bevy_system]
fn setup_ui_camera(mut commands: Commands) {
    commands.spawn(UICamera::bundle());
}
