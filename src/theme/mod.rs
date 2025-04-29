use bevy::prelude::{AssetServer, Color, Font, Handle};

/// The background colour for buttons in the user interface.
pub const BUTTON_COLOR: Color = Color::srgb(0.18, 0.18, 0.18);

/// Get the font for the user interface of DungeonRS.
pub fn font(assets: &AssetServer) -> Handle<Font> {
    assets.load("fonts/opensans.ttf")
}
