# Snapping

**Capabilities**:
- composing: Place Element, Edit Element (the points the Editor sends with them, and a whole Wall or Room moved by an amount)
- export: none of its Commands; the Grid's lines and the snapping marker stay out of an Export

## Problem Statement

A Wall's or a Room's points land exactly where the pointer is, to a fraction of a pixel. Two Rooms meant to stand side by side never quite meet: their edges lie a hair apart or a hair over one another, so they are walled each on their own or open into each other, never two rooms with one Wall and a door between them. A corridor's Walls are never quite straight, and nothing on screen tells the Author where the cells of the Grid are, so drawing to the scale the Assets were made for is guesswork. The functional baseline snaps to its grid by default, lets the author switch that off, and frees a single gesture with a held key.

## Solution

The viewport shows the Grid as faint lines. While the Author draws a Wall or a Room, or drags one of their points, each point snaps: to the nearest point of a Wall or Room on the Level when one lies within a few pixels of the pointer, and otherwise to the nearest corner of a Grid cell. A small marker shows where the next click will land before the Author clicks, and the point placed is exactly the point shown, so a Room drawn against another's corners shares their edges exactly. Dragging a whole Wall or Room moves it by whole cells. Holding Alt, Option on macOS, places and drags freely for as long as it is held, and a Snap switch on the tool strip turns snapping off altogether. Control points, Props, and Portals do not snap, and nothing about snapping is saved in the Project or recorded in the history.

## User Stories

### Seeing the Grid

1. As an Author, I can see the Grid drawn in the viewport as faint lines along the edges of its cells, so that I can draw to the scale my Assets were made for.
2. As an Author, I can see the Grid over my Walls, Rooms, and Props, faint enough that they show through, so that I can line things up anywhere on the Level.
3. As an Author, I can see the Grid however far I pan, the Bounds included and beyond, so that composing outside the Bounds is as easy as inside.
4. As an Author, I can zoom out until the cells are only a few pixels across and see the Grid's lines go away, so that a zoomed-out view is not a solid wash of lines.
5. As an Author, I can export a Level and find no Grid line in the image, so that the Grid helps me compose and never spoils the map.
6. As an Author, I can see the Grid whether snapping is on or off, so that switching snapping off never hides where the cells are.

### Snapping to the Grid

7. As an Author, I can click with the Wall tool and have the point land on the corner of a Grid cell nearest the pointer, so that my Walls run along the Grid without my aiming exactly.
8. As an Author, I can click with the Room tool and have each point land on the nearest Grid corner, so that a Room's corners sit on the Grid.
9. As an Author, I can drag a rectangle with the Room tool and have both of its corners land on Grid corners, so that a rectangular Room covers whole cells.
10. As an Author, I can drag a point of a selected Wall or Room and see it jump from Grid corner to Grid corner as the pointer moves, so that reshaping keeps a Room on the Grid.
11. As an Author, I can drag a whole Wall or Room and see it move by whole cells, so that a Room drawn on the Grid stays on the Grid wherever I put it.
12. As an Author, I can drag a whole Wall or Room that is not on the Grid and see it move by whole cells all the same, keeping its offset from the Grid, so that dragging never reshapes or shifts what I placed freely.

### Snapping to other points

13. As an Author, I can bring the pointer within a few pixels of a corner of another Room and have the point land exactly on that corner, so that two Rooms meet exactly where I meant.
14. As an Author, I can draw a Room against a Room I drew freely, off the Grid, and have its points land exactly on the other Room's points, so that sharing edges never depends on the Grid.
15. As an Author, I can have a point land on a nearby Wall's or Room's point rather than on a Grid corner that is nearer still, so that meeting another Element always wins over the Grid.
16. As an Author, I can rely on the nearest of several nearby points winning, and the topmost Element's point when two are as near, so that where a point lands is predictable.
17. As an Author, I can drag a Room's point onto another Room's point and have it land there exactly, so that I can join Rooms after drawing them.
18. As an Author, I can drag a point and have it land on another point of the same Wall or Room, so that snapping treats my own Element's points like any other's.
19. As an Author, I can drag a point and never have it stick to where it was, so that a point moves as soon as the pointer leaves it.
20. As an Author, I can rely on the points of Walls and Rooms on every Layer of the Level being within reach, so that a building on one Layer lines up with one on another.
21. As an Author, I can rely on the points of Elements on another Level never being within reach, so that the floor above never pulls my points.
22. As an Author, I can rely on a Prop's centre, a Portal, and a curve's control point never pulling a point towards them, so that only corners of Walls and Rooms attract.
23. As an Author, I can rely on the points I have already clicked for the Wall or Room I am drawing not pulling the next point, so that drawing a narrow shape never collapses it.

### Seeing where a point will land

24. As an Author, I can see a small marker where the next click of the Wall or Room tool will land, so that I know where the point goes before I click.
25. As an Author, I can see the marker set apart when it lies on another Element's point, so that I can tell meeting a corner from landing on the Grid.
26. As an Author, I can see the preview's line from the last point, and the Room tool's line back to the first, end at the marker, so that the preview shows the shape the next click makes.
27. As an Author, I can rely on the point placed being exactly where the marker showed it, so that what I see is what I get.

### Placing freely

28. As an Author, I can hold Alt, Option on macOS, while clicking or dragging and have points land exactly under the pointer, so that a crooked wall or an odd corner is one key away.
29. As an Author, I can press or let go of Alt in the middle of a drag and see the drag follow the pointer freely or snap again from that moment, so that I never have to start a drag over.
30. As an Author, I can switch snapping off on the tool strip and draw and drag freely until I switch it on again, so that a free-hand session needs no held key.
31. As an Author, I can rely on snapping being on whenever the editor starts, so that the common path is the default.
32. As an Author opening a Project from a collaborator, I can rely on my own Snap switch being as I set it, whatever theirs was, so that how someone else composes never changes how I compose.

### What does not snap

33. As an Author, I can drag a control point or a straight edge's middle handle and see the curve follow the pointer exactly, so that curves stay smooth.
34. As an Author, I can double-click a line to add a point and have it land on the line where I clicked, so that adding a point never changes the shape.
35. As an Author, I can place and drag Props and Portals exactly as before, so that snapping only ever touches Walls and Rooms.

### History and saving

36. As an Author, I can rely on snapping and switching it never being an undo step, so that undo only ever takes back what I placed or changed.
37. As an Author, I can undo a drag that snapped and see the point return to exactly where the drag began, so that snapping never makes undo approximate.
38. As an Author, I can save and reopen a Project and find every snapped point exactly where it was, so that Rooms sharing an edge still share it after a round trip.

## Rules

### Where a point snaps

**Snapping to the Grid**: with snapping, a point is put at the corner of a Grid cell nearest the pointer, each coordinate rounded to the nearest whole number of cells and a coordinate exactly halfway rounded away from zero, unless a point of a Wall or Room lies within reach (Points win within reach).

**Points win within reach**: with snapping, when points of Walls or Rooms lie within reach of the pointer, eight logical pixels of the view, the point is put at the nearest of them, the topmost Element's on a tie, even where a Grid corner lies nearer.

**What a point snaps to**: the points within reach are the points of every Wall and Room on the Level the Author is working on, on any of its Layers; control points, Props, Portals, and Elements on other Levels are never within reach. Follows from: Levels are independent.

**The dragged point is left out**: while a point is dragged, that point is never within reach of itself; every other point of its Wall or Room is.

**Snapped exactly**: a point snapped to a Grid corner has whole numbers of cells as its coordinates, and one snapped to another point has exactly that point's coordinates, so the two are equal.

**Snapping is not a step**: snapping records nothing in the history and changes nothing in the Project; a Place Element or Edit Element applies the points it carries as they are, whether they were snapped or not.

### What snaps

**Drawing snaps**: with snapping, each point a click of the Wall tool or the Room tool adds, and both corners of a rectangle a Room tool drag draws, is put where snapping puts the pointer.

**Dragged points snap**: with snapping, dragging a point of a selected Wall or Room puts it where snapping puts the pointer at every step of the drag, and the drag stays one history step.

**Moving by whole cells**: with snapping, dragging a whole Wall or Room moves it by the pointer's travel since the press, each coordinate rounded to the nearest whole number of cells, a coordinate exactly halfway rounded away from zero.

**Whole cells stay whole**: moving a Wall or a Room by a whole number of cells in each direction leaves every point that lay on a Grid corner exactly on a Grid corner.

**What does not snap**: control points, a straight segment's or edge's middle handle, the point a double-click adds on a line, the points already added to the Wall or Room being drawn, Props, and Portals are placed and dragged as without snapping, and are never within reach.

**Alt places freely**: while Alt, Option on macOS, is held, every point and every drag that would snap follows the pointer exactly instead; pressing or letting go of it during a drag changes the next step of the drag and leaves it one history step.

**The Snap switch**: the tool strip shows a Snap switch, on when the editor starts; while it is off nothing snaps; switching it is never a history step, and it is kept neither in the Project nor between runs of the editor.

**The marker shows where a click lands**: with the Wall tool or the Room tool chosen and snapping, a small marker shows where the next click would put a point, drawn set apart when that is another Element's point, and the preview's lines to the pointer end at the marker; the point a click adds is exactly the marked one.

### The Grid in the viewport

**The Grid is shown**: the viewport draws a thin, faint line along every edge of every Grid cell in view, at every whole number of cells in each direction, over the Elements, whether or not snapping is on, while a cell is at least eight logical pixels across, and no line while it is smaller.

## Changes to existing behaviour

The composing Rules named here are those the composing spec holds now that Rooms have landed, which this change builds on.

- composing — **Drawing with the Wall tool**: modified so that "each click on the Level adds the point snapping puts it at (Drawing snaps) unless that point lies within a few pixels of the last one; the Wall in progress is previewed with a segment from its last point to that point", because the point a click adds is the snapped one.
- composing — **Drawing with the Room tool**: modified so that "each click on the Level adds the point snapping puts it at unless that point lies within a few pixels of the last one; the outline in progress is previewed with a line from the last point to that point and from it to the first point; a click whose point lies within a few pixels of the first point, with three or more points placed, or Enter, closes it", because the closing click is judged by where it lands, as every other click is.
- composing — **A drag draws a rectangle**: modified so that the rectangle runs between the points snapping puts the press and the release at, and "a rectangle whose corners, so put, lie less than a few screen pixels apart in either direction sends nothing", because a snapped rectangle may collapse onto one Grid line.
- composing — **Handles of the selected Wall**: modified so that "dragging a point puts it where snapping puts the pointer (Dragged points snap)", because a Wall's points snap.
- composing — **Handles of the selected Room**: modified in the same way, and so that "dragging its floor or Walls moves the Room whole, by whole cells with snapping (Moving by whole cells)", because a Room's points snap.
- composing — **Moving the Wall moves every point**: modified to "an Edit Element that changes a Wall's position, or moves it by an amount, moves every point and control point by the same amount", because a whole Wall is now dragged by an amount (Implementation Decisions).
- composing — **Moving the Room moves every point**: modified in the same way, for the same reason.
- export — **Only the Level is exported**: modified to "the Export shows nothing the editor draws over the Level: no Grid line, snapping marker, selection outline, handle, guide line, Wall or Room preview, Portal marker, Brush circle, or stroke band", because the viewport now draws the Grid and the marker over the Level and neither may reach an Export.

## Implementation Decisions

The architecture's split is used as written there: hit-testing, the Element and the nearest point of a line under the pointer, is the Editor's; snapping a placed point to the Grid or into alignment is ShapeEngine's Snap; the Editor may not depend on ShapeEngine, so what Snap answers reaches it through `model` (Derived model component). The composing spec's decisions for Walls, Portals, and Rooms (the derived shape, the once-per-frame deriving, the gesture protocol of Edit Element, the Viewport and its conversions) are extended, not restated.

- **The Pointer**: presentation state in `model` that the Editor owns, beside the Viewport, and writes each frame as it reads the pointer: the Level the Author is working on, the pointer's position in cells, the reach in cells (eight logical pixels at the Viewport's zoom), and what is being snapped: nothing (no tool or drag that snaps, snapping switched off, Alt held, or the pointer off the viewport), a point (with the Element and the index of the point to leave out while one is dragged), or a move (with the position the drag began at). It is never saved and never in the history. _Why_ in `model`: AuthoringManager reads it to ask ShapeEngine, and the Editor may not depend on ShapeEngine.
- **The snapped point**: a derived model component, never saved, that AuthoringManager writes through ShapeEngine's Snap in its system after every Manager has handled its Commands, Undo, and Redo, as Derived model component prescribes: whenever the Pointer changed or a Wall or Room of its Level was placed, edited, or removed, and only when the answer differs. It holds, for a point, the snapped position and, when it lies on another Element's point, that Element's ElementId; for a move, the travel in whole cells; and the Pointer it answers. With nothing being snapped it holds nothing. Its targets are read from the Wall and Room components of the Pointer's Level, with each Element's place in the stacking order for ties; a Layer's Elements are found through the Level's children, so Elements of other Levels are never read.
- **ShapeEngine's Snap** is a plain function over model types: it takes the pointer's position, the reach, the points within the Level with their Elements' places in the stacking order, the point to leave out, and whether it snaps a point or a move. For a point it compares squared distances, so it takes no square root, keeps the nearest point within the reach, the later in the stacking order on a tie, and copies its coordinates; with none, it rounds each coordinate to the nearest whole number, halfway away from zero. For a move it rounds each coordinate of the travel the same way. Rounding and comparing are exact single-precision operations, so the answer is the same on every machine. The targets are scanned linearly: thousands of points cost microseconds, and the system runs only when its inputs change.
- **The Editor's tools use what is shown**: the Wall tool, the Room tool, a point drag, and a whole drag of a Wall or Room take the snapped point last derived, which is the one drawn on screen when the Author clicks or moves, whenever it answers the Pointer the Editor last wrote apart from its position, and the pointer itself otherwise and whenever nothing is being snapped. They store and send snapped points as copies, so no arithmetic stands between a point shown and a point sent; the rectangle's other two corners take one coordinate from each snapped corner. The "few pixels from the last point" and "few pixels from the first point" checks of the Wall and Room tools compare the points as snapped. A point drag sends the snapped point at each step of its gesture, as it sends the pointer today.
- **A move by an amount**: Edit Element gains a move of a Wall or a Room by an amount in cells, which translates every point and control point by that amount through the generic field-setting command, exactly as single-precision addition gives, so a whole number of cells added to a point on a Grid corner lands on a Grid corner. A whole drag of a Wall or a Room sends, at each step of its gesture, the amount moved since the step before (with snapping, the difference between whole-cell travels, itself whole), so the points are never moved by a difference of two box centres; the drag stays one history group that undo returns to where it began. A Prop and a freestanding Portal keep the position change.
- **The Snap switch and Alt**: the switch is Editor state, shown on the tool strip after the tools for every tool, on at start and never written to a file; Alt is read from the keyboard each frame, both the left and the right key, and is ignored while a text field has the keyboard, as every key is. The Editor shows the switch's state on the switch itself and says in its tooltip that Alt places freely.
- **The marker and the preview**: drawn by the Editor with its other overlays after the snapped point is derived, so they show the frame's own state, and not while an Export is written: a small ring at the snapped point in the overlay colour, filled when it lies on another Element's point; the Wall and Room tools' rubber bands end at it.
- **The Grid's lines**: drawn by the Editor as an overlay, never through RenderEngine, so no Export can hold them, and not while an Export is written, as the selection outline is not: one line for every whole number of cells across the visible part of the viewport in each direction, thin, in a dark colour at low opacity over the mid-grey background and the Elements alike, and none while the Viewport's zoom is below eight logical pixels per cell. The Grid has no offset: its corners are the whole-number positions in cells. How the Grid is drawn is not a volatility; the line form is the only one built.

## Testing

- **Composing seam**: `crates/drs-app/tests/snapping.rs`, a headless App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, and AuthoringManager over a fixture Asset Folder holding an image of known pixel size, with no window and no RenderEngine. A test writes the Pointer as the Editor would, runs one update, and asserts the snapped point; it places and edits Walls and Rooms with Apply messages carrying the points the snapped point gave, and asserts the components bit for bit, the answers, and the history. A second Level comes from opening a fixture Project file of two Levels, since the editor has no Add Level yet.
  - **Snapping to the Grid**: `crates/drs-app/tests/snapping.rs::snapping_to_the_grid` (a pointer inside a cell, beyond reach of every point, at negative positions, and exactly halfway)
  - **Points win within reach**: `crates/drs-app/tests/snapping.rs::points_win_within_reach` (a Room's point within reach though a Grid corner is nearer; a point just beyond reach losing to the Grid; two points within reach, the nearer winning; two as near, the later in the stacking order winning)
  - **What a point snaps to**: `crates/drs-app/tests/snapping.rs::what_a_point_snaps_to` (a Wall's and a Room's points reached; a control point, a Prop's centre, and a Portal's centre not; a Room on the other Level of the opened Project not)
  - **The dragged point is left out**: `crates/drs-app/tests/snapping.rs::the_dragged_point_is_left_out` (the left-out point not reached from on top of it; its neighbour on the same Room reached)
  - **Snapped exactly**: `crates/drs-app/tests/snapping.rs::snapped_exactly` (a Room placed off the Grid, a second placed through snapped points against it, the shared points bit-equal, and the same after a save and reopen)
  - **Snapping is not a step**: `crates/drs-app/tests/snapping.rs::snapping_is_not_a_step` (writing the Pointer many times records no history step and leaves the Project unchanged; a point drag of snapped points is one step whose undo returns the point to where it began exactly)
  - **Moving by whole cells**: `crates/drs-app/tests/snapping.rs::moving_by_whole_cells` (a move's travel rounded per coordinate, halfway away from zero)
  - **Whole cells stay whole**: `crates/drs-app/tests/snapping.rs::whole_cells_stay_whole` (a Room with a curved edge whose control point lies off the Grid, moved by whole amounts over a gesture of several steps: every point exactly whole, the control point moved by the same amount, one history step), with the modified **Moving the Wall moves every point** and **Moving the Room moves every point** by the same test and `crates/drs-app/tests/snapping.rs::a_wall_moves_by_an_amount`
  - **What does not snap**, as far as the seam reaches: by `what_a_point_snaps_to` above (control points, Props, and Portals never reached); what the Editor's tools do not snap is checked by hand.
- **By hand**: **Drawing snaps**, **Dragged points snap**, **Alt places freely**, **The Snap switch**, **The marker shows where a click lands**, **What does not snap** for the Editor's tools, and **The Grid is shown**, with the modified **Drawing with the Wall tool**, **Drawing with the Room tool**, **A drag draws a rectangle**, **Handles of the selected Wall**, **Handles of the selected Room**, and **Only the Level is exported**: no automated seam for the egui interface or the viewport's drawing, the accepted deviation of the composing spec; verified by driving the editor with the development-only input script (its `key` step already holds Alt), whose `describe` step logs the Snap switch, the Pointer, and the snapped point, and by exporting a Level while the Grid and the marker are shown and finding neither in the image.
- ShapeEngine's own unit tests check what the Rules rest on, as functions of the Engine alone, and are the coverage of no Rule: rounding to the nearest corner on either side of zero and halfway, the nearest point within reach against the Grid, a tie going to the later point, the left-out point skipped, and a move's travel rounded (`crates/drs-shape-engine/src/snap.rs::tests::rounds_to_the_nearest_corner`, `crates/drs-shape-engine/src/snap.rs::tests::the_nearest_point_within_reach_wins`, `crates/drs-shape-engine/src/snap.rs::tests::a_tie_goes_to_the_later_point`, `crates/drs-shape-engine/src/snap.rs::tests::the_left_out_point_is_skipped`, `crates/drs-shape-engine/src/snap.rs::tests::a_move_rounds_its_travel`).

## Out of Scope

- Snapping to the middles of cell edges or to cell centres, a Grid of another size, an offset Grid, and hexagonal Grids.
- Snapping to anywhere along a Wall's line or a Room's edge, to where lines cross, to the Bounds, or with alignment guides beyond snapping (parked in the open questions).
- Snapping control points, the point a double-click adds, Props, freestanding Portals, and a Portal's place along its Wall.
- A whole Wall or Room drag snapping onto another Element's point rather than by whole cells.
- Snapping to the points already added to the Wall or Room being drawn.
- A shortcut key or a menu entry for the Snap switch, keeping it between runs, and a switch per tool.
- Dots instead of lines, hiding the Grid in the viewport, a Grid colour the Author chooses, and the Grid in an Export (parked in the open questions).
- Leaving the points of hidden or locked Layers out of reach: they come with Layer compositing.

## Further Notes

- **Architecture check**: no new component, contract operation, dependency direction, or restricted crate is needed. ShapeEngine gains Snap, already in its contract; AuthoringManager calls it from a derived-model-component system, as it calls GenerateWalls, so the Editor never depends on ShapeEngine; the Pointer is presentation state the Editor owns in `model`, as the Viewport is, and AuthoringManager only reads it. The architecture's paragraph on `model` names the Viewport as the presentation state the Editor owns; when this change lands, that paragraph names the Pointer beside it.
- The snapped point the Editor uses is the one derived from the Pointer it wrote the frame before, the one on screen when the Author acts, so a point placed is the point shown even when the pointer moves in the frame of the click; a point drag follows the pointer one frame behind, which snapping hides, since the point only jumps between corners.
- A Command carries finished points: tests, the input script, and Projects from any editor place exactly what they say, and snapping exists only between the pointer and the Command. This is why snapping adds no Command and no Rule of Place Element or Edit Element beyond the move by an amount.
- With snapping on, every point the Wall and Room tools place lies on a Grid corner or on another Element's point, so Rooms drawn in one session share their edges exactly; Rooms combine and cut relies on that exact coincidence.
- A point placed freely stays where it is: snapping never moves a point that is not being placed or dragged, and dragging a whole Element that is off the Grid keeps its offset.
