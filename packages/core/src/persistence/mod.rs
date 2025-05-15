//! The persistence module handles saving and restoring DungeonRS' state from (and to) disk.
//!
//! This module contains copies of a lot of data structures found in Core, but this is intentional,
//! the copies contained in this module are intended (and optimised) for serialisation.
//! These copies can be found under [crate::persistence::entities].
//!
//! Serialisation logic itself is implemented using [Serde](https://serde.rs/), keeping the format
//! to which we serialise flexible.
//! Currently, large datasets (like projects) will be serialised using [MessagePack](https://msgpack.org/)
//! while smaller files intended to be edited by users (like config) will be serialised using JSON or [Ron](https://docs.rs/ron).

pub(super) mod entities;
pub(super) mod events;
pub(super) mod save_file;

use crate::persistence::save_file::SaveFile;
use crate::prelude::SaveProjectRequest;
use bevy::app::App;
use bevy::asset::ron::ser::to_string_pretty;
use bevy::prelude::{default, EventReader, FixedPostUpdate, Plugin};
use std::fs::write;

#[derive(Default)]
pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveProjectRequest>();
        app.add_systems(FixedPostUpdate, poll_save_project_events);
    }
}

fn poll_save_project_events(mut save_projects: EventReader<SaveProjectRequest>) {
    for save_project in save_projects.read() {
        let save = SaveFile::new();
        write(
            "output.drs",
            to_string_pretty(&save, default()).expect("FAILED TO SERIALISE"),
        )
            .expect("FAILED TO SAVE");
    }
}
