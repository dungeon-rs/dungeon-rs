# Levels and layers

**Commands**: Add Level, Remove Level, Reorder Levels, Add Layer, Remove Layer, Reorder Layers, Group Layers, Edit Layer, Resize Bounds, Set Ambient Light

## Purpose

A Project is more than the Elements on it: it has a frame that decides what the Export holds, and a structure of Levels and Layers that Elements sit in. This capability lets the Author see the Bounds in the viewport at every zoom, drag their edges and corners or type their place and size, and take every resize back and forward again, so that the Export holds exactly the map the Author drew and never needs a guess.

Resize Bounds is the one Command implemented here; Add Level, Remove Level, Reorder Levels, Add Layer, Remove Layer, Reorder Layers, Group Layers, Edit Layer, and Set Ambient Light are owned here and not implemented.

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

### Resizing and the Elements

30. As an Author, I can shrink the Bounds over Elements and find every Element still where it was, so that trying a smaller map never costs me work.
31. As an Author, I can grow the Bounds back over Elements left outside and see them in the next Export, so that the Bounds are only a frame.
32. As an Author, I can open a Project whose Bounds are larger than this editor lets me make them and save it again with the Bounds as they were, so that an Author with a later editor never loses their map size to me.

### While an Export runs

33. As an Author, I can rely on the Bounds tool, its handles, and its fields doing nothing while an Export runs, so that the Bounds never change under an image being written.

## Rules

### Resize Bounds

**Resized to the Bounds given**: a Resize Bounds sets the Project's Bounds to the lower-left corner, width, and height it carries, in whole cells, as one history step that undo returns to the Bounds before it and redo sets again. Follows from: Every Command can be undone.

**A resize gesture is one step**: Resize Bounds sent as a gesture, from its beginning through every continuation to its end, is one history step, and undo returns the Bounds to where the gesture began. Follows from: Every Command can be undone.

**The same Bounds record nothing**: a Resize Bounds to the Bounds as they are changes nothing and records no history step, and a gesture records none when it never changes the Bounds or ends with them as they were when it began, however far it went in between, leaving what could be redone redoable.

**Elements stay**: a Resize Bounds, its undo, and its redo change no Element on any Layer, inside the Bounds, across their edge, or outside them: each keeps its ElementId, its place in the stacking order, every property, and its derived shape and coverage. Follows from: Bounds only decide what is exported.

**At least a cell, at most a thousand**: a Resize Bounds whose width or height is below 1 or above 1,000 cells is refused with the limits named, changes nothing, and records no history step.
_Why_: Bounds of no cell export nothing, and a thousand cells is far beyond any battle map while keeping the Export's image within reach at a useful resolution.

**Within reach of the origin**: a Resize Bounds any of whose edges lies more than 10,000 cells from the Level's origin, to the left, the right, below, or above, is refused with the limit named, changes nothing, and records no history step.
_Why_: far from the origin a position in cells loses the precision the Export draws at.

**Opened Bounds are kept**: a Project opened with Bounds wider or higher than 1,000 cells, or reaching more than 10,000 cells from the Level's origin, keeps them as they are, and saving writes them unchanged; only a Resize Bounds is held to the limits.

### The Bounds in the editor

**The Bounds are shown**: the viewport always draws the Bounds' four edges as an outline of the same thickness on screen at every zoom, over every Element and over the Grid's lines, whatever tool is chosen, and leaves everything outside the Bounds drawn as it is inside them; it draws no outline while an Export runs.

**The Bounds tool**: the tool strip offers the Bounds tool, also chosen with `O`; while it is chosen a click or a press on the Level selects, places, paints, and removes nothing, and the status line says that dragging an edge or a corner resizes the Bounds. Choosing it, and leaving it by Escape, by choosing another tool, or by choosing an Asset, follow One thing under the pointer and Escape stops placing.

**Handles of the Bounds**: with the Bounds tool chosen, the Bounds show a handle at each corner and at the middle of each edge, and the pointer shows a resizing pointer wherever a press would drag the Bounds.

**Dragging the Bounds**: with the Bounds tool chosen, a press on a corner's handle drags the corner's two edges, and a press on an edge's handle or within a handle's reach of an edge's line drags that edge alone, a corner winning over an edge; from the press to the release each dragged edge lies where it was at the press moved by the pointer's travel across it, rounded to whole cells, a travel exactly halfway rounded away from zero, and every edge not dragged stays where it was; the drag sends Resize Bounds as one gesture, which ends where it was last shown when released over a panel or outside the window.

**Whole cells whatever**: a drag of the Bounds moves its edges by whole cells whether the Snap switch is on or off and whether or not Alt is held.

**A drag stops at the limits**: during a drag each dragged edge stops one cell from the edge opposite it, a thousand cells from that edge, and 10,000 cells from the Level's origin, so a drag never sends a Resize Bounds that would be refused; Bounds opened beyond the limits are brought within them by the first step of a drag.

**The Bounds fields**: with the Bounds tool chosen, the options strip shows the Bounds' left edge, bottom edge, width, and height as whole numbers of cells, as they are after every Resize Bounds, undo, and redo; a value typed into one is sent, as the field lets go of the keyboard, as one Resize Bounds keeping the other three, so a width or a height keeps the left or the bottom edge where it is and a left or a bottom edge moves the Bounds whole; dragging a field changes it by whole cells, stopped at the limits and starting from the Bounds brought within them when they were opened beyond them, as a drag of their edges does, and sends Resize Bounds as one gesture that ends when the field is let go; a refused value leaves the Bounds as they were, the field showing them, and the reason in the status line.

**Undo waits for the Bounds**: undo and redo wait from a press that drags the Bounds to its release, and while a field of the Bounds is held, whether or not anything has changed yet, as Undo waits for the step being made says of every step.

**The Bounds wait for the Export**: while an Export runs the Bounds tool cannot be chosen and its handles, drags, and fields do nothing.

## Implementation Decisions

The technology the architecture fixes (the history's generic field-setting command and its gesture groups, the serialisation registry and the Project snapshot, the Editor's overlays drawn with gizmos and never through RenderEngine, the tiled export through OutputAccess) is used as written there and not restated. The mask tiles of painted Elements are keyed in the Level's pixel plane and never refer to the Bounds, so a resize recomputes no coverage.

- **Ownership**: the Bounds component, on the Project entity, is AuthoringManager's to write; ProjectManager writes it only through the Project lifecycle's exception, for the new Project, whose Bounds are thirty by thirty cells from the origin, and on Open. Its saved shape, its stable name, and its version are those of the Project file, so a resize needs no migration.
- **Apply** carries Resize Bounds: the new Bounds whole (the lower-left corner and the size in whole cells) and a gesture (single, begin, continue, or end), as an Edit Element carries one. It names no Project or Level: the Bounds are the Project's, shared by every Level, and AuthoringManager resizes those of the one Project in the World. _Why_: sending the Bounds whole rather than a side and an amount keeps every step of a gesture exact and lets the fields and the drag send the same message.
- **Resize Bounds** is handled by AuthoringManager: it refuses, before anything is recorded and with a CommandFailed carrying the reason, Bounds below 1 or above 1,000 cells on a side, an edge more than 10,000 cells from the origin (the edges counted in a wider integer, so no sum overflows), and a World with no Project; it records nothing for Bounds equal to the current ones; otherwise it records the generic field-setting command swapping the whole Bounds component of the Project entity, a single Resize Bounds as a step of its own that closes any gesture group left open, and a gesture's as part of its group, as a dragged Edit Element is. The end of a gesture closes its group even when that last Resize Bounds is refused, so the resizes before it remain one step. The Project entity is addressed by its entity: Open replaces the Project and empties the history, so no step outlives the entity it names.
- **The overlay**: the Editor draws the Bounds' outline with its other overlays, after the Grid's lines and before the selection and the Walls' and Rooms' handles, through a gizmo group of its own: lines one physical pixel wide laid side by side, a light band two logical pixels wide over a dark band four logical pixels wide, so it shows on light and dark ground alike at constant screen width whatever the zoom and the display's scale; it is skipped while an Export runs, as the Grid's lines are, and never drawn through RenderEngine, so no Export can hold it. Nothing outside the Bounds is dimmed.
- **The Bounds tool** joins the tool strip after Paint and before the Snap switch, disabled while an Export runs like the other tools, and its key `O` is stated once in the bindings so the strip's tooltip shows the key that works. Its state lives in an Editor module of its own, as the Wall tool's does: the drag under way and the option gesture of its fields. Choosing it, leaving it, and Escape follow the existing tool switching. While it is chosen, a press that hits no corner and no edge does nothing, picking included, and panning and zooming work as with every tool. A drag under way when the tool is left goes on to its release, so its gesture ends.
- **Handles**: the Bounds are seen through the editor-handles guideline's one view of an outline, a closed outline of the four corners with no control points, so the existing handle functions list, hit, and draw a handle at each corner (its points) and at each edge's middle (the middles of straight edges), corners winning over middles within the handle's reach; a press on an edge's line away from a handle is hit as a Wall's line is, within the handle's reach in pixels. The Bounds tool maps what was hit to the edges it drags: a corner to its two edges, a middle or a line to its edge. The pointer's icon is egui's resizing icon for the edge or corner under it, and for the whole of a drag. A corner is drawn as a ring and a middle as one half its radius, those the drag moves in the picked colour.
- **The drag** follows the editor-drag-gesture guideline: the press arms an interaction of its own carrying the edges dragged, the Bounds at the press brought within the limits, and the drag, named by the Editor's check for a step under way so undo and redo wait from the press; each frame the Bounds tool computes the pointer's travel across each dragged edge since the press, rounds it to whole cells, halfway away from zero, clamps the edges to the limits, and steps the drag on the Bounds that gives, sending Resize Bounds as begin and continue and, on the release, the last Bounds sent as the end; a press that never became a drag sends nothing. The Editor rounds the travel itself and the Pointer snaps nothing during the drag: the Bounds are whole cells by their type, so which whole cell an edge lies on is hit-testing, not ShapeEngine's snapping of a placed point, and neither the Snap switch nor Alt applies.
- **The fields** follow the tool-strip option guideline: four integer drag values for the left edge, the bottom edge, the width, and the height, built not to send while showing and not clamped to a range by the widget, a drag sent while held as one gesture of Resize Bounds, stopped by the Editor at the limits, and ended when the field is let go or the strip stops showing it, a typed value sent as typed as a single Resize Bounds, which AuthoringManager refuses with its reason when out of limits; a value beyond what the Bounds' type holds is sent saturated, so it is refused with the limits named. The Editor's step-under-way check includes the option gesture. A press on the viewport while a field has the keyboard is carried out a frame later, as for every field of the strip.
- **The development-only input script**: its `describe` step logs the Bounds, whether the Bounds tool is chosen, the drag under way and the edges it drags, and lists the Bounds' handles and the Bounds tool's fields among the clickable widgets with their rectangles, so a script can aim at them.

## Test seams

The automated seam is a headless App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, and AuthoringManager, with no window and no RenderEngine, over a fixture Asset Folder holding images of known pixel size, a door image among them, and a texture, driven by Apply, Undo, and Redo messages, and asserted on the Project's Bounds, every Element's components, derived shape, and coverage, the answers, and the history.

- **Resized to the Bounds given**: `crates/drs-app/tests/bounds.rs::resized_to_the_bounds_given` (grown on the left, shrunk at the top, moved whole, and one cell by one; undo and redo of each)
- **A resize gesture is one step**: `crates/drs-app/tests/bounds.rs::a_resize_gesture_is_one_step` (a begin, several continuations, and an end: one step, undo back to the Bounds at the beginning), `crates/drs-app/tests/bounds.rs::a_gesture_closes_however_it_ends` (a refused last Resize Bounds leaves the ones before it one step, and a single Resize Bounds sent in the middle closes the gesture and is a step of its own)
- **The same Bounds record nothing**: `crates/drs-app/tests/bounds.rs::the_same_bounds_record_nothing` (a single resize to the Bounds as they are, and a gesture whose every step repeats them), `crates/drs-app/tests/bounds.rs::a_drag_back_to_its_start_records_nothing` (after an undo, a drag away and back to where it began: no step, and the undone step still redoable)
- **Elements stay**: `crates/drs-app/tests/bounds.rs::elements_stay` (a Prop inside, a Wall across the edge with a Portal set into it, a Room and a Terrain of several strokes outside the shrunk Bounds: every component, the order, the derived shapes, and the coverage tile for tile unchanged after the resize, its undo, and its redo)
- **At least a cell, at most a thousand**: `crates/drs-app/tests/bounds.rs::at_least_a_cell_at_most_a_thousand` (no width, no height, and 1,001 refused with the limits named and nothing recorded; 1 and 1,000 accepted)
- **Within reach of the origin**: `crates/drs-app/tests/bounds.rs::within_reach_of_the_origin` (an edge at 10,000 and at -10,000 accepted, one cell farther on each side refused, and a corner near the largest whole number refused rather than overflowing)
- **Opened Bounds are kept**: `crates/drs-app/tests/projects.rs::opened_bounds_are_kept` (a fixture file with Bounds of 2,000 cells reaching 20,000 cells from the origin: opened as they are and saved byte for byte as the file)
- **The Bounds are shown**, **The Bounds tool**, **Handles of the Bounds**, **Dragging the Bounds**, **Whole cells whatever**, **A drag stops at the limits**, **The Bounds fields**, **Undo waits for the Bounds**, **The Bounds wait for the Export**: by hand: no automated seam for the egui interface and the Editor's overlays; verified by driving the editor with the development-only input script, whose `describe` step shows the Bounds, the Bounds tool, and the drag, dragging each edge and a corner past the edge opposite and beyond a thousand cells with snapping off and Alt held, typing and dragging each field, typing a refused width, undoing during a drag, and exporting with the Bounds tool chosen

## Not supported

- The Bounds are never measured in anything but whole cells, and are never per Level: one Bounds is shared by every Level of a Project.
- Resizing never adds, removes, or changes an Element, and never dims or hides what lies outside the Bounds.
- The Bounds' outline and handles are the Editor's own marks and are never part of an Export.

## Notes

- Nine Rules (The Bounds are shown, The Bounds tool, Handles of the Bounds, Dragging the Bounds, Whole cells whatever, A drag stops at the limits, The Bounds fields, Undo waits for the Bounds, The Bounds wait for the Export) have no automated test, against the requirement that every Rule has one. They are behaviour of the egui interface and of the Editor's overlays, for which no headless seam exists. The accepted deviation is verification by hand, driving the editor with the development-only input script.
- Elements stay is tested on one Layer, the only one a Project has.
- A Resize Bounds in a World with no Project is refused by AuthoringManager and is checked by no test, the editor always having a Project.
- With Bounds of at most 1,000 cells a side and an image of at most 100,000 pixels a side, the largest Bounds export at up to 100 pixels per cell, and the default thirty cells at any resolution up to the Export's 1,024.
- Add Level, Remove Level, Reorder Levels, Add Layer, Remove Layer, Reorder Layers, Group Layers, Edit Layer, and Set Ambient Light are owned here and not implemented; a Project has one Level with one Layer, and the Bounds are shared by every Level, so Bounds per Level, moving the Bounds whole by dragging inside them, fitting them to the Elements of a Level, framing the view on them, a Bounds dialog from a menu, choosing them when a Project is created, and snapping Elements' points to their edges do not exist.
