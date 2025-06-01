use bevy::app::App;
use bevy::prelude::{AssetServer, Camera2d, Commands, Plugin, ResMut, Sprite, Startup, Transform};
use bevy::render::view::RenderLayers;
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin, egui};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub struct UIPlugin;

const LAYER_A: usize = 1;
const LAYER_B: usize = 2;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(EguiContextPass, window)
        .add_systems(Startup, setup);
    }
}

pub fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((Camera2d, RenderLayers::layer(LAYER_A)));

    commands.spawn((
        Sprite::from_image(asset_server.load("logo.png")),
        Transform::from_xyz(250.0, 250.0, 0.0),
        RenderLayers::layer(LAYER_A),
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("logo.png")),
        RenderLayers::layer(LAYER_B),
    ));
}

pub fn window(mut contexts: EguiContexts) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });
}
