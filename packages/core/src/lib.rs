pub mod events;
pub mod states;

use bevy::app::App;
use bevy::prelude::Plugin;

/// The core plugin registers all resources, components and systems required for the functionality.
/// This plugin is used in both "visual" and "headless" mode, and handles events, input, rendering
/// and other core functionality such as loading/saving files and so forth.
#[derive(Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, _app: &mut App) {}
}
