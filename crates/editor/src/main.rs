#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::suspicious, clippy::complexity)]

use bevy::prelude::*;

fn main() -> AppExit {
    App::new().add_plugins(DefaultPlugins).run()
}
