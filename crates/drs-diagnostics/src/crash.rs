//! A crash handler that leaves a report and announces it.

use crate::Product;
use std::cell::Cell;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::io::Write as _;
use std::panic::{AssertUnwindSafe, PanicHookInfo};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, Once, PoisonError};
use std::thread::ThreadId;

/// How the crash handler is set up.
#[derive(Debug, Clone)]
pub struct CrashHandler {
    /// Where reports are written: the log directory.
    pub log_directory: PathBuf,
    /// The editor, named in each report and in the dialog.
    pub product: Product,
    /// Whether a crash is announced in a dialog; tests and headless runs say no, and
    /// [`dialogs_possible`] tells the Host.
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

/// What the dialog and the report say for the log file when logging went to the terminal only.
const NO_LOG_FILE: &str = "none: logging went to the terminal only";

/// The handler installed, and the main thread's identity.
static INSTALLED: Mutex<Option<Installed>> = Mutex::new(None);

/// Installs the hook once per process.
static INSTALL: Once = Once::new();

/// Reports written on other threads that the main thread has not announced yet.
static PENDING: Mutex<VecDeque<CrashReport>> = Mutex::new(VecDeque::new());

thread_local! {
    /// Whether the hook is already running on this thread, so a panic inside it is not handled
    /// again.
    static HANDLING: Cell<bool> = const { Cell::new(false) };
    /// Whether this thread catches its own panics, so the hook only logs them.
    static CAUGHT: Cell<bool> = const { Cell::new(false) };
}

/// What the hook knows.
#[derive(Debug, Clone)]
struct Installed {
    /// The handler's set-up.
    handler: CrashHandler,
    /// The thread the handler was installed on: the main thread.
    main_thread: ThreadId,
}

/// Whether a crash dialog can be shown and waited for: not on a continuous-integration run
/// (`CI` set), not when `DRS_NO_DIALOGS` is set, not on Linux without a display
/// (`DISPLAY` and `WAYLAND_DISPLAY` both unset), and in development not while a script drives
/// the editor (`DRS_SCRIPT` set), since nobody is there to click.
#[must_use]
pub fn dialogs_possible() -> bool {
    let set = |variable: &str| std::env::var_os(variable).is_some_and(|value| !value.is_empty());
    if set("CI") || set("DRS_NO_DIALOGS") {
        return false;
    }
    #[cfg(feature = "dev")]
    if set("DRS_SCRIPT") {
        return false;
    }
    if cfg!(target_os = "linux") && !set("DISPLAY") && !set("WAYLAND_DISPLAY") {
        return false;
    }
    true
}

/// Marks whether the calling thread catches its own panics from now on, as a thread does that
/// runs each job of a queue under `std::panic::catch_unwind` and turns a panic into the failure
/// of that one job. While it is marked, a panic on it is logged at `warn` with its message,
/// location, and thread, and leaves no crash report and nothing to announce; the thread marks
/// itself before the job and unmarks itself after.
pub fn mark_panics_caught(caught: bool) {
    CAUGHT.set(caught);
}

/// Installs the crash handler on the calling thread, which is taken to be the main thread.
///
/// Installed before Bevy's plugins; a plugin that sets a hook of its own afterwards is expected
/// to chain the one it finds, as Bevy's log plugin does, so this one keeps running. A panic on
/// any thread then writes a report named `crash-<UTC timestamp>.txt` in the log directory, or
/// in the platform's temporary directory when the log directory cannot be written; prints its
/// path to the terminal; logs it at `error`; and, when dialogs are on, shows the dialog at once
/// on the main thread, or otherwise leaves the report pending for [`announce_pending`]. A
/// panic inside the hook itself, as from a closed terminal or a dialog that cannot open, is
/// caught and changes nothing else, and a panic on a thread that [`mark_panics_caught`] marked is
/// only logged. Installing again replaces the set-up; the hook itself is installed once per
/// process.
pub fn install_crash_handler(handler: CrashHandler) {
    *INSTALLED.lock().unwrap_or_else(PoisonError::into_inner) = Some(Installed {
        handler,
        main_thread: std::thread::current().id(),
    });
    INSTALL.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if HANDLING.replace(true) {
                return;
            }
            if CAUGHT.get() {
                let _ = std::panic::catch_unwind(AssertUnwindSafe(|| log_caught(info)));
            } else {
                let _ = std::panic::catch_unwind(AssertUnwindSafe(|| on_panic(info)));
                let _ = std::panic::catch_unwind(AssertUnwindSafe(|| previous(info)));
                let _ = std::panic::catch_unwind(AssertUnwindSafe(announce_at_once));
            }
            HANDLING.set(false);
        }));
    });
}

/// Announces a report written on another thread that has not been announced yet, if any: shows
/// the dialog that says the editor crashed and names the report and the log file, when dialogs
/// are on, and hands the report back for the status line. Called on the main thread; the
/// Editor asks each frame.
pub fn announce_pending() -> Option<CrashReport> {
    let report = PENDING
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .pop_front()?;
    show_dialog(&report);
    Some(report)
}

/// Shows the dialog for `report` when dialogs are on; shows nothing otherwise.
fn show_dialog(report: &CrashReport) {
    let Some(product) = INSTALLED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .filter(|installed| installed.handler.dialogs)
        .map(|installed| installed.handler.product)
    else {
        return;
    };
    let name = product.name;
    let log_file = report
        .log_file
        .as_ref()
        .map_or_else(|| NO_LOG_FILE.to_owned(), |file| file.display().to_string());
    let dialog = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(format!("{name} crashed"))
        .set_description(format!(
            "{name} crashed: {}\n\nThe crash report was written to:\n{}\n\nThe log file is:\n{log_file}",
            report.message,
            report.path.display(),
        ))
        .set_buttons(rfd::MessageButtons::Ok);
    // A dialog that cannot be shown changes nothing else: the report is already written.
    let _ = std::panic::catch_unwind(AssertUnwindSafe(|| dialog.show()));
}

/// Runs the editor and, when it ends, announces a report that was left pending, so a crash the
/// executor carried from another thread to the main one is announced exactly once.
///
/// A panic that ends the run is resumed after the announcement, so the process still ends as a
/// panic does.
pub fn run_guarded<R>(run: impl FnOnce() -> R) -> R {
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(run));
    announce_pending();
    match outcome {
        Ok(result) => result,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

/// Logs a panic its thread catches itself.
fn log_caught(info: &PanicHookInfo<'_>) {
    let thread = std::thread::current();
    tracing::warn!(
        "a panic on the thread {} was caught there: {} at {}",
        thread.name().unwrap_or("unnamed"),
        panic_message(info),
        location(info)
    );
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
    let name = installed.handler.product.name;
    // A terminal that is gone must not keep the report from being announced elsewhere.
    let Some(path) = write_report(&installed.handler.log_directory, now, &text) else {
        let _ = writeln!(
            std::io::stderr(),
            "{name} crashed and the crash report could not be written anywhere"
        );
        return;
    };
    let _ = writeln!(
        std::io::stderr(),
        "{name} crashed; the crash report is at {}",
        path.display()
    );
    tracing::error!(report = %path.display(), "{name} crashed: {message}");
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
    if on_main_thread {
        announce_pending();
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

/// Where the panic happened, as `file:line:column`.
fn location(info: &PanicHookInfo<'_>) -> String {
    info.location().map_or_else(
        || "unknown".to_owned(),
        |location| {
            format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        },
    )
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
    let _ = writeln!(text, "# {} crash report", handler.product.name);
    let _ = writeln!(text, "\n## When (UTC)\n{}", timestamp(now));
    let _ = writeln!(text, "\n## Version\n{}", handler.product.version);
    let _ = writeln!(
        text,
        "\n## Platform\n{} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    let _ = writeln!(text, "\n## Message\n{message}");
    let _ = writeln!(text, "\n## Location\n{}", location(info));
    let thread = std::thread::current();
    let _ = writeln!(text, "\n## Thread\n{}", thread.name().unwrap_or("unnamed"));
    let _ = writeln!(
        text,
        "\n## Backtrace\n{}",
        std::backtrace::Backtrace::force_capture()
    );
    let log_file =
        log_file.map_or_else(|| NO_LOG_FILE.to_owned(), |file| file.display().to_string());
    let _ = writeln!(text, "\n## Log file\n{log_file}");
    text
}

/// Writes the report into the log directory, or into the temporary directory when the log
/// directory cannot be written; `None` when neither could be.
fn write_report(log_directory: &Path, now: time::OffsetDateTime, text: &str) -> Option<PathBuf> {
    let stem = format!("crash-{}", file_timestamp(now));
    write_into(log_directory, &stem, text)
        .or_else(|| write_into(&std::env::temp_dir(), &stem, text))
}

/// Writes the report as `<stem>.txt` in `directory`, creating the directory first and never
/// overwriting an earlier report: the file is created only if it does not exist yet, so two
/// threads crashing in the same second each keep their own; the path written, or `None`.
fn write_into(directory: &Path, stem: &str, text: &str) -> Option<PathBuf> {
    std::fs::create_dir_all(directory).ok()?;
    for attempt in 1..=1000_u32 {
        let path = if attempt == 1 {
            directory.join(format!("{stem}.txt"))
        } else {
            directory.join(format!("{stem}-{attempt}.txt"))
        };
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                file.write_all(text.as_bytes()).ok()?;
                return Some(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return None,
        }
    }
    None
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
