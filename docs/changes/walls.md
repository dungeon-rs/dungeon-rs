# Walls

**Capabilities**:
- composing: Place Element, Edit Element, Remove Element

## Problem Statement

A dungeon is its walls. Today the Author can place Props and nothing else, so a room is a floor image with no boundary the eye can follow, and the Portals, Rooms, and lighting of later changes have nothing to attach to. The functional baseline draws walls click by click but handles them badly once drawn: moving a point drops or corrupts the doors set into it.

## Solution

A Wall tool in the Editor. The Author clicks points on the Level and the segments between them become a Wall drawn at a thickness in a flat colour; the Wall is finished with Enter or a double-click. Every point stays a handle afterwards: it can be dragged, a point can be added on a segment or taken away, and any segment can be bent into a curve by dragging the handle at its middle and straightened again. A Wall is an Element like a Prop: it sits on a Layer in the stacking order, is selected by clicking its line, is removed with Delete, is exported and saved like everything else, and every step is one undo step. Segments are numbered so that what later changes set into a Wall, a Portal above all, keeps its place along it through every edit.

## User Stories

### Drawing a Wall

1. As an Author, I can choose the Wall tool from the viewport's tool strip, so that my next clicks draw a Wall instead of placing or selecting.
2. As an Author, I can press `W` to choose the Wall tool, so that switching tools never needs the pointer.
3. As an Author, I can click on the Level to add the Wall's points one by one, with a straight segment between each point and the next, so that a Wall is as many clicks as it has corners.
4. As an Author, I can see the Wall I am drawing as a preview, with a segment following the pointer from the last point, so that I see where the next click lands before I make it.
5. As an Author, I can press Enter to finish the Wall with the points I have placed, so that finishing is one key.
6. As an Author, I can double-click to place the last point and finish the Wall at once, so that finishing is one gesture.
7. As an Author, I can keep drawing another Wall after finishing one, so that drawing a dungeon is not a tool change per wall.
8. As an Author, I can press Escape to throw away the Wall I am drawing and go back to selecting, so that a wrong start costs nothing.
9. As an Author, I can rely on a Wall of fewer than two points placing nothing, so that a stray click never leaves an invisible Element behind.
10. As an Author, I can draw a Wall whose points lie outside the Bounds, so that the Bounds never get in the way of composing.
11. As an Author, I can rely on a finished Wall being one undo step, so that undo takes the whole Wall away and redo brings it back whole.
12. As an Author, I can rely on a finished Wall landing on top of the Elements already on the Layer, so that what I draw last is what I see.
13. As an Author, I can set the thickness and the colour the next Wall is drawn with in the tool's options, so that a thin inner wall and a thick outer wall are both one tool.
14. As an Author, I can see a Wall drawn centred on the line I clicked, as thick as I set and in the colour I set, with rounded corners and ends, so that corners meet cleanly whatever their angle.
15. As an Author, I can choose an Asset in the browser while the Wall tool is active and have the tool give way to placing, and choose the Wall tool while an Asset is chosen and have the Asset dropped, so that one thing is ever happening under my pointer.

### Selecting and moving

16. As an Author, I can click on a Wall's line to select it, hit anywhere within its thickness, so that a thick wall is as easy to pick as it looks.
17. As an Author, I can select a thin Wall at a low zoom with a click a few pixels off its line, so that zooming out never makes a Wall unselectable.
18. As an Author, I can rely on the topmost Element under the pointer being selected, Wall or Prop, so that overlapping Elements behave the same whatever their kind.
19. As an Author, I can see the selected Wall's points as handles and, for each segment, a handle at its control point, so that I know what I can drag.
20. As an Author, I can drag a selected Wall by its line to move it whole, with the whole drag one undo step, so that a wall ends up where I let go and undo puts it back where it was.
21. As an Author, I can drag a point of a selected Wall to move it, and only the two segments meeting at it change, so that fixing one corner never disturbs the rest.
22. As an Author, I can rely on a point drag being a single undo step however long it is, so that undo returns the point to where the drag began.
23. As an Author, I can click a point to select it, and click the Wall's line or empty space to deselect it, so that Delete always acts on what I last picked.

### Curves

24. As an Author, I can drag the handle at the middle of a straight segment to bend the segment into a curve, with the handle becoming the curve's control point, so that a curved wall is one drag on a straight one.
25. As an Author, I can drag a curved segment's control point to reshape the curve, with the whole drag one undo step, so that reshaping is as reversible as moving.
26. As an Author, I can see thin guide lines from a control point to its segment's two points, so that I can tell which segment a control point bends.
27. As an Author, I can select a control point and press Delete to straighten its segment, so that a curve is undone as easily as it was made.
28. As an Author, I can rely on the two segments meeting at a point keeping their curves when I move the point, so that moving a corner never flattens a curve.

### Adding and removing points

29. As an Author, I can double-click on a Wall's line to add a point there, so that a straight wall gains a corner where I want one.
30. As an Author, I can rely on adding a point leaving the Wall's shape as it was, on a straight and on a curved segment alike, so that adding a point changes nothing until I drag it.
31. As an Author, I can select a point and press Delete to remove it, with the two segments at it joined into one straight segment, so that a corner goes away in one key.
32. As an Author, I can remove an end point and have the Wall end at its neighbour, so that shortening a wall is one key.
33. As an Author, I can rely on removing a point from a two-point Wall removing the Wall, as one undo step that brings both back, so that a Wall never ends up as a single point.
34. As an Author, I can press Delete with a Wall selected and no point selected to remove the Wall, so that removing is one key.
35. As an Author, I can rely on an undone removal bringing the Wall back exactly as it was, with every point, curve, and its place in the stacking order, so that undo never reshuffles my Level.

### Properties

36. As an Author, I can change the thickness and the colour of a selected Wall in the tool's options, each change one undo step, so that nothing about a Wall is fixed once drawn.
37. As an Author, I am refused a thickness that is zero or less, with nothing changed, so that a Wall never vanishes through a typo.

### Export and saving

38. As an Author, I can export a Level with Walls and see each Wall in the image at its thickness and colour, above and below Props as on screen, so that what I see is what I export.
39. As an Author, I can rely on a Wall with a point outside the Bounds being cut at the edge of the Export, so that the image is exactly the Bounds and nothing else.
40. As an Author, I can save a Project with Walls and reopen it with every Wall's points, curves, thickness, and colour as they were, so that my dungeon survives closing the editor.
41. As an Author sharing a Project with a collaborator whose editor does not know Walls, I can rely on their editor keeping my Walls as placeholders of the right extent and saving them back untouched, so that a round trip through an older editor never loses a wall.

### History

42. As an Author, I can undo and redo every step above with the usual shortcuts, in the order I took them, whether it was a Wall or a Prop, so that history stays one line.
43. As an Author, I can rely on undo and redo waiting while I am drawing a Wall or dragging a handle, so that a step is never taken back while it is still being made.
44. As an Author, I can rely on selecting a Wall, a point, or a control point never being an undo step, so that looking at my work costs me nothing.

## Rules

### The Wall

**A Wall is its points**: a Wall is an ordered list of two or more points in Grid cells with a segment between each point and the next, each segment either straight or curved by one control point, a thickness in cells, and an opaque colour.

**Segments are numbered**: the segment from the first point to the second is the first, and so on; moving the Wall, a point, or a control point, bending or straightening a segment, and changing the thickness or the colour change no segment's number. Follows from: A Portal set into a Wall moves with it.

**Curved by one control point**: a curved segment is the quadratic Bézier curve from its first point to its second point with its control point; a segment without a control point is the straight line between its points.

**The box follows the points**: a Wall's position is the centre of the smallest box around its points and control points, and its size is that box grown by half the thickness on every side.

### Placing and editing

**Placed as one step**: a Place Element of a Wall puts on the given Layer a Wall with the given points, thickness, and colour, every segment straight, as one history step that undo takes away whole and redo brings back whole. Follows from: Every Command can be undone.

**Moving the Wall moves every point**: an Edit Element that changes a Wall's position moves every point and control point by the same amount.

**A point moves alone**: moving a point changes that point and nothing else; every other point and every control point stays where it was, so the two segments meeting at the point keep their curves. Follows from: Nothing is fixed at creation.

**A handle drag is one step**: dragging a point or a control point records a single undo step however long the drag, and undo returns it to where the drag began. Follows from: Every Command can be undone.

**Bending keeps the points**: setting a segment's control point changes that control point only, and unsetting it makes the segment straight; no point moves. Follows from: Nothing is fixed at creation.

**Adding a point keeps the shape**: a point added on a segment splits it into two segments whose joined curve is the one the segment had, straight on a straight segment and curved on a curved one; the segments after it are numbered one higher. Follows from: Nothing is fixed at creation.

**Removing a point joins straight**: removing a point joins the two segments at it into one straight segment; removing an end point removes the first or the last segment; the segments after a removed point are numbered one lower. Follows from: Nothing is fixed at creation.
_Why_: two quadratic curves cannot in general be joined into one, and a predictable straight segment beats an approximation the Author did not ask for.

**Two points or none**: removing a point from a Wall of two points removes the Wall, as one history step that undoes to the Wall with both points.

**Properties stay editable**: a Wall's thickness and colour are each changed through Edit Element, every change a step of its own. Follows from: Nothing is fixed at creation.

**Malformed Walls are refused**: a Place Element of a Wall with fewer than two points or a thickness not above zero, and an Edit Element naming a point or segment the Wall does not have or setting a thickness not above zero, are answered with the reason, change nothing, and record no history step.

### Drawing

**Drawn as a stroke**: a Wall is drawn centred on its line, as wide as its thickness, with round joins at its points and round caps at its ends, in its colour, at its place in the stacking order. Follows from: Every Element that shows a surface is drawn with a Material.

**Exported as drawn**: a Wall appears in the Export as it is drawn in the editor, above the Elements before it and below those after it, and only where it lies inside the Bounds. Follows from: Stacking order.

**Saved as its points**: a saved Wall holds its points, which segments are curved and their control points, its thickness, and its colour, and reopens the same; an editor that does not know the Wall kind keeps it as a placeholder of its size and writes it back unchanged. Follows from: References are never dropped.

### The Wall tool

**Drawing with the Wall tool**: with the Wall tool chosen, from the tool strip or with `W`, each click on the Level adds a point unless it lands within a few pixels of the last one; the Wall in progress is previewed with a segment from its last point to the pointer; Enter finishes it, a double-click finishes it at the point its first click added, a finished Wall of fewer than two points is discarded without a Command, the tool stays chosen for the next Wall, and undo and redo wait while a Wall is being drawn.

**One thing under the pointer**: choosing an Asset leaves the Wall tool and discards a Wall being drawn; choosing the Wall tool drops the chosen Asset and the selection.

**Options follow the selection**: with a Wall selected, the tool's options show its thickness and colour and a change to either is sent as one Edit Element; with none selected, they set the thickness and colour the next Wall is drawn with, an eighth of a cell and a dark grey to start.

**Hit within the thickness**: a Wall is under the pointer when the pointer is no farther from its line than half its thickness or four screen pixels, whichever is more; with a Wall selected, its handles are hit before any Element.

**Handles of the selected Wall**: the selected Wall shows a handle at each point, at each control point with guide lines to its segment's two points, and at the middle of each straight segment; dragging a point moves it, dragging a control point or a straight segment's middle handle puts that segment's control point under the pointer, and a double-click on the Wall's line adds a point at the nearest place on it.

**Delete acts on what is picked**: a click on a handle selects it, and a click on the Wall's line or on empty space lets it go; Delete removes the selected point, straightens the selected control point's segment, or, with no handle selected, removes the selected Wall.

## Changes to existing behaviour

- composing — **Placed on top**: modified to "a new Element is placed above every Element already on its Layer", because a finished Wall lands on top as a Prop does.
- composing — **Anywhere on the Level**: modified to "an Element may lie outside the Bounds, a Prop placed there and a Wall with points there alike", because a Wall's points are as free as a Prop's position.
- composing — **Topmost is selected**: modified to "with the Select tool and no Asset chosen, a click selects the topmost Element under the pointer, a Prop by its rectangle and a Wall by its line; a click on empty space clears the selection; selecting an Element or one of its handles is never a history step", because Walls are selected by their line and have handles.
- composing — **A drag is one step**: modified to "moving an Element by dragging records a single undo step however long the drag, and undo returns the Element to where the drag began", because a Wall is moved whole by its line.
- composing — **Removal is reversible in place**: modified to "undoing a Remove Element restores the Element with every property, its ElementId, and its place in the stacking order", because it holds for Walls as for Props.
- composing — **Escape stops placing**: modified to "pressing Escape drops the chosen Asset or leaves the Wall tool, discarding a Wall being drawn, and clicks select instead", because the Wall tool is a second way of not selecting.

## Implementation Decisions

The technology the architecture fixes (kurbo inside ShapeEngine, Portal anchoring by segment index and parameter, the Element kind registry, RenderEngine's generic mesh-plus-Material path, the generic field-setting history command) is used as written there and not restated.

- **The Wall descriptor**: `model` gains the Wall kind in the Element kind registry, whose descriptor says it is drawn as a stroked path, and the Wall component: the points, one entry per segment holding its optional control point, the thickness, and the colour. It is a serialisable component at version one on the Element tier, under the stable name `wall`. The colour has no alpha. Points and control points are in Grid cells like every position in the model.
- **The derived shape**: `model` also holds a Wall's derived shape, never saved: its line flattened into points, each tagged with the segment it lies on and the parameter along that segment, and its stroke mesh as vertices in cells with indices and the arc length at each vertex. AuthoringManager derives it through ShapeEngine for every Wall whose component changed, once per frame after every Manager has handled its Commands, Undo, and Redo, so a placed, edited, undone, redone, or opened Wall has its shape before anything draws it or hit-tests it; the same system sets the Element's position and size from the points (the box follows the points). _Why_ derived in the model: the Editor hit-tests it and RenderEngine draws it, and neither may depend on ShapeEngine; the resolution table is the precedent for a derived component one Manager writes and others read.
- **ShapeEngine**: GenerateWalls takes a Wall's points, control points, and thickness and returns the derived shape: the line is flattened with the chord never more than a thousandth of a cell off the curve, so that it is at most a pixel off at the highest export resolution, and the stroke is tessellated by ShapeEngine's own stroker with round joins and caps at that tolerance, every angle computed through the deterministic maths functions the architecture prescribes. SplitWall splits a segment at a parameter, exactly, into two segments of the same joined shape (a straight segment into two straight ones, a quadratic by subdivision). Both are plain functions over model types; the Engine defines no message and reads no file.
- **Place Element**: the Apply(Place Element) message gains a Wall payload beside the chosen Asset: the Layer, the points, and the thickness and colour. AuthoringManager spawns the Element with a fresh ElementId as the last child of the Layer with the Wall component and the common Element component, through the same reversible placement as a Prop, resolving nothing from the library. Fewer than two points, or a thickness not above zero, is a CommandFailed.
- **Edit Element**: the Element change gains, beside the position, a point moved (its index and new position), a control point set or unset (the segment and an optional position), a point added (the segment and the parameter along it), a point removed (its index), a thickness, and a colour. Moving a point, setting a control point, and changing the thickness or colour go through the generic field-setting command by reflect path, so the gesture grouping of a drag is exactly the Prop's. Adding and removing a point are a reversible command of their own that remembers what it dropped (the control points of the joined segments) so that revert restores the Wall exactly. Removing a point from a two-point Wall records a group holding the removal of the Wall, through the existing Remove Element command. A position change on a Wall sets the whole point list translated by the difference between the new position and the current one. A point or segment the Wall does not have, and a thickness not above zero, are a CommandFailed.
- **Portal-readiness**: the Commands that add and remove a point are the only ones that renumber segments, so they are where the next change remaps the Portals anchored on the Wall, in the same history group; this change stores nothing about Portals.
- **Rendering**: RenderEngine keeps one mesh with a flat-colour Material per Wall from the derived shape through change detection, blended rather than opaque so that it sorts with the sprites, at the Wall's stacking depth like a Prop's sprite, in the viewport and in RenderRegion alike; the mesh is replaced when the shape changes. An Element of the Wall kind without a derived shape yet is not drawn that frame. Capturing a region waits for nothing new: a mesh has no Asset to load.
- **The tool**: the Editor gains a tool strip over the viewport with Select and Wall, `W` choosing Wall and Escape returning to Select; choosing an Asset leaves the tool, choosing the tool drops the Asset and the selection. The Wall in progress is Editor state, drawn by the Editor as a thin preview line through its points with a rubber band to the pointer; a click adds a point unless it lands within a few pixels of the last one, Enter sends one Apply(Place Element) with the points, a double-click sends it with the point its first click added, Escape discards it, and fewer than two points sends nothing. The options strip shows the thickness and the colour: with a Wall selected they show its values and a change sends Apply(Edit Element) as a single step; otherwise they are the defaults for the next Wall. Undo and redo wait while a Wall is being drawn, as they do during a drag.
- **Picking**: the Editor hit-tests the selected Wall's handles first (points, then control points, then the midpoints of straight segments), then every Element from the topmost Layer down and the last-drawn Element back, a Prop by its rectangle and a Wall by the distance to the nearest flattened chord of its derived shape against half its thickness or four screen pixels converted to cells, whichever is more. A double-click on a Wall's line inserts a point at the nearest chord's segment and parameter, interpolated along the chord. The selection holds an Element and optionally one of its handles; Delete removes the selected point, straightens the selected control point's segment, or removes the Element. A drag of a handle sends Edit Element as a gesture from press to release, as a Prop drag does; a drag of the Wall's line sends position changes the same way.
- **Project file**: ProjectAccess changes nothing: the Wall component travels through the serialisation registry like the Prop, and an editor without the kind keeps its envelope on the Element and writes it back.

## Testing

- **Composing seam**: a headless App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, and AuthoringManager, with no window and no RenderEngine, driven by Apply, Undo, and Redo messages and asserted on the Wall component, the derived shape, the Element box, the Layer's children, and the history.
  - **A Wall is its points**: `crates/drs-app/tests/walls.rs::a_wall_is_its_points`
  - **Segments are numbered**: `crates/drs-app/tests/walls.rs::segments_are_numbered`
  - **Curved by one control point**: `crates/drs-app/tests/walls.rs::curved_by_one_control_point` (the derived line's point halfway along the segment)
  - **The box follows the points**: `crates/drs-app/tests/walls.rs::the_box_follows_the_points`
  - **Placed as one step**: `crates/drs-app/tests/walls.rs::placed_as_one_step`
  - **Moving the Wall moves every point**: `crates/drs-app/tests/walls.rs::moving_the_wall_moves_every_point`
  - **A point moves alone**: `crates/drs-app/tests/walls.rs::a_point_moves_alone`
  - **A handle drag is one step**: `crates/drs-app/tests/walls.rs::a_handle_drag_is_one_step`
  - **Bending keeps the points**: `crates/drs-app/tests/walls.rs::bending_keeps_the_points`
  - **Adding a point keeps the shape**: `crates/drs-app/tests/walls.rs::adding_a_point_keeps_the_shape`
  - **Removing a point joins straight**: `crates/drs-app/tests/walls.rs::removing_a_point_joins_straight`
  - **Two points or none**: `crates/drs-app/tests/walls.rs::two_points_or_none`
  - **Properties stay editable**: `crates/drs-app/tests/walls.rs::properties_stay_editable`
  - **Malformed Walls are refused**: `crates/drs-app/tests/walls.rs::malformed_walls_are_refused`
  - The Wall cases of the modified composing Rules: **Placed on top** `crates/drs-app/tests/walls.rs::walls_are_placed_on_top`, **Anywhere on the Level** `crates/drs-app/tests/walls.rs::walls_lie_anywhere_on_the_level`, **A drag is one step** `crates/drs-app/tests/walls.rs::a_wall_drag_is_one_step`, **Removal is reversible in place** `crates/drs-app/tests/walls.rs::wall_removal_is_reversible_in_place`.
- **Offscreen export seam**: the existing headless App with RenderEngine under Bevy's default plugins without a window, exporting to a temporary PNG and asserting pixels.
  - **Drawn as a stroke**: `crates/drs-app/tests/export.rs::a_wall_is_drawn_as_a_stroke` (the colour along a straight Wall, just past its end within half the thickness, and the background beyond half the thickness beside it and at the corner a square cap would fill), `crates/drs-app/tests/export.rs::a_curved_wall_follows_its_curve` (the colour at the curve's middle and the background at the chord's middle)
  - **Exported as drawn**: `crates/drs-app/tests/export.rs::walls_stack_with_props` (a Wall over one Prop and under another), `crates/drs-app/tests/export.rs::a_wall_is_clipped_at_the_edge` (a Wall with a point outside the Bounds leaving no trace past the edge)
- **Projects seam**: the existing headless App saving and reopening a Project with a straight and a curved Wall, and a second App whose registry lacks the Wall kind opening and saving the same file.
  - **Saved as its points**: `crates/drs-app/tests/projects.rs::walls_are_saved_as_their_points`, `crates/drs-app/tests/projects.rs::unknown_walls_round_trip`
- **By hand**: **Drawing with the Wall tool**, **One thing under the pointer**, **Options follow the selection**, **Hit within the thickness**, **Handles of the selected Wall**, and **Delete acts on what is picked**, with the modified **Topmost is selected** and **Escape stops placing**: no automated seam for the egui interface, the accepted deviation of the composing spec; verified by driving the editor with the development-only input script, which also confirms that a Wall is drawn in the viewport where and as thick as the model says.
- ShapeEngine's own unit tests check what the Rules above rest on, as functions of the Engine alone, and are the coverage of no Rule: the flattening tolerance, the mesh covering the thickness, and the exactness of a split (`crates/drs-shape-engine/src/wall.rs::tests::a_straight_segment_flattens_to_its_ends`, `crates/drs-shape-engine/src/wall.rs::tests::the_chord_stays_within_tolerance`, `crates/drs-shape-engine/src/wall.rs::tests::the_mesh_covers_the_thickness`, `crates/drs-shape-engine/src/wall.rs::tests::a_split_keeps_the_shape`).

## Out of Scope

- Snapping Wall points to the Grid or to other Walls (ShapeEngine's Snap).
- Portals set into Walls, and remapping anything anchored on a Wall when its points change: the next change, which this one leaves segment numbers and parameters for.
- Walls generated from Rooms and Caves, splitting a Wall into two Walls, and joining two Walls into one.
- Cubic curves, arcs, and smoothing a whole Wall through its points.
- Textured Walls: a Material other than the flat colour, Shaders, and a texture repeated along the Wall (the stroke's arc length is in the mesh for later).
- Walls blocking light or sight in the editor: lighting is deferred.
- A property panel beyond the two options of the tool; selecting several Elements or several points at once.
- Drawing a Wall by dragging, or curving a segment while drawing.
- A closed Wall whose last point joins its first: Rooms bring closed outlines.
- A translucent Wall colour: Layer opacity comes with Layer compositing.

## Further Notes

- **Architecture check**: no new component, dependency direction, or restricted crate is needed. ShapeEngine's contract names GenerateWalls for drawn Walls (and, later, Walls generated from Rooms and Caves) and SplitWall beside it; the derived shape in `model` follows the resolution table's precedent of a derived component one Manager writes and others read.
- A Wall's mesh costs a pipeline compilation on the first frame it appears (once per process); the offscreen seam compiles pipelines synchronously, and in the editor the Wall has been on screen for frames before an Export starts.
- Flattening at a thousandth of a cell makes a long curved Wall a few thousand vertices; the shape is derived only when the Wall changes, so this is measured, not feared; a coarser tolerance for the viewport is a later refinement inside ShapeEngine if profiling asks for it.
