#![doc = include_str!("../README.md")]

mod resolve;
mod search;

pub use resolve::{resolve, same_name};
pub use search::{FolderSearch, Match, Matches, SearchOrder, search};

use drs_model::AssetKind;
use std::path::Path;

/// A rule that decides which files in an Asset Folder are Assets, and of what Asset Kind.
pub trait IndexingRule: Send + Sync {
    /// The Asset Kind of the file at `place` in its folder, or `None` when this rule does not
    /// recognise it.
    fn classify(&self, place: &Path) -> Option<AssetKind>;
}

/// The built-in rule: a file with a `png`, `webp`, `jpg`, or `jpeg` extension, in any letter
/// case, is an image Asset.
#[derive(Debug, Clone, Copy, Default)]
pub struct ImageExtensions;

/// The extensions, lower-case, that [`ImageExtensions`] recognises.
const IMAGE_EXTENSIONS: [&str; 4] = ["png", "webp", "jpg", "jpeg"];

impl IndexingRule for ImageExtensions {
    fn classify(&self, place: &Path) -> Option<AssetKind> {
        let extension = place.extension()?.to_str()?.to_lowercase();
        IMAGE_EXTENSIONS
            .contains(&extension.as_str())
            .then_some(AssetKind::IMAGE)
    }
}

/// The Indexing Rules in force, asked in order.
pub struct Classifier {
    /// The rules, first asked first.
    rules: Vec<Box<dyn IndexingRule>>,
}

impl Default for Classifier {
    /// A classifier with the built-in rules.
    fn default() -> Self {
        Self::built_in()
    }
}

impl Classifier {
    /// A classifier holding only the built-in rules.
    #[must_use]
    pub fn built_in() -> Self {
        Self {
            rules: vec![Box::new(ImageExtensions)],
        }
    }

    /// Classify: the Asset Kind of the file at `place` in its folder, or `None` when it is not
    /// an Asset.
    #[must_use]
    pub fn classify(&self, place: &Path) -> Option<AssetKind> {
        self.rules.iter().find_map(|rule| rule.classify(place))
    }
}
