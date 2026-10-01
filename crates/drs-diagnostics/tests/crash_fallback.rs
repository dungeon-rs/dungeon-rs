//! A crash whose log directory cannot be written, through the Utility's public surface: the hook
//! is installed once in this test process over a directory nothing may write into.
#![cfg(unix)]
#![expect(
    clippy::missing_panics_doc,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

use drs_diagnostics::{CrashHandler, install_crash_handler, take_pending_report};
use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use tempfile::TempDir;

/// When the log directory cannot be written, the report is written to the platform's temporary
/// directory instead, and names where.
#[test]
fn an_unwritable_log_directory_sends_the_report_to_the_temporary_directory() {
    let root = TempDir::new().expect("temporary root");
    let logs = root.path().join("logs");
    fs::create_dir_all(&logs).expect("the log directory");
    fs::set_permissions(&logs, fs::Permissions::from_mode(0o000)).expect("the permissions");
    if fs::write(logs.join("probe"), "").is_ok() {
        // Running as a user the permissions do not bind, such as root; nothing to check.
        return;
    }
    let handler = CrashHandler {
        log_directory: logs.clone(),
        version: "0.0.0-test".to_owned(),
        dialogs: false,
    };
    std::thread::spawn(move || install_crash_handler(handler))
        .join()
        .expect("the installing thread");
    let message = "the log directory is not writable".to_owned();

    let worker = std::thread::spawn({
        let message = message.clone();
        move || std::panic::panic_any(message)
    });
    assert!(worker.join().is_err());

    let report = take_pending_report().expect("the report left pending");
    let temporary = std::env::temp_dir()
        .canonicalize()
        .expect("the temporary directory");
    let written_in = report
        .path
        .parent()
        .expect("a parent")
        .canonicalize()
        .expect("the report's directory");
    assert_eq!(written_in, temporary);
    assert!(
        fs::read_to_string(&report.path)
            .expect("the report")
            .contains(&message)
    );
    fs::remove_file(&report.path).expect("the report removed");
    fs::set_permissions(&logs, fs::Permissions::from_mode(0o755)).expect("the permissions");
}
