//! Bundled resources through the Utility's public surface: fixture layouts in a temporary
//! directory, with and without the marker, each played through the executable path the Utility
//! is given.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

use drs_diagnostics::{RESOURCES_MARKER, ResourceError, locate_resources};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// The version the editor looks with.
const VERSION: &str = "0.0.1";

/// An empty file standing for the executable at `path`, its directories made.
fn executable(path: &Path) -> PathBuf {
    fs::create_dir_all(path.parent().expect("a directory")).expect("the directories");
    fs::write(path, "").expect("the executable");
    path.to_path_buf()
}

/// A resource directory at `path`, marked with `version`.
fn marked(path: &Path, version: &str) -> PathBuf {
    fs::create_dir_all(path).expect("the resource directory");
    fs::write(path.join(RESOURCES_MARKER), format!("{version}\n")).expect("the marker");
    path.to_path_buf()
}

/// The resource directory is `resources` beside the executable.
#[test]
fn resources_beside_the_executable_are_found() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("editor").join("dungeon-rs"));
    let resources = marked(&root.path().join("editor").join("resources"), VERSION);

    let found = locate_resources(&editor, VERSION).expect("the resources");

    assert_eq!(found.root(), resources);
    assert_eq!(found.version(), Some(VERSION));
    assert!(!found.version_differs());
}

/// The resource directory is `Resources` beside the executable's directory: the macOS
/// application layout.
#[test]
fn resources_inside_the_macos_application_are_found() {
    let root = TempDir::new().expect("temporary root");
    let contents = root.path().join("DungeonRS.app").join("Contents");
    let editor = executable(&contents.join("MacOS").join("dungeon-rs"));
    let resources = marked(&contents.join("Resources"), VERSION);

    let found = locate_resources(&editor, VERSION).expect("the resources");

    assert_eq!(found.root(), resources);
}

/// The resource directory is `resources` in an ancestor of the executable's directory, nearest
/// first: the workspace layout during development.
#[test]
fn resources_in_a_workspace_ancestor_are_found() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("target").join("debug").join("dungeon-rs"));
    let resources = marked(&root.path().join("resources"), VERSION);

    let found = locate_resources(&editor, VERSION).expect("the resources");

    assert_eq!(found.root(), resources);
}

/// A directory counts as the resource directory only when it holds the marker file; one without
/// it is passed over for a marked one further on.
#[test]
fn a_directory_without_the_marker_does_not_count() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("target").join("debug").join("dungeon-rs"));
    fs::create_dir_all(root.path().join("target").join("debug").join("resources"))
        .expect("the unmarked directory");
    fs::create_dir_all(root.path().join("target").join("resources")).expect("another");
    let resources = marked(&root.path().join("resources"), VERSION);

    let found = locate_resources(&editor, VERSION).expect("the resources");

    assert_eq!(found.root(), resources);
}

/// A marker naming a version other than the editor's is still used, and says so.
#[test]
fn a_marker_naming_another_version_is_still_used() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("editor").join("dungeon-rs"));
    let resources = marked(&root.path().join("editor").join("resources"), "0.0.0");

    let found = locate_resources(&editor, VERSION).expect("the resources");

    assert_eq!(found.root(), resources);
    assert_eq!(found.version(), Some("0.0.0"));
    assert!(found.version_differs());
}

/// The resource directory found does not depend on the directory the editor was started from.
#[test]
fn the_working_directory_never_matters() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("editor").join("dungeon-rs"));
    let resources = marked(&root.path().join("editor").join("resources"), VERSION);
    let elsewhere = TempDir::new().expect("another directory");
    marked(&elsewhere.path().join("resources"), VERSION);
    std::env::set_current_dir(elsewhere.path()).expect("the working directory");

    let found = locate_resources(&editor, VERSION).expect("the resources");

    assert_eq!(found.root(), resources);
}

/// When no location is marked, the error names every location tried.
#[test]
fn no_marked_directory_names_every_location_tried() {
    let root = TempDir::new().expect("temporary root");
    let debug = root.path().join("target").join("debug");
    let editor = executable(&debug.join("dungeon-rs"));
    fs::create_dir_all(debug.join("resources")).expect("an unmarked directory");

    let missing = locate_resources(&editor, VERSION).expect_err("no resources");

    assert_eq!(missing.tried[0], debug.join("resources"));
    assert_eq!(
        missing.tried[1],
        root.path().join("target").join("Resources")
    );
    assert_eq!(
        missing.tried[2],
        root.path().join("target").join("resources")
    );
    assert_eq!(missing.tried[3], root.path().join("resources"));
    assert!(missing.tried.contains(&PathBuf::from("/resources")) || cfg!(windows));
    let said = missing.to_string();
    for tried in &missing.tried {
        assert!(said.contains(&tried.display().to_string()), "{said}");
    }
}

/// A bundled resource asked for by name that is absent is reported with its name.
#[test]
fn a_missing_resource_is_reported_by_name() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("editor").join("dungeon-rs"));
    let resources = marked(&root.path().join("editor").join("resources"), VERSION);
    fs::create_dir_all(resources.join("fonts")).expect("a subdirectory");
    fs::write(resources.join("fonts").join("label.ttf"), "font").expect("a resource");
    let found = locate_resources(&editor, VERSION).expect("the resources");

    assert_eq!(
        found.resource("fonts/label.ttf"),
        Ok(resources.join("fonts").join("label.ttf"))
    );
    assert_eq!(
        found.resource("fonts/missing.ttf"),
        Err(ResourceError::Missing {
            name: "fonts/missing.ttf".to_owned()
        })
    );
    assert!(matches!(
        found.resource("fonts"),
        Err(ResourceError::Unreadable { name, .. }) if name == "fonts"
    ));
}

/// A name that is absolute or holds `..` is refused, so no lookup leaves the resource directory.
#[test]
fn names_that_leave_the_directory_are_refused() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("editor").join("dungeon-rs"));
    marked(&root.path().join("editor").join("resources"), VERSION);
    let found = locate_resources(&editor, VERSION).expect("the resources");

    for name in [
        "/etc/hosts",
        "../dungeon-rs",
        "fonts/../../secret",
        "",
        "C:\\Windows",
    ] {
        assert_eq!(
            found.resource(name),
            Err(ResourceError::NotPlainRelative {
                name: name.to_owned()
            }),
            "{name}"
        );
    }
}

/// Bundled resources load only from the resource directory: a `lib://` name, the Author's
/// Assets, is refused as not a plain relative path.
#[test]
fn a_source_prefixed_name_is_refused() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("editor").join("dungeon-rs"));
    marked(&root.path().join("editor").join("resources"), VERSION);
    let found = locate_resources(&editor, VERSION).expect("the resources");

    assert_eq!(
        found.resource("lib://maps/table.png"),
        Err(ResourceError::NotPlainRelative {
            name: "lib://maps/table.png".to_owned()
        })
    );
}
