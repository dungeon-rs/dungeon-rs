//! The [`I18nPlugin`] handles the registration of the language to use when translating.
//! This also means it forces the global `Lazy` to initialize on startup rather than midway a frame.
use bevy::app::App;
use bevy::prelude::{error, info, Plugin};

/// Plugin that automatically registers the correct translation configuration.
#[derive(Default)]
pub struct I18nPlugin {
    /// The language to initialise the translation system with.
    language: Option<String>,
}

impl I18nPlugin {
    /// Builds a new [`I18nPlugin`] with the given language as default.
    #[must_use = "This plugin must be added to the app"]
    pub fn new(language: &Option<String>) -> Self {
        Self {
            language: language.clone(),
        }
    }
}

impl Plugin for I18nPlugin {
    fn build(&self, _app: &mut App) {
        match crate::LOCALE.initialize_language(&self.language) {
            Ok(lang) => info!("Updated language to {}", lang.to_string()),
            Err(err) => error!("Failed to update locale: {err:?}, falling back to English"),
        }
    }
}