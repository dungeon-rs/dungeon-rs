//! Development tooling: a screenshot of the window, saved where `DRS_SCREENSHOT` points a moment
//! after start, so the editor can be looked at from a terminal that cannot see the screen.

use bevy::ecs::system::{Commands, Local};
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

/// The frame the screenshot is taken on: long enough for the first layout to settle.
const FRAME: u32 = 90;

/// Takes the screenshot once, when `DRS_SCREENSHOT` names a file to save it as.
pub(crate) fn screenshot(mut commands: Commands, mut frames: Local<u32>) {
    *frames = frames.saturating_add(1);
    if *frames != FRAME {
        return;
    }
    let Some(path) = std::env::var_os("DRS_SCREENSHOT") else {
        return;
    };
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}
