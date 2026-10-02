# Diagnostics

**Commands**: none

## Purpose

When the editor misbehaves, what it knew must survive the moment. This capability writes everything the editor logs to a daily file in the editor's own directories, turns a crash into a report the Author can find and send, and finds the Bundled Files the editor ships with from where the executable is, whatever folder it was started from. Its users are the Author who hits a problem and the contributor who reads what it left behind.

## User Stories

### Logging

1. As an Author, I can find everything the editor logged in a file, so that what went wrong is still on disk after the editor has closed.
2. As an Author, I get a new log file each day and only the last week of files kept, so that the logs never grow without bound.
3. As an Author, I can rely on the log files living in the editor's own directories and never in the folder I started the editor from, so that my Projects and Asset Folders never gain stray files.
4. As an Author starting the editor for the first time, I get a log from its first line although its directories did not exist yet, so that a fresh install is as diagnosable as an old one.
5. As an Author, I can start the editor even when its log file cannot be created, with the reason in the terminal, so that a full disk or a locked directory never stops me from working.
6. As a contributor, I can read when each entry was logged, how serious it is, and where in the editor it came from, so that I can follow what happened.
7. As a contributor, I can set how much is logged through the usual `RUST_LOG` variable, so that a problem can be run again with more detail and no rebuild.
8. As a contributor, I get a sensible level without setting anything, so that the editor's own events are recorded and the engine's noise is not.
9. As an Author, I can mistype `RUST_LOG` and still get the default level with a message, so that a typo never silences the log.
10. As an Author, I can read the log file's path in the first line of the log, so that I know where to look from the terminal.
11. As an Author, I can open the folder the logs are in from Help → Show Logs, so that I can attach the log without knowing where my platform keeps it.
12. As an Author, I can find the lines logged just before a crash in the log file, so that the report and the log together tell the whole story.

### Crashes

13. As an Author, I see a dialog when the editor crashes that says so and names the crash report, so that I know what happened and what to send.
14. As an Author, I can find the crash report in the log folder, so that it is still there once the dialog is closed.
15. As an Author, I get the report and the dialog for a crash during startup, before any window exists, so that an editor that will not start on a new device can still be diagnosed.
16. As an Author, I get the report and the dialog for a crash on a background thread, so that nothing depends on which thread failed.
17. As a contributor, I can read in the report the message, where in the code it happened, the backtrace, the editor's version, the operating system and architecture, when it happened, and which log file was current, so that I can diagnose without asking the Author anything.
18. As an Author, I can share the report without reading it first, because it holds nothing about me beyond file paths.
19. As an Author, I can rely on the report being written before any dialog is attempted, so that a problem with the dialog never hides the crash.
20. As an Author, I get the crash report written somewhere even when the log folder cannot be written, with the dialog naming where, so that a crash never goes unrecorded.
21. As a contributor, I can read the crash and the report's path in the log as well, so that the log alone shows how a session ended.
22. As a contributor, I can run the editor headless, in tests, or on a continuous-integration machine and get crash reports without a dialog, so that an automated run never waits for a click.

### Bundled Files

23. As an Author on Windows or Linux, I can run the editor from wherever it was unpacked, because it finds its Bundled Files beside its executable.
24. As an Author on macOS, I can run the editor from the Applications folder, because it finds its Bundled Files inside its application.
25. As a contributor, I can start the editor from the workspace and have it find the Bundled Files there, so that development needs no copying.
26. As an Author, I can start the editor from any folder and have it find its Bundled Files, so that the working directory never matters.
27. As an Author, I am told when the bundle directory cannot be found, with the places that were tried, while the editor starts anyway, so that a broken install explains itself instead of crashing.
28. As a contributor, I get a report naming a Bundled File that is missing or unreadable when I look one up, never a crash, so that one lost file never takes the editor down.
29. As an Author, I can rely on my Asset Folders never being read as Bundled Files and the Bundled Files never appearing as Assets, so that the two are never confused.
30. As a contributor, I can rely on a Bundled File lookup never reaching outside the bundle directory, so that a bad name cannot read an arbitrary file.

## Rules

### Logging

**Logged to a file**: every entry the editor logs at or above the configured level is appended to the current log file.

**Logs live in the editor's directories**: the log directory is `logs` under the editor's cache directory, or `logs` under the root that overrides the editor's directories, or `dungeon-rs/logs` under the platform's temporary directory when the platform names no cache directory; it is never the working directory.
_Why_ the cache directory: logs and crash reports are the editor's own, device-local files, and the cache directory is the one such place every platform names.

**The directory is made**: the log directory is created at start when it does not exist, parents included, and nothing is logged about creating it.

**One file per day**: the log file is named `dungeon-rs.<date>.log` with the UTC date of the day its entries were written, and the first entry of a new day goes to that day's file.

**A week is kept**: at most seven daily log files exist after start and after a new day's file is created; the files with the oldest dates in their names beyond seven are deleted, whatever order they were written in, and crash reports are never deleted.

**Written as it happens**: an entry is in the file once the call that logged it returns; nothing is held back in a buffer.
_Why_: the lines just before a crash are the ones a contributor needs, and a buffer loses exactly those.

**Logging never stops the editor**: when the log directory or the log file cannot be created, the editor starts, logs to the terminal only, and the first line on the terminal says why.

**Entries are dated**: every entry in the file carries its UTC timestamp, its level, and the module it came from.

**Level from `RUST_LOG`**: when the variable is set and well-formed, its directives are laid over the default ones, the more specific winning, in the file and on the terminal alike; otherwise the default applies, and a malformed value is said on the terminal after the line about where the log went.

**Default level**: without `RUST_LOG`, entries at `info` and above are logged, except that `wgpu` logs at `error` and above and `naga` at `warn` and above.

**Location announced**: the first entry logged at start names the current log file's path.

**Show Logs**: the Help menu holds Show Logs, which opens the log directory in the platform's file manager, creating the directory first when it does not exist; a failure to open it is shown in the status line.

### Crashes

**A crash leaves a report**: a panic on any thread writes a crash report named `crash-<UTC timestamp>.txt` in the log directory before anything else is done about it, and never overwrites an earlier report.

**What the report holds**: the UTC timestamp, the editor's version, the operating system and architecture, the panic message, the source location, the name of the thread, a backtrace, and the path of the current log file, each under its own heading.

**Nothing private beyond paths**: the report holds no environment variables, no user name, no host name, and nothing of a Project or an Asset Folder beyond what the message itself contains.

**A crash shows a dialog**: a dialog names the editor, says it crashed, and names the crash report's path and the current log file's path; it is shown at once when the crash is on the main thread, and otherwise on the main thread as soon as it next runs, at the next frame while the editor still runs or when the editor has ended, and exactly once; a crash on another thread is also shown in the status line while the editor still runs.

**The report comes first**: the report is written and its path printed to the terminal before any dialog is attempted, and a dialog that cannot be shown, as without a display, changes nothing else.

**Before any window**: a crash before the window exists writes the report and shows the dialog like any other.

**Somewhere always**: when the log directory cannot be written, the report is written to the platform's temporary directory instead, and the dialog and the terminal name that path.

**A crash is logged**: the panic message and the report's path are logged at `error` once the report is written.

**Dialogs can be off**: the crash handler can be installed without dialogs, and then writes and logs the report and shows nothing; tests install it that way.

**Dialogs only where someone can answer**: the Host installs the crash handler without dialogs on a continuous-integration run (`CI` set), when `DRS_NO_DIALOGS` is set, on Linux without a display (`DISPLAY` and `WAYLAND_DISPLAY` both unset), and in a development build while a script drives the editor.
_Why_: the dialog waits for a click, and a run nobody watches would hang on it.

### Bundled Files

**Found by layout**: the bundle directory is the first of these that is marked as the editor's: `bundle` beside the executable; `Resources` beside the executable's directory, which is the macOS application layout; `bundle` in each ancestor of the executable's directory, nearest first, which is the workspace layout during development.

**Marked as the editor's**: a directory counts as the bundle directory only when it holds the marker file `dungeon-rs.bundle`; a marker naming a version other than the editor's is logged at `warn`, naming the directory and the version, and the directory is used.
_Why_ a marker: without one, a stray `bundle` folder on the way up from the build directory would be taken for the editor's.

**The working directory never matters**: the bundle directory found does not depend on the directory the editor was started from.

**No directory is a report**: when no location is marked, the editor starts, an `error` entry and the status line name every location tried, and the default asset source is rooted at the first location tried, which holds no marker, so the engine's own guess at an asset root never reads anything; when the executable's own location is unknown, nothing is tried, the report says so, and the source is rooted at a directory under the platform's temporary directory that nothing creates.

**A missing Bundled File is a report**: a Bundled File looked up by name that is absent or unreadable yields a report with its name, never a panic.

**Bundled Files stay inside**: a Bundled File is named by a plain relative path; a name that is empty, absolute, holds `..`, begins with `./`, carries a source prefix, or uses `\` or `:` is refused, so no lookup leaves the bundle directory.
_Why_ `\` and `:`: they are not the same path on every platform, so a name holding them could mean one file here and another there.

**Separate sources**: Bundled Files load only from the bundle directory, through the default asset source, and the Author's Assets only through `lib://`, so an Asset Folder is never read as a Bundled File, and the editor never adds the bundle directory as an Asset Folder of its own accord.

## Implementation Decisions

- **The Utility, diagnostics**: the component that owns these behaviours, with the contract StartLogging, InstallCrashHandler, LocateBundledFiles, and RevealLogs. It uses no Bevy. The Host wires it before the App exists and passes values only: the editor's name and version as one Product value, the resolved log directory, whether dialogs are possible, which the Utility itself decides, and the layer the Utility yields, which the Host hands to Bevy's log plugin as its custom layer. The Editor calls the Utility to reveal the logs and to announce a pending crash report.
- **What the start found reaches the Editor as a plugin parameter**: once the subscriber is installed, the Utility logs what the start found and yields it as one value: the log directory, the current log file or none, and the bundle directory or the locations tried. The Host builds the Editor's plugin over that value, which the Editor keeps for Show Logs and the status line, and roots the default asset source where the value says. _Why_ a parameter rather than a shared model type: only the Editor reads it, and nothing in the model should describe the Host's wiring.
- **Order at start**: the Host resolves the editor's directories; installs the crash handler, so that a crash before any window still leaves a report and so that a plugin that sets a hook of its own afterwards finds this one to chain; starts logging; locates the bundle directory from its own executable; adds Bevy's log plugin on its own, so that the first entry names the log file before any other plugin logs; then builds the App with the default asset source rooted at the directory the Utility named, absolute, so the engine's own resolution of an asset root plays no part; and runs the App through the Utility's guarded run, which announces a report the main thread had not yet shown when the App ends.
- **The editor's directories**: the directory overrides in the model gained the log directory and resolve the platform's directories themselves, the log directory being `logs` under the cache directory unless named, so LibraryAccess and the Host share one resolver. The Host's development-only `DRS_DIRECTORIES` places every directory under that root. When the platform names no directories, the Utility names `dungeon-rs/logs` under the platform's temporary directory, so logging has somewhere to go.
- **Logging**: a daily rolling file appender with a maximum of seven files, writing synchronously, over a directory the Utility creates before the appender is built. _Why_ create first: the appender logs an error of its own when the directory it prunes does not exist yet. The Utility prunes by the dates in the file names before the appender is built, to one fewer than the kept count with today's file counted when it already exists, since the appender prunes by file-creation time, which not every file system remembers, both when it is built and at midnight; so pruned, the appender finds fewer files than its maximum and deletes nothing at start. Dates roll on UTC, the appender's clock, and entries carry UTC timestamps for the same reason, so the log and the report never disagree. The file layer's filter is the default directives with those of `RUST_LOG` laid over them when it is well-formed, the more specific winning, which is how Bevy's log plugin reads the variable for the terminal, and the default directives alone otherwise; the Host hands the log plugin the Utility's default filter, so the terminal starts from the same directives as the file and the two agree. When logging goes to the terminal only, the first entry is a warning that says so and why.
- **Crash handler**: a panic hook installed once per process, which records the thread it was installed on as the main thread and chains the hook it replaced. On a panic it writes the report, prints its path to the terminal, logs it at `error`, and leaves the report pending; on the main thread with dialogs on it then shows the dialog at once. The Editor asks each frame, on the main thread, for a pending report, shows the dialog, and puts the report's path in the status line; the guarded run announces one left pending when the App ends, so a panic the executor carries to the main thread is announced exactly once, and resumes the panic afterwards so the process still ends as a panic does. The report file is created only if it does not exist yet, with a numbered suffix when two crashes fall in the same second. The hook catches a panic of its own, as from a closed terminal or a dialog that cannot open, refuses to run again on a thread it is already running on, and ignores every error it meets; when the report cannot be written anywhere, the terminal says so and nothing else happens. The backtrace is captured whatever `RUST_BACKTRACE` says; the operating system and architecture are the standard library's names.
- **Whether a dialog can be shown is the Utility's to say**: not on a continuous-integration run, not when `DRS_NO_DIALOGS` is set, not on Linux without a display, and, in a development build, not while a script drives the editor; the Host passes the answer when installing the handler.
- **The dialog**: a native message dialog. The Utility is granted `rfd` next to the Editor in the Restricted external dependencies table. _Why_: a crash dialog may be needed before the Editor exists, so this is the one place a dialog is shown outside the Client.
- **Bundled Files**: a `bundle` directory at the workspace root holds the editor's Bundled Files and the marker file `dungeon-rs.bundle`, which holds the editor's version; the workspace gate checks that the marker names the workspace's version (the workspace capability's Rule "Bundle marker names the version"). The Utility finds the bundle directory from the executable's location through the layouts above; a marker that names no version marks the directory and is accepted without a warning. It gives one Bundled File by a plain relative name, checked to exist under the directory; no component looks one up, since the editor ships no Bundled File it reads itself, and what the engine loads from the bundle directory is reported by the engine's own log entries. The `lib://` source stays as it is; the two sources share nothing.
- **Show Logs**: the Editor's Help menu holds Show Logs, which calls RevealLogs; the Utility creates the directory and starts the platform's file-manager command on it, detached, and a failure is shown in the status line.
- **Lints**: writing to the terminal warns workspace-wide, so the only places that do it are the ones that run before logging exists, each with an expectation that says why; changing the working directory is a disallowed method, so nothing may come to depend on it.

## Test seams

- **Logged to a file**: `crates/drs-diagnostics/tests/logging.rs::entries_are_logged_to_the_dated_file`
- **Logs live in the editor's directories**: `crates/drs-model/tests/directories.rs::the_log_directory_is_logs_under_the_cache_directory_by_default`, `crates/drs-model/tests/directories.rs::every_directory_is_under_the_root_that_overrides`, `crates/drs-model/tests/directories.rs::the_platform_directories_apply_where_nothing_overrides`, `crates/drs-diagnostics/tests/logging.rs::the_log_directory_is_the_one_resolved_or_under_the_temporary_directory`
- **The directory is made**: `crates/drs-diagnostics/tests/logging.rs::the_log_directory_is_made`
- **One file per day**: `crates/drs-diagnostics/tests/logging.rs::entries_are_logged_to_the_dated_file`, `crates/drs-diagnostics/tests/logging.rs::a_week_of_files_is_kept`
- **A week is kept**: `crates/drs-diagnostics/tests/logging.rs::a_week_of_files_is_kept`, `crates/drs-diagnostics/tests/logging.rs::a_start_on_a_day_with_a_file_keeps_that_file`
- **Written as it happens**: `crates/drs-diagnostics/tests/logging.rs::an_entry_is_written_as_it_happens`
- **Logging never stops the editor**: `crates/drs-diagnostics/tests/logging.rs::a_log_directory_that_cannot_be_made_leaves_the_terminal_only`
- **Entries are dated**: `crates/drs-diagnostics/tests/logging.rs::entries_are_logged_to_the_dated_file`
- **Level from `RUST_LOG`**: `crates/drs-diagnostics/tests/logging.rs::the_level_comes_from_rust_log`, `crates/drs-diagnostics/tests/logging.rs::a_malformed_rust_log_falls_back_to_the_default`
- **Default level**: `crates/drs-diagnostics/tests/logging.rs::the_default_level_applies_without_rust_log`
- **Location announced**: `crates/drs-diagnostics/tests/logging.rs::the_first_entry_names_the_log_file`
- **Show Logs**: no automated test; checked by hand (see Notes).
- **A crash leaves a report**: `crates/drs-diagnostics/tests/crashes.rs::a_crash_on_any_thread_leaves_a_report`
- **What the report holds**: `crates/drs-diagnostics/tests/crashes.rs::the_report_holds_each_field_under_its_heading`
- **Nothing private beyond paths**: `crates/drs-diagnostics/tests/crashes.rs::the_report_holds_nothing_private_beyond_paths`
- **A crash shows a dialog**: no automated test; checked by hand (see Notes).
- **The report comes first**: `crates/drs-diagnostics/tests/crashes.rs::the_report_comes_first`
- **Before any window**: no automated test; checked by hand (see Notes).
- **Somewhere always**: `crates/drs-diagnostics/tests/crash_fallback.rs::an_unwritable_log_directory_sends_the_report_to_the_temporary_directory`
- **A crash is logged**: `crates/drs-diagnostics/tests/crashes.rs::a_crash_is_logged`
- **Dialogs can be off**: `crates/drs-diagnostics/tests/crashes.rs::dialogs_can_be_off`
- **Dialogs only where someone can answer**: no automated test; checked by hand (see Notes).
- **Found by layout**: `crates/drs-diagnostics/tests/bundled_files.rs::the_bundle_beside_the_executable_is_found`, `crates/drs-diagnostics/tests/bundled_files.rs::the_bundle_inside_the_macos_application_is_found`, `crates/drs-diagnostics/tests/bundled_files.rs::the_bundle_in_a_workspace_ancestor_is_found`
- **Marked as the editor's**: `crates/drs-diagnostics/tests/bundled_files.rs::a_directory_without_the_marker_does_not_count`, `crates/drs-diagnostics/tests/bundled_files.rs::a_marker_naming_another_version_is_still_used`, `crates/drs-diagnostics/tests/logging.rs::a_marker_naming_another_version_is_logged_as_a_warning`
- **The working directory never matters**: `crates/drs-diagnostics/tests/bundled_files.rs::the_working_directory_never_matters`
- **No directory is a report**: `crates/drs-diagnostics/tests/bundled_files.rs::no_marked_directory_names_every_location_tried`, `crates/drs-diagnostics/tests/logging.rs::no_bundle_directory_is_logged_as_an_error_naming_every_location_tried`, `crates/drs-diagnostics/tests/logging.rs::the_first_entry_names_the_log_file`
- **A missing Bundled File is a report**: `crates/drs-diagnostics/tests/bundled_files.rs::a_missing_bundled_file_is_reported_by_name`
- **Bundled Files stay inside**: `crates/drs-diagnostics/tests/bundled_files.rs::names_that_leave_the_directory_are_refused`, `crates/drs-diagnostics/tests/bundled_files.rs::a_source_prefixed_name_is_refused`
- **Separate sources**: `crates/drs-diagnostics/tests/bundled_files.rs::a_source_prefixed_name_is_refused`, `crates/drs-diagnostics/tests/logging.rs::a_marker_naming_another_version_is_logged_as_a_warning`, `crates/drs-diagnostics/tests/logging.rs::no_bundle_directory_is_logged_as_an_error_naming_every_location_tried`

## Not supported

- Crash reports and logs never leave the device: the editor sends nothing anywhere and never asks the Author to send a report.
- The editor never writes a log file or a crash report into the working directory, a Project, or an Asset Folder.
- The bundle directory is never listed as an Asset Folder, and an Asset Folder is never read as Bundled Files.
- A crash on a background thread that the engine does not carry to the main thread does not end the editor: the report is announced and the editor keeps running.

## Notes

- Four Rules (Show Logs, A crash shows a dialog, Before any window, Dialogs only where someone can answer) and the status-line half of No directory is a report have no automated test, and five Rules are tested in part: what the terminal shows is not read back for Logging never stops the editor, The report comes first, and Somewhere always, the terminal's own filter is not read back for Level from `RUST_LOG`, which agrees with the file by construction, and the Asset Folder half of Separate sources is a statement about what the editor never does. All of these are against the requirement that every Rule has one. behaviour of the egui interface, of a native dialog, of the terminal, or of the environment the process runs in, for which no headless seam exists: the dialog blocks for a click, Show Logs starts the platform's file manager, and whether dialogs are possible is read from the process's own environment, which a test cannot set without unsafe code. The accepted deviation is verification by hand, driving the editor with the development-only switches: `DRS_CRASH_TEST` set to `main`, `thread`, or `startup` forces a crash on the main thread on the second frame, on a spawned thread on the second frame, or while the plugins build before any window exists. That is tooling for verification, not behaviour of the capability.
- The macOS application layout is checked against a fixture layout; a built application is checked by hand.
- The crash handler is installed process-wide, so each test file that installs it is the only one in its process that may, and its tests take turns; the test for Somewhere always runs on Unix only, where a directory can be made unwritable.
- Rolling at midnight is the appender's behaviour and is not driven by a test, since the appender exposes no clock; One file per day is checked through the file's name and A week is kept through pruning at start. The tests that name the day take it once before starting and give up when it has changed since, rather than fail at midnight.
- Nothing private beyond paths is stated as what the report does not hold; the message a panic carries may itself name a path, and a backtrace frame may name the home directory.
- "Contributor" is the engineering role; it is not the domain's Author.
