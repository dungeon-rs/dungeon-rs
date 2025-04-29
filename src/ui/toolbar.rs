use crate::theme;
use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::Val::{Percent, Px};
use bevy::prelude::{
    AlignItems, BackgroundColor, Button, Changed, ChildOf, Color, Commands, Component, Event,
    EventWriter, Interaction, JustifyContent, Name, Node, Plugin, PostStartup, Query, Res, Text,
    TextColor, TextFont, UiRect, Update, With, default,
};

pub(super) struct Toolbar;

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
    commands
        .spawn((
            Node {
                width: Percent(100.0),
                height: Px(36.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.75)),
        ))
        .with_children(|parent| {
            create_button(parent, &assets, ToolbarAction::New);
            create_button(parent, &assets, ToolbarAction::Open);
            create_button(parent, &assets, ToolbarAction::Save);
            create_button(parent, &assets, ToolbarAction::Export);
        });
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

/// Helper function to create a button with the given label and tag.
fn create_button(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    assets: &AssetServer,
    tag: ToolbarAction,
) {
    // Generate the accessibility label from the tag.
    let name = match tag {
        ToolbarAction::New => "New",
        ToolbarAction::Open => "Open",
        ToolbarAction::Save => "Save",
        ToolbarAction::Export => "Export",
    };

    parent
        .spawn((
            tag,
            Button,
            Name::new(name),
            AccessibilityNode(accesskit::Node::new(Role::Button)).set_label(name),
            Node {
                width: Px(72.0),
                height: Px(26.0),
                margin: UiRect::horizontal(Px(4.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(name),
                TextFont {
                    font: theme::font(&assets),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}
