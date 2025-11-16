#![cfg(test)]
#![allow(missing_docs)]
#![allow(clippy::pedantic)]

use bevy::prelude::*;
use bevy::time::TimePlugin;
use drs_core::*;
use drs_data::Project;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn load_project_message() -> anyhow::Result<()> {
    // Holds output files for this test, we hold the variable since it's deleted on drop.
    let temp_dir = tempdir()?;
    let mut app = App::new();
    app.add_plugins((TimePlugin, CorePlugin));
    let mut input = PathBuf::from(temp_dir.path());
    input.push("testfile");

    let mut file = File::create_new(input.clone())?;
    file.write_all(
        br#"{
  "name": "example project",
  "levels": [
    {
      "name": "default",
      "visible": false,
      "layers": [
        {
          "name": "default",
          "visible": false,
          "order": 0.0,
          "items": []
        }
      ]
    }
  ]
}"#,
    )?;

    // run the schedules once to process Setup and spawn
    app.update();

    assert_eq!(
        app.world_mut()
            .query::<&Project>()
            .iter(app.world())
            .count(),
        0,
        "Project should not be loaded yet"
    );

    app.world_mut().write_message(LoadProjectMessage { input });

    // advance world to send message and once more to run systems
    app.update();
    app.update();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_secs(2));
    app.world_mut().run_schedule(FixedPostUpdate);

    assert_eq!(
        app.world_mut()
            .query::<&Project>()
            .iter(app.world())
            .count(),
        1,
        "Project should have been loaded"
    );

    Ok(())
}
