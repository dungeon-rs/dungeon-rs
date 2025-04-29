use crate::theme;
use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::color::Color;
use bevy::prelude::Val::Px;
use bevy::prelude::{
    AlignItems, AssetServer, BackgroundColor, Bundle, Button, Component, JustifyContent, Name,
    Node, UiRect, children, default,
};
use bevy::prelude::{SpawnRelated, Text, TextColor, TextFont};

/// Generates a button.
pub(crate) fn button(
    assets: &AssetServer,
    tag: impl Component,
    label: &'static str,
    background_color: Color,
) -> impl Bundle {
    (
        tag,
        Button,
        Name::new(label),
        AccessibilityNode(accesskit::Node::new(Role::Button)).set_label(label),
        Node {
            width: Px(72.0),
            height: Px(26.0),
            margin: UiRect::horizontal(Px(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(background_color),
        children![(
            Text::new(label),
            TextFont {
                font: theme::font(&assets),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
        )],
    )
}
