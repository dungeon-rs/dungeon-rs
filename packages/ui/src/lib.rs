mod widgets;

use crate::widgets::toolbar::toolbar;
use bevy::prelude::*;
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin};
use bevy_inspector_egui::{DefaultInspectorConfigPlugin, reflect_inspector};

/// The UI plugin handles registering and setting up all user interface elements of the editor.
/// This module is unavailable in headless mode and should not contain any actual functionality,
/// just build the user interface.
#[derive(Default)]
pub struct UIPlugin;

#[derive(Resource)]
struct EntityToInspect {
    entity: Entity,
}

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_systems(Startup, setup)
        .add_systems(EguiContextPass, editor_interface);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Pickable::default(),
            Name::from("left"),
            Sprite::from_image(asset_server.load("logo.png")),
            Transform::from_xyz(-100.0, 0.0, 0.0),
        ))
        .observe(on_click_sprite);

    commands
        .spawn((
            Pickable::default(),
            Name::from("right"),
            Sprite::from_image(asset_server.load("logo.png")),
            Transform::from_xyz(100.0, 0.0, 0.0),
        ))
        .observe(on_click_sprite);
}

fn editor_interface(
    mut contexts: EguiContexts,
    entity: Option<Res<EntityToInspect>>,
    mut query: Query<&mut Transform>,
    registry: Res<AppTypeRegistry>,
) {
    let Some(context) = contexts.try_ctx_mut() else {
        return;
    };

    toolbar(context);

    egui::SidePanel::right("inspector_panel")
        .resizable(true)
        .show(context, |ui| {
            //         side_panel.ui(ui, &mut Viewer);
            if let Some(entity) = entity {
                let registry = registry.read();
                let mut transform = query.get_mut(entity.entity).unwrap();

                reflect_inspector::ui_for_value(&mut transform.translation, ui, &*registry);
                reflect_inspector::ui_for_value(&mut transform.rotation, ui, &*registry);
                reflect_inspector::ui_for_value(&mut transform.scale, ui, &*registry);
            }
        });
}

fn on_click_sprite(
    event: Trigger<Pointer<Pressed>>,
    mut commands: Commands,
    mut query: Query<&Name>,
) {
    let Ok(name) = query.get_mut(event.target) else {
        return;
    };

    info!("Clicked on {}", name);
    commands.insert_resource(EntityToInspect {
        entity: event.target,
    });
}
