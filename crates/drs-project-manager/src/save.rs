//! Save: the whole Project written to its file.

use crate::ProjectManagerError;
use crate::snapshot::gather;
use bevy_ecs::world::World;
use drs_history::{History, Position};
use drs_model::{
    PROJECT_EXTENSION, Project, ProjectSaved, SavedMark, project_name_of, with_extension_if_missing,
};
use drs_project_access::write_project;
use std::path::PathBuf;

/// The history's position, or the position of an empty history when the World has none.
pub(crate) fn history_position(world: &World) -> Position {
    world
        .get_resource::<History>()
        .map_or_else(Position::default, History::position)
}

/// Save: writes the Project to `path` with the Project extension added when it lacks it, or to
/// the file the Project was last saved to or opened from, and records that file and the
/// history's position as the saved mark. Nothing in the World changes but the Project's name,
/// which becomes its file's once the file is written; a refusal changes nothing.
///
/// # Errors
///
/// [`ProjectManagerError::NoFile`] when no path is given and the Project has no file yet,
/// [`ProjectManagerError::NoProject`] without exactly one Project, or the error of gathering or
/// writing the file.
pub(crate) fn save_project(
    world: &mut World,
    path: Option<PathBuf>,
) -> Result<ProjectSaved, ProjectManagerError> {
    let path = match path {
        Some(path) => with_extension_if_missing(path, PROJECT_EXTENSION),
        None => world
            .get_resource::<SavedMark>()
            .and_then(|mark| mark.file.clone())
            .ok_or(ProjectManagerError::NoFile)?,
    };
    let projects = crate::projects(world);
    let [project] = projects.as_slice() else {
        return Err(ProjectManagerError::NoProject);
    };
    let project = *project;

    let snapshot = gather(world, project)?;
    write_project(&path, &snapshot)?;
    if let Some(mut current) = world.get_mut::<Project>(project) {
        current.name = project_name_of(&path);
    }

    let position = history_position(world);
    world.insert_resource(SavedMark {
        file: Some(path.clone()),
        position,
    });
    Ok(ProjectSaved { path })
}
