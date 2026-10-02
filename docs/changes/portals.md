# Portals

**Capabilities**:
- composing: Set Portal into Wall, Free Portal, Place Element, Edit Element, Remove Element

## Problem Statement

A dungeon's walls are drawn, but nothing gets through them: there is no door, no window, no arch. The functional baseline has doors, and they are where it fails worst: moving a point of a wall drops the doors set into it or leaves them hanging beside it, so an Author who reshapes a corridor fixes every door by hand, or finds out at the table that one is gone.

## Solution

A Portal tool in the Editor. The Author chooses the Portal tool and an image Asset, such as a door, and hovers a Wall: a marker shows where on the Wall's line the Portal will sit and which side it will face. A click places the Portal there, set into the Wall: it is drawn turned to the Wall, centred on its line, and the Wall is left out along the Portal's width so the Portal stands in a gap. A click away from any Wall places the Portal freestanding, like a Prop that can be turned. A Portal set into a Wall is anchored to a segment of it and a place along that segment, so it moves with the Wall through every edit: dragging a point, bending a segment, or moving the whole Wall carries it along; adding a point keeps it where it was; removing a point carries it onto the joined segment. It is removed only when the part of the Wall it sits in is removed, in the same undo step and with the Author told. The Author slides a Portal along its Wall by dragging it, flips the side it faces, frees it where it stands, or sets a freestanding one into the nearest Wall; every step is one undo step, and the Export and the saved Project keep Portals and their gaps exactly.

## User Stories

### Placing Portals

1. As an Author, I can choose the Portal tool from the viewport's tool strip or with `P`, so that my next clicks place Portals instead of Props or Walls.
2. As an Author, I can choose the Portal's image in the browser, before or after choosing the Portal tool, so that a door is any image I own.
3. As an Author, I am told in the status line to choose an Asset when I click with the Portal tool and none is chosen, and nothing is placed, so that a click without an image never leaves an invisible Element.
4. As an Author, I can hover a Wall with the Portal tool and see a marker across its line at the nearest point, pointing to the side my pointer is on, so that I see where the Portal will sit and which way it will face before I click.
5. As an Author, I can see the marker jump to a Wall when my pointer comes within half a cell of its line, so that I need not hit a thin Wall exactly.
6. As an Author, I can rely on the nearest Wall being the one the marker snaps to when several are within reach, so that a door between two close walls goes into the one I mean.
7. As an Author, I can click while the marker shows to place the Portal set into that Wall at that point, facing that side, so that placing a door is one gesture.
8. As an Author, I can click away from any Wall to place the Portal freestanding, upright and centred on the click, so that a door leaning against a cave wall or a gate in the open is as easy as a Prop.
9. As an Author, I can rely on placing a Portal into a Wall being one undo step, so that undo takes the Portal away whole and leaves the Wall unbroken.
10. As an Author, I can see a placed Portal at its image's natural size, its width along the Wall, so that a vendor's door is as wide as the vendor meant.
11. As an Author, I can keep placing Portals after placing one, so that the doors of a whole dungeon are one tool choice.
12. As an Author, I can press Escape to leave the Portal tool and go back to selecting, so that I never place a Portal by accident.
13. As an Author, I can choose the Wall tool while the Portal tool is active and have the Portal tool give way, and choose the Portal tool while drawing a Wall and have the unfinished Wall thrown away, so that one thing is ever happening under my pointer.

### How a Portal looks

14. As an Author, I can see a Portal set into a Wall drawn centred on the Wall's line and turned to follow the Wall's direction at that point, on a straight segment and on a curved one alike, so that a door sits in its wall however the wall runs.
15. As an Author, I can see the Wall left out along the Portal's width, ending squarely at either side of the Portal, so that the Portal stands in a real gap.
16. As an Author, I can see a Portal facing the side I chose, its image mirrored across the Wall's line when it faces the other side, so that a door opens into the room I mean.
17. As an Author, I can place a Portal across a point of a Wall and see the gap follow the Wall around the corner, so that a door at a corner is drawn as it would be built.
18. As an Author, I can place a Portal near the end of a Wall and see the gap stop at the Wall's end, so that a doorway at the end of a wall needs no special care.
19. As an Author, I can place two Portals that overlap, and see both drawn in stacking order and the Wall left out along both, so that a door and its frame can be two images.
20. As an Author, I can see a freestanding Portal drawn centred on its position and turned by its rotation, so that it looks like a Prop I can turn.
21. As an Author, I can see a Portal whose image is missing or broken drawn as a placeholder of its size, turned and set into its Wall like the image would be, with the gap still cut, so that a missing door still shows where the doorway is.

### Selecting and editing Portals

22. As an Author, I can click a Portal to select it, hit anywhere within its turned image, so that a diagonal door is as easy to pick as an upright one.
23. As an Author, I can click a Portal standing in a gap and get the Portal rather than the Wall, whatever their stacking order, so that the gap belongs to the door.
24. As an Author, I can drag a Portal set into a Wall and have it slide along the Wall, following the nearest point on the Wall's line, across the Wall's points, so that moving a door along its wall never pulls it out of the wall.
25. As an Author, I can rely on sliding a Portal being a single undo step however long the drag, so that undo returns it to where the drag began.
26. As an Author, I can drag a freestanding Portal to move it as I move a Prop, as one undo step, so that freestanding Portals behave like Props.
27. As an Author, I can flip the side a selected Portal faces with `X` or from the options strip, as one undo step, so that a door opening the wrong way is one key away from right.
28. As an Author, I can change a selected Portal's width in the options strip, its height following the image's proportions, as one undo step, so that a double door or a narrow window is the same image at another size.
29. As an Author, I am refused a width that is zero or less, with nothing changed, so that a Portal never vanishes through a typo.
30. As an Author, I can change a freestanding Portal's rotation in degrees in the options strip, as one undo step, so that it stands at any angle.
31. As an Author, I can press Delete with a Portal selected to remove it, and see the gap in its Wall close, so that removing a door restores the wall.
32. As an Author, I can rely on undoing that removal bringing the Portal back set into the same place of the same Wall, so that undo never loses an anchor.

### Freeing and setting

33. As an Author, I can free a selected Portal set into a Wall with `F` or from the options strip, and see it stay exactly where it stood, so that a door can be taken out of its wall without moving.
34. As an Author, I can rely on a freed Portal no longer following its former Wall, and on the Wall closing behind it, so that freeing means free.
35. As an Author, I can set a selected freestanding Portal into the nearest Wall within reach of its centre with `F` or from the options strip, at the nearest point on that Wall's line, so that a door dropped beside a wall snaps into it.
36. As an Author, I am told in the status line that no Wall is within reach when I set a Portal that stands far from every Wall, and nothing changes, so that a key press that cannot work says why.
37. As an Author, I can rely on setting and freeing each being one undo step that undo returns exactly to how the Portal stood, so that trying out a door costs me nothing.

### Through every Wall edit

38. As an Author, I can drag a point of a Wall and see the Portals set into its two segments move with the segments, each keeping its place along its segment, so that reshaping a room never leaves a door hanging.
39. As an Author, I can drag a point two segments away from a Portal and see that Portal not move at all, so that fixing one corner never disturbs doors elsewhere.
40. As an Author, I can bend or straighten a segment and see the Portals on it follow the curve, turned to its direction at their place, so that a curved wall keeps its doors.
41. As an Author, I can move a whole Wall and see its Portals move with it, so that a wall and its doors move as one.
42. As an Author, I can change a Wall's thickness and see its Portals and their gaps unchanged along the line, so that a thicker wall keeps its doors.
43. As an Author, I can add a point to a Wall, even on the segment a Portal sits on, and see every Portal stay exactly where it was, so that adding a corner never moves a door.
44. As an Author, I can remove a point of a Wall and see the Portals on the two segments it joined carried onto the joined segment, each at the same share of the way along, so that taking out a corner keeps the doors beside it.
45. As an Author, I can remove a point that a Portal covers and see that Portal removed with the point, and be told in the status line how many Portals went, so that a door is never left floating over a corner that no longer exists, and never silently.
46. As an Author, I can remove an end point of a Wall and see the Portals on its last segment removed with the segment and be told, so that a door goes when the stretch of wall it stood in goes.
47. As an Author, I can remove a Wall and see the Portals set into it removed with it, and be told how many, so that no door is left standing in no wall.
48. As an Author, I can undo any of these removals and get the Wall, its points, and every Portal back exactly, set where they were, so that undo restores the doors with the wall.
49. As an Author, I can rely on no Wall edit moving or removing a freestanding Portal, so that a freestanding Portal is independent of every Wall.
50. As an Author, I can rely on a Portal on a segment I dragged down to no length keeping the angle it had, so that collapsing a segment never spins a door.

### Export, saving, and sharing

51. As an Author, I can export a Level and see each Portal turned and mirrored as in the editor and each Wall left out along its Portals, so that the doors I see are the doors I export.
52. As an Author, I can save a Project with Portals and reopen it with every Portal set into the same place of the same Wall, facing the same side, or freestanding where it was, so that my doors survive closing the editor.
53. As an Author sharing a Project with a collaborator whose editor knows Walls but not Portals, I can rely on their editor keeping my Portals as placeholders and saving them back untouched, so that a round trip through an older editor never loses a door.
54. As an Author opening a Project in which such a collaborator removed the part of a Wall a Portal sat in, I can see that Portal standing where it was saved, kept and saved back unchanged, and free it, so that a door is never dropped because someone else's editor did not know it.

### History

55. As an Author, I can undo and redo every step above with the usual shortcuts, in the order I took them, whether it was a Portal, a Wall, or a Prop, so that history stays one line.
56. As an Author, I can rely on selecting a Portal and hovering with the Portal tool never being an undo step, so that looking costs me nothing.

## Rules

### The Portal

**A Portal is an image with a width**: a Portal shows an image Asset at a width in cells, with a height that keeps the image's proportions, and is either freestanding, with a position, a rotation, and whether it is mirrored, or set into a Wall, anchored by that Wall's ElementId, one of its segments, a parameter along that segment from 0 to 1, and a side, left or right of the segment's direction from its first point to its second.

**Natural width**: a placed Portal's width and height are its image's pixel width and height divided by the Grid's 256 pixels per cell.

**Set Portals stand on the line**: a Portal set into a Wall is centred on the point of its segment at its parameter, with its width along the segment's direction there and its image's top facing its side: drawn as it is when it faces the left, mirrored across the Wall's line when it faces the right. Follows from: A Portal set into a Wall moves with it.

**A Portal covers its width**: a Portal set into a Wall covers the stretch of the Wall's line that reaches half its width either way from its centre, measured along the line and across the Wall's points, and stopping at the Wall's ends.

**The Wall gives way**: a Wall is not drawn along any stretch a Portal set into it covers; at each end of such a stretch the stroke ends squarely across the line, and an end of the Wall that a stretch reaches has no cap; overlapping stretches leave out what either covers.

**Freestanding like a Prop**: a freestanding Portal is drawn centred on its position, turned counter-clockwise by its rotation, and mirrored across its length when it is mirrored.

**Portals stack like Elements**: every Portal is drawn at its place in the stacking order, Portals that overlap each other included, and a Portal whose image cannot be drawn is a placeholder of its size, turned, mirrored, and covering its stretch as the image would. Follows from: Stacking order.

**Portals export as drawn**: Portals and the stretches of Wall they leave out appear in the Export as they are drawn in the editor. Follows from: Stacking order.

### Set Portal into Wall and Free Portal

**Placed into a Wall at once**: a Place Element of a Portal with an anchor places the Portal already set into the Wall, as one history step that undo takes away whole and redo brings back set into the same place. Follows from: Every Command can be undone.

**Set into a Wall**: a Set Portal into Wall anchors a Portal, freestanding or set into any Wall, to the given Wall, segment, parameter, and side, as one history step that undo returns to the anchor, position, rotation, and mirroring it had. Follows from: Every Command can be undone.

**Freed where it stands**: a Free Portal makes a Portal set into a Wall freestanding with the position, rotation, and mirroring it had, so nothing moves on the Level and its Wall is drawn whole again, as one history step that undo returns to the same anchor.

**Refused anchors**: a Set Portal into Wall or a Place Element of a Portal whose anchor names an Element that is not a Wall, a Wall on another Level, a segment the Wall does not have, or a parameter outside 0 to 1, a Free Portal of a freestanding Portal, and a Set Portal into Wall or Free Portal of an Element that is not a Portal are answered with the reason, change nothing, and record no history step.

### Editing Portals

**Sliding along the Wall**: an Edit Element changing a set Portal's segment and parameter moves it along its Wall with its side kept, and a drag of it records a single history step however long, which undo returns to where the drag began.

**Portals stay editable**: a Portal's width, a set Portal's side, and a freestanding Portal's position, rotation, and mirroring are each changed through Edit Element, every change a step of its own outside a drag; a width not above zero, a segment or parameter its Wall does not have, a position, rotation, or mirroring of a set Portal, which follow its Wall, and a side or a place along a Wall of a freestanding Portal are refused with the reason, change nothing, and record no history step. Follows from: Nothing is fixed at creation.

**Removed Portals return set**: undoing the removal of a Portal set into a Wall restores it with its ElementId, its anchor, and its place in the stacking order, and its Wall gives way again.

### Through Wall edits

**Moves with its Wall**: moving a Wall, moving a point, setting or unsetting a control point, and changing the thickness or the colour change no Portal's segment, parameter, or side; each Portal set into the Wall stands at its parameter on its segment as the segment now is. Follows from: A Portal set into a Wall moves with it.

**Adding a point keeps Portals in place**: adding a point on segment *k* at parameter *s* moves a Portal on segment *k* at a parameter *t* below *s* to parameter *t* / *s* on segment *k*, one at or above *s* to parameter (*t* − *s*) / (1 − *s*) on segment *k* + 1, and every Portal on a later segment one segment on, in the same history step, so no Portal moves on the Level. Follows from: A Portal set into a Wall moves with it.

**Removing a point carries the Portals beside it**: removing an inner point moves each Portal on the two segments it joins whose stretch does not cover it onto the joined segment, at the parameter equal to the share of the two segments' combined length that lay before the Portal's centre, and every Portal on a later segment one segment back; removing the first point moves every Portal on a remaining segment one segment back; all in the same history step. Follows from: A Portal set into a Wall moves with it.

**Gone with its part of the Wall**: a Portal whose stretch covers an inner point being removed, a Portal on the segment that removing an end point takes away, and every Portal set into a Wall being removed, by Remove Element or by removing a point of a two-point Wall, are removed in the same history step, which undo restores whole; the Author is told in the status line how many Portals were removed. Follows from: A Portal set into a Wall moves with it.

**Freestanding Portals stay put**: no edit of any Wall moves, turns, or removes a freestanding Portal.

**No direction keeps the rotation**: a Portal set into a Wall at a place where its segment has no direction, a segment of no length or a curve whose control point lies on its end, keeps the rotation it had.

### Saving

**Saved with its anchor**: a saved Portal holds its image's Asset Reference, its width, its position, rotation, and mirroring, and its anchor with the Wall's ElementId, segment, parameter, and side, and reopens the same: set into the same place of the same Wall, or freestanding where it was. Follows from: References are never dropped.

**A lost Wall leaves the Portal standing**: a Portal whose anchor names no Wall on its Level, or a segment its Wall does not have, as an editor that does not know Portals may leave it, is drawn at its saved position, rotation, and mirroring, makes no Wall give way, is saved back unchanged, and can be freed. Follows from: References are never dropped.

### The Portal tool

**Placing with the Portal tool**: with the Portal tool chosen, from the tool strip or with `P`, the Asset chosen before or while it is chosen is its image; with no Asset chosen a click places nothing and the status line asks for one; within reach of a Wall, no farther from its line than half a cell or half its thickness, whichever is more, the pointer shows a marker across the nearest such Wall's line at the nearest point on it, pointing to the side of the line the pointer is on, the topmost Wall winning a tie; a click there sends a Place Element of a Portal anchored at that point and side, a click out of reach sends one freestanding, unturned and centred on the click; the tool stays chosen.

**Picking Portals**: a Portal is under the pointer anywhere within its turned rectangle, and a Wall is not under the pointer where the nearest point of its line lies in a stretch a Portal covers.

**Dragging Portals**: dragging a selected Portal set into a Wall sends Edit Element changes of its segment and parameter to the nearest point on its own Wall's line, from press to release as one gesture; dragging a freestanding Portal sends position changes as dragging a Prop does.

**Portal options and keys**: with a Portal selected, the options strip shows its width, its rotation in degrees when freestanding, a flip button, and a Free Portal or Set into Wall button; `X` flips the side of a set Portal or the mirroring of a freestanding one; `F` frees a set Portal, or sets a freestanding one into the nearest Wall within reach of its centre, at the nearest point on that Wall's line and facing the right when it is mirrored or the left when it is not, and with no Wall within reach says so in the status line and sends nothing; Delete removes it.

## Changes to existing behaviour

The Rules of Walls named here are those the composing spec holds once Walls lands, which this change builds on.

- composing — **Placed where clicked**: modified to "with an Asset chosen and neither the Wall tool nor the Portal tool chosen, a click on the Level places a Prop of that Asset on the current Layer, centred on the clicked point", because a click with the Portal tool places a Portal.
- composing — **Placement records a reference**: modified to "placing a Prop or a Portal records in the Project an Asset Reference …; placing a second Element of the same Asset adds no second Asset Reference", because a Portal's image is an Asset like a Prop's.
- composing — **Placement records the folder**: modified to "the first Prop or Portal placed from an Asset Folder records that folder's Canonical Name and version in the Project; later Elements from the same folder add no second record", for the same reason.
- composing — **Topmost is selected**: modified to "with the Select tool and no Asset chosen, a click selects the topmost Element under the pointer, a Prop by its rectangle, a Portal by its turned rectangle, and a Wall by its line outside the stretches its Portals cover; a click on empty space clears the selection; selecting an Element or one of its handles is never a history step", because Portals are turned and stand in their Wall's gaps.
- composing — **Escape stops placing**: modified to "pressing Escape drops the chosen Asset, leaves the Wall tool, discarding a Wall being drawn, or leaves the Portal tool, and clicks select instead", because the Portal tool is a third way of not selecting.
- composing — **One history**: modified to "Add Asset Folder, Place Element, Edit Element, Remove Element, Set Portal into Wall, and Free Portal are each one undo step, and undo walks back through them in the order they were applied whichever Manager handled them", because the two Portal Commands join the history.
- composing — **Redo repeats exactly**: modified to "redoing a Place Element, Edit Element, Remove Element, Set Portal into Wall, or Free Portal leaves the Level as it was before the undo", for the same reason.
- composing — **A failed Command is reported**: modified to "a Place Element, Edit Element, Remove Element, Set Portal into Wall, or Free Portal that cannot be carried out is answered with the reason, places or changes nothing, and records no history step", for the same reason.
- composing — **Drawn as a stroke**: modified to "a Wall is drawn centred on its line, as wide as its thickness, with round joins at its points and round caps at its ends, in its colour, at its place in the stacking order, except along the stretches its Portals cover (The Wall gives way)", because a Portal stands in a gap.
- composing — **Two points or none**: modified to "removing a point from a Wall of two points removes the Wall and the Portals set into it, as one history step that undoes to the Wall with both points and its Portals", because a Portal exists only while its part of the Wall exists.
- composing — **Malformed Walls are refused**: modified to add "an Edit Element adding a point at a parameter not strictly between 0 and 1", because such a point would make a segment of no length and leave no parameter to carry a Portal to.
- composing — **Hit within the thickness**: modified to "a Wall is under the pointer when the pointer is no farther from its line than half its thickness or four screen pixels, whichever is more, and the nearest point of its line lies in no stretch a Portal covers; with a Wall selected, its handles are hit before any Element", because the gap belongs to the Portal.
- composing — **One thing under the pointer**: modified to "choosing an Asset leaves the Wall tool and discards a Wall being drawn, unless the Portal tool is chosen, which makes it the Portal's image; choosing the Wall tool drops the chosen Asset and the selection and leaves the Portal tool; choosing the Portal tool leaves the Wall tool, discarding a Wall being drawn, drops the selection, and keeps a chosen Asset as the Portal's image", because the Portal tool joins the tool strip and takes its image from the browser.

## Implementation Decisions

The technology the architecture fixes (Portal anchoring by Wall ElementId, segment index, parameter, and side, never by arc length, with the outline-changing Command removing a Portal whose part of the Wall vanishes in the same undo group; ShapeEngine's AnchorPortals and GenerateWalls; the Element kind registry; the generic field-setting and reflection-snapshot history commands; the deterministic maths functions) is used as written there and not restated.

- **The Portal descriptor**: `model` gains the Portal kind in the Element kind registry, drawn as an image like a Prop, and the Portal component: the Asset Reference row of its image, its width in cells, its rotation in radians counter-clockwise, whether it is mirrored, and an optional anchor holding the Wall's ElementId, the segment's number, the parameter, and the side. It is a serialisable component at version one on the Element tier, under the stable name `portal`. The Element's position is the Portal's centre and its size the width by the height the image's recorded pixel size gives; a set Portal's position, rotation, and mirroring are kept equal to what its anchor gives, so freeing it is clearing the anchor and nothing else, and the file holds the same values the editor draws.
- **The derived shape**: the Wall's derived shape in `model` gains the stretches its Portals cover, as ranges along its line from a segment and parameter to a segment and parameter, so that the Editor can leave them out of hit-testing and RenderEngine draws a stroke that already leaves them out.
- **ShapeEngine**: AnchorPortals takes a Wall's points, control points, and the anchors and widths of the Portals set into it, and returns, for each Portal, its centre, the angle of the line's direction there, and the stretch it covers; given a point added at a segment and parameter, or a point removed, it also returns where each anchor goes: kept on a new segment and parameter, or gone. The centre and direction are evaluated on the exact curve; lengths along the line are measured over the flattened line the Walls change derives, summing its chords, so the stretch, the share of length at a removed point, and the gap's ends use square roots and arithmetic alone and come out the same on every machine; the angle is computed through the deterministic maths functions, and a direction of no length is reported as none, so the Portal keeps its rotation. GenerateWalls takes the stretches as well and leaves them out of the stroke, ending each run squarely across the line and capping only the Wall's own ends that no stretch reaches. Both stay plain functions over model types.
- **Deriving**: the once-per-frame system that derives every changed Wall's shape after every Manager has handled its Commands, Undo, and Redo now also runs for a Wall when a Portal set into it is placed, edited, set, freed, or removed, and for every Portal whose component changed: it asks AnchorPortals where the Wall's Portals stand, writes each Portal's position, rotation, and mirroring when they differ, sets every changed Portal's size from its width and its image's recorded pixel size, and passes the stretches to GenerateWalls. A Portal whose anchor names no Wall of its Level, or a segment its Wall lacks, is left as it is and adds no stretch. An opened Project therefore draws each Portal as saved, and saving it again writes the same bytes.
- **Place Element**: the Apply(Place Element) message gains a Portal payload: the Layer, the chosen Asset, the centre for a freestanding Portal, and an optional anchor. AuthoringManager resolves the Asset as it does for a Prop (LoadAsset, the Asset Reference and folder rows) and checks the anchor before anything is recorded, then spawns the Element with a fresh ElementId as the last child of the Layer, already set into the Wall when anchored, through the same reversible placement as a Prop: one step.
- **Set Portal into Wall and Free Portal**: Apply gains a Set Portal into Wall (the Portal, the Wall, the segment, the parameter, the side) and a Free Portal (the Portal), both handled by AuthoringManager and each closing any gesture group left open before recording. Set Portal into Wall is a reversible command of its own that remembers the anchor, position, rotation, and mirroring before it, so undo returns a freestanding Portal to the angle it stood at rather than the Wall's. Free Portal clears the anchor through the generic field-setting command, since the position, rotation, and mirroring already hold what it stood at.
- **Edit Element**: the Element change gains a Portal's width, its rotation, its mirroring, its side, and its place along its Wall (segment and parameter); each goes through the generic field-setting command by reflect path, so a slide is grouped from press to release exactly as a Prop's drag. A width not above zero, a position, rotation, or mirroring of a set Portal, a side or place along the Wall of a freestanding Portal, and a segment or parameter its Wall lacks are a CommandFailed.
- **Wall edits carry their Portals**: the commands that add and remove a Wall's points ask AnchorPortals where the Portals set into that Wall go and record, in the same history group, the remapped anchors through the generic field-setting command and the removal of each Portal that is gone through the existing Remove Element command, the Portals' removals before the point's so that revert restores the point first and the Portals after it. Remove Element of a Wall, and the removal of a two-point Wall by removing a point, record a group holding the removal of every Portal set into it before the Wall's. Moving the Wall, a point, or a control point records nothing for its Portals: deriving moves them. When a Command removes Portals, AuthoringManager answers with a message in `model` naming the Wall and the ElementIds of the Portals removed, and the Editor writes how many to the status line, as Failure shown in the status line prescribes for a success.
- **Rendering**: RenderEngine draws a Portal as it draws a Prop, through the resolution table and the `lib://` handle, at its stacking depth, with the sprite turned by its rotation, the turn built through the deterministic maths functions so the Export is the same on every machine, and flipped across its length when mirrored; the placeholder of a Portal that cannot be drawn is turned and flipped the same way. The Wall's mesh comes from the derived shape as before and so already leaves the stretches out. Capturing a region waits for a Portal's image as it waits for a Prop's.
- **The tool**: the tool strip gains Portal, `P` choosing it and Escape returning to Select. The Portal tool's image is the chosen Asset, kept when the tool is chosen and replaced when another Asset is chosen while it is. Snapping is the Editor's: it finds the nearest point on each Wall's flattened line within reach, interpolating the segment and parameter along the nearest chord as a double-click on a Wall does, takes the side from which side of the chord's direction the pointer lies on, and draws the marker as a short line across the Wall with an arrowhead to the side. A click sends one Apply(Place Element) with the Portal payload. No preview of the image is drawn before the click, since its size is known only once the Asset is loaded at placement.
- **Picking and editing**: the Editor hit-tests a Portal by mapping the pointer into the Portal's turned frame against its rectangle, and leaves out of a Wall's hit the chords inside the derived shape's stretches. A slide maps the pointer to the nearest point on the selected Portal's own Wall and sends the segment and parameter as a gesture from press to release. `F` and `X` act on a selected Portal and do nothing otherwise; the options strip shows the width, the rotation in degrees while freestanding, the flip button, and the Free Portal or Set into Wall button, each change sent as a single step. The selection is dropped when its Portal is removed, by Delete or by a Wall edit.
- **Project file**: ProjectAccess changes nothing: the Portal component travels through the serialisation registry, the anchor's Wall written as its ElementId; an editor without the Portal kind keeps the envelope on the Element and writes it back.

## Testing

- **Composing seam**: a headless App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, and AuthoringManager over a fixture Asset Folder holding a door image with known pixel size, with no window and no RenderEngine, driven by Apply, Undo, and Redo messages and asserted on the Portal and Element components, the Wall's derived shape and its stretches, the Layer's children, the answers, and the history.
  - **A Portal is an image with a width**: `crates/drs-app/tests/portals.rs::a_portal_is_an_image_with_a_width`
  - **Natural width**: `crates/drs-app/tests/portals.rs::natural_width`
  - **Set Portals stand on the line**: `crates/drs-app/tests/portals.rs::set_portals_stand_on_the_line` (centre, rotation, and mirroring on a straight and a curved segment, on either side)
  - **A Portal covers its width**: `crates/drs-app/tests/portals.rs::a_portal_covers_its_width` (a stretch inside a segment, across a point, and stopped at an end)
  - **Placed into a Wall at once**: `crates/drs-app/tests/portals.rs::placed_into_a_wall_at_once`
  - **Set into a Wall**: `crates/drs-app/tests/portals.rs::set_into_a_wall`
  - **Freed where it stands**: `crates/drs-app/tests/portals.rs::freed_where_it_stands`
  - **Refused anchors**: `crates/drs-app/tests/portals.rs::refused_anchors`
  - **Sliding along the Wall**: `crates/drs-app/tests/portals.rs::sliding_along_the_wall`
  - **Portals stay editable**: `crates/drs-app/tests/portals.rs::portals_stay_editable`
  - **Removed Portals return set**: `crates/drs-app/tests/portals.rs::removed_portals_return_set`
  - **Moves with its Wall**: `crates/drs-app/tests/portals.rs::moves_with_its_wall` (a point drag on the Portal's segment and two segments away, a bend, a whole-Wall move, a thickness change)
  - **Adding a point keeps Portals in place**: `crates/drs-app/tests/portals.rs::adding_a_point_keeps_portals_in_place` (on the Portal's segment either side of the new point, and upstream; the Portal's centre unchanged on a straight and a curved segment)
  - **Removing a point carries the Portals beside it**: `crates/drs-app/tests/portals.rs::removing_a_point_carries_the_portals_beside_it`
  - **Gone with its part of the Wall**: `crates/drs-app/tests/portals.rs::gone_with_a_covered_point`, `crates/drs-app/tests/portals.rs::gone_with_an_end_segment`, `crates/drs-app/tests/portals.rs::gone_with_the_wall`, `crates/drs-app/tests/portals.rs::gone_with_a_two_point_wall` (each with the answer naming the removed Portals and an undo restoring them)
  - **Freestanding Portals stay put**: `crates/drs-app/tests/portals.rs::freestanding_portals_stay_put`
  - **No direction keeps the rotation**: `crates/drs-app/tests/portals.rs::no_direction_keeps_the_rotation`
  - The Portal cases of the modified composing Rules: **Placement records a reference** `crates/drs-app/tests/portals.rs::portals_record_a_reference`, **Placement records the folder** `crates/drs-app/tests/portals.rs::portals_record_the_folder`, **One history** `crates/drs-app/tests/portals.rs::portal_commands_share_the_history`, **Redo repeats exactly** `crates/drs-app/tests/portals.rs::portal_commands_redo_exactly`, **A failed Command is reported** by `refused_anchors` above, **Two points or none** by `gone_with_a_two_point_wall` above, and **Malformed Walls are refused** `crates/drs-app/tests/walls.rs::a_point_at_a_segment_end_is_refused`.
- **Offscreen export seam**: the existing headless App with RenderEngine under Bevy's default plugins without a window, exporting to a temporary PNG and asserting pixels.
  - **The Wall gives way** and the stroke part of the modified **Drawn as a stroke**: `crates/drs-app/tests/export.rs::a_wall_gives_way_to_its_portal` (a Portal image shorter than the Wall's thickness: its colour at its centre, the background within half the thickness beside it inside the stretch, and the Wall's colour just past the stretch's end), `crates/drs-app/tests/export.rs::a_gap_follows_the_corner` (a Portal across a point: the background on both segments within the stretch)
  - **Set Portals stand on the line** as drawn: `crates/drs-app/tests/export.rs::a_portal_faces_its_side` (an image of two colours, its top half's colour on the chosen side of a vertical Wall, swapped after a flip)
  - **Freestanding like a Prop**: `crates/drs-app/tests/export.rs::a_freestanding_portal_is_turned` (a wide image turned a quarter turn covering a tall area)
  - **Portals stack like Elements**: `crates/drs-app/tests/export.rs::overlapping_portals_stack` (the later Portal's colour where two overlap)
  - **Portals export as drawn**: covered by the four tests above, all of which export.
- **Projects seam**: the existing headless App saving and reopening a Project with a set and a freestanding Portal, and a second App whose registry lacks the Portal kind opening the same file, removing the end point under a Portal, and saving it for the first App to open.
  - **Saved with its anchor**: `crates/drs-app/tests/projects.rs::portals_are_saved_with_their_anchor`
  - **A lost Wall leaves the Portal standing**: `crates/drs-app/tests/projects.rs::a_lost_wall_leaves_the_portal_standing` (the Portal drawn at its saved place, no stretch on the Wall, the file written back unchanged, and Free Portal accepted)
- **By hand**: **Placing with the Portal tool**, **Picking Portals**, **Dragging Portals**, and **Portal options and keys**, with the modified **Placed where clicked**, **Topmost is selected**, **Escape stops placing**, **Hit within the thickness**, and **One thing under the pointer**: no automated seam for the egui interface, the accepted deviation of the composing spec; verified by driving the editor with the development-only input script, which also confirms that a Portal is drawn in the viewport turned, mirrored, and in its gap where the model says, and that the status line names the Portals a Wall edit removed.
- ShapeEngine's own unit tests check what the Rules above rest on, as functions of the Engine alone, and are the coverage of no Rule: the centre and direction on a quadratic, the stretch measured across a point and stopped at an end, the remap through an added point leaving the centre where it was, and the stroke ending squarely at a stretch (`crates/drs-shape-engine/src/portal.rs::tests::the_centre_lies_on_the_curve`, `crates/drs-shape-engine/src/portal.rs::tests::a_stretch_crosses_points_and_stops_at_ends`, `crates/drs-shape-engine/src/portal.rs::tests::an_added_point_keeps_the_centre`, `crates/drs-shape-engine/src/wall.rs::tests::a_stretch_ends_the_stroke_squarely`).

## Out of Scope

- Portals set into Walls generated from Rooms, and what happens to them when Rooms combine and cut: the Rooms changes, which use this change's anchor with a Room's edge in place of a Wall's segment.
- Splitting a Wall into two Walls at a Portal, and joining two Walls into one.
- Open and closed Portals, and Portals blocking light: lighting is deferred.
- Snapping a Portal to the Grid, or to a Wall while a freestanding Portal is dragged; dragging a set Portal off its Wall or onto another Wall.
- A preview of the Portal's image before it is placed.
- Rotating Props, and a rotation handle on the Level for freestanding Portals.
- A Material other than the image's default, a separate door frame, and Portals that change the Wall's look beside them.
- Telling doors from windows: both are Portals, told apart by their image.
- Selecting several Portals at once, and copying them.
- Portals in a VTT export: VTT export is later.

## Further Notes

- **Architecture check**: no new component, dependency direction, or restricted crate is needed. The anchor is the architecture's (Wall ElementId, segment index, parameter, side), with the side relative to the segment's direction from its first point to its second; AnchorPortals and GenerateWalls stay ShapeEngine's, extended with the stretches and the remap through a point edit; the stretches join the derived shape that AuthoringManager writes and the Editor and RenderEngine read; the Portal Commands and the answer naming removed Portals are `model` messages AuthoringManager handles and sends. The Rooms changes reuse the same anchor and the same rule for a vanished part of a Wall.
- **Settled open questions**: overlapping Portals are allowed, on any Wall, and drawn in stacking order, with the Wall left out along what either covers. A Portal may straddle a point of a Wall, its gap following the stroke around it; only removing a point it covers removes it. What happens to a Portal straddling the cut when Rooms combine and cut stays open for that change.
- A point removed beside Portals that do not cover it carries them onto the joined straight segment by share of length, so a straight Wall that gained and then lost such a point puts those Portals back exactly where they were.
- A Portal wider than its whole Wall leaves the Wall undrawn and unpickable by its line until the Portal is narrowed, slid, freed, or removed; the Wall stays an Element and its handles show when it is selected through undo or after the Portal changes.
