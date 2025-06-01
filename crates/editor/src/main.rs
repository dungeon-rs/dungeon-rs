#![doc = include_str!("../README.md")]

use bevy::prelude::*;
use io::IOPlugin;
use ui::UIPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, IOPlugin, UIPlugin))
        .run()
}
