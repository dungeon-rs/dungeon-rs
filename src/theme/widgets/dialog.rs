use bevy::app::App;
use bevy::prelude::Val::{Percent, Px};
use bevy::prelude::{
    BackgroundColor, Color, Display, FlexDirection, JustifyContent, SpawnRelated, Text, UiRect,
};
use bevy::prelude::{Commands, Node, Plugin, PositionType, Startup, children, default};

pub struct DialogPlugin;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Percent(50.),
            top: Percent(50.),
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
                children![],
            )
        ],
    ));
}
