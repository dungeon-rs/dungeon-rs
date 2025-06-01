use anyhow::Context;
use serialization::{Deserialize, SerializationFormat, Serialize, serialize_to};
use std::collections::HashMap;
use std::env::current_exe;
use std::fs::File;
use std::path::PathBuf;

/// Configuration for the `DungeonRS` application.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Configuration {
    /// A list of recently opened files,
    /// used in the UI to show recently opened projects.
    pub recents: Vec<PathBuf>,
    /// A map of asset libraries currently known to the application.
    /// This is used so that assets can be referred to relative to a library, which in turn is referred
    /// to by name.
    /// The result is that if the same library is located in different paths (as is often the case
    /// across multiple devices and/or users), asset references will still work consistently.
    pub libraries: HashMap<String, PathBuf>,
}

const CONFIG_FILE_NAME: &str = "config.toml";

impl Configuration {
    /// Attempts to save the configuration.
    ///
    /// # Errors
    /// - [`std::io::Error`] returned when the underlying calls to either [`std::env::current_exe`]
    ///   or [`std::io::File::create`] fails.
    /// - [`serialization::SerializationError`] Thrown when a serialisation-related error occurs.
    pub fn save(&self) -> anyhow::Result<()> {
        let mut path = current_exe().with_context(|| "Failed to get current executable path")?;
        path.pop(); // Remove the executable name
        path.push(CONFIG_FILE_NAME); // and add the config file name
        let file = File::create(path).with_context(|| "Failed to create config file")?;

        serialize_to(&self, &SerializationFormat::Toml, file)?;
        Ok(())
    }
}
