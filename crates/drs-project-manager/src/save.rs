//! Save: the whole Project written to its file.

use crate::ProjectManagerError;
use crate::file::gather;
use bevy_ecs::world::World;
use drs_history::{History, Position};
use drs_model::{Project, ProjectSaved, SavedMark, project_name_of, with_project_extension};
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
/// which is its file's; a refusal leaves even that as it was.
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
        Some(path) => with_project_extension(path),
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

    let previous_name = world
        .get_mut::<Project>(project)
        .map(|mut current| std::mem::replace(&mut current.name, project_name_of(&path)))
        .ok_or(ProjectManagerError::NoProject)?;
    let written = gather(world, project).and_then(|file| Ok(write_project(&path, &file)?));
    if let Err(error) = written {
        if let Some(mut current) = world.get_mut::<Project>(project) {
            current.name = previous_name;
        }
        return Err(error);
    }

    let position = history_position(world);
    world.insert_resource(SavedMark {
        file: Some(path.clone()),
        position,
    });
    Ok(ProjectSaved { path })
}
