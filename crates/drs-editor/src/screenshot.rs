//! Development tooling: a screenshot of the window, saved where `DRS_SCREENSHOT` points a moment
//! after start, so the editor can be looked at from a terminal that cannot see the screen.

use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Commands, Local, Res};
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use std::path::PathBuf;

/// The frame the screenshot is taken on: long enough for the first layout to settle.
const FRAME: u32 = 90;

/// Where the screenshot is saved, as `DRS_SCREENSHOT` named it when the plugins built.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScreenshotPath(PathBuf);

impl ScreenshotPath {
    /// Reads `DRS_SCREENSHOT` once, while the plugins build; `None` when it is unset.
    pub(crate) fn from_environment() -> Option<Self> {
        let path = std::env::var_os("DRS_SCREENSHOT")?;
        Some(Self(PathBuf::from(path)))
    }
}

/// Takes the screenshot once, on the frame the first layout has settled.
pub(crate) fn screenshot(
    mut commands: Commands,
    path: Res<ScreenshotPath>,
    mut frames: Local<u32>,
) {
    *frames = frames.saturating_add(1);
    if *frames != FRAME {
        return;
    }
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path.0.clone()));
}
