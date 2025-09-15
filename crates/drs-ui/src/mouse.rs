//! This module provides a way for getting the mouse position in world space.
//!
//! The mouse position is calculated using the `Camera` and `GlobalTransform` components
//! of the `Camera` entity, see <https://github.com/bevyengine/bevy/discussions/7970>.

use bevy::ecs::system::SystemParam;
use bevy::prelude::{BevyError, Camera, GlobalTransform, Query, Vec2, Window};

/// A [`SystemParam`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.SystemParam.html) that
/// provides convenient access to the mouse position, calculated on demand.
#[derive(SystemParam)]
pub struct MousePosition<'w, 's> {
    /// We need the `Window` to get the physical mouse position.
    window: Query<'w, 's, &'static Window>,
    /// We use the camera and it's relative position to calculate the world coordinates of the mouse.
    camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform)>,
}

impl MousePosition<'_, '_> {
    /// Attempts to calculate the current world coordinates of the mouse.
    ///
    /// # Errors
    /// This method may return an error if there is no active Window, Camera or if the physical
    /// mouse position could not be determined.
    pub fn get(&self) -> Result<Vec2, BevyError> {
        let window = self.window.single()?;
        let (camera, transform) = self.camera.single()?;

        match window.cursor_position() {
            None => Err(BevyError::from("Failed to get physical mouse position")),
            Some(position) => Ok(camera.viewport_to_world_2d(transform, position)?),
        }
    }
}
