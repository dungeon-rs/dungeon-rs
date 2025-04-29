mod camera_controls;
#[cfg(feature = "dev")]
mod debug;
mod theme;
mod ui;

use crate::{camera_controls::CameraControlsPlugin, theme::ThemePlugin, ui::UiPlugin};
use bevy::app::plugin_group;

pub use crate::theme::ToolbarAction;

#[cfg(feature = "dev")]
use debug::DebugPlugin;

plugin_group! {
    /// This plugin group will add all the plugins for DungeonRS:
    pub struct DungeonRsPlugin {
        :UiPlugin,
        :ThemePlugin,
        :CameraControlsPlugin,
        #[cfg(feature = "dev")]
        :DebugPlugin,
    }
}
