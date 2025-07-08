//! Defines the [`UIPlugin`] which inserts all UI related functionality into the bevy `App`.

use crate::camera::{camera_control_system, setup_ui_camera};
use crate::dialogs::Dialogs;
use crate::layout::render_editor_layout;
use crate::notifications::Notifications;
use crate::state::UiState;
use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::prelude::{Commands, Plugin, PostUpdate, ResMut, Sprite, Startup};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

/// A [Bevy](https://bevyengine.org/) plugin that adds UI to the app it's added to.
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .insert_resource(Notifications::default())
            .insert_resource(Dialogs::default());

        // Camera controls
        app.add_systems(PostUpdate, camera_control_system)
            .add_systems(Startup, setup_ui_camera);

        // editor docking layout
        app.insert_resource(UiState::default())
            .add_systems(EguiPrimaryContextPass, render_editor_layout);

        // for testing purposes.
        app.add_systems(Startup, spawn_demo_asset);
    }
}

/// Test system that spawns an image, used to validate resource loading.
#[utils::bevy_system]
fn spawn_demo_asset(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(Sprite::from_image(asset_server.load("logo.png")));
}
