//! The headless editor opens on a new Project.
#![expect(clippy::missing_panics_doc, reason = "a test asserts")]

use bevy_app::App;
use bevy_ecs::hierarchy::ChildOf;
use drs_model::{AssetReferences, Bounds, Grid, Layer, Level, ModelPlugin, Project};
use drs_project_manager::ProjectManagerPlugin;

/// The editor starts with one new, unsaved Project holding one Level named `Level 1` with one
/// Layer named `Layer 1`, on a Grid of 256 pixels per cell.
#[test]
fn a_project_to_start_with() {
    let mut app = App::new();
    app.add_plugins((ModelPlugin, ProjectManagerPlugin));
    app.update();
    let world = app.world_mut();

    let projects: Vec<_> = world
        .query::<(
            bevy_ecs::entity::Entity,
            &Project,
            &Grid,
            &Bounds,
            &AssetReferences,
        )>()
        .iter(world)
        .map(|(entity, project, grid, bounds, references)| {
            (entity, project.clone(), *grid, *bounds, references.clone())
        })
        .collect();
    assert_eq!(projects.len(), 1, "exactly one Project");
    let (project, _, grid, bounds, references) = &projects[0];
    assert_eq!(grid.pixels_per_cell, 256);
    assert_eq!(*bounds, Bounds::default());
    assert!(references.assets.is_empty() && references.folders.is_empty());

    let levels: Vec<_> = world
        .query::<(bevy_ecs::entity::Entity, &Level, &ChildOf)>()
        .iter(world)
        .map(|(entity, level, parent)| (entity, level.name.clone(), parent.parent()))
        .collect();
    assert_eq!(levels.len(), 1, "exactly one Level");
    assert_eq!(levels[0].1, "Level 1");
    assert_eq!(levels[0].2, *project, "the Level belongs to the Project");

    let layers: Vec<_> = world
        .query::<(&Layer, &ChildOf)>()
        .iter(world)
        .map(|(layer, parent)| (layer.name.clone(), parent.parent()))
        .collect();
    assert_eq!(layers.len(), 1, "exactly one Layer");
    assert_eq!(layers[0].0, "Layer 1");
    assert_eq!(layers[0].1, levels[0].0, "the Layer belongs to the Level");
}
