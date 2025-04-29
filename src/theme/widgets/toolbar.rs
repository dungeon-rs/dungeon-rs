use crate::theme::widgets::button::button;
use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::prelude::SpawnRelated;
use bevy::prelude::Val::{Percent, Px};
use bevy::prelude::{
    AlignItems, BackgroundColor, Button, Changed, Color, Commands, Component, Event, EventWriter,
    Interaction, JustifyContent, Node, Plugin, PostStartup, Query, Res, UiRect, Update, With,
    children, default,
};

pub(in crate::theme) struct Toolbar;

/// The buttons on the toolbar, dispatched as events when clicked.
#[derive(Component, Event, Clone, Copy, Debug)]
pub enum ToolbarAction {
    New,
    Open,
    Save,
    Export,
}

impl Plugin for Toolbar {
    fn build(&self, app: &mut App) {
        app.add_event::<ToolbarAction>()
            .add_systems(PostStartup, setup)
            .add_systems(Update, on_interaction);
    }
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        Node {
            width: Percent(100.0),
            height: Px(36.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0., 0., 0., 0.75)),
        children![
            button(&assets, ToolbarAction::New, "New", Color::NONE),
            button(&assets, ToolbarAction::Open, "Open", Color::NONE),
            button(&assets, ToolbarAction::Save, "Save", Color::NONE),
            button(&assets, ToolbarAction::Export, "Export", Color::NONE),
        ],
    ));
}

/// When a button on the toolbar is clicked, dispatch the corresponding event.
fn on_interaction(
    mut q: Query<(&ToolbarAction, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut writer: EventWriter<ToolbarAction>,
) {
    for (action, interaction) in &mut q {
        if *interaction == Interaction::Pressed {
            writer.write(*action);
        }
    }
}
