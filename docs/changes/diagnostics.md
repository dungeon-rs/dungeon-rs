# Diagnostics

**Capabilities**:
- diagnostics: none. The capability owns no domain Command; it is an engineering capability like the workspace, and its "user" is the Author who hits a problem and the contributor who reads what it left behind.

## Problem Statement

When the editor misbehaves, nothing of it survives the moment. What it logged went to a terminal the Author may not have had open, and is gone when the window closes. A crash takes the window down without a word, leaving the Author with no idea what happened and nothing to send; a crash before the window even appears looks like the editor refusing to start. Where the editor's own files land depends on where it was started from, so a run from a Project's folder can leave stray files among the Author's work. And the editor finds the resources it ships with only when started the way the engine assumes, so an installed copy, or the application on macOS, may not find its own shaders and fonts, and fails in ways that look like anything but what they are.

## Solution

Everything the editor logs goes to a daily file in the editor's own directories, with a week of files kept, and the Author can open that folder from the Help menu. A crash writes a report with everything a contributor needs to diagnose it, then shows a dialog that says the editor crashed and names the report, whether the crash came on the main thread, on a background thread, or before any window existed. How much is logged is set with the usual `RUST_LOG` variable, with a sensible default. The resources the editor ships with are found by the layout they were installed in: beside the executable, inside the macOS application, or in the workspace during development, and a resource that is not there is reported, never a crash. The Author's Assets and the editor's own resources never meet.

## User Stories

### Logging

1. As an Author, I want everything the editor logs written to a file, so that what went wrong is still on disk after the editor has closed.
2. As an Author, I want a new log file each day and only the last week of files kept, so that the logs never grow without bound.
3. As an Author, I want the log files kept in the editor's own directories and never in the folder I started the editor from, so that my Projects and Asset Folders never gain stray files.
4. As an Author starting the editor for the first time, I want it to log from its first line although its directories do not exist yet, so that a fresh install is as diagnosable as an old one.
5. As an Author, I want the editor to start even when its log file cannot be created, with the reason in the terminal, so that a full disk or a locked directory never stops me from working.
6. As an Author, I want each entry to carry when it was logged, how serious it is, and where in the editor it came from, so that a contributor can follow what happened.
7. As a contributor, I want to set how much is logged through the usual `RUST_LOG` variable, so that a problem can be run again with more detail and no rebuild.
8. As a contributor, I want a sensible level without setting anything, so that the editor's own events are recorded and the engine's noise is not.
9. As an Author, I want a mistyped `RUST_LOG` to fall back to the default with a message, so that a typo never silences the log.
10. As an Author, I want the first line of the log to name the file's path, so that I know where to look from the terminal.
11. As an Author, I want a Show Logs entry in the Help menu that opens the folder the logs are in, so that I can attach the log without knowing where my platform keeps it.
12. As an Author, I want the lines logged just before a crash to be in the log file, so that the report and the log together tell the whole story.

### Crashes

13. As an Author, I want a crash to show a dialog that says the editor crashed and names the crash report, so that I know what happened and what to send.
14. As an Author, I want the crash report left in the log folder, so that it is still there once the dialog is closed.
15. As an Author, I want a crash during startup, before any window exists, to leave the report and show the dialog, so that an editor that will not start on a new device can still be diagnosed.
16. As an Author, I want a crash on a background thread to leave the report and show the dialog, so that nothing depends on which thread failed.
17. As a contributor, I want the report to hold the message, where in the code it happened, the backtrace, the editor's version, the operating system and architecture, when it happened, and which log file was current, so that I can diagnose without asking the Author anything.
18. As an Author, I want the report to hold nothing about me beyond file paths, so that I can share it without reading it first.
19. As an Author, I want the report written before any dialog is attempted, so that a problem with the dialog never hides the crash.
20. As an Author, I want the crash report written somewhere even when the log folder cannot be written, with the dialog naming where, so that a crash never goes unrecorded.
21. As an Author, I want the crash and the report's path in the log as well, so that the log alone shows how a session ended.
22. As a contributor, I want headless runs and tests to leave crash reports without showing a dialog, so that an automated run never waits for a click.

### Bundled resources

23. As an Author on Windows or Linux, I want the editor to find its bundled resources beside its executable, so that it runs from wherever it was unpacked.
24. As an Author on macOS, I want the editor to find its bundled resources inside its application, so that it works from the Applications folder.
25. As a contributor, I want the editor started from the workspace to find the resources in the workspace, so that development needs no copying.
26. As an Author, I want the editor to find its resources whatever folder I start it from, so that the working directory never matters.
27. As an Author, I want a resource directory that cannot be found reported, with the places that were tried, while the editor starts anyway, so that a broken install explains itself instead of crashing.
28. As an Author, I want a bundled resource that is missing or unreadable reported by name, not a crash, so that one lost file never takes the editor down.
29. As an Author, I want my Asset Folders never to be read as bundled resources and the bundled resources never to appear as Assets, so that the two are never confused.
30. As a contributor, I want a resource lookup never to reach outside the resource directory, so that a bad name cannot read an arbitrary file.

## Rules

### Logging

**Logged to a file**: every entry the editor logs at or above the configured level is appended to the current log file.

**Logs live in the editor's directories**: the log directory is `logs` under the editor's cache directory, or `logs` under the root that overrides the editor's directories; it is never the working directory.
_Why_ the cache directory: logs and crash reports are the editor's own, device-local files, and the cache directory is the one such place every platform names.

**The directory is made**: the log directory is created at start when it does not exist, parents included, and nothing is logged about creating it.

**One file per day**: the log file is named `dungeon-rs.<date>.log` with the UTC date of the day its entries were written, and the first entry of a new day goes to that day's file.

**A week is kept**: at most seven daily log files exist after start and after a new day's file is created; the oldest beyond seven are deleted, and crash reports are never deleted.

**Written as it happens**: an entry is in the file once the call that logged it returns; nothing is held back in a buffer.
_Why_: the lines just before a crash are the ones a contributor needs, and a buffer loses exactly those.

**Logging never stops the editor**: when the log directory or the log file cannot be created, the editor starts, logs to the terminal only, and the first line on the terminal says why.

**Entries are dated**: every entry in the file carries its UTC timestamp, its level, and the module it came from.

**Level from `RUST_LOG`**: when the variable is set and well-formed, its directives decide which entries are logged; otherwise the default applies, and a malformed value is said on the terminal.

**Default level**: without `RUST_LOG`, entries at `info` and above are logged, except that `wgpu` logs at `error` and above and `naga` at `warn` and above.

**Location announced**: the first entry logged at start names the current log file's path.

**Show Logs**: the Help menu holds Show Logs, which opens the log directory in the platform's file manager, creating the directory first when it does not exist; a failure to open it is shown in the status line.

### Crashes

**A crash leaves a report**: a panic on any thread writes a crash report named `crash-<UTC timestamp>.txt` in the log directory before anything else is done about it.

**What the report holds**: the UTC timestamp, the editor's version, the operating system and architecture, the panic message, the source location, the name of the thread, a backtrace, and the path of the current log file, each under its own heading.

**Nothing private beyond paths**: the report holds no environment variables, no user name, no host name, and nothing of a Project or an Asset Folder beyond what the message itself contains.

**A crash shows a dialog**: a dialog names the editor, says it crashed, and names the crash report's path and the current log file's path; it is shown at once when the crash is on the main thread, and otherwise on the main thread as soon as it next runs: at the next frame while the editor still runs, or when the editor has ended.

**The report comes first**: the report is written and its path printed to the terminal before any dialog is attempted, and a dialog that cannot be shown, as without a display, changes nothing else.

**Before any window**: a crash before the window exists writes the report and shows the dialog like any other.

**Somewhere always**: when the log directory cannot be written, the report is written to the platform's temporary directory instead, and the dialog and the terminal name that path.

**A crash is logged**: the panic message and the report's path are logged at `error` once the report is written.

**Dialogs can be off**: the crash handler can be installed without dialogs, and then writes and logs the report and shows nothing; tests and headless runs install it that way.

### Bundled resources

**Found by layout**: the resource directory is the first of these that is marked as the editor's: `resources` beside the executable; `Resources` beside the executable's directory, which is the macOS application layout; `resources` in each ancestor of the executable's directory, nearest first, which is the workspace layout during development.

**Marked as the editor's**: a directory counts as the resource directory only when it holds the marker file `dungeon-rs.resources`; a marker naming a version other than the editor's is logged at `warn` and the directory is used.
_Why_ a marker: without one, a stray `resources` folder on the way up from the build directory would be taken for the editor's.

**The working directory never matters**: the resource directory found does not depend on the directory the editor was started from.

**No directory is a report**: when no location is marked, the editor starts, and an `error` entry and the status line name every location tried.

**A missing resource is a report**: a bundled resource asked for by name that is absent or unreadable is reported with its name and the editor keeps running.

**Resources stay inside**: a bundled resource is named by a plain relative path; a name that is absolute, holds `..`, or carries a source prefix is refused, so no lookup leaves the resource directory.

**Separate sources**: bundled resources load only from the resource directory, through the default asset source, and the Author's Assets only through `lib://`, so an Asset Folder is never read as a resource and the resource directory is never listed as an Asset Folder.

## Implementation Decisions

- **A new Utility, diagnostics**: the component that owns these behaviours. The Host wires it before the App exists and holds no logic of its own; the Editor calls it to reveal the logs and to announce a pending crash report. It uses no Bevy: its logging operation yields a layer the Host hands to Bevy's log plugin as its custom layer, so the filter the plugin builds from `RUST_LOG` and its defaults applies to the terminal and the file alike. Contract: StartLogging (the layer and the current file's path), InstallCrashHandler, LocateResources (and the lookup of one resource under the directory found), RevealLogs.
- **Order at start**: the Host installs the crash handler first, so that the panic handler Bevy's log plugin builds chains ours; then starts logging; then locates the resource directory; then builds the App with the default asset source rooted at that directory and with a `model` resource holding what diagnostics found (the log directory and current file, and the resource directory or the locations tried), which the Editor reads for Show Logs and the status line; and runs the App through the Utility's guarded run, which announces a report the main thread had not yet shown when the App ends. The directory overrides in `model` gain the log directory, and the Host's dev-only `DRS_DIRECTORIES` places it under that root like the others.
- **Logging**: a daily rolling file appender with a maximum of seven files, writing synchronously, in the log directory the Utility creates before the appender is built. _Why_ create first: the appender logs an error of its own when the directory it prunes does not exist yet. Dates roll on UTC, the appender's clock; entries carry UTC timestamps for the same reason, so the log and the report never disagree. The Host leaves the log plugin's level and filter at their defaults, which are the Default level above; `RUST_LOG` overrides them as the plugin already allows.
- **Crash handler**: a panic hook that records the main thread's identity when installed, and on a panic writes the report, prints its path to the terminal, logs it, and then: on the main thread with dialogs on, shows the dialog at once; on another thread, leaves the report as pending. The Editor asks each frame for a pending report and shows the dialog on the main thread; the guarded run shows one left pending when the App ends, so a panic the executor carries to the main thread is announced exactly once. The hook never panics itself and ignores every error it meets. The backtrace is captured whatever `RUST_BACKTRACE` says; the operating system and architecture are the standard library's names; the editor's version is given by the Host when installing the handler.
- **The dialog**: a native message dialog. The Utility is granted `rfd` next to the Editor in the Restricted external dependencies table. _Why_: a crash dialog may be needed before the Editor exists, so this is the one place a dialog is shown outside the Client.
- **Resources**: a `resources` directory at the workspace root holds the editor's bundled resources and the marker file, which holds the editor's version. The Host sets the default asset source's root to the directory found, absolute, so that the engine's own resolution of the asset root plays no part. One resource looked up by name is checked to exist under the directory and refused when its name is not a plain relative path. The `lib://` source stays as it is; the two sources share nothing.
- **Show Logs**: the Editor adds a Help menu with Show Logs, which calls RevealLogs; opening the folder uses the platform's file manager through a small cross-platform opener.

## Testing

Test references take the form `path/to/file.rs::test_fn`. Two seams.

- **Seam: the Utility's public surface over temporary directories.** Logging: the layer the Utility yields is composed into a subscriber scoped to the test, entries are emitted at several levels and targets, and the file is read back. Covers: Logged to a file, The directory is made, One file per day (the file's name carries today's UTC date), A week is kept (a fixture of ten dated files is pruned to seven at start and the crash reports among them stay), Written as it happens (the entry is in the file before the subscriber is dropped), Logging never stops the editor (a directory that is a file yields a terminal-only outcome with the reason), Entries are dated, Level from `RUST_LOG` and Default level (the filter the Utility derives from a given value, or from none), Location announced. Crashes: the hook the Utility builds, with dialogs off, is installed once in the test process; a spawned thread panics, is joined, and the report is read back. Covers: A crash leaves a report, What the report holds, Nothing private beyond paths, Somewhere always (a directory that cannot be written sends the report to the temporary directory), A crash is logged, Dialogs can be off, The report comes first (the report exists and the path was printed with nothing else attempted). Resources: fixture layouts in a temporary directory, with and without the marker, each played through the executable path the Utility is given. Covers: Found by layout, Marked as the editor's, The working directory never matters, No directory is a report (the locations tried are named), A missing resource is a report, Resources stay inside, Separate sources (a `lib://` name is refused as not a plain relative path).
- **Checked by hand**, by the author or a verification agent driving the editor: Show Logs, the Help menu, the dialog as shown on each platform, A crash shows a dialog from the main thread, from a background thread, and after the App ends, Before any window (a panic forced before the window exists in a development build), the status line of No directory is a report, and the macOS application layout against a built bundle.

## Out of Scope

- Release packaging: building the application bundle, installers, icons, the Windows console window, signing and notarisation, and the release profile's debug information, so a release backtrace is as named as the build happens to allow.
- Telemetry, sending reports anywhere, and asking the Author to send one.
- Reporting what the engine itself fails to load from the resource directory beyond the engine's own log entries; no bundled resource is used yet, so no component reads one.
- Translations of the dialog and the menu.
- Log rotation by size, compression of old logs, and a log level set from the menu.
- A crash on a task-pool thread that does not reach the main thread ending the editor; the editor keeps running and the report is announced at the next frame.

## Further Notes

- The crash handler is installed process-wide, so the test that installs it is the only one in its process that may; the other tests of the Utility do not panic.
- Rolling at midnight itself is the appender's behaviour and is not driven by a test, since the appender exposes no clock; One file per day is checked through the file's name and A week is kept through pruning at start.
- Nothing private beyond paths is stated as what the report does not hold; the message a panic carries may itself name a path, which the Author is told the report may contain.
