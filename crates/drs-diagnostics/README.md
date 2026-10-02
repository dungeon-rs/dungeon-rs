# drs-diagnostics

The Utility that keeps the editor's own files for diagnosis and brings a
fault to the Author: logging to daily files in the editor's directories, a
crash handler that leaves a report and announces it, and the location of
the editor's Bundled Files on each platform. It uses no Bevy: the Host
wires it before the App exists, and the Editor calls it to reveal the logs
and to announce a pending crash report.

## Contract

- **`StartLogging`**: [`start_logging`](crate::start_logging) creates the log
  directory ([`log_directory`](crate::log_directory) names it: the one the
  editor's directories resolved to, or `logs` under the temporary directory
  when they resolved to none), builds a layer that appends to
  `dungeon-rs.<UTC date>.log`, keeps the last seven daily files, and writes
  each entry as it happens. The Host hands the layer to Bevy's log plugin
  through [`take_layer`](crate::take_layer), and the entries carry their UTC
  timestamp, level, and module. The level comes from `RUST_LOG` through
  [`log_directives`](crate::log_directives): the default,
  [`DEFAULT_LEVEL`](crate::DEFAULT_LEVEL), with the variable's directives on
  top, and the default alone, said on the terminal, when the variable is
  malformed. When the directory or the file cannot be created, the reason is
  printed and the editor logs to the terminal only; what came out is a
  [`Logging`](crate::Logging).
- **`InstallCrashHandler`**:
  [`install_crash_handler`](crate::install_crash_handler), installed on the
  main thread before Bevy's plugins, with the editor's name and version as a
  [`Product`](crate::Product). A panic on any thread writes
  `crash-<UTC timestamp>.txt` in the log directory, or in the platform's
  temporary directory when the log directory cannot be written, holding
  under its own heading the UTC time, the editor's version, the operating
  system and architecture, the message, the source location, the thread, a
  backtrace, and the current log file, and nothing else about the Author.
  The path is printed to the terminal and logged at `error` before any
  dialog. On the main thread the dialog is shown at once; on another thread
  the report waits for [`announce_pending`](crate::announce_pending), which
  the Editor calls each frame to show the dialog and get the report for the
  status line, and [`run_guarded`](crate::run_guarded) announces what is
  still pending when the editor ends. With `dialogs` off nothing is shown, for tests and
  headless runs; [`dialogs_possible`](crate::dialogs_possible) tells the Host
  when that is: never on a continuous-integration run (`CI` set), when
  `DRS_NO_DIALOGS` is set, on Linux without a display, or, in development,
  while `DRS_SCRIPT` drives the editor, since nobody is there to click and a
  dialog would hold the run.
- **`LocateBundledFiles`**:
  [`locate_bundled_files_of_this_executable`](crate::locate_bundled_files_of_this_executable)
  finds the bundle directory of the running editor, through
  [`locate_bundled_files`](crate::locate_bundled_files), which looks from a
  given executable's location: `bundle` beside
  it, `Resources` beside its directory (the macOS application), or `bundle`
  in each ancestor of its directory, nearest first (the workspace). Only a
  directory holding the marker [`BUNDLE_MARKER`](crate::BUNDLE_MARKER),
  which names the editor's version, counts. None found is a
  [`BundledFilesNotFound`](crate::BundledFilesNotFound) naming every location
  tried. [`BundledFiles::file`](crate::BundledFiles::file) gives one Bundled
  File by a plain relative name and refuses a name that is absolute, holds
  `..`, or carries a source prefix.
- **`RevealLogs`**: [`reveal_logs`](crate::reveal_logs) opens the log
  directory in the platform's file manager, creating it first.

[`log_start`](crate::log_start) logs, once the subscriber is installed, the
log file's path and what was found about the Bundled Files, and yields a
[`Started`](crate::Started): the log directory and file and the bundle
directory, or the locations tried, which the Host hands to the Editor's
plugin, and whose `asset_root` is where the default asset source is rooted:
the bundle directory, or the first location tried, which holds nothing the
engine should read.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
