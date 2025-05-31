#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::suspicious, clippy::complexity)]

use bevy::prelude::*;
use io::IOPlugin;
use ui::UIPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, IOPlugin, UIPlugin))
        .run()
}
