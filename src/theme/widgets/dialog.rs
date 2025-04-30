use bevy::app::App;
use bevy::prelude::Val::{Percent, Px};
use bevy::prelude::{
    AlignItems, BackgroundColor, Bundle, Changed, Color, Component, Display, Entity, FlexDirection,
    Interaction, JustifyContent, Query, SpawnRelated, Text, UiRect, Update, With,
};
use bevy::prelude::{Commands, Node, Plugin, PositionType, children, default};
use bevy::ui::FocusPolicy;

pub struct DialogPlugin;

/// Marker component for dialogs.
#[derive(Component)]
struct DialogWindow;

/// Marker component for close buttons.
#[derive(Component)]
#[require(Node, FocusPolicy::Block, Interaction)]
struct DialogCloseButton;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_close_button_click);
    }
}

pub fn dialog(children: impl Bundle) -> impl Bundle {
    (
        DialogWindow,
        Node {
            position_type: PositionType::Absolute,
            width: Percent(100.),
            height: Percent(100.),
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(
            Node {
                width: Px(350.),
                height: Px(350.),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            children![
                // Toolbar
                (
                    Node {
                        position_type: PositionType::Relative,
                        width: Percent(100.),
                        display: Display::Flex,
                        justify_content: JustifyContent::End,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                    children![
                        // Close button
                        (
                            DialogCloseButton,
                            Node {
                                margin: UiRect::horizontal(Px(5.)),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            Text::new("X"),
                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                        ),
                    ]
                ),
                // Content
                (
                    Node {
                        flex_grow: 1.,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                    children,
                )
            ],
        )],
    )
}

fn on_close_button_click(
    mut commands: Commands,
    mut query: Query<
        (&DialogCloseButton, &Interaction),
        (Changed<Interaction>, With<DialogCloseButton>),
    >,
    mut dialogs: Query<Entity, With<DialogWindow>>,
) {
    for (_button, &interaction) in &mut query {
        if interaction == Interaction::Pressed {
            if let Ok(dialog) = dialogs.single_mut() {
                commands.entity(dialog).despawn();
            }
        }
    }
}
