#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use bevy::prelude::*;
    use tempfile::tempdir;
    use data::Project;
    use io::{IOPlugin, LoadProjectEvent};

    #[test]
    fn load_project_event() -> anyhow::Result<()> {
        // Holds output files for this test, we hold the variable since it's deleted on drop.
        let temp_dir = tempdir()?;
        let mut app = App::new();
        app.add_plugins(IOPlugin);
        let mut input = PathBuf::from(temp_dir.path());
        input.push("testfile");

        let mut file = File::create_new(input.clone())?;
        file.write_all(b"{}")?;

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
        
        app.world_mut().send_event(LoadProjectEvent { input });

        // advance world to send event and once more to run systems
        app.update();
        app.update();

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
}
