#![cfg(test)]
#![allow(missing_docs)]
#![allow(clippy::pedantic)]

use anyhow::Context;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use data::{Layer, Level, Project};
use io::*;
use std::time::Duration;
use std::{fs::read_to_string, path::PathBuf};
use tempfile::tempdir;
use utils::CorePlugin;

/// Advance the world (similar to the pattern used in utils tests)
fn advance_world(app: &mut App) {
    app.update();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_secs(2));
    app.world_mut().run_schedule(FixedPostUpdate);
    app.update();
}

/// Continuously advance the world until all AsyncComponents have been processed.
fn process_async_components(app: &mut App) {
    // advance world to send event and process async components
    advance_world(app);

    // Give the async task time to complete by advancing the world multiple times
    // The async IO task needs time to complete, so we advance until the AsyncComponent is removed
    for _ in 0..100 {
        let remaining_components = app
            .world_mut()
            .query_filtered::<Entity, With<utils::AsyncComponent>>()
            .iter(app.world())
            .count();

        if remaining_components == 0 {
            // AsyncComponent has been removed, task completed
            break;
        }

        advance_world(app);
    }
}

#[test]
fn save_project_event() -> anyhow::Result<()> {
    // Holds output files for this test, we hold the variable since it's deleted on drop.
    let temp_dir = tempdir()?;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CorePlugin, IOPlugin));
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

    process_async_components(&mut app);

    let mut system_state: SystemState<EventReader<SaveProjectCompleteEvent>> =
        SystemState::new(app.world_mut());
    let mut events = system_state.get_mut(app.world_mut());
    let event = events.read().next();

    assert!(
        event.is_some(),
        "A completed event should have been dispatched"
    );
    assert_eq!(
        event.unwrap().project,
        project,
        "The completed event should have been for the project"
    );
    assert_eq!(
        event.unwrap().output,
        output,
        "The completed event should have been for the  output"
    );
    // TODO: this fails to query the event.
    // assert_eq!(
    //     app.world_mut().resource::<Events<SaveProjectCompleteEvent>>().len(),
    //     1,
    //     "SaveProjectEvent should have been emitted"
    // );

    let json = read_to_string(output.clone())
        .with_context(|| format!("Output file {} could not be opened", output.display()))?;

    assert!(json.starts_with('{'));
    Ok(())
}
