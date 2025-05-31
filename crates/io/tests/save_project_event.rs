#[cfg(test)]
mod tests {
    use anyhow::Context;
    use bevy::prelude::*;
    use data::{Layer, Level, Project};
    use io::*;
    use std::{fs::read_to_string, path::PathBuf};
    use tempfile::tempdir;

    #[test]
    fn save_project_event() -> anyhow::Result<()> {
        // Holds output files for this test, we hold the variable since it's deleted on drop.
        let temp_dir = tempdir()?;
        let mut app = App::new();
        app.add_plugins(IOPlugin);
        app.world_mut().spawn((
            Project::new("Example Project"),
            children![(
                Level::new("First Level"),
                children![(Layer::new("First Layer"), children![])]
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
        // TODO: currently tests don't seem to fail when the system internally fails (for whatever reason).

        // Validate event has been processed
        let events = app.world().resource::<Events<SaveProjectEvent>>();
        assert!(events.is_empty(), "SaveProjectEvent was not processed!");

        // TODO: validate file contents
        let json = read_to_string(output.clone())
            .with_context(|| format!("Output file {} could not be opened", output.display()))?;

        assert!(json.starts_with("{"));
        Ok(())
    }
}
