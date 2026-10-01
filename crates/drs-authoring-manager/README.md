# drs-authoring-manager

The Manager that applies the Author's Commands to the Level: Place Element, Edit Element, and
Remove Element, each recorded in the history so that it can be undone and redone.

The Editor sends it `Apply`, `Undo`, and `Redo` messages; a Command that cannot be carried out is
answered with a `CommandFailed` message that says why, and an undo or redo that cannot be with a
`HistoryFailed`.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
