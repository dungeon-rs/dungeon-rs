mod camera_controls;
#[cfg(feature = "dev")]
mod debug;
mod theme;

use bevy::app::plugin_group;
use crate::{
    theme::ThemePlugin,
    camera_controls::CameraControlsPlugin,
};

pub use crate::theme::ToolbarAction;

#[cfg(feature = "dev")]
use debug::DebugPlugin;

plugin_group! {
    /// This plugin group will add all the plugins for DungeonRS:
    pub struct DungeonRsPlugin {
        :ThemePlugin,
        :CameraControlsPlugin,
        #[cfg(feature = "dev")]
        :DebugPlugin,
    }
}
