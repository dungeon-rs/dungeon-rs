#![cfg(test)]
#![allow(missing_docs)]
#![allow(clippy::pedantic)]

use anyhow::Context;
use bevy::prelude::*;
use data::{Layer, Level, Project};
use io::*;
use std::time::Duration;
use std::{fs::read_to_string, path::PathBuf};
use tempfile::tempdir;

#[test]
fn save_project_event() -> anyhow::Result<()> {
    // Holds output files for this test, we hold the variable since it's deleted on drop.
    let temp_dir = tempdir()?;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, IOPlugin));
    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs(1)));
    app.world_mut().spawn((
        Project::new("Example Project"),
        children![(
            Level::new("First Level"),
            children![(Layer::new("First Layer", Transform::IDENTITY), children![])]
        )],
    ));
    let (_, project) = app
        .world_mut()
        .query::<(&Project, Entity)>()
        .single(app.world())?;

    let mut output = PathBuf::from(temp_dir.path());
    output.push("save_project_event_test_output.json"); // set output filename
    // run the schedules once to process Setup and spawn
    app.update();

    app.world_mut()
        .send_event(SaveProjectEvent::new(project, output.clone()));

    // advance world to send event and once more to run systems
    app.update();
    app.update();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_secs(2));
    app.world_mut().run_schedule(FixedPostUpdate);
    // TODO: currently tests don't seem to fail when the system internally fails (for whatever reason).

    let json = read_to_string(output.clone())
        .with_context(|| format!("Output file {} could not be opened", output.display()))?;

    assert!(json.starts_with('{'));
    Ok(())
}
