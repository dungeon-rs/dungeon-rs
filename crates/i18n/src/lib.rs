#![doc = include_str!("../README.md")]

mod macros;
mod plugin;

use anyhow::Context;
pub use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{ArcLoader, LanguageIdentifier, Loader};
pub use plugin::I18nPlugin;
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{LazyLock, RwLock};

/// The fallback language to use by default.
const FALLBACK: LanguageIdentifier = unic_langid::langid!("en-GB");

/// The globally available localisation instance.
///
/// See [`Locale`].
pub static LOCALE: LazyLock<Locale> = LazyLock::new(|| {
    let locales_path = utils::resource_path()
        .with_context(|| "Failed to resolve resources path")
        .unwrap()
        .join("locales");

    let loader = ArcLoader::builder(locales_path.as_path(), FALLBACK)
        .build()
        .unwrap_or_else(|_| {
            panic!(
                "Failed to build locale loader for {}",
                locales_path.display()
            )
        });

    Locale {
        language: RwLock::new(FALLBACK),
        loader,
    }
});

/// Thin wrapper around Fluent's API that automatically sets the correct language.
pub struct Locale {
    /// The language we're currently translating with.
    language: RwLock<LanguageIdentifier>,
    /// The translation loader.
    loader: ArcLoader,
}

impl Locale {
    /// Attempts to set the language to the given string, falls back to the system language if no
    /// language is passed.
    ///
    /// # Errors
    /// Return an error if it's unable to fetch the system's locale, unable to parse it or unable to set it.
    ///
    /// See also error conditions for [`Locale::set_language`].
    pub fn initialise_language(
        &self,
        language: &Option<String>,
    ) -> Result<LanguageIdentifier, String> {
        if let Some(language) = language {
            let language =
                LanguageIdentifier::from_str(language).map_err(|_| "Failed to parse locale")?;
            self.set_language(language)?;
        }

        let Some(locale) = sys_locale::get_locale() else {
            return Err(String::from("Failed to get system locale"));
        };

        let locale = LanguageIdentifier::from_str(&locale).map_err(|_| "Failed to parse locale")?;
        self.set_language(locale.clone())?;
        Ok(locale)
    }

    /// Sets the given `LanguageIdentifier` as the currently used translation key.
    ///
    /// # Errors
    /// This method may fail if it's unable to acquire a lock on the current language.
    pub fn set_language(&self, language: LanguageIdentifier) -> Result<(), String> {
        let mut current_language = self
            .language
            .write()
            .map_err(|_| "Failed to acquire lock")?;

        *current_language = language;
        Ok(())
    }

    /// Translate a given translation ID without additional parameters.
    #[must_use]
    pub fn translate(&self, id: &str) -> String {
        self.language.read().map_or_else(
            |_| String::from(id),
            |locale| self.loader.lookup(&locale, id),
        )
    }

    /// Translates a given translation ID and passes in additional arguments.
    #[must_use]
    pub fn translate_with_arguments(
        &self,
        id: &str,
        args: &HashMap<Cow<'static, str>, FluentValue>,
    ) -> String {
        self.language.read().map_or_else(
            |_| String::from(id),
            |locale| self.loader.lookup_with_args(&locale, id, args),
        )
    }
}
