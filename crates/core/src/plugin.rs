use crate::async_ecs::handle_async_components;
use bevy::app::App;
use bevy::prelude::{FixedPostUpdate, Plugin};

#[derive(Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, handle_async_components);
    }
}
