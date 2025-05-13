use bevy::app::plugin_group;
use core::CorePlugin;
use ui::UIPlugin;

plugin_group! {
    pub(super) struct EditorPlugin {
        :UIPlugin,
        :CorePlugin,
    }
}
