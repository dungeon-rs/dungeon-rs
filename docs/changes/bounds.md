# Bounds

**Capabilities**:
- levels-and-layers: Resize Bounds
- export: Export Level (the image follows the Bounds as they were resized, and an image too large to write is refused)
- composing: none of its Commands; Resize Bounds joins the one history, and the Bounds tool joins the tools
- projects: none of its Commands; resized Bounds are saved and opened, and Bounds of no cell are a bad file

## Problem Statement

Every Project exports thirty by thirty cells from the Level's origin, and the Author can neither see where that square lies nor change it. A dungeon drawn east of the origin is cut in half in the Export, a small encounter map comes out mostly black, and the only way to find out is to export and look. The functional baseline sets the map's size in a dialog and shows its edge while composing; editors of every kind let the author drag that edge into place.

## Solution

The Bounds are always shown in the viewport as an outline, at every zoom and whatever tool is chosen, so the Author sees at a glance what the Export will hold; nothing outside them is dimmed, because composing beyond the Bounds is as ordinary as composing inside them. A Bounds tool, chosen from the tool strip or with `O`, shows handles at the Bounds' corners and at the middles of their edges: dragging an edge, or a handle on it, moves that edge, and dragging a corner moves its two edges, always by whole cells, never closer than one cell to the edge opposite, and as one undo step. With the Bounds tool chosen, the options strip shows the Bounds' left edge, bottom edge, width, and height in cells, each of which the Author can type or drag. A Bounds of less than one cell or more than a thousand cells on a side, or reaching farther than ten thousand cells from the Level's origin, is refused with the limits named. Resizing never touches an Element: what falls outside the Bounds stays where it is, and comes back into the Export when the Bounds grow over it again. The Export follows the Bounds as they stand, and refuses before writing anything an image more than a hundred thousand pixels on a side. Saving keeps the Bounds, as it always has, and a new Project still starts with thirty by thirty cells from the origin.

## User Stories

### Seeing the Bounds

1. As an Author, I can see the Bounds outlined in the viewport whatever tool I have chosen, so that I always know what the Export will hold.
2. As an Author, I can see the outline at the same thickness at every zoom, however far I zoom out, so that the Bounds of a large map stay visible when the Grid's lines are gone.
3. As an Author, I can see the outline over every Element and over the Grid's lines, so that a floor or a Wall along the edge never hides where the Bounds run.
4. As an Author, I can see the outline over light and dark ground alike, so that it never disappears on a pale floor or in a dark cave.
5. As an Author, I can see Elements outside the Bounds exactly as I see those inside, not dimmed, so that composing beyond the Bounds is as easy as composing inside them.
6. As an Author, I can rely on the outline never appearing in an Export, so that the Bounds help me compose and never spoil the map.

### The Bounds tool

7. As an Author, I can choose the Bounds tool from the tool strip or with `O`, so that resizing the Bounds is one key away.
8. As an Author, I can see handles at the four corners of the Bounds and at the middle of each edge while the Bounds tool is chosen, so that I know where to grab them.
9. As an Author, I can see the pointer change to a resizing pointer over an edge or a corner, so that I know a drag there resizes the Bounds.
10. As an Author, I can read in the status line what a drag does while the Bounds tool is chosen, so that the tool explains itself.
11. As an Author, I can rely on the Bounds tool selecting, placing, and painting nothing, so that I never change an Element while I resize the Bounds.
12. As an Author, I can rely on choosing the Bounds tool dropping the chosen Asset and the selection, and leaving the tool I was in with whatever I was drawing discarded, so that only one thing is ever under the pointer.
13. As an Author, I can press Escape, choose another tool, or choose an Asset to leave the Bounds tool, so that I go back to composing the way I leave any tool.
14. As an Author, I can pan and zoom while the Bounds tool is chosen, so that I can reach an edge that lies out of view.

### Dragging the Bounds

15. As an Author, I can drag an edge, or the handle at its middle, to move that edge alone, so that I can grow or shrink the Bounds on any side.
16. As an Author, I can drag a corner to move its two edges at once, so that I can grow the Bounds towards a corner in one gesture.
17. As an Author, I can see the Bounds follow the pointer by whole cells while I drag, so that I see what I will get before I let go.
18. As an Author, I can rely on the Bounds moving by whole cells whether or not snapping is on and whether or not Alt is held, so that the Bounds always line up with the Grid.
19. As an Author, I can drag an edge towards the edge opposite and see it stop one cell short of it, so that the Bounds never turn inside out or vanish.
20. As an Author, I can drag an edge outwards and see it stop once the Bounds are a thousand cells across or ten thousand cells from the origin, so that a drag never asks for Bounds the editor refuses.
21. As an Author, I can undo a whole drag of the Bounds in one step, however long it was, so that a resize is as easy to take back as to make.
22. As an Author, I can let go of a drag over a panel or outside the window and have it end where it was last shown, so that a sweeping drag never leaves the Bounds somewhere I did not see.
23. As an Author, I can drag an edge and let go where it started and find nothing added to the history, so that a hesitant press costs nothing.

### Typing the Bounds

24. As an Author, I can read the Bounds' left edge, bottom edge, width, and height in cells in the options strip while the Bounds tool is chosen, so that I know their exact size and place.
25. As an Author, I can type a width or a height and have the Bounds grow or shrink to it from their left or bottom edge, so that a map of an exact size is one entry.
26. As an Author, I can type a left or a bottom edge and have the Bounds move there whole, keeping their size, so that I can put the map exactly where my drawing lies.
27. As an Author, I can drag one of the fields and see the Bounds change by whole cells as one undo step, so that I can nudge them without typing.
28. As an Author, I am told in the status line why a value I typed was refused, with the limits named, and see the field show the Bounds as they are, so that a mistyped size costs a retry and nothing else.
29. As an Author, I can see the fields follow an undo, a redo, or a drag of the Bounds, so that what I read is always what the Bounds are.

### Resizing and the Project

30. As an Author, I can shrink the Bounds over Elements and find every Element still where it was, so that trying a smaller map never costs me work.
31. As an Author, I can grow the Bounds back over Elements left outside and see them in the next Export, so that the Bounds are only a frame.
32. As an Author, I can undo and redo every resize in the same history as everything else I do, in the order I did it, so that the Bounds are never a special case.
33. As an Author, I can rely on a resize giving the Project unsaved changes, and undoing back to where I saved taking them away again, so that the unsaved marker tells the truth.
34. As an Author, I can save a Project with resized Bounds and open it again, here or on another device, and find the same Bounds, so that my collaborator exports exactly what I export.
35. As an Author, I can open a Project whose Bounds are larger than this editor lets me make them and save it again with the Bounds as they were, so that an Author with a later editor never loses their map size to me.
36. As an Author, I am refused a file whose Bounds have no width or no height, with the reason, so that a damaged file never leaves me with nothing to export.
37. As an Author, I can start a new Project and find the Bounds thirty by thirty cells from the Level's origin, as before, so that the common case needs no setup.

### Exporting what the Bounds hold

38. As an Author, I can export after resizing and get an image exactly as large as the resized Bounds at the resolution I chose, showing exactly what lies inside them, so that the Export is the frame I drew.
39. As an Author, I can read the resized Bounds and the image size they give in the Export dialog, so that I know what I am about to produce.
40. As an Author, I am refused an Export whose image would be more than a hundred thousand pixels on a side, before anything is written, with the limit and the largest resolution my Bounds allow named, so that I never wait for an image that cannot be written.
41. As an Author, I can rely on the Bounds tool's handles never appearing in an Export, whatever tool is chosen while I export, so that I need not leave the tool to export.
42. As an Author, I can rely on the Bounds tool, its handles, and its fields doing nothing while an Export runs, so that the Bounds never change under an image being written.

## Rules

### Resize Bounds (levels-and-layers capability)

**Resized to the Bounds given**: a Resize Bounds sets the Project's Bounds to the lower-left corner, width, and height it carries, in whole cells, as one history step that undo returns to the Bounds before it and redo sets again. Follows from: Every Command can be undone.

**A resize gesture is one step**: Resize Bounds sent as a gesture, from its beginning through every continuation to its end, is one history step, and undo returns the Bounds to where the gesture began. Follows from: Every Command can be undone.

**The same Bounds record nothing**: a Resize Bounds to the Bounds as they are changes nothing and records no history step, and a gesture none of whose Resize Bounds changes the Bounds records none.

**Elements stay**: a Resize Bounds, its undo, and its redo change no Element on any Layer, inside the Bounds, across their edge, or outside them: each keeps its ElementId, its place in the stacking order, every property, and its derived shape and coverage. Follows from: Bounds only decide what is exported.

**At least a cell, at most a thousand**: a Resize Bounds whose width or height is below 1 or above 1,000 cells is refused with the limits named, changes nothing, and records no history step.
_Why_: Bounds of no cell export nothing, and a thousand cells is far beyond any battle map while keeping the Export's image within reach at a useful resolution.

**Within reach of the origin**: a Resize Bounds any of whose edges lies more than 10,000 cells from the Level's origin, to the left, the right, below, or above, is refused with the limit named, changes nothing, and records no history step.
_Why_: far from the origin a position in cells loses the precision the Export draws at.

**Opened Bounds are kept**: a Project opened with Bounds wider or higher than 1,000 cells, or reaching more than 10,000 cells from the Level's origin, keeps them as they are, and saving writes them unchanged; only a Resize Bounds is held to the limits.

### The Bounds in the editor (levels-and-layers capability)

**The Bounds are shown**: the viewport always draws the Bounds' four edges as an outline of the same thickness on screen at every zoom, over every Element and over the Grid's lines, whatever tool is chosen, and leaves everything outside the Bounds drawn as it is inside them; it draws no outline while an Export runs.

**The Bounds tool**: the tool strip offers the Bounds tool, also chosen with `O`; choosing it drops the chosen Asset and the selection and leaves the Wall, the Room, the Portal, or the Paint tool, discarding a Wall, an outline, or a stroke being drawn; while it is chosen a click or a press on the Level selects, places, paints, and removes nothing, the status line says that dragging an edge or a corner resizes the Bounds, and Escape, choosing another tool, or choosing an Asset leaves it, the Asset then being chosen for placing.

**Handles of the Bounds**: with the Bounds tool chosen, the Bounds show a handle at each corner and at the middle of each edge, and the pointer shows a resizing pointer wherever a press would drag the Bounds.

**Dragging the Bounds**: with the Bounds tool chosen, a press on a corner's handle drags the corner's two edges, and a press on an edge's handle or within a handle's reach of an edge's line drags that edge alone, a corner winning over an edge; from the press to the release each dragged edge lies where it was at the press moved by the pointer's travel across it, rounded to whole cells, a travel exactly halfway rounded away from zero, and every edge not dragged stays where it was; the drag sends Resize Bounds as one gesture, which ends where it was last shown when released over a panel or outside the window.

**Whole cells whatever**: a drag of the Bounds moves its edges by whole cells whether the Snap switch is on or off and whether or not Alt is held.

**A drag stops at the limits**: during a drag each dragged edge stops one cell from the edge opposite it, a thousand cells from that edge, and 10,000 cells from the Level's origin, so a drag never sends a Resize Bounds that would be refused; Bounds opened beyond the limits are brought within them by the first step of a drag.

**The Bounds fields**: with the Bounds tool chosen, the options strip shows the Bounds' left edge, bottom edge, width, and height as whole numbers of cells, as they are after every Resize Bounds, undo, and redo; a value typed into one is sent, as the field lets go of the keyboard, as one Resize Bounds keeping the other three, so a width or a height keeps the left or the bottom edge where it is and a left or a bottom edge moves the Bounds whole; dragging a field changes it by whole cells and sends Resize Bounds as one gesture that ends when the field is let go; a refused value leaves the Bounds as they were, the field showing them, and the reason in the status line.

**Undo waits for the Bounds**: undo and redo, from the keys or the menu, wait from a press that drags the Bounds to its release, and while a field of the Bounds is held, whether or not anything has changed yet.

**The Bounds wait for the Export**: while an Export runs the Bounds tool cannot be chosen and its handles, drags, and fields do nothing.

### Export (export capability)

**Image within limits**: an Export whose image would be more than 100,000 pixels wide or high, the Bounds' width or height in cells times the resolution, is refused before anything is written, with the limit and the largest resolution the Bounds allow named, or, when no resolution fits, with the Bounds named as too large to export.
_Why_: the Export holds one band of tiles as wide as the image in memory, 410 MB at 100,000 pixels.

## Changes to existing behaviour

The composing Rules named here are those composing holds once Layers has landed, which comes before this change on the roadmap; the clauses below are added to what those Rules then say.

- export — **Exactly the Bounds**: modified to "the Export is as many pixels wide as the Bounds' width in cells times the resolution, and as many high as the height times the resolution, and shows exactly the Bounds as they stand when the Export Level is handled, however they were resized before it; a Resize Bounds handled while the Export runs changes nothing of it", because the Bounds can now be resized.
- export — **Only the Level is exported**: modified to add the Bounds' outline and the Bounds tool's handles to what the Export never shows, because the editor now draws both over the Level.
- composing — **One history**: modified to add Resize Bounds to the Commands that are each one undo step in the one history, because resizing the Bounds joins the history.
- composing — **Redo repeats exactly**: modified to add Resize Bounds, so that redoing it leaves the Bounds as they were before the undo, for the same reason.
- composing — **A failed Command is reported**: modified to add Resize Bounds, for the same reason.
- composing — **Undo waits for the step being made**: modified to add a press that drags the Bounds, to its release, and a field of the Bounds held (Undo waits for the Bounds), because a drag of the Bounds is one step like a drag of a handle.
- composing — **Escape stops placing**: modified to add that Escape leaves the Bounds tool, because the Bounds tool is left as every other tool is.
- composing — **One thing under the pointer**: modified to add that choosing the Bounds tool drops the chosen Asset and the selection and leaves the Wall, the Room, the Portal, or the Paint tool, discarding what is being drawn, and that choosing an Asset or any of those tools leaves the Bounds tool (The Bounds tool), because the Bounds tool joins the tools.
- projects — **A bad file is refused**: modified to add a file whose Bounds have a width or a height of no cell, because Bounds of no cell export nothing and no Command can make them.

## Implementation Decisions

The technology the architecture fixes (the history's generic field-setting command and its gesture groups, the serialisation registry and the Project snapshot, the Editor's overlays drawn with gizmos and never through RenderEngine, the tiled export through OutputAccess) is used as written there and not restated. The mask tiles of painted Elements are keyed in the Level's pixel plane and never refer to the Bounds, so a resize recomputes no coverage.

- **Ownership**: the Bounds component, on the Project entity, becomes AuthoringManager's to write; ProjectManager writes it only through the Project lifecycle's exception, for the new Project, whose Bounds stay thirty by thirty cells from the origin, and on Open. Its saved shape, its stable name, and its version are unchanged, so no migration is needed.
- **Apply** gains Resize Bounds, carrying the new Bounds whole (the lower-left corner and the size in whole cells) and a gesture (single, begin, continue, or end), as an Edit Element carries one. It names no Project or Level: the Bounds are the Project's, shared by every Level, and AuthoringManager resizes those of the one Project in the World. Sending the Bounds whole rather than a side and an amount keeps every step of a gesture exact and lets the fields and the drag send the same message.
- **Resize Bounds** is handled by AuthoringManager: it refuses, before anything is recorded and with a CommandFailed carrying the reason, Bounds below 1 or above 1,000 cells on a side, an edge more than 10,000 cells from the origin (the edges counted in a wider integer, so no sum overflows), and a World with no Project; it records nothing for Bounds equal to the current ones; otherwise it records the generic field-setting command swapping the whole Bounds component of the Project entity, a single Resize Bounds as a step of its own that closes any gesture group left open, and a gesture's as part of its group, as a dragged Edit Element is. The Project entity is addressed by its entity: Open replaces the Project and empties the history, so no step outlives the entity it names.
- **The overlay**: the Editor draws the Bounds' outline with its other overlays, after the Grid's lines and before the selection and the handles, through a gizmo group of its own: a white line two logical pixels wide over a black line four pixels wide, so it shows on light and dark ground alike, at constant screen width whatever the zoom; it is skipped while an Export runs, as the Grid's lines are, and never drawn through RenderEngine, so no Export can hold it. Nothing outside the Bounds is dimmed.
- **The Bounds tool** joins the tool strip after Paint and before the Snap switch, disabled while an Export runs like the other tools, and its key `O` is stated once in the bindings so the strip's tooltip shows the key that works. Its state lives in an Editor module of its own, as the Wall tool's does: the drag under way and the option gesture of its fields. Choosing it, leaving it, and Escape follow the existing tool switching. While it is chosen, a press that hits no corner and no edge does nothing, picking included, and panning and zooming work as with every tool.
- **Handles**: the Bounds are seen through the editor-handles guideline's one view of an outline, a closed outline of the four corners with no control points, so the existing handle functions list, hit, and draw a handle at each corner (its points) and at each edge's middle (the middles of straight edges), corners winning over middles within the handle's reach; a press on an edge's line away from a handle is hit as a Wall's line is, within the handle's reach in pixels. The Bounds tool maps what was hit to the edges it drags: a corner to its two edges, a middle or a line to its edge. The pointer's icon is egui's resizing icon for the edge or corner under it.
- **The drag** follows the editor-drag-gesture guideline: the press arms an interaction of its own carrying the edges dragged, the Bounds at the press, and the drag, named by the Editor's check for a step under way so undo and redo wait from the press; each frame the Bounds tool computes the pointer's travel across each dragged edge since the press, rounds it to whole cells, halfway away from zero, clamps the edges to the limits, and steps the drag on the Bounds that gives, sending Resize Bounds as begin and continue and, on the release, the last Bounds sent as the end. The Editor rounds the travel itself and the Pointer snaps nothing during the drag: the Bounds are whole cells by their type, so which whole cell an edge lies on is hit-testing, not ShapeEngine's snapping of a placed point, and neither the Snap switch nor Alt applies.
- **The fields** follow the tool-strip option guideline: four integer drag values for the left edge, the bottom edge, the width, and the height, built not to send while showing and not clamped to a range, a drag sent while held as one gesture of Resize Bounds ended when the field is let go or the strip stops showing it, a typed value sent as typed as a single Resize Bounds, which AuthoringManager refuses with its reason when out of limits; the Editor's step-under-way check includes the option gesture. A press on the viewport while a field has the keyboard is carried out a frame later, as for every field of the strip.
- **The Export**: ProjectManager adds to its refusals, before any file exists, an image wider or higher than 100,000 pixels, naming the limit and the largest whole resolution the Bounds allow, or the Bounds as too large when not even 1 pixel per cell fits; this replaces the refusal of Bounds whose pixel size cannot be counted, which it covers. The Export job keeps the Bounds it read when the request was handled, as it does now, so a Resize Bounds handled during the job changes nothing of it. The Export dialog applies the same limit to the resolution it shows, refusing in words naming it and disabling Export as it does for the resolution's own limits, and names the Bounds as they are each frame. While an Export runs the Bounds' outline and handles are among the overlays not drawn.
- **Open** checks the Bounds with the rest of a file's data: a width or a height of no cell makes the file bad and refused with the reason; larger Bounds, or Bounds far from the origin, are materialised as they are.
- **The export seam's fixtures**: where an export test needs small Bounds, its fixture sends Resize Bounds instead of setting them in the World directly.
- **The development-only input script**: its `describe` step logs the Bounds, whether the Bounds tool is chosen, the drag under way and the edges it drags, and lists the Bounds' handles and the Bounds tool's fields among the clickable widgets with their rectangles, so a script can aim at them.

## Testing

- **Bounds seam**: a headless App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, and AuthoringManager, with no window and no RenderEngine, over a fixture Asset Folder holding images of known pixel size, a door image among them, and a texture, driven by Apply, Undo, and Redo messages, and asserted on the Project's Bounds, every Element's components, derived shape, and coverage, the answers, and the history.
  - **Resized to the Bounds given**: `crates/drs-app/tests/bounds.rs::resized_to_the_bounds_given` (grown on the left, shrunk at the top, moved whole, and one cell by one; undo and redo of each)
  - **A resize gesture is one step**: `crates/drs-app/tests/bounds.rs::a_resize_gesture_is_one_step` (a begin, several continuations, and an end: one step, undo back to the Bounds at the beginning)
  - **The same Bounds record nothing**: `crates/drs-app/tests/bounds.rs::the_same_bounds_record_nothing` (a single resize to the Bounds as they are, and a gesture whose every step repeats them)
  - **Elements stay**: `crates/drs-app/tests/bounds.rs::elements_stay` (a Prop inside, a Wall across the edge with a Portal set into it, a Room and a Terrain of several strokes outside the shrunk Bounds: every component, the order, the derived shapes, and the coverage tile for tile unchanged after the resize, its undo, and its redo)
  - **At least a cell, at most a thousand**: `crates/drs-app/tests/bounds.rs::at_least_a_cell_at_most_a_thousand` (no width, no height, and 1,001 refused with the limits named and nothing recorded; 1 and 1,000 accepted)
  - **Within reach of the origin**: `crates/drs-app/tests/bounds.rs::within_reach_of_the_origin` (an edge at 10,000 and at −10,000 accepted, one cell farther on each side refused, and a corner near the largest whole number refused rather than overflowing)
  - The modified composing Rules: **One history** `crates/drs-app/tests/bounds.rs::resizing_shares_the_history` (a resize between two placements, undone and redone in order), **Redo repeats exactly** by `resized_to_the_bounds_given` above, **A failed Command is reported** by `at_least_a_cell_at_most_a_thousand` and `within_reach_of_the_origin` above.
- **Offscreen export seam**: the existing headless App with RenderEngine under Bevy's default plugins without a window, exporting to a temporary PNG and asserting pixels and answers.
  - **Exactly the Bounds** as resized: `crates/drs-app/tests/export.rs::an_export_follows_resized_bounds` (Bounds resized to a few cells away from the origin: the image's size, a Prop's colour inside, and a Prop left outside by the shrink leaving no trace; then grown back over it and exported again, its colour where it lies), `crates/drs-app/tests/export.rs::an_export_keeps_the_bounds_it_was_asked_for` (a Resize Bounds sent in the frame after the Export Level: the image of the Bounds before it)
  - **Image within limits**: `crates/drs-app/tests/export.rs::image_within_limits` (Bounds of 1,000 cells at 101 pixels per cell refused, naming the limit and 100 as the largest resolution, with no file at the path; Bounds of 200,000 cells, as an opened file may hold and set in the World as one would be, refused as too large to export; the existing size checks keep the accepted side)
- **Projects seam**: the existing headless App saving and reopening Projects in temporary directories.
  - The existing projects Rules **Everything the Project is**, **Opened as saved**, and **Saving is a fixed point** across a resize: `crates/drs-app/tests/projects.rs::resized_bounds_are_saved` (resized, saved, opened: the same Bounds, and the second save byte for byte the first)
  - The existing projects Rule **Unsaved means a step since the save** for a resize: `crates/drs-app/tests/projects.rs::a_resize_is_an_unsaved_change` (unsaved after a resize, saved again after undoing back)
  - **Opened Bounds are kept**: `crates/drs-app/tests/projects.rs::opened_bounds_are_kept` (a fixture file with Bounds of 2,000 cells reaching 20,000 cells from the origin: opened as they are and saved byte for byte as the file)
  - The modified projects Rule **A bad file is refused**: `crates/drs-app/tests/projects.rs::bounds_of_no_cell_are_refused` (a fixture file with a width of no cell: refused with the reason, the current Project untouched)
- **By hand**: **The Bounds are shown**, **The Bounds tool**, **Handles of the Bounds**, **Dragging the Bounds**, **Whole cells whatever**, **A drag stops at the limits**, **The Bounds fields**, **Undo waits for the Bounds**, **The Bounds wait for the Export**, the modified composing Rules **Undo waits for the step being made**, **Escape stops placing**, and **One thing under the pointer**, and the modified export Rule **Only the Level is exported** for the outline and the handles: no automated seam for the egui interface and the Editor's overlays, the accepted deviation of the composing and export specs; verified by driving the editor with the development-only input script, whose `describe` step shows the Bounds, the Bounds tool, and the drag, dragging each edge and a corner past the edge opposite and beyond a thousand cells with snapping off and Alt held, typing and dragging each field, typing a refused width, undoing during a drag, and exporting with the Bounds tool chosen.

## Out of Scope

- Moving the Bounds whole by dragging inside them; typing the left or bottom edge moves them.
- Dimming or shading what lies outside the Bounds.
- A Bounds dialog from a menu, and choosing the Bounds when a Project is created (a new-Project dialog).
- Fitting the Bounds to the Elements of a Level, and framing the view on the Bounds.
- Bounds measured in anything but whole cells, and Bounds per Level: the domain shares one Bounds among every Level of a Project.
- Snapping Elements' points to the Bounds' edges.
- Progress and cancellation of a large Export, and Export formats other than PNG.

## Further Notes

- **Architecture check**: within the architecture. Resize Bounds is AuthoringManager's, sent by the Editor as Apply; its step is the history's generic field-setting command; the outline and the handles are Editor overlays, which no Export can draw; the Export's new limit is ProjectManager's refusal before OutputAccess is asked for anything. No new component, contract operation, or dependency direction. The Editor rounds a drag of the Bounds to whole cells itself rather than through ShapeEngine's Snap, reading the Bounds' integer edges as hit-testing; the architecture's Snapping split may want to say so.
- **The capability**: Layers, earlier on the roadmap, creates the levels-and-layers pinned spec, and this change adds Resize Bounds to its implemented Commands. Should Bounds land before Layers, this change creates that pinned spec, with the header Layers plans: Add Level, Remove Level, Reorder Levels, Add Layer, Remove Layer, Reorder Layers, Group Layers, Edit Layer, Resize Bounds, and Set Ambient Light, every one defined by the domain, owned by no other capability, and on the roadmap, all but Resize Bounds stated as owned and not implemented.
- **The limits**: with Bounds of at most 1,000 cells a side and an image of at most 100,000 pixels a side, the largest Bounds export at up to 100 pixels per cell, and the default thirty cells at any resolution up to the existing 1,024. The largest accepted image is not exported by an automated test, which would write gigabytes; its memory is the band the architecture's Export bullet describes.
