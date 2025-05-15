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

use crate::components::{Layer, Level, Project};
use crate::persistence::save_file::SaveFile;
use crate::prelude::SaveProjectRequest;
use bevy::app::App;
use bevy::asset::Assets;
use bevy::asset::ron::ser::to_string_pretty;
use bevy::prelude::{
    Children, ColorMaterial, EventReader, FixedPostUpdate, Mesh2d, MeshMaterial2d, Name, Plugin,
    Query, Res, Result, Transform, With, default, info,
};
use std::fs::write;

#[derive(Default)]
pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveProjectRequest>();
        app.add_systems(FixedPostUpdate, poll_save_project_events);
    }
}

#[allow(clippy::too_many_arguments)]
fn poll_save_project_events(
    mut save_projects: EventReader<SaveProjectRequest>,
    project_query: Query<&Children, With<Project>>,
    children_query: Query<&Children>,
    level_query: Query<&Name, With<Level>>,
    layer_query: Query<(&Layer, &Name)>,
    mesh_query: Query<&Mesh2d>,
    transform_query: Query<&Transform>,
    material_query: Query<&MeshMaterial2d<ColorMaterial>>,
    materials: Res<Assets<ColorMaterial>>,
) -> Result {
    for save_project in save_projects.read() {
        let save = SaveFile::new(
            project_query,
            children_query,
            level_query,
            layer_query,
            mesh_query,
            transform_query,
            material_query,
            &materials,
        )?;

        write(
            save_project.path.as_path(),
            to_string_pretty(&save, default()).expect("FAILED TO SERIALISE"),
        )
        .expect("FAILED TO SAVE");
        info!("Saved to {}", save_project.path.display());
    }

    Ok(())
}
