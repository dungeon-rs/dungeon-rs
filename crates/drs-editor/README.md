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
Name. Edit → Undo and Redo are offered while no drag, Wall, Room, or stroke
being drawn, option held while it changes, or Export is under way. Help → Show
Logs opens the log directory in the platform's file manager.

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
whether an Export is being written, the tool, the Wall, the Room outline, or the
stroke being drawn, the option being changed, the Brush, the selected stroke,
and the Snap switch. It writes the model's `Pointer` each frame: the Level being
worked on, where the pointer is, a reach of eight pixels, and what snaps.
Clicking in the viewport places the chosen Asset or selects the topmost Element
under the pointer, a Prop by its rectangle, a Portal by its turned rectangle,
a Wall by its line outside the stretches its Portals cover, and a Room by its
floor or by its Walls outside those stretches, never a Terrain, dragging a
selected Element moves it as one gesture, Delete removes it, Escape stops
placing, and the platform's usual shortcuts undo and redo. Scrolling pans, a
wheel or a pinch zooms, and the middle button or Space with the left button
drags the view.

A tool strip over the viewport offers Select, Wall, Portal, and Room, a Snap
switch after the tools, and the thickness and colour of the selected Wall, or of
the next Wall while none is selected. With the Wall tool, chosen there or with
`W`, each click adds a point of a Wall previewed with a rubber band to the
pointer, and Enter or a double-click finishes it as one Place Element; choosing
an Asset leaves the tool, and choosing the tool drops the chosen Asset and the
selection. The selected Wall shows a handle at each point, at each control point
with guide lines, and at the middle of each straight segment: dragging one moves
the point or bends the segment as one gesture, a double-click on the line adds a
point there, and Delete removes the selected point or straightens the selected
control point's segment.

The viewport draws the Grid as thin, faint lines along the edges of the cells in
view, over the Elements, while a cell is at least eight pixels across; it is the
Editor's own overlay, so no Export holds it. With snapping, a point a click of
the Wall or the Room tool adds, both corners of a Room rectangle, and a point of a
selected Wall or Room being dragged go where the authoring Manager's snapped point
puts the pointer: on the nearest point of a Wall or Room of the Level within
reach, or else on the nearest Grid corner. A small ring shows where the next click
of either tool lands, filled when it lies on another Element's point, and the
rubber bands end at it; the point placed is the one shown, the snapped point
derived from the Pointer written the frame before. A whole Wall or Room dragged
moves by whole cells, each step sending the amount moved since the last. Control
points, middles, the point a double-click adds, Props, and Portals never snap.
The Snap switch, on at start and never saved, turns snapping off, and holding
Alt, Option on macOS, places and drags freely while it is held, a drag included.
Nothing else drops snapping from a drag: carried over a panel or out of the
window, it keeps snapping until the button is released.

With the Room tool, chosen in the strip or with `R`, each click adds a point of
an outline previewed with rubber bands from the last point to the pointer and
from the pointer back to the first; a click on the first point or Enter closes
it as one Place Element once it has three points, and with no point placed a
drag draws a rectangle, placed on release unless it is only a few pixels wide or
high. The strip then shows the wall thickness, the wall colour, and the floor
colour of the next Room, or of the selected Room, a change to it sent as one
Edit Element. Choosing the tool drops the chosen Asset and the selection and
leaves the Wall, the Portal, or the Paint tool, discarding a stroke being drawn;
choosing an Asset or another tool leaves it, discarding the outline. A selected Room has a Wall's handles round its closed
outline, the edge from the last point to the first included, and moves whole
when dragged by its floor or its Walls.

With the Portal tool, chosen in the strip or with `P`, the chosen Asset is the
Portal's image, kept when the tool is chosen; a marker across the nearest Wall
or Room's Walls within half a cell or half its thickness of the pointer shows
where the Portal will sit and which side it will face, and a click places it set
into that line, or freestanding where none is in reach. Choosing the tool leaves
the Wall and the Room tool and drops the selection; Escape goes back to Select. A
selected Portal set into a Wall or a Room slides along its line when dragged, as
one gesture, round a Room past its first point; `X` flips its side, or the
mirroring of a freestanding Portal or of one whose Wall is gone, which is also
dragged and turned as a freestanding one; `F` frees it where it stands, or sets
a freestanding one into the nearest Wall or Room within reach of its centre; both wait,
as undo does, while a step is being made. Where two Walls are equally near, the
marker, a click, and `F` all take the topmost. The strip shows a selected
Portal's width, its rotation in degrees while freestanding, a Flip button, and a
Free Portal or Set into Wall button. When a Wall edit removes Portals, the
status line says how many, and so does a Room edit.

The tool strip also offers Paint, chosen there or with `B` painting and with `E`
erasing, which leaves the Wall or the Room tool, discarding what is being drawn,
and drops the selection but keeps a chosen Asset as the image the Brush paints
with. Its options choose Paint, Erase, or Edit strokes. Painting or erasing, a
circle as large as the Brush follows the pointer; a press starts a stroke that
paints or erases as the tool did at the press, moving adds the pointer to its
path whenever it is more than an eighth of the Brush's size from the last point,
the stroke is shown as a translucent band as wide as the Brush, both in a warm
red while erasing, and the release sends one Paint onto the current Layer with
the path, the Brush's settings, and whether it erases, a stroke that paints
naming the chosen Asset or, with none, no image, and an erase naming none; with
no Asset chosen and no Terrain on the Layer a press paints nothing and the
status line asks for an Asset, and with no Terrain a press erases nothing and
the status line says there is nothing to erase. Editing strokes, a click picks
the latest laid stroke of the current Layer's Terrain within its radius or four
screen pixels of the pointer, shown as a band as wide as its Brush with its path
and a handle at each point; dragging a handle moves that point and dragging the
stroke moves it whole, each as one gesture, and Delete removes it. The options
show the Brush's size in cells, its hardness and strength as percentages,
starting at two cells, 50 %, and 100 %, or the selected stroke's, with whether
it paints or erases, a change to a stroke sent as an Edit Element, a held drag
as one gesture; and the image it paints with, with, when an Asset is chosen and
the Layer's Terrain shows another image, a button that sends the Edit Element
making the Terrain show it. The Brush, the mode, and the selected stroke are the
Editor's own, never a history step and never saved; the selected stroke is let
go on undo and redo, on leaving the tool or switching to painting or erasing,
and when its Terrain no longer has it. Escape and choosing the Wall or the Room
tool discard a stroke being drawn.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development. With `DRS_SCREENSHOT` set to a file path, a
  screenshot of the window is saved there a moment after start. With `DRS_SCRIPT` set
  to a file, the editor is driven by its steps, one per frame (`wait`, `move`, `down`,
  `up`, `click`, `drag`, `key`, `hold`, `release`, `text`, `scroll`, `pinch`,
  `screenshot`, `describe`, which also logs the Wall and Room tools, every Wall
  and Room and the stretches it gives way along, every Portal, every Terrain with
  its strokes, the band its coverage is shown at, and how many tiles it holds at
  the base and at that band, the Snap switch, the Pointer, and the snapped point,
  `close`,
  `quit`), fed in as the messages the window would send so egui and the viewport see
  them alike; `describe` logs every clickable widget and every cell of the grid with
  its rectangle.
  With `DRS_PICK_FOLDER` set, Add Asset Folder… takes that folder instead of opening
  the dialog; with `DRS_PICK_FILE` set, Open… takes that file; with `DRS_SAVE_FILE`
  set, Save As… writes to that path (the extension is added when it lacks one), and so
  does Export Level… while `DRS_EXPORT_FILE` is unset. With `DRS_EXPORT_FILE` set,
  Export Level… writes to that path instead, so one script can both save and export.
  An empty value stands for a cancelled dialog.
  With `DRS_CRASH_TEST` set to `main`, `thread`, or `startup`, the editor panics on
  purpose on the main thread on its second frame, on a spawned thread on its second
  frame, or while its plugins build before any window exists, so the crash handler can
  be seen at work.
