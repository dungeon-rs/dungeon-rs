# drs-editor

The Client: the egui interface through which the Author works.

The [`EditorPlugin`](crate::EditorPlugin) lays the window out with `egui_dock`:
an Assets panel on the left that lists the Assets of every added Asset Folder
under its Canonical Name, filtered by name, and the viewport in the centre,
which is left transparent so the Level drawn by the render Engine shows
through. A menu bar offers Library → Add Asset Folder…, which opens the
platform's folder dialog and then asks for the Canonical Name, Edit → Undo
and Redo, and Help → Show Logs, which opens the log directory in the
platform's file manager; a status line at the bottom reports what happened
last and what the Author is doing, including a resource directory that was
not found and a crash report another thread left, which is also announced
in the crash dialog on the main thread.

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
- `dev`: debug tooling for development. With `DRS_SCREENSHOT` set to a file path, a
  screenshot of the window is saved there a moment after start. With `DRS_SCRIPT` set
  to a file, the editor is driven by its steps, one per frame (`wait`, `move`, `down`,
  `up`, `click`, `drag`, `key`, `hold`, `release`, `text`, `scroll`, `pinch`,
  `screenshot`, `quit`), fed
  in as the messages the window would send so egui and the viewport see them alike.
  With `DRS_PICK_FOLDER` set, Add Asset Folder… takes that folder instead of opening
  the dialog; an empty value stands for a cancelled dialog. With `DRS_CRASH_TEST` set
  to `main`, `thread`, or `startup`, the editor panics on purpose on the main thread
  on its second frame, on a spawned thread on its second frame, or while its plugins
  build before any window exists, so the crash handler can be seen at work.
