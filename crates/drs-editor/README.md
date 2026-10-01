# drs-editor

The Client: the egui interface through which the Author works.

The [`EditorPlugin`](crate::EditorPlugin) lays the window out with `egui_dock`:
an Assets panel on the left that lists the Assets of every added Asset Folder
under its Canonical Name, filtered by name, and the viewport in the centre,
which is left transparent so the Level drawn by the render Engine shows
through. A menu bar offers Library → Add Asset Folder…, which opens the
platform's folder dialog and then asks for the Canonical Name, and Edit →
Undo and Redo; a status line at the bottom reports what happened last and
what the Author is doing.

The panels read the World and send Commands as messages; they never own
domain state. The Editor writes only the model's `Viewport` (panning and
zooming) and its own state: the chosen Asset, the selection, the filter, and
the prompt in progress. Clicking in the viewport places the chosen Asset or
selects the topmost Prop under the pointer, dragging a selected Prop moves it
as one gesture, Delete removes it, Escape stops placing, and the platform's
usual shortcuts undo and redo. Scrolling pans, a wheel or a pinch zooms, and
the middle button or Space with the left button drags the view.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
