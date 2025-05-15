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

use crate::components::{Layer, Level, Project, Texture};
use crate::persistence::events::load_project_request::LoadProjectRequest;
use crate::persistence::save_file::SaveFile;
use crate::prelude::SaveProjectRequest;
use bevy::app::App;
use bevy::asset::{AssetServer, Assets};
use bevy::prelude::{
    Children, ColorMaterial, Commands, Entity, EventReader, FixedPostUpdate, Mesh, Mesh2d,
    MeshMaterial2d, Name, Plugin, Query, Res, ResMut, Result, Transform, With, info,
};
use std::fs::write;

#[derive(Default)]
pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveProjectRequest>()
            .add_event::<LoadProjectRequest>();
        app.add_systems(
            FixedPostUpdate,
            (poll_save_project_events, poll_load_project_events),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn poll_save_project_events(
    mut save_projects: EventReader<SaveProjectRequest>,
    project_query: Query<(&Project, &Name, &Children), With<Project>>,
    children_query: Query<&Children>,
    level_query: Query<&Name, With<Level>>,
    layer_query: Query<(&Layer, &Name)>,
    mesh_query: Query<(&Texture, Option<&Name>), With<Mesh2d>>,
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

        #[cfg(feature = "dev")]
        write(
            save_project.path.as_path(),
            serde_json::to_string_pretty(&save)?,
        )
        .expect("FAILED TO SAVE");

        #[cfg(not(feature = "dev"))]
        write(save_project.path.as_path(), rmp_serde::to_vec_named(&save)?)
            .expect("FAILED TO SAVE");

        info!("Saved to {}", save_project.path.display());
    }

    Ok(())
}

fn poll_load_project_events(
    mut load_projects: EventReader<LoadProjectRequest>,
    mut commands: Commands,
    project: Query<Entity, With<Project>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) -> Result {
    for load_project in load_projects.read() {
        let content = std::fs::read_to_string(&load_project.path)?;
        #[cfg(feature = "dev")]
        let save: SaveFile = serde_json::from_str(&content)?;
        #[cfg(not(feature = "dev"))]
        let save: SaveFile = rmp_serde::from_slice(content.as_bytes())?;

        if let Ok(project) = project.single() {
            info!("Despawning existing hierarchy");
            commands.entity(project).despawn();
        }

        save.restore(&mut commands, &mut meshes, &mut materials, &asset_server);
    }

    Ok(())
}
