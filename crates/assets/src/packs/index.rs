//! The actual indexation logic of the [`AssetPack`] is split out in this module to keep it separate
//! from the rest of the resolution logic.

use std::path::{Path, PathBuf};
use tantivy::collector::TopDocs;
use tantivy::query::TermQuery;
use tantivy::schema::{Field, IndexRecordOption, STORED, STRING, Schema, TEXT, Value};
use tantivy::{Index, TantivyDocument, Term};
use thiserror::Error;
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
    CreateIndex(PathBuf, #[source] tantivy::TantivyError),

    /// An error occurred while opening the underlying index.
    #[error("Failed to open index at {0}")]
    OpenIndex(PathBuf, #[source] tantivy::TantivyError),

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

    pub fn index(&self, _root: &Path) -> Result<(), AssetPackIndexError> {
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
}
