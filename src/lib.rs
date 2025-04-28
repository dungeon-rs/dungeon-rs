mod camera_controls;
#[cfg(feature = "dev")]
mod debug;

use bevy::app::plugin_group;
use camera_controls::CameraControlsPlugin;

#[cfg(feature = "dev")]
use debug::DebugPlugin;

plugin_group! {
    /// This plugin group will add all the plugins for DungeonRS:
    pub struct DungeonRsPlugin {
        :CameraControlsPlugin,
        #[cfg(feature = "dev")]
        :DebugPlugin,
    }
}
