# drs-app

The Host: registers the plugins of every other crate and starts the editor.
It holds no logic: `main` installs the crash handler before anything else,
starts logging to the daily file and hands its layer to Bevy's log plugin,
locates the Bundled Files and roots the default asset source there,
registers the `lib://` asset source before Bevy's asset plugin builds, sets
the window title, adds the model, the history, the library access, the
library Manager, the project Manager, the authoring Manager, the render
Engine, and the Editor, and runs the App under the crash handler's guard.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development. With `DRS_DIRECTORIES` set to a directory, the
  editor keeps its Manifests, index caches, logs, and crash reports under it instead
  of the platform's configuration and cache directories.
