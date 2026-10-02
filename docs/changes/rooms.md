# Rooms

**Capabilities**:
- composing: Place Element, Edit Element, Remove Element, Set Portal into Wall, Free Portal

## Problem Statement

A dungeon is mostly rooms, and today each one is a Wall drawn round by hand with nothing inside it: the floor is whatever lies below, the last corner never quite meets the first, and reshaping a room means moving every Wall around it. The functional baseline draws a room as a floor with walls around it in one gesture, but its walls are fixed to the shape it was drawn with, and the doors set into them are dropped or corrupted when the room is edited.

## Solution

A Room tool in the Editor. The Author clicks the corners of a Room on the Level and closes it by clicking the first corner again or pressing Enter, or drags a rectangle; the Room is drawn at once as a floor in a flat colour with its Walls stroked around the outline, closed and without ends. A Room is edited exactly as a Wall is: its points are handles, a point can be added on an edge or taken away, and any edge bends into a curve by its middle handle, the floor and the Walls following every change. The whole Room is one Element: it is selected by its floor or its Wall, moved whole, stacked, removed with Delete, exported, and saved like any other, every step one undo step. Portals are set into a Room's Walls as into a drawn Wall, with the Portal tool's marker reaching them, and they stay anchored to the Room's edges through every edit. Each Room stands on its own in this change: two Rooms that overlap are both drawn, in stacking order.

## User Stories

### Drawing a Room

1. As an Author, I can choose the Room tool from the viewport's tool strip, so that my next clicks draw a Room instead of placing, selecting, or drawing a Wall.
2. As an Author, I can press `R` to choose the Room tool, so that switching tools never needs the pointer.
3. As an Author, I can click on the Level to add the Room's points one by one, with a straight edge between each point and the next, so that a Room is as many clicks as it has corners.
4. As an Author, I can see the outline I am drawing as a preview, with a line from the last point to the pointer and another from the pointer back to the first point, so that I see the Room the next click would make.
5. As an Author, I can click the first point again to close the outline and place the Room, so that drawing a Room ends where it began.
6. As an Author, I can press Enter to close the outline with the points I have placed, so that closing is one key.
7. As an Author, I can rely on an outline of fewer than three points placing nothing when I press Enter, and on a click on the first point doing nothing until there are three, so that a stray click never leaves a Room with no floor.
8. As an Author, I can press Escape to throw away the outline I am drawing and go back to selecting, so that a wrong start costs nothing.
9. As an Author, I can drag on the Level with the Room tool to draw a rectangular Room from where I pressed to where I let go, so that the most common Room is one gesture.
10. As an Author, I can see the rectangle as a preview while I drag, so that I see the Room before I let go.
11. As an Author, I can rely on a drag that is narrower or lower than a few pixels placing nothing, so that a shaky click never leaves a sliver of a Room.
12. As an Author, I can rely on a Room drawn as a rectangle being an outline like any other, so that dragging one corner later moves only that corner.
13. As an Author, I can keep drawing another Room after placing one, so that laying out a dungeon is not a tool change per Room.
14. As an Author, I can draw a Room whose points lie outside the Bounds, so that the Bounds never get in the way of composing.
15. As an Author, I can rely on a placed Room being one undo step, so that undo takes the whole Room away, floor and Walls, and redo brings it back whole.
16. As an Author, I can rely on a placed Room landing on top of the Elements already on the Layer, so that what I draw last is what I see.
17. As an Author, I can set the wall thickness, the wall colour, and the floor colour the next Room is drawn with in the tool's options, so that a cellar and a great hall are both one tool.
18. As an Author, I can choose an Asset, the Wall tool, or the Portal tool while drawing a Room and have the outline thrown away, and choose the Room tool while drawing a Wall and have the Wall thrown away, so that one thing is ever happening under my pointer.

### How a Room looks

19. As an Author, I can see a Room's floor fill the whole outline in the floor colour, so that a Room reads as a place to stand.
20. As an Author, I can see a Room's Walls drawn centred on its outline, as thick as I set and in the wall colour, rounded at every corner including the one where the outline closes, so that the last corner meets the first as cleanly as any other.
21. As an Author, I can see the Walls drawn over the floor, so that the floor never covers the inner half of a Wall.
22. As an Author, I can see the floor and the Walls follow a curved edge, so that a round tower is drawn round.
23. As an Author, I can drag a point so that two edges cross and still see the Room drawn, its floor filling every part the outline goes around, so that an awkward drag never makes a Room vanish.
24. As an Author, I can draw two Rooms that overlap and see both, each with its floor and Walls, the later one drawn over the earlier, so that overlapping is never refused.
25. As an Author, I can see an Element placed after a Room drawn over its floor and its Walls, and one placed before it hidden under them, so that a Room stacks like any other Element.

### Selecting and moving

26. As an Author, I can click a Room's floor to select it, so that a large Room is easy to pick.
27. As an Author, I can click a Room's Wall to select the Room, hit anywhere within the Wall's thickness, including its outer half, so that a Room is picked where it looks to be.
28. As an Author, I can rely on the topmost Element under the pointer being selected, so that a Prop on a floor is picked before the Room beneath it.
29. As an Author, I can see the selected Room's points as handles and, for each edge, a handle at its control point or at its middle, including the edge from the last point back to the first, so that I know what I can drag.
30. As an Author, I can drag a selected Room by its floor or its Wall to move it whole, as one undo step, so that a Room ends up where I let go and undo puts it back.
31. As an Author, I can drag a point of a selected Room to move it, and only the two edges meeting at it change, so that fixing one corner never disturbs the rest.
32. As an Author, I can rely on a point drag being a single undo step however long it is, so that undo returns the point to where the drag began.

### Curves, points, and properties

33. As an Author, I can drag the handle at the middle of a straight edge to bend it into a curve, and drag its control point to reshape it, each drag one undo step, so that a curved Room is one drag on a straight one.
34. As an Author, I can select a control point and press Delete to straighten its edge, so that a curve is undone as easily as it was made.
35. As an Author, I can rely on the two edges meeting at a point keeping their curves when I move the point, so that moving a corner never flattens a curve.
36. As an Author, I can double-click on a Room's Wall to add a point there, without changing the Room's shape, on a straight edge and on a curved one, so that a Room gains a corner where I want one.
37. As an Author, I can select a point and press Delete to remove it, the two edges at it joined into one straight edge, the first point included, so that a corner goes away in one key.
38. As an Author, I can rely on removing a point from a three-point Room removing the Room, as one undo step that brings it back with its points, so that a Room never ends up as a line.
39. As an Author, I can press Delete with a Room selected and no handle selected to remove the Room, so that removing is one key.
40. As an Author, I can rely on an undone removal bringing the Room back exactly as it was, with every point, curve, colour, and its place in the stacking order, so that undo never reshuffles my Level.
41. As an Author, I can change the wall thickness, the wall colour, and the floor colour of a selected Room in the tool's options, each change one undo step, so that nothing about a Room is fixed once drawn.
42. As an Author, I am refused a wall thickness that is zero or less, with nothing changed, so that a Room's Walls never vanish through a typo.

### Portals in a Room's Walls

43. As an Author, I can hover a Room's Wall with the Portal tool and see the marker snap to it, as it does to a drawn Wall, so that doors go into Rooms the way they go into Walls.
44. As an Author, I can click while the marker shows on a Room's Wall to place the Portal set into it, facing the side I pointed to, so that a Room's door is one gesture.
45. As an Author, I can see a Room's Wall left out along a Portal's width, the floor still reaching the outline there, so that the doorway opens onto the Room's floor.
46. As an Author, I can place a Portal across the point where a Room's outline closes and see the gap follow the Wall around that corner, so that no corner of a Room is special.
47. As an Author, I can slide a Portal along a Room's Wall by dragging it, round every corner and past the point where the outline closes, so that a door travels the whole Room.
48. As an Author, I can set a freestanding Portal into the nearest Room's Wall with `F`, and free a Portal from a Room's Wall where it stands, so that a Room's doors behave like any Wall's.
49. As an Author, I can move a Room, drag its points, bend its edges, or change its thickness and colours, and see its Portals move with the Walls, each keeping its place along its edge, so that reshaping a Room never leaves a door hanging.
50. As an Author, I can add a point to a Room, even on the edge a Portal sits on, and see every Portal stay exactly where it was, so that adding a corner never moves a door.
51. As an Author, I can remove a point of a Room and see the Portals on the two edges it joined carried onto the joined edge, and a Portal covering that point removed and be told, so that a door is never left floating over a corner that no longer exists.
52. As an Author, I can remove a Room and see the Portals set into it removed with it, and be told how many, so that no door is left standing in no wall.
53. As an Author, I can undo any of these removals and get the Room and every Portal back exactly, set where they were, so that undo restores the doors with the Room.

### Export and saving

54. As an Author, I can export a Level with Rooms and see each floor and its Walls in the image as in the editor, its Portals' gaps included, so that what I see is what I export.
55. As an Author, I can rely on a Room with points outside the Bounds being cut at the edge of the Export, so that the image is exactly the Bounds.
56. As an Author, I can save a Project with Rooms and reopen it with every Room's points, curves, thickness, and colours as they were, and every Portal set into the same place of the same Room, so that my dungeon survives closing the editor.
57. As an Author sharing a Project with a collaborator whose editor knows Walls and Portals but not Rooms, I can rely on their editor keeping my Rooms as placeholders and my Portals standing where they were saved, and saving both back untouched, so that a round trip through an older editor never loses a Room or its doors.

### History

58. As an Author, I can undo and redo every step above with the usual shortcuts, in the order I took them, whatever kind of Element it was, so that history stays one line.
59. As an Author, I can rely on undo and redo waiting while I am drawing a Room or dragging a handle, so that a step is never taken back while it is still being made.
60. As an Author, I can rely on selecting a Room or one of its handles never being an undo step, so that looking at my work costs me nothing.

## Rules

### The Room

**A Room is its outline**: a Room is an ordered list of three or more points in Grid cells with an edge from each point to the next and from the last point back to the first, each edge either straight or curved by one control point, a wall thickness in cells, an opaque wall colour, and an opaque floor colour.

**Edges are numbered**: the edge from the first point to the second is the first, and so on, the edge from the last point back to the first being the last; moving the Room, a point, or a control point, bending or straightening an edge, and changing the wall thickness or either colour change no edge's number. Follows from: A Portal set into a Wall moves with it.

**An edge curves by one control point**: a curved edge is the quadratic Bézier curve from its first point to its second point with its control point; an edge without a control point is the straight line between its points.

**The box follows the outline**: a Room's position is the centre of the smallest box around its points and control points, and its size is that box grown by half the wall thickness on every side.

### Placing and editing

**A Room is placed as one step**: a Place Element of a Room puts on the given Layer a Room with the given points, wall thickness, wall colour, and floor colour, every edge straight, as one history step that undo takes away whole and redo brings back whole. Follows from: Every Command can be undone.

**Moving the Room moves every point**: an Edit Element that changes a Room's position moves every point and control point by the same amount.

**A Room's point moves alone**: moving a point of a Room changes that point and nothing else; every other point and every control point stays where it was, so the two edges meeting at the point keep their curves. Follows from: Nothing is fixed at creation.

**A Room's handle drag is one step**: dragging a point or a control point of a Room records a single undo step however long the drag, and undo returns it to where the drag began. Follows from: Every Command can be undone.

**Bending an edge keeps the points**: setting an edge's control point changes that control point only, and unsetting it makes the edge straight; no point moves. Follows from: Nothing is fixed at creation.

**Adding a point keeps the Room's shape**: a point added on an edge at a parameter strictly between 0 and 1 splits it into two edges whose joined curve is the one the edge had, straight on a straight edge and curved on a curved one; the new point comes after the edge's first point, and every later point and edge is numbered one higher, so a point added on the last edge becomes the last point. Follows from: Nothing is fixed at creation.

**Removing a point joins the Room straight**: removing a point joins the edge that ends at it and the edge that starts at it into one straight edge from the point before it to the point after it, which takes the place of the edge that ended at it; every later point and edge is numbered one lower, so removing the first point makes the joined edge the last. Follows from: Nothing is fixed at creation.
_Why_: two quadratic curves cannot in general be joined into one, as on a Wall.

**Three points or none**: removing a point from a Room of three points removes the Room and the Portals set into it, as one history step that undoes to the Room with its three points and its Portals.

**Room properties stay editable**: a Room's wall thickness, wall colour, and floor colour are each changed through Edit Element, every change a step of its own. Follows from: Nothing is fixed at creation.

**Malformed Rooms are refused**: a Place Element of a Room with fewer than three points or a wall thickness not above zero, and an Edit Element naming a point or edge the Room does not have, setting a wall thickness not above zero, or adding a point at a parameter not strictly between 0 and 1, are answered with the reason, change nothing, and record no history step.

### Drawing

**The floor fills the outline**: a Room's floor covers every place its outline winds around, including every part of an outline whose edges cross, up to the outline's line, in the floor colour. Follows from: Every Element that shows a surface is drawn with a Material.

**The Walls close around the floor**: a Room's Walls are its outline drawn centred on the line, as wide as the wall thickness, with a round join at every point, the first included, and no caps, in the wall colour, except along the stretches its Portals cover, where they end squarely across the line. Follows from: Every Element that shows a surface is drawn with a Material.

**The floor lies under its Walls**: a Room is drawn at its place in the stacking order, its floor first and its Walls over it; every Element before the Room is drawn under both, and every Element after it over both. Follows from: Stacking order.

**Rooms export as drawn**: a Room's floor and Walls appear in the Export as they are drawn in the editor, the stretches its Portals leave out included, and only where they lie inside the Bounds. Follows from: Bounds only decide what is exported.

**Saved as its outline**: a saved Room holds its points, which edges are curved and their control points, its wall thickness, its wall colour, and its floor colour, and reopens the same; an editor that does not know the Room kind keeps it as a placeholder of its size and writes it back unchanged. Follows from: References are never dropped.

### Portals in a Room's Walls

**Portals set into Rooms**: a Portal can be set into a Room's Walls, anchored by the Room's ElementId, one of its edges, a parameter along that edge from 0 to 1, and a side, left or right of the edge's direction from its first point to its second; it then stands on the outline, faces its side, is placed, set, freed, slid, flipped, resized, and removed exactly as a Portal set into a Wall, with the Room's edges in place of the Wall's segments. Follows from: A Portal set into a Wall moves with it.

**A stretch runs round the Room**: a Portal set into a Room's Walls covers the stretch of the outline that reaches half its width either way from its centre, measured along the outline, across every point including the first, and never stopped by an end; a Portal wider than the whole outline covers all of it.

**Moves with its Room**: moving a Room, moving a point, setting or unsetting a control point, and changing the wall thickness or either colour change no Portal's edge, parameter, or side; each Portal set into the Room stands at its parameter on its edge as the edge now is. Follows from: A Portal set into a Wall moves with it.

**Adding a point keeps a Room's Portals in place**: adding a point on edge *k* at parameter *s* moves a Portal on edge *k* at a parameter *t* below *s* to parameter *t* / *s* on edge *k*, one at or above *s* to parameter (*t* − *s*) / (1 − *s*) on edge *k* + 1, and every Portal on a later edge one edge on, in the same history step, so no Portal moves on the Level. Follows from: A Portal set into a Wall moves with it.

**Removing a point carries a Room's Portals**: removing a point moves each Portal on the two edges it joins whose stretch does not cover it onto the joined edge, at the parameter equal to the share of the two edges' combined length that lay before the Portal's centre, and every Portal on a later edge one edge back, all in the same history step. Follows from: A Portal set into a Wall moves with it.

**Gone with its part of the Room**: a Portal whose stretch covers a point being removed from its Room, and every Portal set into a Room being removed, by Remove Element or by removing a point of a three-point Room, are removed in the same history step, which undo restores whole; the Author is told in the status line how many Portals were removed. Follows from: A Portal set into a Wall moves with it.

### The Room tool

**Drawing with the Room tool**: with the Room tool chosen, from the tool strip or with `R`, each click on the Level adds a point unless it lands within a few pixels of the last one; the outline in progress is previewed through its points with a line from the last point to the pointer and from the pointer to the first point; a click within a few pixels of the first point with three or more points placed, or Enter, closes it and sends one Place Element of a Room; Enter with fewer than three points, and a click on the first point with fewer than three, send nothing; the tool stays chosen for the next Room, and undo and redo wait while an outline is being drawn.

**A drag draws a rectangle**: with the Room tool chosen and no point placed, a drag from press to release previews a rectangle and, on release, sends one Place Element of a Room of four points at the rectangle's corners, from its lower-left corner counter-clockwise, so the bottom edge is the first; a rectangle less than a few screen pixels wide or high sends nothing; with points placed, a drag adds no point.

**Room options follow the selection**: with a Room selected, the Room tool's options show its wall thickness, wall colour, and floor colour, and a change to any is sent as one Edit Element; with none selected, they set what the next Room is drawn with, an eighth of a cell, the Wall tool's dark grey, and a light grey floor to start.

**Picking Rooms**: a Room is under the pointer anywhere inside its floor, and wherever a Wall of its thickness along its outline would be under it, outside the stretches its Portals cover; with a Room selected, its handles are hit before any Element.

**Handles of the selected Room**: the selected Room shows a handle at each point, at each control point with guide lines to its edge's two points, and at the middle of each straight edge, the edge from the last point to the first included; dragging them behaves as on a Wall, a double-click on its Walls adds a point at the nearest place on the outline, and dragging its floor or Walls moves the Room whole.

## Changes to existing behaviour

The Rules of Walls and Portals named here are those the composing spec holds once those changes land, which this change builds on.

- composing — **Anywhere on the Level**: modified to "an Element may lie outside the Bounds, a Prop placed there and a Wall or a Room with points there alike", because a Room's points are as free as a Wall's.
- composing — **Topmost is selected**: modified to "with the Select tool and no Asset chosen, a click selects the topmost Element under the pointer, a Prop by its rectangle, a Portal by its turned rectangle, a Wall by its line outside the stretches its Portals cover, and a Room by its floor or by its Walls outside the stretches its Portals cover; a click on empty space clears the selection; selecting an Element or one of its handles is never a history step", because a Room is picked by its floor as well as its Walls.
- composing — **Escape stops placing**: modified to "pressing Escape drops the chosen Asset, leaves the Wall tool, discarding a Wall being drawn, leaves the Room tool, discarding an outline being drawn, or leaves the Portal tool, and clicks select instead", because the Room tool is a fourth way of not selecting.
- composing — **One thing under the pointer**: modified to add "choosing the Room tool drops the chosen Asset and the selection and leaves the Wall tool, discarding a Wall being drawn, or the Portal tool; choosing an Asset, the Wall tool, or the Portal tool leaves the Room tool, discarding an outline being drawn", because the Room tool joins the tool strip.
- composing — **Delete acts on what is picked**: modified to "a click on a handle selects it, and a click on the Element's line, its floor, or empty space lets it go; Delete removes the selected point, straightens the selected control point's segment or edge, or, with no handle selected, removes the selected Wall or Room", because a Room has the same handles as a Wall.
- composing — **A Portal is an image with a width**: modified so that a set Portal is "anchored by the ElementId of the Wall or Room it is set into, one of the Wall's segments or the Room's edges, a parameter along it from 0 to 1, and a side, left or right of its direction from its first point to its second", because Portals are set into Rooms' Walls.
- composing — **A Portal covers its width**: modified to "a Portal set into a Wall covers the stretch of the Wall's line that reaches half its width either way from its centre, measured along the line and across the Wall's points, and stopping at the Wall's ends; a Portal set into a Room covers its stretch round the Room (A stretch runs round the Room)", because a Room's outline has no ends.
- composing — **Refused anchors**: modified so that the anchor is refused when it "names an Element that is neither a Wall nor a Room, a Wall or Room on another Level, a segment or edge it does not have, or a parameter outside 0 to 1", because a Room is now a valid anchor.
- composing — **Freestanding Portals stay put**: modified to "no edit of any Wall or Room moves, turns, or removes a freestanding Portal", because a Room edit is a Wall edit too.
- composing — **A lost Wall leaves the Portal standing**: modified to "a Portal whose anchor names neither a Wall nor a Room on its Level, or a segment or edge it does not have, as an editor that does not know Portals or Rooms may leave it, is drawn at its saved position, rotation, and mirroring, makes no Wall give way, is saved back unchanged, and can be freed", because a Room may be unknown to a collaborator's editor.
- composing — **Saved with its anchor**: modified so that the anchor holds "the Wall's or Room's ElementId, the segment or edge, the parameter, and the side", because a Portal may be set into a Room.
- composing — **Placing with the Portal tool**: modified so that the marker snaps to "the nearest line within reach, a Wall's line or a Room's outline, no farther from it than half a cell or half its thickness, whichever is more, the topmost Element winning a tie", and a click there anchors the Portal to that Wall's segment or that Room's edge, because a Room's Walls take doors as a Wall does.
- composing — **Picking Portals**: modified to "a Portal is under the pointer anywhere within its turned rectangle, and a Wall, or a Room's Walls, is not under the pointer where the nearest point of the line lies in a stretch a Portal covers; a Room's floor is picked there as anywhere else", because the gap belongs to the Portal but the floor still lies under it.
- composing — **Dragging Portals**: modified so that a set Portal slides "to the nearest point on its own Wall's line or its own Room's outline, round the outline past its first point", because a Room's outline is closed.
- composing — **Portal options and keys**: modified so that `F` sets a freestanding Portal "into the nearest Wall or Room within reach of its centre, at the nearest point on that line", because a Room's Walls are within reach like any Wall.

## Implementation Decisions

The technology the architecture fixes (Rooms keeping their editable source outlines as the truth, Portal anchoring by the Wall's or Room's ElementId with a segment or edge index, parameter, and side, lyon_tessellation filling outlines, ShapeEngine's own stroker with deterministic joins, the Element kind registry, the generic field-setting and reflection-snapshot history commands) is used as written there, and the composing spec's decisions for Walls and Portals (the derived shape in `model`, the once-per-frame deriving, the reversible point commands, the answer naming removed Portals) are extended, not restated.

- **The Room descriptor**: `model` gains the Room kind in the Element kind registry, whose descriptor says it is drawn as a filled outline under a stroked one, and the Room component: the points, one entry per edge holding its optional control point (as many entries as points, the last for the edge back to the first), the wall thickness, the wall colour, and the floor colour, both colours without alpha. It is a serialisable component at version one on the Element tier, under the stable name `room`. The points keep the order the Author gave them; nothing depends on the outline's direction.
- **The derived shape**: a Room carries the same derived shape as a Wall, never saved: its outline flattened into a closed line, each point tagged with the edge it lies on and the parameter along it, its stroke mesh, and the stretches its Portals cover, plus its floor: a fill mesh in cells. The same once-per-frame system that derives every changed Wall derives every changed Room after every Manager has handled its Commands, Undo, and Redo, sets its position and size from its outline, and runs for a Room when a Portal set into it changes, so a placed, edited, undone, redone, or opened Room has its floor and Walls before anything draws or picks it. A stretch on a Room may run past the first point, so it is a range from an edge and parameter to an edge and parameter that wraps round the outline.
- **ShapeEngine**: CombineOutlines takes Rooms' outlines and returns the derived outline and the floor: in this change it is given one Room at a time and combines nothing, flattening the outline with the Walls change's tolerance and filling what it winds around, under the non-zero rule, through lyon_tessellation, so the floor's edge and the Walls' centre line are the same chords. GenerateWalls strokes a closed line as well as an open one: a round join at the first point and no caps, its stretches wrapping. AnchorPortals works on a closed outline: lengths along it wrap past the first point, a stretch is never stopped by an end, and its remap through an added or removed point follows the Room's numbering, the joined edge of a removed first point being the last. SplitWall splits a Room's edge exactly as a Wall's segment. All stay plain functions over model types.
- **Place Element**: the Apply(Place Element) message gains a Room payload: the Layer, the points, the wall thickness, the wall colour, and the floor colour. AuthoringManager spawns the Element with a fresh ElementId as the last child of the Layer with the Room component and the common Element component, through the same reversible placement as a Wall, resolving nothing from the library. Fewer than three points, or a thickness not above zero, is a CommandFailed.
- **Edit Element**: the outline changes the Walls change added (a point moved, a control point set or unset, a point added, a point removed, the thickness, the colour) address a Room's points, edges, wall thickness, and wall colour as they do a Wall's, and the Element change gains the floor colour. Moving a point, setting a control point, and changing the thickness or a colour go through the generic field-setting command by reflect path. The reversible command that adds or removes a point works over closed outlines too, remembering what it dropped so that revert restores the Room exactly, and asks AnchorPortals where the Room's Portals go, recording their remapped anchors and their removals in the same history group, the Portals' removals before the point's. Removing a point from a three-point Room, and Remove Element of a Room, record a group holding the removal of every Portal set into it before the Room's. A position change on a Room sets the whole point and control point list translated by the difference. The answer naming the removed Portals names the Room they were set into, and the Editor writes how many to the status line as for a Wall.
- **Portal anchors**: the anchor's ElementId, saved as its `host` field, names the Wall or the Room the Portal is set into, and its `index` field is the Wall's segment or the Room's edge, read by the kind of the Element it names; wherever this spec speaks of an anchor's segment or edge, that is the saved `index`; the Portal component's shape does not change, so it stays at version one under `portal`. Placing, Set Portal into Wall, sliding, and the checks of a refused anchor accept a Room on the Portal's Level, and deriving treats an anchor naming an Element that is neither a Wall nor a Room as lost.
- **Rendering**: RenderEngine keeps two meshes with flat-colour Materials per Room from the derived shape through change detection, the floor and the Walls, both blended like a Wall's so they sort with the sprites, at the Room's stacking depth with the floor a fraction of a unit below its Walls and above the Element before it, in the viewport and in RenderRegion alike; both are replaced when the shape changes. An Element of the Room kind without a derived shape yet is not drawn that frame. Capturing a region waits for nothing new.
- **The tool**: the tool strip gains Room, `R` choosing it and Escape returning to Select; choosing it or leaving it follows One thing under the pointer. The outline in progress is Editor state, drawn by the Editor as a thin preview through its points with rubber bands from the last point to the pointer and from the pointer to the first; a click adds a point unless it lands within a few pixels of the last, a click within a few pixels of the first point with three or more placed or Enter sends one Apply(Place Element) with the Room payload, and fewer than three points sends nothing. A drag with no point placed, told from a click as a Prop drag is, previews a rectangle and sends its four corners on release when it is at least a few pixels wide and high. The options strip shows the wall thickness, the wall colour, and the floor colour: with a Room selected they show its values and a change sends Apply(Edit Element) as a single step; otherwise they are the defaults for the next Room. Undo and redo wait while an outline is being drawn or a rectangle dragged.
- **Picking**: the Editor hit-tests the selected Room's handles first, as a Wall's, then every Element from the topmost Layer down and the last-drawn Element back; a Room is hit when the pointer is inside its floor, by the non-zero winding of its derived closed line around the pointer, or within its Walls, by the distance to the nearest chord outside its stretches against half its thickness or four screen pixels, whichever is more. A double-click on its Walls inserts a point at the nearest chord's edge and parameter. Dragging a selected Room by its floor or Walls sends position changes as a gesture from press to release; dragging a handle sends Edit Element as a Wall's does.
- **The Portal tool**: snapping looks at every Room's derived closed line beside every Wall's, within half a cell or half its thickness, the nearest winning and the topmost on a tie, and takes the side from the chord's direction as on a Wall; a slide of a Portal set into a Room maps the pointer to the nearest point on that Room's closed line, so it passes the first point freely; `F` looks for the nearest Wall or Room within reach.
- **Project file**: ProjectAccess changes nothing: the Room component travels through the serialisation registry like the Wall, and an editor without the kind keeps its envelope on the Element and writes it back.

## Testing

- **Composing seam**: a headless App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, and AuthoringManager over a fixture Asset Folder holding a door image of known pixel size, with no window and no RenderEngine, driven by Apply, Undo, and Redo messages and asserted on the Room, Portal, and Element components, the Room's derived shape with its floor and stretches, the Layer's children, the answers, and the history.
  - **A Room is its outline**: `crates/drs-app/tests/rooms.rs::a_room_is_its_outline`
  - **Edges are numbered**: `crates/drs-app/tests/rooms.rs::edges_are_numbered`
  - **An edge curves by one control point**: `crates/drs-app/tests/rooms.rs::an_edge_curves_by_one_control_point` (the derived line's point halfway along the closing edge)
  - **The box follows the outline**: `crates/drs-app/tests/rooms.rs::the_box_follows_the_outline`
  - **A Room is placed as one step**: `crates/drs-app/tests/rooms.rs::a_room_is_placed_as_one_step`
  - **Moving the Room moves every point**: `crates/drs-app/tests/rooms.rs::moving_the_room_moves_every_point`
  - **A Room's point moves alone**: `crates/drs-app/tests/rooms.rs::a_rooms_point_moves_alone`
  - **A Room's handle drag is one step**: `crates/drs-app/tests/rooms.rs::a_rooms_handle_drag_is_one_step`
  - **Bending an edge keeps the points**: `crates/drs-app/tests/rooms.rs::bending_an_edge_keeps_the_points`
  - **Adding a point keeps the Room's shape**: `crates/drs-app/tests/rooms.rs::adding_a_point_keeps_the_rooms_shape` (on a straight edge, a curved one, and the last edge)
  - **Removing a point joins the Room straight**: `crates/drs-app/tests/rooms.rs::removing_a_point_joins_the_room_straight` (an inner point and the first point)
  - **Three points or none**: `crates/drs-app/tests/rooms.rs::three_points_or_none`
  - **Room properties stay editable**: `crates/drs-app/tests/rooms.rs::room_properties_stay_editable`
  - **Malformed Rooms are refused**: `crates/drs-app/tests/rooms.rs::malformed_rooms_are_refused`
  - **The floor fills the outline**, as derived: `crates/drs-app/tests/rooms.rs::the_floor_fills_the_outline` (the floor mesh covering a point inside a rectangle and inside both lobes of a crossed outline, and not a point outside)
  - **Portals set into Rooms**: `crates/drs-app/tests/rooms.rs::portals_set_into_rooms` (placed into, set into, freed from, and slid along a Room's edge; centre, rotation, and mirroring on either side)
  - **A stretch runs round the Room**: `crates/drs-app/tests/rooms.rs::a_stretch_runs_round_the_room` (a stretch across the first point, and one wider than the outline)
  - **Moves with its Room**: `crates/drs-app/tests/rooms.rs::moves_with_its_room`
  - **Adding a point keeps a Room's Portals in place**: `crates/drs-app/tests/rooms.rs::adding_a_point_keeps_a_rooms_portals_in_place`
  - **Removing a point carries a Room's Portals**: `crates/drs-app/tests/rooms.rs::removing_a_point_carries_a_rooms_portals` (an inner point and the first point)
  - **Gone with its part of the Room**: `crates/drs-app/tests/rooms.rs::gone_with_a_covered_point_of_the_room`, `crates/drs-app/tests/rooms.rs::gone_with_the_room`, `crates/drs-app/tests/rooms.rs::gone_with_a_three_point_room` (each with the answer naming the removed Portals and an undo restoring them)
  - The Room cases of the modified composing Rules: **Anywhere on the Level** `crates/drs-app/tests/rooms.rs::rooms_lie_anywhere_on_the_level`, **A Portal is an image with a width** and **Saved with its anchor** by `portals_set_into_rooms` above and the projects seam below, **A Portal covers its width** by `a_stretch_runs_round_the_room` above, **Refused anchors** `crates/drs-app/tests/rooms.rs::room_anchors_are_refused_as_wall_anchors` (a Room on another Level, an edge it lacks, a parameter outside 0 to 1), **Freestanding Portals stay put** `crates/drs-app/tests/rooms.rs::freestanding_portals_ignore_rooms`.
- **Offscreen export seam**: the existing headless App with RenderEngine under Bevy's default plugins without a window, exporting to a temporary PNG and asserting pixels.
  - **The floor fills the outline**: `crates/drs-app/tests/export.rs::a_room_fills_its_floor` (the floor colour at the centre of a rectangular Room, the background outside beyond half the thickness), `crates/drs-app/tests/export.rs::a_curved_room_fills_its_curve` (the floor colour between a bulging edge's chord and its curve)
  - **The Walls close around the floor**: `crates/drs-app/tests/export.rs::a_room_is_walled_all_round` (the wall colour on every edge's middle and at the first point's corner, within half the thickness either side of the outline)
  - **The floor lies under its Walls**: `crates/drs-app/tests/export.rs::a_rooms_floor_lies_under_its_walls` (the wall colour on the inner half of a Wall, a Prop placed before the Room hidden by its floor, and one placed after it shown over the floor)
  - **Rooms export as drawn**: `crates/drs-app/tests/export.rs::a_room_wall_gives_way_to_its_portal` (inside the stretch, the floor colour on the inner half and the background on the outer half beside a narrow Portal image), `crates/drs-app/tests/export.rs::a_room_is_clipped_at_the_edge`
- **Projects seam**: the existing headless App saving and reopening a Project with a straight and a curved Room and a Portal set into one, and a second App whose registry knows Walls and Portals but lacks the Room kind opening and saving the same file for the first App to reopen.
  - **Saved as its outline**: `crates/drs-app/tests/projects.rs::rooms_are_saved_as_their_outline`, `crates/drs-app/tests/projects.rs::unknown_rooms_round_trip`
  - **A lost Wall leaves the Portal standing**, the Room case: `crates/drs-app/tests/projects.rs::unknown_rooms_round_trip` (the Portal standing at its saved place in the second App, written back unchanged, and set into its Room again when the first App reopens the file)
- **By hand**: **Drawing with the Room tool**, **A drag draws a rectangle**, **Room options follow the selection**, **Picking Rooms**, and **Handles of the selected Room**, with the modified **Topmost is selected**, **Escape stops placing**, **One thing under the pointer**, **Delete acts on what is picked**, **Placing with the Portal tool**, **Picking Portals**, **Dragging Portals**, and **Portal options and keys**: no automated seam for the egui interface, the accepted deviation of the composing spec; verified by driving the editor with the development-only input script, which also confirms that a Room's floor and Walls are drawn in the viewport where the model says and that the Portal tool's marker snaps to a Room's Walls.
- ShapeEngine's own unit tests check what the Rules above rest on, as functions of the Engine alone, and are the coverage of no Rule: the floor's fill under the non-zero rule, a closed stroke with a join at the first point and no caps, a stretch wrapping past the first point, and the remap through a removed first point (`crates/drs-shape-engine/src/room.rs::tests::the_floor_fills_a_crossed_outline`, `crates/drs-shape-engine/src/room.rs::tests::a_closed_stroke_joins_its_first_point`, `crates/drs-shape-engine/src/portal.rs::tests::a_stretch_wraps_past_the_first_point`, `crates/drs-shape-engine/src/portal.rs::tests::removing_the_first_point_remaps_round_the_outline`).

## Out of Scope

- Rooms that overlap combining into one, a Room cutting another, party walls, and what happens to Portals in them: the next change, Rooms combine and cut. Here overlapping Rooms are simply both drawn, in stacking order.
- A textured floor or Walls: a Material other than the flat colours, Shaders, and a texture repeated along the Walls.
- A Room without Walls, or with some edges left unwalled; a Room with holes.
- Snapping a Room's points to the Grid or to other Rooms and Walls (ShapeEngine's Snap).
- Turning a Wall into a Room or a Room into Walls, and splitting a Room's Walls into Walls of their own.
- Squares, circles, and other shapes by drag beyond the rectangle; constraining the rectangle with a modifier; curving an edge while drawing.
- Cubic curves, arcs, and smoothing a whole outline through its points.
- Roofs over a Room, and Rooms blocking light or sight in the editor: lighting is deferred.
- Selecting several Rooms or points at once, and a property panel beyond the tool's options.
- A translucent floor or wall colour: Layer opacity comes with Layer compositing.

## Further Notes

- **Architecture check**: no new component, contract operation, dependency direction, or restricted crate is needed. The architecture already anchors a Portal by "Wall or Room ElementId, segment or edge index", so the Portal anchor needs no kind of its own: the kind of the Element it names says how to read the index. CombineOutlines is used for one Room at a time in this change, which is where the next change adds the combination through i_overlay; GenerateWalls, AnchorPortals, and SplitWall gain closed outlines. The Room's generated Walls are part of the Room Element, not Elements of their own, which is what "Walls generated around its outline" and the Room-anchored Portal need.
- The points keep the order the Author drew them in, clockwise or not, and a Portal's side stays relative to its edge's direction, so the next change, whose combined pieces may come out reversed, does not change which way a Portal faces.
- A Portal set into a Room and stacked below it, which only setting an older freestanding Portal into a newer Room can produce until Restack exists, is hidden where the floor covers it; it is still picked outside the floor, and the Room's Walls give way to it.
