//! Contains wrapper functionality for hashing various data structures.
//!
//! If we ever need a different hash algorithm, updating this module should update the hashing implementation
//! across all crates.

use std::path::Path;

/// Generates a hash of the given `PathBuf`.
pub fn hash_path(data: impl AsRef<Path>) -> String {
    blake3::hash(data.as_ref().as_os_str().as_encoded_bytes()).to_string()
}

/// Generates a hash of the given `String` or `&str`.
pub fn hash_string(data: impl AsRef<str>) -> String {
    blake3::hash(data.as_ref().as_bytes()).to_string()
}
