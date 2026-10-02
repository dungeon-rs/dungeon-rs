//! Bundled Files through the Utility's public surface: fixture layouts in a temporary
//! directory, with and without the marker, each played through the executable path the Utility
//! is given.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

use drs_diagnostics::{BUNDLE_MARKER, BundledFileError, locate_bundled_files};
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

/// A bundle directory at `path`, marked with `version`.
fn marked(path: &Path, version: &str) -> PathBuf {
    fs::create_dir_all(path).expect("the bundle directory");
    fs::write(path.join(BUNDLE_MARKER), format!("{version}\n")).expect("the marker");
    path.to_path_buf()
}

/// The bundle directory is `bundle` beside the executable.
#[test]
fn the_bundle_beside_the_executable_is_found() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("editor").join("dungeon-rs"));
    let bundle = marked(&root.path().join("editor").join("bundle"), VERSION);

    let found = locate_bundled_files(&editor, VERSION).expect("the Bundled Files");

    assert_eq!(found.root(), bundle);
    assert_eq!(found.version(), Some(VERSION));
    assert!(!found.version_differs());
}

/// The bundle directory is `Resources` beside the executable's directory: the macOS
/// application layout.
#[test]
fn the_bundle_inside_the_macos_application_is_found() {
    let root = TempDir::new().expect("temporary root");
    let contents = root.path().join("DungeonRS.app").join("Contents");
    let editor = executable(&contents.join("MacOS").join("dungeon-rs"));
    let bundle = marked(&contents.join("Resources"), VERSION);

    let found = locate_bundled_files(&editor, VERSION).expect("the Bundled Files");

    assert_eq!(found.root(), bundle);
}

/// The bundle directory is `bundle` in an ancestor of the executable's directory, nearest
/// first: the workspace layout during development.
#[test]
fn the_bundle_in_a_workspace_ancestor_is_found() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("target").join("debug").join("dungeon-rs"));
    let bundle = marked(&root.path().join("bundle"), VERSION);

    let found = locate_bundled_files(&editor, VERSION).expect("the Bundled Files");

    assert_eq!(found.root(), bundle);
}

/// A directory counts as the bundle directory only when it holds the marker file; one without
/// it is passed over for a marked one further on.
#[test]
fn a_directory_without_the_marker_does_not_count() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("target").join("debug").join("dungeon-rs"));
    fs::create_dir_all(root.path().join("target").join("debug").join("bundle"))
        .expect("the unmarked directory");
    fs::create_dir_all(root.path().join("target").join("bundle")).expect("another");
    let bundle = marked(&root.path().join("bundle"), VERSION);

    let found = locate_bundled_files(&editor, VERSION).expect("the Bundled Files");

    assert_eq!(found.root(), bundle);
}

/// A marker naming a version other than the editor's is still used, and says so.
#[test]
fn a_marker_naming_another_version_is_still_used() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("editor").join("dungeon-rs"));
    let bundle = marked(&root.path().join("editor").join("bundle"), "0.0.0");

    let found = locate_bundled_files(&editor, VERSION).expect("the Bundled Files");

    assert_eq!(found.root(), bundle);
    assert_eq!(found.version(), Some("0.0.0"));
    assert!(found.version_differs());
}

/// The bundle directory found does not depend on the directory the editor was started from.
///
/// The working directory is process-wide; the test runner gives each test its own process, so
/// the change reaches no other test.
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "the one place the working directory is changed on purpose, to show it does not matter"
)]
fn the_working_directory_never_matters() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("editor").join("dungeon-rs"));
    let bundle = marked(&root.path().join("editor").join("bundle"), VERSION);
    let elsewhere = TempDir::new().expect("another directory");
    marked(&elsewhere.path().join("bundle"), VERSION);
    std::env::set_current_dir(elsewhere.path()).expect("the working directory");

    let found = locate_bundled_files(&editor, VERSION).expect("the Bundled Files");

    assert_eq!(found.root(), bundle);
}

/// When no location is marked, the error names every location tried.
#[test]
fn no_marked_directory_names_every_location_tried() {
    let root = TempDir::new().expect("temporary root");
    let debug = root.path().join("target").join("debug");
    let editor = executable(&debug.join("dungeon-rs"));
    fs::create_dir_all(debug.join("bundle")).expect("an unmarked directory");

    let missing = locate_bundled_files(&editor, VERSION).expect_err("no bundle");

    assert_eq!(missing.tried[0], debug.join("bundle"));
    assert_eq!(
        missing.tried[1],
        root.path().join("target").join("Resources")
    );
    assert_eq!(missing.tried[2], root.path().join("target").join("bundle"));
    assert_eq!(missing.tried[3], root.path().join("bundle"));
    assert!(missing.tried.contains(&PathBuf::from("/bundle")) || cfg!(windows));
    let said = missing.to_string();
    for tried in &missing.tried {
        assert!(said.contains(&tried.display().to_string()), "{said}");
    }
}

/// A Bundled File asked for by name that is absent is reported with its name.
#[test]
fn a_missing_bundled_file_is_reported_by_name() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("editor").join("dungeon-rs"));
    let bundle = marked(&root.path().join("editor").join("bundle"), VERSION);
    fs::create_dir_all(bundle.join("fonts")).expect("a subdirectory");
    fs::write(bundle.join("fonts").join("label.ttf"), "font").expect("a Bundled File");
    let found = locate_bundled_files(&editor, VERSION).expect("the Bundled Files");

    assert_eq!(
        found.file("fonts/label.ttf"),
        Ok(bundle.join("fonts").join("label.ttf"))
    );
    assert_eq!(
        found.file("fonts/missing.ttf"),
        Err(BundledFileError::Missing {
            name: "fonts/missing.ttf".to_owned()
        })
    );
    assert!(matches!(
        found.file("fonts"),
        Err(BundledFileError::Unreadable { name, .. }) if name == "fonts"
    ));
}

/// A name that is absolute or holds `..` is refused, so no lookup leaves the bundle directory.
#[test]
fn names_that_leave_the_directory_are_refused() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("editor").join("dungeon-rs"));
    marked(&root.path().join("editor").join("bundle"), VERSION);
    let found = locate_bundled_files(&editor, VERSION).expect("the Bundled Files");

    for name in [
        "/etc/hosts",
        "../dungeon-rs",
        "fonts/../../secret",
        "",
        "C:\\Windows",
    ] {
        assert_eq!(
            found.file(name),
            Err(BundledFileError::NotPlainRelative {
                name: name.to_owned()
            }),
            "{name}"
        );
    }
}

/// Bundled Files load only from the bundle directory: a `lib://` name, the Author's Assets, is
/// refused as not a plain relative path.
#[test]
fn a_source_prefixed_name_is_refused() {
    let root = TempDir::new().expect("temporary root");
    let editor = executable(&root.path().join("editor").join("dungeon-rs"));
    marked(&root.path().join("editor").join("bundle"), VERSION);
    let found = locate_bundled_files(&editor, VERSION).expect("the Bundled Files");

    assert_eq!(
        found.file("lib://maps/table.png"),
        Err(BundledFileError::NotPlainRelative {
            name: "lib://maps/table.png".to_owned()
        })
    );
}
