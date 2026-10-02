//! How a background thread tells the crash handler about the panics it catches itself.

use bevy_ecs::resource::Resource;

/// Marks whether the calling thread catches its own panics from now on, so that the crash
/// handler logs a panic on it instead of reporting a crash. The Host puts in the diagnostics
/// Utility's marker; without it, as in a headless test, marking does nothing.
#[derive(Resource, Debug, Clone, Copy)]
pub struct CaughtPanics(pub fn(bool));

impl Default for CaughtPanics {
    fn default() -> Self {
        Self(|_| {})
    }
}
