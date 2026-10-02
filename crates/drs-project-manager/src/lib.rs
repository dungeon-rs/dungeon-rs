#![doc = include_str!("../README.md")]

mod export;
mod open;
mod resolve;
mod save;
mod snapshot;

pub use export::ExportError;

use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::message::MessageReader;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::system::{Commands, SystemState};
use bevy_ecs::world::World;
use drs_model::{
    Layer, Level, ManagerSystems, OpenProject, Project, ProjectRefused, ProjectRequest,
    SaveProject, SavedMark, SerialisationError,
};
use drs_project_access::ProjectAccessError;
use std::path::PathBuf;

/// Why a Project request could not be carried out.
#[derive(Debug, thiserror::Error)]
pub enum ProjectManagerError {
    /// Save was asked of a Project that has no file yet, so where to save it must be chosen.
    #[error("the Project has not been saved yet; choose where to save it")]
    NoFile,
    /// The World holds no Project, or more than one.
    #[error("there is not exactly one Project to save")]
    NoProject,
    /// The World holds no serialisation registry, so nothing can be written or read.
    #[error("the World has no serialisation registry")]
    NoRegistry,
    /// The file could not be read or written.
    #[error(transparent)]
    Access(#[from] ProjectAccessError),
    /// A component could not be written or read.
    #[error(transparent)]
    Serialisation(#[from] SerialisationError),
    /// The file is a Project file whose parts do not fit together.
    #[error("{} is malformed: {reason}", path.display())]
    Malformed {
        /// The file.
        path: PathBuf,
        /// What does not fit.
        reason: String,
    },
}

/// Creates the new Project the editor opens on, and handles [`SaveProject`], [`OpenProject`],
/// and [`drs_model::ExportLevel`], answering each with its report or with its refusal.
pub struct ProjectManagerPlugin;

impl Plugin for ProjectManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SavedMark>()
            .add_systems(Startup, create_new_project)
            .add_systems(Update, handle_save.in_set(ManagerSystems::Commands))
            .add_systems(Update, handle_open.in_set(ManagerSystems::Commands))
            .add_systems(
                Update,
                export::handle_export_level.in_set(ManagerSystems::Commands),
            )
            .add_systems(
                Update,
                resolve::handle_folder_changed.after(ManagerSystems::Redo),
            )
            .add_systems(
                Update,
                resolve::resolve_changed_references.after(ManagerSystems::Redo),
            );
    }
}

/// Spawns a new, unsaved Project with one Level holding one Layer.
///
/// The Grid, the Bounds, the Asset Reference table, and the resolution table come with the
/// Project by default.
fn create_new_project(mut commands: Commands) {
    let project = commands
        .spawn(Project {
            name: "Untitled".to_owned(),
        })
        .id();
    let level = commands
        .spawn((
            Level {
                name: "Level 1".to_owned(),
            },
            ChildOf(project),
        ))
        .id();
    commands.spawn((
        Layer {
            name: "Layer 1".to_owned(),
        },
        ChildOf(level),
    ));
}

/// Every Project in the World.
pub(crate) fn projects(world: &mut World) -> Vec<Entity> {
    world
        .query::<(Entity, &Project)>()
        .iter(world)
        .map(|(entity, _)| entity)
        .collect()
}

/// Carries out every [`SaveProject`] request, answering each with [`drs_model::ProjectSaved`]
/// or [`ProjectRefused`].
fn handle_save(world: &mut World, requests: &mut SystemState<MessageReader<SaveProject>>) {
    let requests: Vec<SaveProject> = match requests.get_mut(world) {
        Ok(mut reader) => reader.read().cloned().collect(),
        Err(_) => return,
    };
    for SaveProject { path } in requests {
        match save::save_project(world, path.clone()) {
            Ok(saved) => {
                world.write_message(saved);
            }
            Err(error) => {
                world.write_message(ProjectRefused {
                    request: ProjectRequest::Save { path },
                    reason: error.to_string(),
                });
            }
        }
    }
}

/// Carries out every [`OpenProject`] request, answering each with [`drs_model::ProjectOpened`]
/// or [`ProjectRefused`].
fn handle_open(world: &mut World, requests: &mut SystemState<MessageReader<OpenProject>>) {
    let requests: Vec<OpenProject> = match requests.get_mut(world) {
        Ok(mut reader) => reader.read().cloned().collect(),
        Err(_) => return,
    };
    for OpenProject { path } in requests {
        match open::open_project(world, path.clone()) {
            Ok(opened) => {
                world.write_message(opened);
            }
            Err(error) => {
                world.write_message(ProjectRefused {
                    request: ProjectRequest::Open { path },
                    reason: error.to_string(),
                });
            }
        }
    }
}
