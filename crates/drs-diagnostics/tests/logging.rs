//! Logging through the Utility's public surface over a temporary directory: the layer it yields
//! is composed into a subscriber scoped to the test, entries are emitted, and the file is read
//! back.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

use drs_diagnostics::{
    BUNDLE_MARKER, BundledFilesNotFound, DEFAULT_LEVEL, KEPT_LOG_FILES, LogDirectives,
    LoggingError, locate_bundled_files, log_directives, log_directory, log_start, start_logging,
    take_layer,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tracing_subscriber::layer::SubscriberExt as _;

/// Starts logging into `directory` with `level` and installs the layer in a subscriber scoped to
/// the test; the guard keeps it installed.
fn subscriber(
    directory: &Path,
    level: LogDirectives,
) -> (PathBuf, tracing::subscriber::DefaultGuard) {
    let logging = start_logging(directory, level);
    let file = logging.file.expect("a log file in a writable directory");
    let layer = take_layer().expect("the layer logging yields");
    let subscriber = tracing_subscriber::registry::Registry::default().with(layer);
    (file, tracing::subscriber::set_default(subscriber))
}

/// Today's UTC date as the file name carries it. A test that names the day takes it before
/// starting and gives up when it has changed since, rather than fail at midnight.
fn today() -> String {
    let format = time::macros::format_description!("[year]-[month]-[day]");
    time::OffsetDateTime::now_utc()
        .date()
        .format(format)
        .expect("a formatted date")
}

/// The daily log files in `directory`, by name.
fn log_files(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(directory)
        .expect("the log directory")
        .map(|entry| {
            entry
                .expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| {
            name.starts_with("dungeon-rs.")
                && Path::new(name)
                    .extension()
                    .is_some_and(|extension| extension == "log")
        })
        .collect();
    names.sort();
    names
}

/// Every entry the editor logs at or above the configured level is appended to the current log
/// file, which is named with the UTC date of the day and carries each entry's UTC timestamp,
/// level, and module.
#[test]
fn entries_are_logged_to_the_dated_file() {
    let root = TempDir::new().expect("temporary root");
    let logs = root.path().join("logs");
    let day = today();
    let (file, guard) = subscriber(&logs, log_directives(None));

    tracing::info!(target: "drs_test::module", "the first entry");
    tracing::warn!(target: "drs_test::module", "the second entry");
    drop(guard);

    if today() != day {
        return;
    }
    assert_eq!(file, logs.join(format!("dungeon-rs.{day}.log")));
    let text = fs::read_to_string(&file).expect("the log file");
    let first = text.lines().next().expect("a first line");
    assert!(first.contains("the first entry"), "{first}");
    assert!(first.contains(" INFO "), "{first}");
    assert!(first.contains("drs_test::module"), "{first}");
    assert!(
        first.contains(&day) && first.contains('T') && first.contains('Z'),
        "{first}"
    );
    assert!(
        text.lines()
            .nth(1)
            .is_some_and(|line| line.contains("WARN")),
        "{text}"
    );
}

/// The log directory is created at start when it does not exist, parents included.
#[test]
fn the_log_directory_is_made() {
    let root = TempDir::new().expect("temporary root");
    let logs = root.path().join("cache").join("logs");

    let logging = start_logging(&logs, log_directives(None));

    assert!(logs.is_dir());
    assert!(logging.file.is_some());
    assert!(logging.failure.is_none());
    assert_eq!(log_files(&logs).len(), 1);
}

/// At most seven daily log files exist after start: the oldest by the date in their names are
/// deleted beyond seven, whatever order they were written in, and crash reports are never
/// deleted.
#[test]
fn a_week_of_files_is_kept() {
    let root = TempDir::new().expect("temporary root");
    let logs = root.path().join("logs");
    fs::create_dir_all(&logs).expect("the log directory");
    // Written newest first, so that the file system's creation order says nothing useful.
    for day in (1..=10).rev() {
        fs::write(
            logs.join(format!("dungeon-rs.2026-09-{day:02}.log")),
            "old\n",
        )
        .expect("an old log file");
    }
    let report = logs.join("crash-2026-09-05T10-00-00Z.txt");
    fs::write(&report, "a report\n").expect("a crash report");
    let day = today();

    let logging = start_logging(&logs, log_directives(None));

    if today() != day {
        return;
    }
    let kept = log_files(&logs);
    assert_eq!(kept.len(), KEPT_LOG_FILES, "{kept:?}");
    assert!(kept.contains(&format!("dungeon-rs.{day}.log")), "{kept:?}");
    for day in 1..=4 {
        assert!(
            !kept.contains(&format!("dungeon-rs.2026-09-{day:02}.log")),
            "{kept:?}"
        );
    }
    assert!(kept.contains(&"dungeon-rs.2026-09-10.log".to_owned()));
    assert!(report.is_file());
    assert!(logging.file.is_some());
}

/// A start on a day that already has its file keeps that file and what it holds, and still
/// leaves at most seven: the oldest by date go, never today's, whatever the file system
/// remembers about creation.
#[test]
fn a_start_on_a_day_with_a_file_keeps_that_file() {
    let root = TempDir::new().expect("temporary root");
    let logs = root.path().join("logs");
    fs::create_dir_all(&logs).expect("the log directory");
    let day = today();
    let todays = logs.join(format!("dungeon-rs.{day}.log"));
    fs::write(&todays, "earlier today\n").expect("today's file");
    for day in 1..=10 {
        fs::write(
            logs.join(format!("dungeon-rs.2026-09-{day:02}.log")),
            "old\n",
        )
        .expect("an old log file");
    }

    let (_, guard) = subscriber(&logs, log_directives(None));
    tracing::info!(target: "drs_test", "later today");
    drop(guard);

    if today() != day {
        return;
    }
    let kept = log_files(&logs);
    assert!(kept.len() <= KEPT_LOG_FILES, "{kept:?}");
    assert!(
        kept.contains(&"dungeon-rs.2026-09-10.log".to_owned()),
        "{kept:?}"
    );
    let text = fs::read_to_string(&todays).expect("today's file");
    assert!(text.starts_with("earlier today"), "{text}");
    assert!(text.contains("later today"), "{text}");
}

/// An entry is in the file once the call that logged it returns; nothing is held back in a
/// buffer.
#[test]
fn an_entry_is_written_as_it_happens() {
    let root = TempDir::new().expect("temporary root");
    let (file, _guard) = subscriber(&root.path().join("logs"), log_directives(None));

    tracing::info!(target: "drs_test", "written at once");

    let text = fs::read_to_string(&file).expect("the log file while the subscriber lives");
    assert!(text.contains("written at once"), "{text}");
}

/// When the log directory cannot be created, the editor starts and logs to the terminal only,
/// with the reason.
#[test]
fn a_log_directory_that_cannot_be_made_leaves_the_terminal_only() {
    let root = TempDir::new().expect("temporary root");
    let logs = root.path().join("logs");
    fs::write(&logs, "a file where the directory should be").expect("the file in the way");

    let logging = start_logging(&logs, log_directives(None));

    assert_eq!(logging.directory, logs);
    assert!(logging.file.is_none());
    assert!(
        matches!(&logging.failure, Some(LoggingError::Directory { path, .. }) if *path == logs),
        "{:?}",
        logging.failure
    );
    assert!(take_layer().is_none());
}

/// When `RUST_LOG` is set and well-formed, its directives are laid over the default ones, the
/// more specific winning, as the terminal reads it: a default the variable does not name stays,
/// and one it names is replaced.
#[test]
fn the_level_comes_from_rust_log() {
    let root = TempDir::new().expect("temporary root");
    let level = log_directives(Some("debug,drs_quiet=error,naga=info"));
    assert!(level.fallback.is_none());
    let (file, _guard) = subscriber(&root.path().join("logs"), level);

    tracing::debug!(target: "drs_test", "a debug entry");
    tracing::warn!(target: "drs_quiet", "a quiet warning");
    tracing::error!(target: "drs_quiet", "a quiet error");
    tracing::warn!(target: "wgpu::device", "a wgpu warning");
    tracing::info!(target: "naga::front", "a naga info entry");

    let text = fs::read_to_string(&file).expect("the log file");
    assert!(text.contains("a debug entry"), "{text}");
    assert!(!text.contains("a quiet warning"), "{text}");
    assert!(text.contains("a quiet error"), "{text}");
    assert!(!text.contains("a wgpu warning"), "{text}");
    assert!(text.contains("a naga info entry"), "{text}");
}

/// Without `RUST_LOG`, entries at `info` and above are logged, except that `wgpu` logs at
/// `error` and above and `naga` at `warn` and above.
#[test]
fn the_default_level_applies_without_rust_log() {
    let root = TempDir::new().expect("temporary root");
    let (file, _guard) = subscriber(&root.path().join("logs"), log_directives(None));

    tracing::debug!(target: "drs_test", "an editor debug entry");
    tracing::info!(target: "drs_test", "an editor info entry");
    tracing::warn!(target: "wgpu::device", "a wgpu warning");
    tracing::error!(target: "wgpu::device", "a wgpu error");
    tracing::info!(target: "naga::front", "a naga info entry");
    tracing::warn!(target: "naga::front", "a naga warning");

    let text = fs::read_to_string(&file).expect("the log file");
    assert!(!text.contains("an editor debug entry"), "{text}");
    assert!(text.contains("an editor info entry"), "{text}");
    assert!(!text.contains("a wgpu warning"), "{text}");
    assert!(text.contains("a wgpu error"), "{text}");
    assert!(!text.contains("a naga info entry"), "{text}");
    assert!(text.contains("a naga warning"), "{text}");
}

/// A malformed `RUST_LOG` leaves the default in place and is said on the terminal.
#[test]
fn a_malformed_rust_log_falls_back_to_the_default() {
    let root = TempDir::new().expect("temporary root");
    let level = log_directives(Some("drs_test=loud"));
    assert!(
        level
            .fallback
            .as_deref()
            .is_some_and(|note| note.contains("RUST_LOG") && note.contains(DEFAULT_LEVEL)),
        "{:?}",
        level.fallback
    );
    let (file, _guard) = subscriber(&root.path().join("logs"), level);

    tracing::debug!(target: "drs_test", "a debug entry");
    tracing::info!(target: "drs_test", "an info entry");

    let text = fs::read_to_string(&file).expect("the log file");
    assert!(!text.contains("a debug entry"), "{text}");
    assert!(text.contains("an info entry"), "{text}");
}

/// The log directory is the one the editor's directories resolved to; when they resolved to
/// none, it is under the platform's temporary directory, never the working directory.
#[test]
fn the_log_directory_is_the_one_resolved_or_under_the_temporary_directory() {
    let resolved = PathBuf::from("/editor/cache/logs");

    assert_eq!(log_directory(Some(resolved.clone())), resolved);
    let fallback = log_directory(None);
    assert!(
        fallback.starts_with(std::env::temp_dir()),
        "{}",
        fallback.display()
    );
    assert!(fallback.ends_with("logs"), "{}", fallback.display());
}

/// The first entry logged at start names the current log file's path.
#[test]
fn the_first_entry_names_the_log_file() {
    let root = TempDir::new().expect("temporary root");
    let logs = root.path().join("logs");
    let logging = start_logging(&logs, log_directives(None));
    let file = logging.file.clone().expect("a log file");
    let layer = take_layer().expect("the layer logging yields");
    let _guard = tracing::subscriber::set_default(
        tracing_subscriber::registry::Registry::default().with(layer),
    );

    let started = log_start(&logging, Err(BundledFilesNotFound { tried: Vec::new() }));

    let text = fs::read_to_string(&file).expect("the log file");
    let first = text.lines().next().expect("a first line");
    assert!(first.contains(&file.display().to_string()), "{first}");
    assert_eq!(started.log_file.as_deref(), Some(file.as_path()));
    assert_eq!(started.logs, logs);
    // With the executable's location unknown, nothing was tried, and the asset root is a
    // directory under the temporary directory that nothing creates.
    let asset_root = started.asset_root();
    assert!(
        asset_root.starts_with(std::env::temp_dir()),
        "{}",
        asset_root.display()
    );
    assert!(!asset_root.exists(), "{}", asset_root.display());
}

/// A bundle directory whose marker names a version other than the editor's is used, and the
/// start logs a warning naming the directory and the version.
#[test]
fn a_marker_naming_another_version_is_logged_as_a_warning() {
    let root = TempDir::new().expect("temporary root");
    let editor = root.path().join("editor").join("dungeon-rs");
    let bundle = root.path().join("editor").join("bundle");
    fs::create_dir_all(&bundle).expect("the bundle directory");
    fs::write(&editor, "").expect("the executable");
    fs::write(bundle.join(BUNDLE_MARKER), "0.0.0\n").expect("the marker");
    let found = locate_bundled_files(&editor, "0.0.1").expect("the Bundled Files");
    let logging = start_logging(&root.path().join("logs"), log_directives(None));
    let file = logging.file.clone().expect("a log file");
    let _guard = tracing::subscriber::set_default(
        tracing_subscriber::registry::Registry::default()
            .with(take_layer().expect("the layer logging yields")),
    );

    let started = log_start(&logging, Ok(found));

    assert_eq!(started.asset_root(), bundle);
    let text = fs::read_to_string(&file).expect("the log file");
    let warning = text
        .lines()
        .find(|line| line.contains(" WARN "))
        .expect("a warning");
    assert!(warning.contains(&bundle.display().to_string()), "{warning}");
    assert!(warning.contains("0.0.0"), "{warning}");
}

/// When no location is marked, the start logs an error naming every location tried, and the
/// asset root is the first of them, so nothing else is read.
#[test]
fn no_bundle_directory_is_logged_as_an_error_naming_every_location_tried() {
    let root = TempDir::new().expect("temporary root");
    let tried = vec![
        root.path().join("editor").join("bundle"),
        root.path().join("Resources"),
    ];
    let logging = start_logging(&root.path().join("logs"), log_directives(None));
    let file = logging.file.clone().expect("a log file");
    let _guard = tracing::subscriber::set_default(
        tracing_subscriber::registry::Registry::default()
            .with(take_layer().expect("the layer logging yields")),
    );

    let started = log_start(
        &logging,
        Err(BundledFilesNotFound {
            tried: tried.clone(),
        }),
    );

    assert_eq!(started.asset_root(), tried[0]);
    let text = fs::read_to_string(&file).expect("the log file");
    let error = text
        .lines()
        .find(|line| line.contains(" ERROR "))
        .expect("an error");
    for location in &tried {
        assert!(error.contains(&location.display().to_string()), "{error}");
    }
}
