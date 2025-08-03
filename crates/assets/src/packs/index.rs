//! The actual indexation logic of the [`AssetPack`] is split out in this module to keep it separate
//! from the rest of the resolution logic.

use crate::packs::thumbnails::{AssetPackThumbnailError, AssetPackThumbnails};
use crate::scripting::IndexEntry;
use bevy::prelude::{trace, warn};
use rhai::{AST, Array, Engine, OptimizationLevel, Scope};
use std::path::{Path, PathBuf};
use tantivy::collector::TopDocs;
use tantivy::query::TermQuery;
use tantivy::schema::{Field, IndexRecordOption, STORED, STRING, Schema, TEXT, Value};
use tantivy::{Index, IndexWriter, TantivyDocument, TantivyError, Term, doc};
use thiserror::Error;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use tracing_indicatif::style::ProgressStyle;
use utils::file_name;
use walkdir::WalkDir;

/// The default script for filtering when no custom script was passed into the `AssetPack`.
const DEFAULT_FILTER_SCRIPT: &str = include_str!("../../scripts/filter.rhai");

/// The default script for indexing when no custom script was passed into the `AssetPack`.
const DEFAULT_INDEX_SCRIPT: &str = include_str!("../../scripts/index.rhai");

/// All errors that can occur while creating, opening or working with the underlying index.
#[derive(Error, Debug)]
pub enum AssetPackIndexError {
    /// An error occurred while creating the underlying index.
    #[error("Failed to create index at {0}")]
    CreateIndex(PathBuf, #[source] TantivyError),

    /// An error occurred while opening the underlying index.
    #[error("Failed to open index at {0}")]
    OpenIndex(PathBuf, #[source] TantivyError),

    /// An error occurred while indexing or reading.
    #[error("An error or occurred indexing {0}")]
    Index(PathBuf, #[source] TantivyError),

    /// Thrown when a Rhai script fails to compile (usually syntax errors)
    #[error("An error occurred while compiling {0} script: {1}")]
    CompileScript(&'static str, String),

    /// Thrown when a Rhai script fails to execute
    #[error("An error occurred while executing {0} script: {1}")]
    RunScript(&'static str, String),

    /// Thrown when thumbnail generation failed.
    #[error("Failed to generate thumbnail for {0}: {1:?}")]
    Thumbnail(PathBuf, #[source] AssetPackThumbnailError),
}

/// Encapsulates all the indexing-related data structures for an `AssetPack`.
#[derive(Debug)]
pub struct AssetPackIndex {
    /// The underlying Tantivy index of the `Assetpack`.
    index: Index,
    /// The name field from Tantivy's schema.
    name: Field,
    /// The categories field from Tantivy's schema.
    categories: Field,
    /// The path field from Tantivy's schema.
    path: Field,
    /// The thumbnail ID field from Tantivy's schema.
    thumbnail: Field,
}

impl AssetPackIndex {
    /// Create a new index in the given `path`.
    ///
    /// # Errors
    /// Will return an [`AssetPackIndexError`] when the underlying index fails to be created.
    pub fn new(index_path: PathBuf) -> Result<Self, AssetPackIndexError> {
        let (schema, name, categories, path, thumbnail) = Self::schema();
        let index = Index::create_in_dir(index_path.clone(), schema)
            .map_err(|error| AssetPackIndexError::CreateIndex(index_path, error))?;

        Ok(Self {
            index,
            name,
            categories,
            path,
            thumbnail,
        })
    }

    /// Opens an existing index at the given `path`.
    ///
    /// # Errors
    /// Will return an [`AssetPackIndexError`] when the underlying index fails to be opened.
    pub fn open(index_path: PathBuf) -> Result<Self, AssetPackIndexError> {
        let (_schema, name, categories, path, thumbnail) = Self::schema();
        let index = Index::open_in_dir(index_path.clone())
            .map_err(|error| AssetPackIndexError::OpenIndex(index_path, error))?;

        Ok(Self {
            index,
            name,
            categories,
            path,
            thumbnail,
        })
    }

    /// TODO.
    ///
    /// # Errors
    /// TODO.
    pub fn index(
        &self,
        index_root: &Path,
        thumbnails: Option<&AssetPackThumbnails>,
        filter_script: Option<&String>,
        index_script: Option<&String>,
    ) -> Result<(), AssetPackIndexError> {
        let walker = WalkDir::new(index_root);
        let (engine, filter_script, index_script) = Self::scripting(filter_script, index_script)?;
        let mut scope = Scope::new();

        let span = bevy::prelude::info_span!(
            "Indexing",
            path = index_root.to_path_buf().display().to_string()
        );
        span.pb_set_style(&ProgressStyle::with_template("{wide_bar} {pos}/{len} {msg}").unwrap());
        span.pb_set_length(WalkDir::new(index_root).into_iter().count() as u64);
        let _guard = span.enter();

        let mut writer: IndexWriter = self
            .index
            .writer(100_000_000)
            .map_err(|error| AssetPackIndexError::Index(index_root.to_path_buf(), error))?;

        writer
            .delete_all_documents()
            .map_err(|error| AssetPackIndexError::Index(index_root.to_path_buf(), error))?;

        let mut current: u64 = 0;
        for entry in walker.sort_by_file_name().into_iter().flatten() {
            span.pb_set_position(current); // If we're logging to consoles, this will properly set the progressbar.
            current += 1;

            let Some(file_name) = file_name(entry.path()) else {
                warn!(
                    "Automatically skipping invalid entry: '{path:?}', this is most likely a bug.",
                    path = entry.path()
                );
                continue;
            };

            if !engine
                .call_fn::<bool>(&mut scope, &filter_script, "filter", (file_name.clone(),))
                .map_err(|error| AssetPackIndexError::RunScript("filter", error.to_string()))?
            {
                trace!(
                    "Skipping {path} because filter script returned false",
                    path = entry.path().display()
                );
                continue;
            } else if !entry.path().is_file() {
                trace!(
                    "Skipping {path} because it's not a file",
                    path = entry.path().display()
                );

                continue;
            }

            {
                // Explicitly cast to an `Array` to avoid interop problems
                let components: Array = index_root
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().to_string().into())
                    .collect::<Vec<_>>();

                let result = engine
                    .call_fn::<IndexEntry>(
                        &mut scope,
                        &index_script,
                        "index",
                        (file_name, components),
                    )
                    .map_err(|error| AssetPackIndexError::RunScript("index", error.to_string()))?;

                trace!(
                    "Indexing {entry} as {result:?}",
                    entry = entry.path().display(),
                    result = result
                );

                let thumbnail_id = result.thumbnail.as_str().to_string();
                writer
                    .add_document(self.to_document(result))
                    .map_err(|error| {
                        AssetPackIndexError::Index(entry.path().to_path_buf(), error)
                    })?;

                if let Some(thumbnails) = thumbnails {
                    trace!("Generating thumbnail for {}", entry.path().display());

                    thumbnails
                        .generate(entry.path(), thumbnail_id)
                        .map_err(|error| {
                            AssetPackIndexError::Thumbnail(entry.path().to_path_buf(), error)
                        })?
                }
            }
        }

        writer
            .commit()
            .map_err(|error| AssetPackIndexError::Index(index_root.to_path_buf(), error))?;
        Ok(())
    }

    /// Attempt to fetch the path from the index by the ID.
    ///
    /// If any error was encountered, or no documents returned this method will return `None`.
    pub fn find_by_id(&self, id: &str) -> Option<PathBuf> {
        let Ok(reader) = self.index.reader() else {
            return None;
        };
        let searcher = reader.searcher();
        let query = TermQuery::new(
            Term::from_field_text(self.name, id),
            IndexRecordOption::Basic,
        );

        let Ok(results) = searcher.search(&query, &TopDocs::with_limit(10)) else {
            return None;
        };

        if let Some((_score, address)) = results.into_iter().next() {
            let Ok(document) = searcher.doc::<TantivyDocument>(address) else {
                return None;
            };

            return document
                .get_first(self.path)
                .and_then(|value| value.as_str())
                .map(PathBuf::from);
        }

        None
    }

    /// Builds the schema and returns it alongside all fields so they can be cached.
    ///
    /// The fields returned are in the following order:
    /// - name
    /// - categories
    /// - path
    /// - thumbnail identifier
    fn schema() -> (Schema, Field, Field, Field, Field) {
        let mut builder = Schema::builder();
        let name = builder.add_text_field("name", TEXT);
        let categories = builder.add_text_field("categories", STRING | STORED);
        let path = builder.add_text_field("path", TEXT | STORED);
        let thumbnail = builder.add_text_field("thumbnail", STRING | STORED);

        (builder.build(), name, categories, path, thumbnail)
    }

    /// Generates a `TantivyDocument` from the [`IndexEntry`].
    ///
    /// This method converts the scripts indexing to Tantivy's indexing.
    fn to_document(&self, entry: IndexEntry) -> TantivyDocument {
        let mut document = doc!(
            self.name => entry.name.as_str(),
            self.thumbnail => entry.thumbnail.as_str(),
        );

        // Add all categories
        for category in entry.categories {
            match category.into_string() {
                Ok(value) => {
                    document.add_text(self.categories, value);
                }
                Err(error) => {
                    warn!(
                        "Script returned an invalid string value in categories: '{error}', it will be skipped in the index."
                    );
                }
            }
        }

        document
    }

    /// Builds a Rhai scripting engine and pre-compiles the filter and indexation script.
    ///
    /// It also performs optimisation passes on the passed scripts.
    ///
    /// # Errors
    /// This method compiles either the given `filter_script` or `index_script`s, or the built-in ones.
    /// When an error occurs during compilation (syntax errors, missing variables, ...) it will propagate.
    fn scripting(
        filter_script: Option<&String>,
        index_script: Option<&String>,
    ) -> Result<(Engine, AST, AST), AssetPackIndexError> {
        let engine = crate::scripting::build_engine();
        let scope = Scope::new();

        let filter_script = filter_script.map_or(DEFAULT_FILTER_SCRIPT, |string| string.as_str());
        let filter_script = engine
            .compile(filter_script)
            .map_err(|error| AssetPackIndexError::CompileScript("filter", error.to_string()))?;

        let index_script = index_script.map_or(DEFAULT_INDEX_SCRIPT, |string| string.as_str());
        let index_script = engine
            .compile(index_script)
            .map_err(|error| AssetPackIndexError::CompileScript("index", error.to_string()))?;

        let filter_script = engine.optimize_ast(&scope, filter_script, OptimizationLevel::Full);
        let index_script = engine.optimize_ast(&scope, index_script, OptimizationLevel::Full);

        Ok((engine, filter_script, index_script))
    }
}
