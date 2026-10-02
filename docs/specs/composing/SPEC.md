# Composing

**Commands**: Place Element, Edit Element, Remove Element, Restack, Paint, Set Portal into Wall, Free Portal

## Purpose

The Author builds a Level by putting Elements on its Layers. This capability lets the Author choose an Asset and place it as a Prop where they click, at the size the vendor meant, draw Walls point by point and bend, reshape, and recolour them at any time, move and remove either, and take every step back and forward again, so that composing is a line of small, reversible gestures. The editor opens on a Project to compose on at once.

## User Stories

### Placing Props

1. As an Author, I can open the editor on a new, unsaved Project with one Level and one Layer, so that I can place something at once.
2. As an Author, I can choose an Asset and click on the Level to place a Prop centred where I clicked, so that placing is one gesture.
3. As an Author, I can see a placed Prop at its natural size in Grid cells, so that a vendor's table is as big as the vendor meant.
4. As an Author, I can rely on each new Element landing on top of the Elements already on the Layer, so that what I place or draw last is what I see.
5. As an Author, I can place the same Asset many times, so that a room gets as many barrels as it needs.
6. As an Author, I can place a Prop outside the Bounds, so that the Bounds never get in the way of composing.
7. As an Author, I can press Escape to stop placing and go back to selecting, so that I never place by accident.

### Drawing a Wall

8. As an Author, I can choose the Wall tool from the viewport's tool strip, or press `W`, so that my next clicks draw a Wall instead of placing or selecting, and switching tools never needs the pointer.
9. As an Author, I can click on the Level to add the Wall's points one by one, with a straight segment between each point and the next, so that a Wall is as many clicks as it has corners.
10. As an Author, I can see the Wall I am drawing as a preview, with a segment following the pointer from the last point, so that I see where the next click lands before I make it.
11. As an Author, I can press Enter, or double-click to place the last point, to finish the Wall, so that finishing is one key or one gesture.
12. As an Author, I can keep drawing another Wall after finishing one, so that drawing a dungeon is not a tool change per wall.
13. As an Author, I can press Escape to throw away the Wall I am drawing and go back to selecting, so that a wrong start costs nothing.
14. As an Author, I can rely on a Wall of fewer than two points placing nothing, so that a stray click never leaves an invisible Element behind.
15. As an Author, I can draw a Wall whose points lie outside the Bounds, so that the Bounds never get in the way of composing.
16. As an Author, I can rely on a finished Wall being one undo step, so that undo takes the whole Wall away and redo brings it back whole.
17. As an Author, I can set the thickness and the colour the next Wall is drawn with in the tool's options, so that a thin inner wall and a thick outer wall are both one tool.
18. As an Author, I can see a Wall drawn centred on the line I clicked, as thick as I set and in the colour I set, with rounded corners and ends, so that corners meet cleanly whatever their angle.
19. As an Author, I can choose an Asset in the browser while the Wall tool is active and have the tool give way to placing, and choose the Wall tool while an Asset is chosen and have the Asset dropped, so that one thing is ever happening under my pointer.

### Selecting and moving

20. As an Author, I can click a Prop, or a Wall's line anywhere within its thickness, to select it, with the topmost Element winning when they overlap whatever its kind, so that I always get the one I see.
21. As an Author, I can select a thin Wall at a low zoom with a click a few pixels off its line, so that zooming out never makes a Wall unselectable.
22. As an Author, I can see the selected Wall's points as handles and, for each segment, a handle at its control point or its middle, so that I know what I can drag.
23. As an Author, I can drag a selected Prop, or a selected Wall by its line, to move it and have the whole drag be a single undo step, so that undo takes the Element back to where the drag began, not one pixel back.
24. As an Author, I can drag a point of a selected Wall to move it, and only the two segments meeting at it change, keeping their curves, with the whole drag one undo step, so that fixing one corner never disturbs the rest.
25. As an Author, I can click a point or a control point to select it, and click the Wall's line, a straight segment's middle, or empty space to let it go, so that Delete always acts on what I last picked.

### Curves

26. As an Author, I can drag the handle at the middle of a straight segment to bend the segment into a curve, with the handle becoming the curve's control point, so that a curved wall is one drag on a straight one.
27. As an Author, I can drag a curved segment's control point to reshape the curve, with the whole drag one undo step, so that reshaping is as reversible as moving.
28. As an Author, I can see thin guide lines from a control point to its segment's two points, so that I can tell which segment a control point bends.
29. As an Author, I can select a control point and press Delete to straighten its segment, so that a curve is undone as easily as it was made.

### Adding and removing points

30. As an Author, I can double-click on a Wall's line to add a point there, leaving the Wall's shape as it was on a straight and on a curved segment alike, so that a wall gains a corner where I want one and nothing changes until I drag it.
31. As an Author, I can select a point and press Delete to remove it, with the two segments at it joined into one straight segment, or an end point and have the Wall end at its neighbour, so that a corner goes away in one key.
32. As an Author, I can rely on removing a point from a two-point Wall removing the Wall, as one undo step that brings both back, so that a Wall never ends up as a single point.

### Properties

33. As an Author, I can change the thickness and the colour of a selected Wall in the tool's options, each change one undo step, so that nothing about a Wall is fixed once drawn.
34. As an Author, I am refused a thickness that is zero or less, with the reason and nothing changed, so that a Wall never vanishes through a typo.

### Removing

35. As an Author, I can press Delete to remove the selected Prop, or the selected Wall when no point or control point is selected, so that removing is one key.
36. As an Author, I can rely on an undone removal bringing the Element back exactly as it was, a Wall with every point and curve, including its place in the stacking order, so that undo never reshuffles my Level.

### History and the view

37. As an Author, I can undo and redo with the usual shortcuts every step above, in the order I took them, whether it was a Wall or a Prop, so that I never need a menu to take a step back and history stays one line.
38. As an Author, I can rely on a new action after undoing discarding the undone steps, so that history stays a single line I can reason about.
39. As an Author, I can rely on undo and redo waiting while I am drawing a Wall or dragging, so that a step is never taken back while it is still being made.
40. As an Author, I can rely on selecting an Element, a point, or a control point never being an undo step, so that looking at my work costs me nothing.
41. As an Author, I can pan and zoom the viewport without that showing up in undo, so that looking around never costs me a step.
42. As an Author, I can see a Prop whose image cannot be loaded as a placeholder of the right size while the editor keeps running, so that a deleted or broken file never takes the editor down.
43. As an Author, I am told in the status line when a Command could not be carried out, with nothing changed, so that a failure costs me a retry and never a crash.

## Rules

### Placing and history

**A Project to start with**: the editor opens with a new, unsaved Project holding one Level named `Level 1` with one Layer named `Layer 1`.

**Placed where clicked**: with an Asset chosen, a click on the Level places a Prop of that Asset on the current Layer, centred on the clicked point.

**Natural size**: a Prop's size in Grid cells is its image's pixel size divided by the Grid's pixels per cell, which is 256.
_Why_: the convention of the functional baseline, so its libraries place at the size their vendors meant.

**Placed on top**: a new Element is placed above every Element already on its Layer.

**Placement records a reference**: placing a Prop records in the Project an Asset Reference holding the Asset's name, the Canonical Name of its Asset Folder, the place in that folder it was placed from as the first place it is known to sit, its byte size, its pixel size, and its content fingerprint; placing a second Prop of the same Asset adds no second Asset Reference. Follows from: A Project is device-independent.

**Placement records the folder**: the first Prop placed from an Asset Folder records that folder's Canonical Name and version in the Project; later Props from the same folder add no second record. Follows from: A Missing Asset is always explainable.

**Anywhere on the Level**: an Element may lie outside the Bounds, a Prop placed there and a Wall with points there alike. Follows from: Bounds only decide what is exported.

**Many of the same**: several Props placed from the same Asset are independent Elements, each with its own ElementId. Follows from: No Element kind is limited to one per Level.

**Escape stops placing**: pressing Escape drops the chosen Asset or leaves the Wall tool, discarding a Wall being drawn, and clicks select instead.

**Topmost is selected**: with the Select tool and no Asset chosen, a click selects the topmost Element under the pointer, a Prop by its rectangle and a Wall by its line; a click on empty space clears the selection; selecting an Element or one of its handles is never a history step.

**A drag is one step**: moving an Element by dragging records a single undo step however long the drag, and undo returns the Element to where the drag began.

**Removal is reversible in place**: undoing a Remove Element restores the Element with every property, its ElementId, and its place in the stacking order.

**Identity survives undo**: an Element removed by undoing a Place Element and brought back by redo has the ElementId it had before.

**Redo repeats exactly**: redoing a Place Element, Edit Element, or Remove Element leaves the Level as it was before the undo.

**A new step clears redo**: a Command applied after an undo discards the undone steps.

**One history**: Add Asset Folder, Place Element, Edit Element, and Remove Element are each one undo step, and undo walks back through them in the order they were applied whichever Manager handled them. Follows from: Every Command can be undone.

**View is not a step**: panning and zooming the viewport are not Commands and never appear in the history.

**A failed load is a placeholder**: an Element whose Asset cannot be loaded or decoded, an Element whose Asset is Missing, and an Element of a kind this editor does not know are drawn as the same placeholder of their recorded size, stay on their Layer, and the editor keeps running.

**A failed Command is reported**: a Place Element, Edit Element, or Remove Element that cannot be carried out is answered with the reason, places or changes nothing, and records no history step.

### The Wall

**A Wall is its points**: a Wall is an ordered list of two or more points in Grid cells with a segment between each point and the next, each segment either straight or curved by one control point, a thickness in cells, and an opaque colour.

**Segments are numbered**: the segment from the first point to the second is the first, and so on; moving the Wall, a point, or a control point, bending or straightening a segment, and changing the thickness or the colour change no segment's number. Follows from: A Portal set into a Wall moves with it.

**Curved by one control point**: a curved segment is the quadratic Bézier curve from its first point to its second point with its control point; a segment without a control point is the straight line between its points.

**The box follows the points**: a Wall's position is the centre of the smallest box around its points and control points, and its size is that box grown by half the thickness on every side.

### Placing and editing Walls

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

**Malformed Walls are refused**: a Place Element of a Wall with fewer than two points, a point that is not finite, or a thickness not above zero or not finite, and an Edit Element naming a point or segment the Wall does not have, adding a point not strictly between its segment's two points, putting the Wall, a point, or a control point where it is not finite, or setting such a thickness, are answered with the reason, change nothing, and record no history step.

**Drawn as a stroke**: a Wall is drawn centred on its line, as wide as its thickness, with round joins at its points and round caps at its ends, in its colour, at its place in the stacking order. Follows from: Every Element that shows a surface is drawn with a Material.

### The Wall tool

**Drawing with the Wall tool**: with the Wall tool chosen, from the tool strip or with `W`, each click on the Level adds a point unless it lands within a few pixels of the last one; the Wall in progress is previewed with a segment from its last point to the pointer; Enter finishes it, a double-click finishes it at the point its first click added, a finished Wall of fewer than two points is discarded without a Command, the tool stays chosen for the next Wall, and undo and redo wait while a Wall is being drawn.

**One thing under the pointer**: choosing an Asset leaves the Wall tool and discards a Wall being drawn; choosing the Wall tool drops the chosen Asset and the selection.

**Options follow the selection**: with a Wall selected, the tool's options show its thickness and colour and a change to either is sent as one Edit Element, a typed thickness of zero or less as typed, so that it is refused with the reason; with none selected, they set the thickness and colour the next Wall is drawn with, an eighth of a cell and a dark grey to start, and a typed thickness of zero or less leaves the next Wall's as it was; the thickness offered goes up to sixteen cells, dragged or typed, and a drag never takes it below a hundredth of a cell.

**Hit within the thickness**: a Wall is under the pointer when the pointer is no farther from its line than half its thickness or four screen pixels, whichever is more; with a Wall selected, its handles are hit before any Element.

**Handles of the selected Wall**: the selected Wall shows a handle at each point, at each control point with guide lines to its segment's two points, and at the middle of each straight segment; dragging a point moves it, dragging a control point or a straight segment's middle handle puts that segment's control point under the pointer, and a double-click on the Wall's line adds a point at the nearest place on it.

**Delete acts on what is picked**: a click on a point or a control point selects it, and a click on a straight segment's middle handle, on the Wall's line, or on empty space lets it go, the middle handle being for dragging only; Delete removes the selected point, straightens the selected control point's segment, or, with no handle selected, removes the selected Element.

## Implementation Decisions

- **The new Project** is created at startup by ProjectManager, the owner of the Project lifecycle: named `Untitled`, with one Level, one Layer, a Grid of 256 pixels per cell, and default Bounds of thirty by thirty cells from the origin, which are neither drawn nor edited.
- **Model**: the Project is an entity carrying its Grid, its Bounds, its Asset Reference table, and the resolution table that says where each Asset Reference loads from on this device, which ProjectManager alone writes and no file holds; its Levels are its children, each Level's Layers are that Level's children, and each Layer's Elements are that Layer's children in stacking order, the first drawn first. Every Element carries its kind, its position (its centre in Grid cells, `x` to the right and `y` upwards from the Level's origin), its size in cells, and an ElementId: a stable identity Commands and the history address it by, never the entity handle, which changes whenever an Element is respawned. The Element kind registry holds Prop, drawn as an image, which refers to its Asset by the Project-local row of the Asset Reference table, and Wall, drawn as a stroked path.
- **The Wall component** holds the points, one entry per segment with its optional control point, the thickness, and the colour, without alpha; points and control points are in Grid cells like every position in the model. It is a serialisable component on the Element tier under the stable name `wall`, and it says itself what makes a Wall malformed, so the Commands and the reading of a file refuse the same Walls for the same reasons.
- **The derived shape**: `model` also holds a Wall's derived shape, never saved: its line flattened into points, each tagged with the segment it lies on and the parameter along that segment, and its stroke mesh as vertices in cells with indices and the arc length at each vertex. AuthoringManager derives it through ShapeEngine for every Wall whose points, segments, or thickness changed, once per frame after every Manager has handled its Commands, Undo, and Redo, so a placed, edited, undone, redone, or opened Wall has its shape before anything draws it or hit-tests it; a new colour keeps the shape. The same system sets the Element's position and size from the points. _Why_ derived in the model: the Editor hit-tests it and RenderEngine draws it, and neither may depend on ShapeEngine.
- **ShapeEngine**: GenerateWalls takes a Wall's points, control points, and thickness and returns the derived shape: the line is flattened with the chord never more than a thousandth of a cell off the curve, so that it is at most a pixel off at the highest export resolution, in at most 4096 chords a segment, past which only a control point tens of thousands of cells from its segment goes, and the stroke is tessellated by ShapeEngine's own stroker with round joins and caps at that tolerance, computed with no trigonometry, through correctly rounded square roots only, so that an Export is the same on every machine. SplitWall splits a segment at a parameter strictly between its points, exactly to single precision, into two segments of the same joined shape (a straight segment into two straight ones, a quadratic by subdivision). Both are plain functions over model types.
- **Place Element**: the Editor sends AuthoringManager Apply(Place Element) with the Layer and what to place: a Prop's position and chosen Asset (folder key and place), or a Wall's points, thickness, and colour. For a Prop, AuthoringManager finds the Asset in the added folder's index, asks LibraryAccess to LoadAsset, which reads the file once for its byte size, pixel size, and fingerprint, records the rows, and spawns the Element with a fresh ElementId as the last child of the Layer; ProjectManager resolves the table's new row in the same frame, before anything draws it. A Wall resolves nothing from the library: it is placed by a step of its own that spawns the Element with its Wall component as the last child of the Layer, the same spawning on top as a Prop's. Undo despawns either; redo spawns it again with the same identity, appended, which is on top because every later step was undone first.
- **Edit Element**: an Edit Element carries one change: a position, or on a Wall a point moved (its index and new position), a control point set or unset (the segment and an optional position), a point added (the segment and the parameter along it), a point removed (its index), a thickness, or a colour. The Editor sends a drag as a sequence of Apply(Edit Element) messages, marked as the beginning, continuation, and end of one gesture from press to release, the end sent at the pointer's last position; AuthoringManager records them as one history Group of the generic field-setting command, so the step undoes to where the gesture began. A change marked as single is a step of its own. A position change on a Prop sets its position; on a Wall it sets the whole Wall component, every point and control point translated by the difference between the new position and the current one. Moving a point, setting a control point, and changing the thickness or the colour set that field of the Wall through the same command. Adding and removing a point, the only changes that renumber segments, are a step of their own that keeps the whole Wall as it was, so that undo restores it exactly. Removing a point from a two-point Wall records the removal of the Wall through Remove Element instead. AuthoringManager builds the Wall each change would leave and refuses it for the reason the Wall component gives before anything is recorded. A press that moves the pointer less than a few pixels is a click, not a drag. Undo and redo, from the keys or the menu, wait while a drag or a Wall being drawn is under way. _Why_: the step is still being made.
- **Remove Element**: the generic reflection-snapshot command, extended with the Element's index among its Layer's children; undo restores the entity, a Wall with its points and curves, and rebuilds the Layer's whole order with it at that index, so the Elements above it keep their places.
- **Steps of their own**: Place Element, Remove Element, Add Asset Folder, and adding or removing a point each close any gesture group left open before recording, so none joins a drag.
- **Failures**: an authoring Command that cannot be carried out (a chosen Asset in no added folder or not in its index, a file that cannot be read or is not an image, a target that is not a Layer or an Element, a change only a Wall has sent for another Element, a malformed Wall, a point added at either end of its segment) is answered with a CommandFailed message carrying the reason, and nothing is recorded. An undo or redo that fails is answered with a HistoryFailed message, and the step stays where it was in the history. The Editor shows either in its status line.
- **Undo and redo** are handled by AuthoringManager for the one history, whichever Manager recorded the step: an Undo or Redo message takes back or repeats the most recent step, be it a placement or an added folder.
- **Ordering**: `model` orders every Manager's handling within a frame, Commands before Undo before Redo, so a Command and the Undo sent in the same frame apply in the order the Author gave them whichever Manager handles each.
- **The Viewport** (the cell at the centre of the view, the zoom in logical pixels per cell between 4 and 1024, and the area of the window the Level is shown in) is presentation state in `model`, written only by the Editor and followed by RenderEngine's projection; it starts centred on the origin at 64 pixels per cell; its conversions between cells and screen points are the ones picking and drawing share. _Why_ it lives in `model`: the Editor may not depend on RenderEngine, so the type both use sits with the other shared contracts. Panning is the middle button, Space with the left button, or a plain trackpad scroll; zooming is the wheel, a pinch, or a scroll with Command or Control held, around the pointer.
- **Rendering**: RenderEngine draws each Element as its kind's descriptor in the Element kind registry says, through change detection, placed one unit apart along the camera's axis in stacking order through every Level, so a later Element is drawn over an earlier one. An Element drawn as an image keeps one sprite: a Prop's image is loaded through the `lib://` handle from the folder key and the place, spelled as on disk, that the Project's resolution table gives for its Asset Reference, at the Element's size and position; RenderEngine never looks a folder up by Canonical Name itself. An image that fails to load, a Missing Asset, a row not yet resolved, and an Element of a kind this editor does not know give the same flat coloured placeholder of the Element's size. An Element drawn as a stroked path keeps one mesh from its derived shape's stroke, replaced when the shape changes, with a flat-colour Material shared by every Wall of that colour, blended rather than opaque so that it sorts with the sprites by depth; one without a derived shape yet is not drawn that frame. The projection is a 2D camera with one cell per world unit that follows the Viewport, over a mid grey that a Wall's default dark grey and the editor's dark panels both stand apart from.
- **The tool**: the Editor shows a tool strip over the viewport with Select and Wall and the Wall tool's options, the thickness and the colour. The tool, the Wall being drawn, and the next Wall's thickness and colour are Editor state; the Wall being drawn is drawn by the Editor as a thin preview line through its points with a rubber band to the pointer. Enter sends one Apply(Place Element) with the points, a double-click sends it with the point its first click added, and fewer than two points sends nothing. A thickness or colour changed on the selected Wall by dragging, or while the colour picker is open, is sent as one gesture.
- **Picking and selection** are the Editor's: it maps pointer positions to cells through the Viewport and hit-tests the selected Wall's handles first (points, then control points, then the middles of straight segments), then every Element from the topmost Layer down and the last-drawn Element back, a Prop by its rectangle and a Wall by the distance to the nearest flattened chord of its derived shape against half its thickness or four screen pixels converted to cells, whichever is more. A double-click on a Wall's line inserts a point at the nearest chord's segment and parameter, interpolated along the chord and kept strictly between the segment's points. The selection holds an Element and optionally one of its points or control points; it lives in the Editor, is outlined on the Level, is dropped when its Element is gone, when an Asset is chosen for placing, or when the Wall tool is chosen, and is never in the history. A drag of a handle sends Edit Element as a gesture from press to release, as a Prop drag does, and selects the control point a middle handle's drag creates; a drag of the Wall's line sends position changes the same way. The current Layer is the Project's only Layer.
- **Keyboard**: Escape leaves placing or the Wall tool; `W` chooses the Wall tool; Enter finishes the Wall being drawn; Delete, and Backspace on macOS too, removes or straightens what is selected; the platform's standard undo and redo shortcuts drive history, stated once so that the menu shows exactly the keys that work. Nothing happens while a text field has the keyboard.

## Test seams

The automated seam is a headless Bevy App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, and AuthoringManager over one fixture Asset Folder of images with known pixel sizes, driven by Apply, Undo, and Redo messages and asserted on the World (for Walls, the Wall component, the derived shape, the Element box, the Layer's children, and the history); it has no window and no RenderEngine. `A Project to start with` runs ProjectManager alone over `model`. `Drawn as a stroke` runs on the export seam, a headless App with RenderEngine under Bevy's default plugins without a window, asserting on the pixels of an exported PNG.

- **A Project to start with**: `crates/drs-project-manager/tests/new_project.rs::a_project_to_start_with`
- **Placed where clicked**: `crates/drs-app/tests/composing.rs::placed_where_clicked` (the Command's centring; the mapping from the click to cells is checked by hand)
- **Natural size**: `crates/drs-app/tests/composing.rs::natural_size`
- **Placed on top**: `crates/drs-app/tests/composing.rs::placed_on_top`, `crates/drs-app/tests/walls.rs::walls_are_placed_on_top`
- **Placement records a reference**: `crates/drs-app/tests/composing.rs::placement_records_a_reference`
- **Placement records the folder**: `crates/drs-app/tests/composing.rs::placement_records_the_folder`
- **Anywhere on the Level**: `crates/drs-app/tests/composing.rs::anywhere_on_the_level`, `crates/drs-app/tests/walls.rs::walls_lie_anywhere_on_the_level`
- **Many of the same**: `crates/drs-app/tests/composing.rs::many_of_the_same`
- **Escape stops placing**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Topmost is selected**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **A drag is one step**: `crates/drs-app/tests/composing.rs::a_drag_is_one_step`, `crates/drs-app/tests/walls.rs::a_wall_drag_is_one_step`
- **Removal is reversible in place**: `crates/drs-app/tests/composing.rs::removal_is_reversible_in_place`, `crates/drs-app/tests/walls.rs::wall_removal_is_reversible_in_place`
- **Identity survives undo**: `crates/drs-app/tests/composing.rs::identity_survives_undo`
- **Redo repeats exactly**: `crates/drs-app/tests/composing.rs::redo_repeats_exactly`
- **A new step clears redo**: `crates/drs-app/tests/composing.rs::a_new_step_clears_redo`
- **One history**: `crates/drs-app/tests/composing.rs::one_history`
- **View is not a step**: `crates/drs-app/tests/composing.rs::view_is_not_a_step` (a changed Viewport records nothing; the mapping from the pointer and the wheel to the Viewport is checked by hand)
- **A failed load is a placeholder**: `crates/drs-app/tests/projects.rs::missing_assets_stay`, `crates/drs-app/tests/projects.rs::unknown_kinds_are_kept` (the Element staying on its Layer; the drawing is by hand: no headless seam renders the viewport; verified by driving the editor with the dev-only input script)
- **A failed Command is reported**: `crates/drs-app/tests/composing.rs::a_failed_command_is_reported`
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
- **Drawn as a stroke**: `crates/drs-app/tests/export.rs::a_wall_is_drawn_as_a_stroke` (the colour along a straight Wall, just past its end within half the thickness, and the background beyond half the thickness beside it and at the corner a square cap would fill), `crates/drs-app/tests/export.rs::a_curved_wall_follows_its_curve` (the colour at the curve's middle and the background at the chord's middle); the viewport's drawing is checked by hand
- **Drawing with the Wall tool**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **One thing under the pointer**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Options follow the selection**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script (the refusal of a thickness not above zero is `crates/drs-app/tests/walls.rs::malformed_walls_are_refused`)
- **Hit within the thickness**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Handles of the selected Wall**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script (the Commands a drag and a double-click send are covered by `crates/drs-app/tests/walls.rs::a_handle_drag_is_one_step`, `crates/drs-app/tests/walls.rs::bending_keeps_the_points`, and `crates/drs-app/tests/walls.rs::adding_a_point_keeps_the_shape`)
- **Delete acts on what is picked**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script (the Commands Delete sends are covered by `crates/drs-app/tests/walls.rs::removing_a_point_joins_straight`, `crates/drs-app/tests/walls.rs::bending_keeps_the_points`, and `crates/drs-app/tests/walls.rs::wall_removal_is_reversible_in_place`)

ShapeEngine's own unit tests check what the Wall Rules rest on, as functions of the Engine alone, and are the coverage of no Rule: the flattening tolerance, the mesh covering the thickness, and the exactness of a split (`crates/drs-shape-engine/src/wall.rs::tests::a_straight_segment_flattens_to_its_ends`, `crates/drs-shape-engine/src/wall.rs::tests::the_chord_stays_within_tolerance`, `crates/drs-shape-engine/src/wall.rs::tests::the_mesh_covers_the_thickness`, `crates/drs-shape-engine/src/wall.rs::tests::a_split_keeps_the_shape`).

## Not supported

- Panning, zooming, and selecting are never Commands and never history steps.
- An Element is never addressed by its entity handle across a Command or a history step; only its ElementId is stable.
- Removing a point never approximates the joined segments' curves: the joined segment is always straight.

## Notes

- `A Project to start with` lives here because it is the Project the Author composes on; saving and reopening a Project are the projects capability's.
- Eight Rules (Escape stops placing, Topmost is selected, Drawing with the Wall tool, One thing under the pointer, Options follow the selection, Hit within the thickness, Handles of the selected Wall, Delete acts on what is picked) have no automated test, against the requirement that every Rule has one, and the viewport's drawing of A failed load is a placeholder and Drawn as a stroke is checked only by hand. They are behaviour of the egui interface and of the viewport's rendering, for which no headless seam exists. The accepted deviation is verification by hand, driving the editor with the development-only input script, whose `describe` step logs the Wall tool and every Wall, and which also confirms that a placed Prop and a drawn Wall are shown where and as large as the model says.
- Add Asset Folder changes device state, not Project state, yet it sits in the same history as composing steps because every Manager records into the one history. Undo therefore always reaches a Prop before the folder it came from.
- Adding and removing a point are the only changes that renumber a Wall's segments.
- Snapping Wall points, splitting or joining Walls, closed Walls, cubic curves and arcs, textured or translucent Walls, Walls blocking light or sight, drawing a Wall by dragging, and selecting several Elements or points at once do not exist.
