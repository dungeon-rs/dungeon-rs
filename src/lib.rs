mod camera_controls;

use bevy::app::plugin_group;
use camera_controls::CameraControlsPlugin;

plugin_group! {
    /// This plugin group will add all the plugins for DungeonRS:
    pub struct DungeonRsPlugin {
        :CameraControlsPlugin,
    }
}
