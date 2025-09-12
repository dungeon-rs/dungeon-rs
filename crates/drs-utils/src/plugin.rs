//! This module defines the [`UtilsPlugin`].

use crate::async_ecs::handle_async_components;
use bevy::app::App;
use bevy::prelude::{FixedPostUpdate, Plugin};

/// The [`UtilsPlugin`] registers systems for handling asynchronous components.
#[derive(Default)]
pub struct UtilsPlugin;

impl Plugin for UtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, handle_async_components);
    }
}
