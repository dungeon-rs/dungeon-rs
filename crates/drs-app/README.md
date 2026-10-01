# drs-app

The Host: registers the plugins of every other crate and starts the editor.
It holds no logic: `main` registers the `lib://` asset source before Bevy's
asset plugin builds, sets the window title, and adds the model, the history,
the library access, the library Manager, the project Manager, the authoring
Manager, the render Engine, and the Editor.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development. With `DRS_DIRECTORIES` set to a directory, the
  editor keeps its Manifests and index caches under it instead of the platform's
  configuration and cache directories.
