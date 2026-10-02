//! Where the editor keeps its own files, resolved from the overrides the model holds.
#![expect(
    clippy::missing_panics_doc,
    reason = "a test stops at the first thing that is not as expected"
)]

use drs_model::EditorDirectories;
use std::path::Path;

/// Every directory, the logs included, is under the root that overrides the editor's
/// directories, so a test or a development run leaves nothing elsewhere.
#[test]
fn every_directory_is_under_the_root_that_overrides() {
    let root = Path::new("/editor-root");

    let resolved = EditorDirectories::under(root)
        .resolve()
        .expect("overridden directories need no platform");

    assert_eq!(resolved.configuration, root.join("configuration"));
    assert_eq!(resolved.cache, root.join("cache"));
    assert_eq!(resolved.logs, root.join("logs"));
}

/// The log directory is `logs` under the cache directory unless named.
#[test]
fn the_log_directory_is_logs_under_the_cache_directory_by_default() {
    let root = Path::new("/editor-root");
    let overrides = EditorDirectories {
        configuration: Some(root.join("configuration")),
        cache: Some(root.join("cache")),
        logs: None,
    };

    let resolved = overrides
        .resolve()
        .expect("overridden directories need no platform");

    assert_eq!(resolved.logs, root.join("cache").join("logs"));
}

/// Where nothing overrides, the platform's directories apply, with the logs under the cache
/// directory; a platform that names no directories is a report, not a crash.
#[test]
fn the_platform_directories_apply_where_nothing_overrides() {
    if let Ok(resolved) = EditorDirectories::default().resolve() {
        assert_eq!(resolved.logs, resolved.cache.join("logs"));
        assert!(resolved.configuration.is_absolute());
        assert!(resolved.cache.is_absolute());
    }
}
