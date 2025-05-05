use bevy::app::plugin_group;
use dungeonrs_core::CorePlugin;
use dungeonrs_ui::UIPlugin;

plugin_group! {
    pub(super) struct EditorPlugin {
        :UIPlugin,
        :CorePlugin,
    }
}
