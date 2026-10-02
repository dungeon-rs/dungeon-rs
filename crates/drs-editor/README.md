# drs-editor

The Client: the egui interface through which the Author works.

The [`EditorPlugin`](crate::EditorPlugin), built over what the diagnostics
Utility set up and found at start, lays the window out with `egui_dock`:
an Assets panel on the left that shows the Assets of every added Asset Folder
as a grid of thumbnails with their names beneath, ordered by the Canonical Name
of their folder, compared as the library Manager's search compares it, and then
by place, or, with text typed in its search field, the
matches the library Manager answers for the text, in its order; above the grid
a line says how many Assets the library holds or how many match, or that none
matches the text, and one line per folder how many of its Assets are shown. Each
change of the text is sent as it is typed and starts the grid at the top. The
viewport sits in the centre,
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
Level… asks for a resolution in pixels per cell, with presets and a typed value
that is refused in words while it lies outside the limits, shows the image size
that results and how many placeholders would be exported as shown, then the
platform's save dialog proposing `<Project> - <Level>.png`; while the Export is
written the viewport, Undo, Redo, and another Export wait for it. Library → Add
Asset Folder… opens the platform's folder dialog and then asks for the Canonical
Name. Edit → Undo and Redo are offered while no drag, Wall or stroke being
drawn, option held while it changes, or Export is under way. Help → Show Logs
opens the log directory in the platform's file manager.

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
selection, the search text, the prompt, question, report, or dialog in progress,
whether an Export is being written, the tool, the Wall or the stroke being
drawn, the option being changed, and the Brush.
Clicking in the viewport places the chosen Asset or selects the topmost Element
under the pointer, a Prop by its rectangle, a Portal by its turned rectangle,
and a Wall by its line outside the stretches its Portals cover, never a
Terrain, dragging a
selected Element moves it as one gesture, Delete removes it, Escape stops
placing, and the platform's usual shortcuts undo and redo. Scrolling pans, a
wheel or a pinch zooms, and the middle button or Space with the left button
drags the view.

A tool strip over the viewport offers Select, Wall, and Portal, and the
thickness and colour of the selected Wall, or of the next Wall while none is
selected. With
the Wall tool, chosen there or with `W`, each click adds a point of a Wall
previewed with a rubber band to the pointer, and Enter or a double-click
finishes it as one Place Element; choosing an Asset leaves the tool, and
choosing the tool drops the chosen Asset and the selection. The selected Wall
shows a handle at each point, at each control point with guide lines, and at the
middle of each straight segment: dragging one moves the point or bends the
segment as one gesture, a double-click on the line adds a point there, and
Delete removes the selected point or straightens the selected control point's
segment.

With the Portal tool, chosen in the strip or with `P`, the chosen Asset is the
Portal's image, kept when the tool is chosen; a marker across the nearest Wall
within half a cell or half its thickness of the pointer shows where the Portal
will sit and which side it will face, and a click places it set into that Wall,
or freestanding where no Wall is in reach. Choosing the tool leaves the Wall
tool and drops the selection; Escape goes back to Select. A selected Portal set
into a Wall slides along it when dragged, as one gesture; `X` flips its side,
or a freestanding one's mirroring; `F` frees it where it stands, or sets a
freestanding one into the nearest Wall within reach of its centre; both wait,
as undo does, while a step is being made. Where two Walls are equally near, the
marker, a click, and `F` all take the topmost. The strip shows a selected
Portal's width, its rotation in degrees while freestanding, a Flip button, and a
Free Portal or Set into Wall button. When a Wall edit removes Portals, the
status line says how many.

The tool strip also offers Paint, chosen there or with `B`, which leaves the
Wall tool and drops the selection but keeps a chosen Asset as the image the
Brush paints with. A circle as large as the Brush follows the pointer; a press
starts a stroke, moving adds the pointer to its path whenever it is more than an
eighth of the Brush's size from the last point, the stroke is shown as a
translucent band as wide as the Brush, and the release sends one Paint onto the
current Layer with the path and the Brush's settings, naming the chosen Asset
or, with none, no image; with no Asset chosen and no Terrain on the Layer, a
press paints nothing and the status line asks for an Asset. The options show
the Brush's size in cells, its hardness and strength as percentages, starting at
two cells, 50 %, and 100 %, and the image it paints with; when an Asset is chosen
and the Layer's Terrain shows another image, a button sends the Edit Element that
makes the Terrain show it. The Brush is the Editor's own, never a history step
and never saved. Escape and choosing the Wall tool discard a stroke being
drawn.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development. With `DRS_SCREENSHOT` set to a file path, a
  screenshot of the window is saved there a moment after start. With `DRS_SCRIPT` set
  to a file, the editor is driven by its steps, one per frame (`wait`, `move`, `down`,
  `up`, `click`, `drag`, `key`, `hold`, `release`, `text`, `scroll`, `pinch`,
  `screenshot`, `describe`, which also logs the Wall tool, every Wall and the
  stretches it gives way along, and every Portal, `close`,
  `quit`), fed in as the messages the window would send so egui and the viewport see
  them alike; `describe` logs every clickable widget and every cell of the grid with
  its rectangle.
  With `DRS_PICK_FOLDER` set, Add Asset Folder… takes that folder instead of opening
  the dialog; with `DRS_PICK_FILE` set, Open… takes that file; with `DRS_SAVE_FILE`
  set, Save As… and Export Level… write to that path (the extension is added when it
  lacks one). An empty value stands for a cancelled dialog. With `DRS_CRASH_TEST` set
  to `main`, `thread`, or `startup`, the editor panics on purpose on the main thread
  on its second frame, on a spawned thread on its second frame, or while its plugins
  build before any window exists, so the crash handler can be seen at work.
