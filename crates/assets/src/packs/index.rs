//! The actual indexation logic of the [`crate::AssetPack`] is split out in this module to keep it separate
//! from the rest of the resolution logic.

use crate::packs::thumbnails::{AssetPackThumbnailError, AssetPackThumbnails};
use crate::scripting::IndexEntry;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::{Event, info_span, trace, warn};
use rhai::{AST, Array, Engine, OptimizationLevel, Scope};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use tantivy::collector::TopDocs;
use tantivy::query::{QueryParser, QueryParserError, TermQuery};
use tantivy::schema::{Field, IndexRecordOption, STORED, STRING, Schema, TEXT, Value};
use tantivy::{Index, IndexWriter, TantivyDocument, TantivyError, Term, doc};
use thiserror::Error;
use utils::{Sender, file_name, report_progress};
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

/// All errors that can occur when searching for assets in the [`crate::AssetPack`].
#[derive(Error, Debug)]
pub enum AssetPackSearchError {
    /// An error occurred while opening / reading from the index.
    #[error("Could not perform search because of an index error: {0}")]
    OpenIndex(#[from] TantivyError),

    /// An error occurred while parsing the query given to the search method.
    #[error("The provided query is malformed and could not be parsed: {0}")]
    ParseQuery(#[from] QueryParserError),
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

/// A wrapper struct for the result of a search operation.
pub struct AssetPackSearchResult {
    /// The document from which the fields will be written.
    document: TantivyDocument,
    /// The name field from Tantivy's schema.
    name: Field,
    /// The categories field from Tantivy's schema.
    categories: Field,
    /// The path field from Tantivy's schema.
    path: Field,
    /// The thumbnail ID field from Tantivy's schema.
    thumbnail: Field,
}

/// An event emitted when indexing an asset pack progresses.
#[derive(Event)]
pub struct AssetPackIndexProgressEvent {
    /// The ID of the [`crate::AssetPack`] being indexed.
    pub id: String,
    /// The number of processed entries finished.
    pub current: usize,
    /// The total number of entries that need to be processed in this pack.
    pub total: usize,
}

/// An event emitted when indexing an asset pack is completed.
#[derive(Event)]
pub struct AssetPackIndexCompletedEvent {
    /// The ID of the [`crate::AssetPack`] being indexed.
    pub id: String,
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

    /// Indexes all assets in the specified directory using filtering and indexing scripts.
    ///
    /// This method walks through all files in `index_root`, applies the filter script to determine
    /// which files should be indexed, then processes each accepted file through the index script
    /// to extract metadata before adding it to the search index. Optionally generates thumbnails
    /// for supported file formats.
    ///
    /// The `id` parameter is used for tracing and logging purposes. Custom `filter_script` and
    /// `index_script` can be provided, otherwise the default built-in scripts will be used.
    ///
    /// # Errors
    /// Will return an [`AssetPackIndexError`] in the following situations:
    /// - Failed to create an index writer or perform index operations
    /// - Script compilation fails due to syntax errors or missing variables
    /// - Script execution fails during filtering or indexing operations
    /// - Thumbnail generation fails for supported file formats
    pub fn index(
        &self,
        id: &String,
        index_root: &Path,
        thumbnails: Option<&AssetPackThumbnails>,
        filter_script: Option<&String>,
        index_script: Option<&String>,
        sender: Sender<CommandQueue>,
    ) -> Result<(), AssetPackIndexError> {
        let walker = WalkDir::new(index_root);
        let (engine, filter_script, index_script) = Self::scripting(filter_script, index_script)?;
        let mut scope = Scope::new();

        let total_amount = WalkDir::new(index_root).into_iter().count();
        let span = info_span!(
            "indexing",
            id = id,
            path = index_root.to_path_buf().display().to_string(),
            length = total_amount as u64
        );
        let _guard = span.enter();

        let mut writer: IndexWriter = self
            .index
            .writer(100_000_000)
            .map_err(|error| AssetPackIndexError::Index(index_root.to_path_buf(), error))?;

        writer
            .delete_all_documents()
            .map_err(|error| AssetPackIndexError::Index(index_root.to_path_buf(), error))?;

        let mut current: usize = 0;
        #[allow(
            clippy::explicit_counter_loop,
            reason = "The suggested syntax reads very awkward and is just obtuse for no reason"
        )]
        for entry in walker.sort_by_file_name().into_iter().flatten() {
            current += 1;
            let _ = report_progress(
                &sender,
                AssetPackIndexProgressEvent {
                    id: id.clone(),
                    current,
                    total: total_amount,
                },
            );

            let Some(file_name) = file_name(entry.path()) else {
                warn!(
                    "Automatically skipping invalid entry: '{path:?}', this is most likely a bug.",
                    path = entry.path()
                );
                continue;
            };

            if file_name == super::MANIFEST_FILE_NAME {
                trace!("Skipping configuration file");
                continue;
            } else if !engine
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
            } else if !AssetPackThumbnails::is_supported(entry.path()) {
                warn!(
                    "Skipping {path} because it's format is not supported by thumbnail generation",
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
                    .add_document(self.to_document(result, entry.path()))
                    .map_err(|error| {
                        AssetPackIndexError::Index(entry.path().to_path_buf(), error)
                    })?;

                if let Some(thumbnails) = thumbnails {
                    trace!("Generating thumbnail for {}", entry.path().display());

                    thumbnails
                        .generate(entry.path(), thumbnail_id)
                        .map_err(|error| {
                            AssetPackIndexError::Thumbnail(entry.path().to_path_buf(), error)
                        })?;
                }
            }
        }

        writer
            .commit()
            .map_err(|error| AssetPackIndexError::Index(index_root.to_path_buf(), error))?;
        let _ = report_progress(&sender, AssetPackIndexCompletedEvent { id: id.clone() });
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

    /// Executes a query on the index to search for arbitrary entries within the asset pack.
    ///
    /// The passed `query` must be a valid Tantivy query (see [QueryParser](https://docs.rs/tantivy/0.24.2/tantivy/query/struct.QueryParser.html)).
    /// You can control the (maximum) number of entries returned for this query with `amount`.
    ///
    /// The passed `id` is used for tracing.
    ///
    /// # Errors
    /// There are 2 situations where this method may return an error.
    /// - Tantivy throws an error when opening or reading from the index itself
    /// - The passed `query` could not be parsed, see the above `QueryParser` link for more information.
    ///
    /// # Panics
    /// As described in [TopDocs::with_limit](https://docs.rs/tantivy/0.24.2/tantivy/collector/struct.TopDocs.html#method.with_limit),
    /// this method will panic if the `amount` passed is `0`.
    pub fn query(
        &self,
        id: &String,
        query: impl AsRef<str>,
        amount: usize,
    ) -> Result<Vec<AssetPackSearchResult>, AssetPackSearchError> {
        let _ = info_span!("querying", id = id).entered();

        let reader = self
            .index
            .reader()
            .map_err(AssetPackSearchError::OpenIndex)?;

        let searcher = reader.searcher();
        let parser = QueryParser::for_index(&self.index, vec![self.name]);
        let query = parser
            .parse_query(query.as_ref())
            .map_err(AssetPackSearchError::ParseQuery)?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(amount))
            .map_err(AssetPackSearchError::OpenIndex)?;

        let mut documents = Vec::with_capacity(top_docs.len());
        for (_score, address) in top_docs {
            let document: TantivyDocument = searcher.doc::<TantivyDocument>(address)?;

            documents.push(AssetPackSearchResult::new(document, self));
        }

        Ok(documents)
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
        let name = builder.add_text_field("name", TEXT | STORED);
        let categories = builder.add_text_field("categories", STRING | STORED);
        let path = builder.add_text_field("path", TEXT | STORED);
        let thumbnail = builder.add_text_field("thumbnail", STRING | STORED);

        (builder.build(), name, categories, path, thumbnail)
    }

    /// Generates a `TantivyDocument` from the [`IndexEntry`].
    ///
    /// This method converts the scripts indexing to Tantivy's indexing.
    fn to_document(&self, entry: IndexEntry, path: &Path) -> TantivyDocument {
        let path = path.display().to_string();
        let mut document = doc!(
            self.name => entry.name.as_str(),
            self.path => path,
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

impl AssetPackSearchResult {
    /// Create a new `AssetPackSearchResult`.
    fn new(document: TantivyDocument, index: &AssetPackIndex) -> Self {
        // Cloning the `Field`s is cheap as they are just `i32` wrappers.
        Self {
            document,
            name: index.name,
            categories: index.categories,
            path: index.path,
            thumbnail: index.thumbnail,
        }
    }

    /// Attempt to resolve the `name` field from the result.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.document
            .get_first(self.name)
            .and_then(|value| value.as_str())
    }

    /// Resolves the `categories` field from the result.
    ///
    /// If no values are set an empty list is returned.
    #[must_use]
    pub fn categories(&self) -> Vec<&str> {
        self.document
            .get_all(self.categories)
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>()
    }

    /// Attempt to resolve the `path` field from the result.
    #[must_use]
    pub fn path(&self) -> Option<PathBuf> {
        self.document
            .get_first(self.path)
            .and_then(|value| value.as_str())
            .map(PathBuf::from)
    }

    /// Attempt to resolve the `thumbnail` field from the result.
    #[must_use]
    pub fn thumbnail(&self) -> Option<PathBuf> {
        self.document
            .get_first(self.thumbnail)
            .and_then(|value| value.as_str())
            .map(PathBuf::from)
    }
}

impl Display for AssetPackSearchResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = self.name().unwrap_or("<no value>");
        let path = self
            .path()
            .map_or(String::from("<no value>"), |path| utils::to_string(&path));
        let categories = self.categories();
        let thumbnail = self
            .path()
            .map_or(String::from("<no value>"), |path| utils::to_string(&path));

        writeln!(f, "name: {name}")?;
        writeln!(f, "categories:")?;
        for category in categories {
            writeln!(f, "- {category}")?;
        }
        writeln!(f, "path: {path}")?;
        writeln!(f, "thumbnail: {thumbnail}")?;

        Ok(())
    }
}
