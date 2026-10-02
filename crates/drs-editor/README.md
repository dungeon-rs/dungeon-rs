# drs-editor

The Client: the egui interface through which the Author works.

The [`EditorPlugin`](crate::EditorPlugin), built over what the diagnostics
Utility set up and found at start, lays the window out with `egui_dock`:
an Assets panel on the left that shows the Assets of every added Asset Folder
as a grid of thumbnails with their names beneath, ordered by the Canonical Name
of their folder and then by place, filtered by name, with one line per folder
above the grid giving how many of its Assets are shown, and the viewport in the
centre,
which is left transparent so the Level drawn by the render Engine shows
through. A status line at the bottom reports what happened last and what the
Author is doing, including a bundle directory that was not found and a crash
report another thread left, which is also announced in the crash dialog on the
main thread. The window itself is described by
[`window_plugin`](crate::window_plugin), which the Host sets on Bevy's default
plugins: its title is `<Project> — DungeonRS`, with `• ` in front while the
Project has unsaved changes, and its close button is answered by the Editor
rather than closing the window, so the unsaved-changes question is asked first.

The menu bar offers File, Library, Edit, and Help. File → Open…, Save, Save As…,
Export Level…, and Quit, each with the platform's usual shortcut, send the
project Manager its requests: Save goes to the Project's file or, while it has
none, becomes Save As…, whose dialog proposes the Project's name; Open… and Quit
first ask whether to save, discard, or cancel while there are unsaved changes,
and a save that is refused keeps the question open with the reason. Opening a
Project that cannot be shown in full lists each Missing Asset and unknown
Element kind in a report the Author dismisses; the placeholders stay. Export
Level… asks for a resolution in pixels per cell, with presets and a typed
value that is refused in words while it lies outside the limits, shows the
image size that results and how many placeholders would be exported as shown,
then the platform's save dialog proposing `<Project> - <Level>.png`; while the
Export is written the viewport, Undo, Redo, and another Export wait for it.
Library → Add Asset Folder… opens the platform's folder dialog and then asks
for the Canonical Name. Edit → Undo and Redo are offered while no drag, Wall
being drawn, or Export is under way. Help → Show Logs opens the log directory in the
platform's file manager.

The grid lays out only the rows in view, as many 128-point cells as the panel's
width holds. Each cell shows a neutral square until its thumbnail is generated,
a placeholder of the thumbnail's proportions until it is decoded, then the
thumbnail at one physical pixel of the display per pixel, never enlarged, or a
crossed-out square for a file that is not an image; hovering shows the Asset's
name, its folder's Canonical Name, and its place, and a click chooses it for
placing. The Assets of the rows laid out and two rows either side are named to
the library Manager with Browse whenever they change, so they are generated
first; thumbnails are loaded through the `thumb://` asset source, so the asset
system decodes them off the main thread, the rows either side are loaded ahead,
a thumbnail is registered with egui only while its row is laid out, so none
while the panel is behind another tab, and at most 512 decoded thumbnails are
kept, the least recently shown dropped first.

The panels read the World and send Commands and requests as messages; they
never own domain state. The Editor writes only the model's `Viewport` (panning
and zooming) and its own state: the chosen Asset, the thumbnails it holds, the
selection, the filter, the prompt, question, report, or dialog in progress,
whether an Export is being written, the tool, and the Wall being drawn.
Clicking in the viewport places the chosen Asset or selects the topmost Element
under the pointer, a Prop by its rectangle and a Wall by its line, dragging a
selected Element moves it as one gesture, Delete removes it, Escape stops
placing, and the platform's usual shortcuts undo and redo. Scrolling pans, a
wheel or a pinch zooms, and the middle button or Space with the left button
drags the view.

A tool strip over the viewport offers Select and Wall, and the thickness and
colour of the selected Wall, or of the next Wall while none is selected. With
the Wall tool, chosen there or with `W`, each click adds a point of a Wall
previewed with a rubber band to the pointer, and Enter or a double-click
finishes it as one Place Element; choosing an Asset leaves the tool, and
choosing the tool drops the chosen Asset and the selection. The selected Wall
shows a handle at each point, at each control point with guide lines, and at the
middle of each straight segment: dragging one moves the point or bends the
segment as one gesture, a double-click on the line adds a point there, and
Delete removes the selected point or straightens the selected control point's
segment.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development. With `DRS_SCREENSHOT` set to a file path, a
  screenshot of the window is saved there a moment after start. With `DRS_SCRIPT` set
  to a file, the editor is driven by its steps, one per frame (`wait`, `move`, `down`,
  `up`, `click`, `drag`, `key`, `hold`, `release`, `text`, `scroll`, `pinch`,
  `screenshot`, `describe`, which also logs the Wall tool and every Wall, `close`,
  `quit`), fed
  in as the messages the window would send so egui and the viewport see them alike;
  `describe` logs every clickable widget and every cell of the grid with its rectangle.
  With `DRS_PICK_FOLDER` set, Add Asset Folder… takes that folder instead of opening
  the dialog; with `DRS_PICK_FILE` set, Open… takes that file; with `DRS_SAVE_FILE`
  set, Save As… and Export Level… write to that path (the extension is added when it
  lacks one). An empty value stands for a cancelled dialog. With `DRS_CRASH_TEST` set
  to `main`, `thread`, or `startup`, the editor panics on purpose on the main thread
  on its second frame, on a spawned thread on its second frame, or while its plugins
  build before any window exists, so the crash handler can be seen at work.
