//! A crash handler that leaves a report and announces it.

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::panic::{AssertUnwindSafe, PanicHookInfo};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, Once, PoisonError};
use std::thread::ThreadId;

/// How the crash handler is set up.
#[derive(Debug, Clone)]
pub struct CrashHandler {
    /// Where reports are written: the log directory.
    pub log_directory: PathBuf,
    /// The editor's version, named in each report.
    pub version: String,
    /// Whether a crash is announced in a dialog; tests and headless runs say no.
    pub dialogs: bool,
}

/// A crash report that was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashReport {
    /// Where the report is.
    pub path: PathBuf,
    /// The panic's message.
    pub message: String,
    /// The log file current when the crash happened, if any.
    pub log_file: Option<PathBuf>,
}

/// The name of the editor, as the dialog names it.
const EDITOR: &str = "DungeonRS";

/// The handler installed, and the main thread's identity.
static INSTALLED: Mutex<Option<Installed>> = Mutex::new(None);

/// Installs the hook once per process.
static INSTALL: Once = Once::new();

/// Reports written on other threads that the main thread has not announced yet.
static PENDING: Mutex<VecDeque<CrashReport>> = Mutex::new(VecDeque::new());

/// What the hook knows.
#[derive(Debug, Clone)]
struct Installed {
    /// The handler's set-up.
    handler: CrashHandler,
    /// The thread the handler was installed on: the main thread.
    main_thread: ThreadId,
}

/// Installs the crash handler on the calling thread, which is taken to be the main thread.
///
/// Installed before Bevy's plugins, so that the panic hook they build chains this one. A panic
/// on any thread then writes a report named `crash-<UTC timestamp>.txt` in the log directory,
/// or in the platform's temporary directory when the log directory cannot be written; prints
/// its path to the terminal; logs it at `error`; and, when dialogs are on, shows the dialog at
/// once on the main thread, or otherwise leaves the report pending for [`take_pending_report`].
/// The hook never panics and ignores every error it meets. Installing again replaces the
/// set-up; the hook itself is installed once per process.
pub fn install_crash_handler(handler: CrashHandler) {
    *INSTALLED.lock().unwrap_or_else(PoisonError::into_inner) = Some(Installed {
        handler,
        main_thread: std::thread::current().id(),
    });
    INSTALL.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            on_panic(info);
            previous(info);
            announce_at_once();
        }));
    });
}

/// A report written on another thread that has not been announced yet, if any.
///
/// The Editor asks each frame on the main thread and announces what it gets.
#[must_use]
pub fn take_pending_report() -> Option<CrashReport> {
    PENDING
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .pop_front()
}

/// Shows the dialog that says the editor crashed and names the report and the log file, when
/// dialogs are on; shows nothing otherwise. Called on the main thread.
pub fn announce(report: &CrashReport) {
    let dialogs = INSTALLED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .is_some_and(|installed| installed.handler.dialogs);
    if !dialogs {
        return;
    }
    let log_file = report.log_file.as_ref().map_or_else(
        || "none: logging went to the terminal only".to_owned(),
        |file| file.display().to_string(),
    );
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(format!("{EDITOR} crashed"))
        .set_description(format!(
            "{EDITOR} crashed: {}\n\nThe crash report was written to:\n{}\n\nThe log file is:\n{log_file}",
            report.message,
            report.path.display(),
        ))
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

/// Runs the editor and, when it ends, announces a report that was left pending, so a crash the
/// executor carried from another thread to the main one is announced exactly once.
///
/// A panic that ends the run is resumed after the announcement, so the process still ends as a
/// panic does.
pub fn run_guarded<R>(run: impl FnOnce() -> R) -> R {
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(run));
    if let Some(report) = take_pending_report() {
        announce(&report);
    }
    match outcome {
        Ok(result) => result,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

/// Writes the report, prints its path, logs it, and leaves it for the announcement.
fn on_panic(info: &PanicHookInfo<'_>) {
    let Some(installed) = INSTALLED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
    else {
        return;
    };
    let now = time::OffsetDateTime::now_utc();
    let message = panic_message(info);
    let log_file = crate::logging::current_log_file(now);
    let text = report_text(&installed.handler, info, &message, now, log_file.as_deref());
    let Some(path) = write_report(&installed.handler.log_directory, now, &text) else {
        eprintln!("{EDITOR} crashed and the crash report could not be written anywhere");
        return;
    };
    eprintln!(
        "{EDITOR} crashed; the crash report is at {}",
        path.display()
    );
    tracing::error!(report = %path.display(), "{EDITOR} crashed: {message}");
    PENDING
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push_back(CrashReport {
            path,
            message,
            log_file,
        });
}

/// Announces the pending report at once when the panic is on the main thread; another thread
/// leaves it for the main thread.
fn announce_at_once() {
    let on_main_thread = INSTALLED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .is_some_and(|installed| installed.main_thread == std::thread::current().id());
    if !on_main_thread {
        return;
    }
    if let Some(report) = take_pending_report() {
        announce(&report);
    }
}

/// The panic's message, as the standard hook prints it.
fn panic_message(info: &PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "a panic without a message".to_owned()
    }
}

/// The report: each field under its own heading, nothing about the Author beyond paths.
fn report_text(
    handler: &CrashHandler,
    info: &PanicHookInfo<'_>,
    message: &str,
    now: time::OffsetDateTime,
    log_file: Option<&Path>,
) -> String {
    let mut text = String::new();
    // Writing to a `String` cannot fail.
    let _ = writeln!(text, "# {EDITOR} crash report");
    let _ = writeln!(text, "\n## When (UTC)\n{}", timestamp(now));
    let _ = writeln!(text, "\n## Version\n{}", handler.version);
    let _ = writeln!(
        text,
        "\n## Platform\n{} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    let _ = writeln!(text, "\n## Message\n{message}");
    let location = info.location().map_or_else(
        || "unknown".to_owned(),
        |location| {
            format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        },
    );
    let _ = writeln!(text, "\n## Location\n{location}");
    let thread = std::thread::current();
    let _ = writeln!(text, "\n## Thread\n{}", thread.name().unwrap_or("unnamed"));
    let _ = writeln!(
        text,
        "\n## Backtrace\n{}",
        std::backtrace::Backtrace::force_capture()
    );
    let log_file = log_file.map_or_else(
        || "none: logging went to the terminal only".to_owned(),
        |file| file.display().to_string(),
    );
    let _ = writeln!(text, "\n## Log file\n{log_file}");
    text
}

/// Writes the report into the log directory, or into the temporary directory when the log
/// directory cannot be written; `None` when neither could be.
fn write_report(log_directory: &Path, now: time::OffsetDateTime, text: &str) -> Option<PathBuf> {
    let name = format!("crash-{}.txt", file_timestamp(now));
    write_into(log_directory, &name, text)
        .or_else(|| write_into(&std::env::temp_dir(), &name, text))
}

/// Writes the report as `name` in `directory`, creating the directory first and never
/// overwriting an earlier report of the same name; the path written, or `None`.
fn write_into(directory: &Path, name: &str, text: &str) -> Option<PathBuf> {
    std::fs::create_dir_all(directory).ok()?;
    let mut path = directory.join(name);
    let mut attempt = 1;
    while path.exists() {
        attempt += 1;
        path = directory.join(format!("{}-{attempt}.txt", name.trim_end_matches(".txt")));
    }
    std::fs::write(&path, text).ok()?;
    Some(path)
}

/// The timestamp as the report states it.
fn timestamp(now: time::OffsetDateTime) -> String {
    let format =
        time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
    now.format(format).unwrap_or_else(|_| "unknown".to_owned())
}

/// The timestamp as a file name carries it: no colons, which Windows refuses.
fn file_timestamp(now: time::OffsetDateTime) -> String {
    let format =
        time::macros::format_description!("[year]-[month]-[day]T[hour]-[minute]-[second]Z");
    now.format(format)
        .unwrap_or_else(|_| now.unix_timestamp().to_string())
}
