//! Crashes through the Utility's public surface: the hook, with dialogs off, is installed once
//! in this test process; each test panics on a spawned thread, joins it, and reads the report
//! back. The tests take turns, since the hook and the pending reports are process-wide.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

use drs_diagnostics::{
    CrashHandler, install_crash_handler, level_filter, start_logging, take_layer,
    take_pending_report,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};
use tempfile::TempDir;
use tracing_subscriber::layer::SubscriberExt as _;

/// The version the handler is installed with.
const VERSION: &str = "7.8.9-test";

/// The process-wide fixture: the temporary root, the log directory, and the log file.
struct Fixture {
    /// Keeps the temporary directory alive for the whole process.
    _root: TempDir,
    /// The log directory, where reports land.
    logs: PathBuf,
    /// The log file entries go to.
    log_file: PathBuf,
}

/// The fixture, made once.
static FIXTURE: OnceLock<Fixture> = OnceLock::new();

/// Lets one test at a time use the process-wide hook.
static TURN: Mutex<()> = Mutex::new(());

/// Installs the handler and the logging once, from a thread that is not any test's, so every
/// test's panic is on a thread other than the one the handler takes for the main thread.
fn setup() -> (&'static Fixture, MutexGuard<'static, ()>) {
    let turn = TURN.lock().unwrap_or_else(PoisonError::into_inner);
    let fixture = FIXTURE.get_or_init(|| {
        let root = TempDir::new().expect("temporary root");
        let logs = root.path().join("logs");
        let logging = start_logging(&logs, level_filter(None));
        let log_file = logging.file.clone().expect("a log file");
        let layer = take_layer().expect("the layer logging yields");
        tracing::subscriber::set_global_default(
            tracing_subscriber::registry::Registry::default().with(layer),
        )
        .expect("the first subscriber of the process");
        let handler = CrashHandler {
            log_directory: logs.clone(),
            version: VERSION.to_owned(),
            dialogs: false,
        };
        std::thread::spawn(move || install_crash_handler(handler))
            .join()
            .expect("the installing thread");
        Fixture {
            _root: root,
            logs,
            log_file,
        }
    });
    (fixture, turn)
}

/// Panics with `message` on a thread named `worker` and waits for it to end.
#[expect(clippy::panic, reason = "the crash under test")]
fn crash_on_a_thread(message: String) {
    let worker = std::thread::Builder::new()
        .name("worker".to_owned())
        .spawn(move || {
            let payload = message;
            std::panic::panic_any(payload);
        })
        .expect("the worker thread");
    assert!(worker.join().is_err(), "the worker should have panicked");
}

/// The crash report in `logs` that holds `message`.
fn report_holding(logs: &Path, message: &str) -> (PathBuf, String) {
    let mut found = fs::read_dir(logs)
        .expect("the log directory")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("crash-"))
        })
        .filter_map(|path| {
            let text = fs::read_to_string(&path).ok()?;
            text.contains(message).then_some((path, text))
        });
    let report = found.next().expect("a report holding the message");
    assert!(found.next().is_none(), "one report per crash");
    report
}

/// A panic on any thread writes a crash report named `crash-<UTC timestamp>.txt` in the log
/// directory.
#[test]
fn a_crash_on_any_thread_leaves_a_report() {
    let (fixture, _turn) = setup();
    let message = "the editor fell over while placing a Prop".to_owned();

    crash_on_a_thread(message.clone());

    let (path, _) = report_holding(&fixture.logs, &message);
    let name = path.file_name().expect("a name").to_string_lossy();
    assert!(
        name.starts_with("crash-") && name.ends_with("Z.txt"),
        "{name}"
    );
    assert!(name.contains('T') && !name.contains(':'), "{name}");
}

/// The report holds the UTC timestamp, the editor's version, the operating system and
/// architecture, the panic message, the source location, the name of the thread, a backtrace,
/// and the path of the current log file, each under its own heading.
#[test]
fn the_report_holds_each_field_under_its_heading() {
    let (fixture, _turn) = setup();
    let message = "every field is in the report".to_owned();

    crash_on_a_thread(message.clone());

    let (_, text) = report_holding(&fixture.logs, &message);
    let section = |heading: &str| -> String {
        let start = text
            .find(&format!("## {heading}\n"))
            .unwrap_or_else(|| panic!("the heading {heading} in {text}"));
        let body = &text[start + heading.len() + 4..];
        body.split("\n## ")
            .next()
            .expect("a section")
            .trim()
            .to_owned()
    };
    assert!(section("When (UTC)").ends_with('Z'), "{text}");
    assert_eq!(section("Version"), VERSION);
    assert_eq!(
        section("Platform"),
        format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
    );
    assert_eq!(section("Message"), message);
    assert!(section("Location").contains("crashes.rs"), "{text}");
    assert_eq!(section("Thread"), "worker");
    assert!(!section("Backtrace").is_empty(), "{text}");
    assert_eq!(section("Log file"), fixture.log_file.display().to_string());
}

/// The report holds no environment variables, no user name, and no host name.
#[test]
fn the_report_holds_nothing_private_beyond_paths() {
    let (fixture, _turn) = setup();
    let message = "nothing private is in the report".to_owned();

    crash_on_a_thread(message.clone());

    let (_, text) = report_holding(&fixture.logs, &message);
    let path_variable = std::env::var("PATH").expect("the PATH variable of the test process");
    assert!(!text.contains(&path_variable), "{text}");
    for variable in ["PATH=", "HOME=", "USER=", "HOSTNAME=", "COMPUTERNAME="] {
        assert!(!text.contains(variable), "{text}");
    }
}

/// The panic message and the report's path are logged at `error` once the report is written.
#[test]
fn a_crash_is_logged() {
    let (fixture, _turn) = setup();
    let message = "the crash reaches the log".to_owned();

    crash_on_a_thread(message.clone());

    let (path, _) = report_holding(&fixture.logs, &message);
    let log = fs::read_to_string(&fixture.log_file).expect("the log file");
    let entry = log
        .lines()
        .find(|line| line.contains(&message))
        .expect("the crash entry");
    assert!(entry.contains("ERROR"), "{entry}");
    assert!(entry.contains(&path.display().to_string()), "{entry}");
}

/// The report is written before any dialog is attempted: with the crash on another thread, the
/// report exists and waits for the main thread, and nothing else has been done about it.
#[test]
fn the_report_comes_first() {
    let (fixture, _turn) = setup();
    let message = "the report waits for the main thread".to_owned();

    crash_on_a_thread(message.clone());

    let (path, _) = report_holding(&fixture.logs, &message);
    let pending = take_pending_report().expect("the report left pending");
    assert_eq!(pending.path, path);
    assert_eq!(pending.message, message);
    assert_eq!(
        pending.log_file.as_deref(),
        Some(fixture.log_file.as_path())
    );
    assert!(take_pending_report().is_none());
}

/// Installed without dialogs, the handler writes and logs the report and shows nothing: the
/// tests of this file run to the end without a dialog.
#[test]
fn dialogs_can_be_off() {
    let (fixture, _turn) = setup();
    let message = "no dialog is shown".to_owned();

    crash_on_a_thread(message.clone());

    let (path, _) = report_holding(&fixture.logs, &message);
    assert!(path.is_file());
}
