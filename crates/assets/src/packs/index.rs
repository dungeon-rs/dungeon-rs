//! The actual indexation logic of the [`AssetPack`] is split out in this module to keep it separate
//! from the rest of the resolution logic.

use bevy::prelude::{trace, warn};
use rhai::{AST, Array, Engine, OptimizationLevel, Scope};
use std::path::{Path, PathBuf};
use tantivy::collector::TopDocs;
use tantivy::query::TermQuery;
use tantivy::schema::{Field, IndexRecordOption, STORED, STRING, Schema, TEXT, Value};
use tantivy::{doc, Index, IndexWriter, TantivyDocument, TantivyError, Term};
use thiserror::Error;
use utils::file_name;
use walkdir::WalkDir;
use crate::scripting::IndexEntry;

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

    #[error("An error or occured indexing {0}")]
    Index(PathBuf, #[source] TantivyError),

    /// Thrown when a Rhai script fails to compile (usually syntax errors)
    #[error("An error occurred while compiling {0} script: {1}")]
    CompileScript(&'static str, String),

    /// Thrown when a Rhai script fails to execute
    #[error("An error occurred while executing {0} script: {1}")]
    RunScript(&'static str, String),
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
}

impl AssetPackIndex {
    /// Create a new index in the given `path`.
    ///
    /// # Errors
    /// Will return an [`AssetPackIndexError`] when the underlying index fails to be created.
    pub fn new(index_path: PathBuf) -> Result<Self, AssetPackIndexError> {
        let (schema, name, categories, path) = Self::schema();
        let index = Index::create_in_dir(index_path.clone(), schema)
            .map_err(|error| AssetPackIndexError::CreateIndex(index_path, error))?;

        Ok(Self {
            index,
            name,
            categories,
            path,
        })
    }

    /// Opens an existing index at the given `path`.
    ///
    /// # Errors
    /// Will return an [`AssetPackIndexError`] when the underlying index fails to be opened.
    pub fn open(index_path: PathBuf) -> Result<Self, AssetPackIndexError> {
        let (_schema, name, categories, path) = Self::schema();
        let index = Index::open_in_dir(index_path.clone())
            .map_err(|error| AssetPackIndexError::OpenIndex(index_path, error))?;

        Ok(Self {
            index,
            name,
            categories,
            path,
        })
    }

    /// TODO.
    ///
    /// # Errors
    /// TODO.
    pub fn index(
        &self,
        root: &Path,
        filter_script: Option<&String>,
        index_script: Option<&String>,
    ) -> Result<(), AssetPackIndexError> {
        let walker = WalkDir::new(root);
        let (engine, filter_script, index_script) = self.scripting(filter_script, index_script)?;
        let mut scope = Scope::new();

        let mut writer: IndexWriter = self
            .index
            .writer(100_000_000)
            .map_err(|error| AssetPackIndexError::Index(root.to_path_buf(), error))?;
        for entry in walker.sort_by_file_name().into_iter().flatten() {
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
            }

            {
                #[cfg(feature = "dev")]
                let _span = bevy::prelude::info_span!("Indexing", name = "indexing").entered();

                // Explicitly cast to an `Array` to avoid interop problems
                let components: Array = root
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

                trace!("Indexing {entry} as {result:?}", entry = entry.path().display(), result = result);
                writer.add_document(self.to_document(&result))
                    .map_err(|error| AssetPackIndexError::Index(entry.path().to_path_buf(), error))?;
            }
        }

        writer
            .commit()
            .map_err(|error| AssetPackIndexError::Index(root.to_path_buf(), error))?;
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
    fn schema() -> (Schema, Field, Field, Field) {
        let mut builder = Schema::builder();
        let name = builder.add_text_field("name", TEXT);
        let categories = builder.add_text_field("categories", STRING | STORED);
        let path = builder.add_text_field("path", TEXT | STORED);

        (builder.build(), name, categories, path)
    }

    /// Generates a `TantivyDocument` from the [`IndexEntry`].
    ///
    /// This method converts the scripts indexing to Tantivy's indexing.
    fn to_document(&self, entry: &IndexEntry) -> TantivyDocument {
        doc!()
    }

    /// Builds a Rhai scripting engine and pre-compiles the filter and indexation script.
    ///
    /// It also performs optimisation passes on the passed scripts.
    ///
    /// # Errors
    /// This method compiles either the given `filter_script` or `index_script`s, or the built-in ones.
    /// When an error occurs during compilation (syntax errors, missing variables, ...) it will propagate.
    fn scripting(
        &self,
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
