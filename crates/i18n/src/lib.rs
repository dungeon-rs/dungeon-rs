#![doc = include_str!("../README.md")]

mod macros;

use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{ArcLoader, LanguageIdentifier, Loader};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// The fallback language to use by default.
const FALLBACK: LanguageIdentifier = unic_langid::langid!("en-GB");

/// The globally available localisation instance.
///
/// See [`Locale`].
pub static LOCALE: LazyLock<RwLock<Locale>> = LazyLock::new(|| {
    let loader = ArcLoader::builder("locales", FALLBACK).build().unwrap();

    RwLock::new(Locale {
        language: FALLBACK,
        loader,
    })
});

/// Thin wrapper around Fluent's API that automatically sets the correct language.
pub struct Locale {
    /// The language we're currently translating with.
    language: LanguageIdentifier,
    /// The translation loader.
    loader: ArcLoader,
}

impl Locale {
    pub fn set_to_system(&mut self) {
        //
    }

    pub fn set_language(&mut self, language: LanguageIdentifier) {
        self.language = language;
    }

    /// Translate a given translation ID without additional parameters.
    #[must_use]
    pub fn translate(&self, id: &str) -> String {
        self.loader.lookup(&self.language, id)
    }

    /// Translates a given translation ID and passes in additional arguments.
    #[must_use]
    pub fn translate_with_arguments(
        &self,
        id: &str,
        args: &HashMap<Cow<'static, str>, FluentValue>,
    ) -> String {
        self.loader.lookup_with_args(&self.language, id, args)
    }
}
