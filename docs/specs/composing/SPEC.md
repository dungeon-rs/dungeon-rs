# Composing

**Commands**: Place Element, Edit Element, Remove Element, Restack, Paint, Set Portal into Wall, Free Portal

## Purpose

The Author builds a Level by putting Elements on its Layers. This capability lets the Author choose an Asset and place it as a Prop where they click, at the size the vendor meant, draw Walls point by point and bend, reshape, and recolour them at any time, set doors and windows into them as Portals that follow every Wall edit or stand free, paint the ground as Terrain with a soft Brush whose every stroke stays editable, move and remove any of them, and take every step back and forward again, so that composing is a line of small, reversible gestures. The editor opens on a Project to compose on at once.

## User Stories

### Placing Props

1. As an Author, I can open the editor on a new, unsaved Project with one Level and one Layer, so that I can place something at once.
2. As an Author, I can choose an Asset and click on the Level to place a Prop centred where I clicked, so that placing is one gesture.
3. As an Author, I can see a placed Prop at its natural size in Grid cells, so that a vendor's table is as big as the vendor meant.
4. As an Author, I can rely on each new Element I place or draw landing on top of the Elements already on the Layer, so that what I place or draw last is what I see.
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

20. As an Author, I can click a Prop, a Portal anywhere within its turned image, or a Wall's line anywhere within its thickness, to select it, with the topmost Element winning when they overlap whatever its kind, so that I always get the one I see.
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

35. As an Author, I can press Delete to remove the selected Prop or Portal, or the selected Wall when no point or control point is selected, so that removing is one key.
36. As an Author, I can rely on an undone removal bringing the Element back exactly as it was, a Wall with every point and curve, including its place in the stacking order, so that undo never reshuffles my Level.

### Placing Portals

37. As an Author, I can choose the Portal tool from the viewport's tool strip or with `P`, so that my next clicks place Portals instead of Props or Walls.
38. As an Author, I can choose the Portal's image in the browser, before or after choosing the Portal tool, so that a door is any image I own.
39. As an Author, I am told in the status line to choose an Asset when I click with the Portal tool and none is chosen, and nothing is placed, so that a click without an image never leaves an invisible Element.
40. As an Author, I can hover a Wall with the Portal tool and see a marker across its line at the nearest point, pointing to the side my pointer is on, so that I see where the Portal will sit and which way it will face before I click.
41. As an Author, I can see the marker jump to a Wall when my pointer comes within half a cell of its line, so that I need not hit a thin Wall exactly.
42. As an Author, I can rely on the nearest Wall being the one the marker snaps to when several are within reach, the topmost when two are as near, so that a door between two close walls goes into the one I mean.
43. As an Author, I can click while the marker shows to place the Portal set into that Wall at that point, facing that side, so that placing a door is one gesture.
44. As an Author, I can click away from any Wall to place the Portal freestanding, upright and centred on the click, so that a door leaning against a cave wall or a gate in the open is as easy as a Prop.
45. As an Author, I can rely on placing a Portal into a Wall being one undo step, so that undo takes the Portal away whole and leaves the Wall unbroken.
46. As an Author, I can see a placed Portal at its image's natural size, its width along the Wall, so that a vendor's door is as wide as the vendor meant.
47. As an Author, I can keep placing Portals after placing one, so that the doors of a whole dungeon are one tool choice.
48. As an Author, I can press Escape to leave the Portal tool and go back to selecting, so that I never place a Portal by accident.
49. As an Author, I can choose the Wall tool while the Portal tool is active and have the Portal tool give way, and choose the Portal tool while drawing a Wall and have the unfinished Wall thrown away, so that one thing is ever happening under my pointer.

### How a Portal looks

50. As an Author, I can see a Portal set into a Wall drawn centred on the Wall's line and turned to follow the Wall's direction at that point, on a straight segment and on a curved one alike, so that a door sits in its wall however the wall runs.
51. As an Author, I can see the Wall left out along the Portal's width, ending squarely at either side of the Portal, so that the Portal stands in a real gap.
52. As an Author, I can see a Portal facing the side I chose, its image mirrored across the Wall's line when it faces the other side, so that a door opens into the room I mean.
53. As an Author, I can place a Portal across a point of a Wall and see the gap follow the Wall around the corner, so that a door at a corner is drawn as it would be built.
54. As an Author, I can place a Portal near the end of a Wall and see the gap stop at the Wall's end, with no rounded end left beyond it, so that a doorway at the end of a wall needs no special care.
55. As an Author, I can place two Portals that overlap, and see both drawn in stacking order and the Wall left out along both, so that a door and its frame can be two images.
56. As an Author, I can see a freestanding Portal drawn centred on its position, turned by its rotation, and flipped when mirrored, so that it looks like a Prop I can turn.
57. As an Author, I can see a Portal whose image is missing or broken drawn as a placeholder of its size, turned and set into its Wall like the image would be, with the gap still cut, so that a missing door still shows where the doorway is.

### Selecting and editing Portals

58. As an Author, I can click a Portal standing in a gap and get the Portal rather than the Wall, whatever their stacking order, so that the gap belongs to the door.
59. As an Author, I can drag a Portal set into a Wall and have it slide along the Wall, following the nearest point on the Wall's line, across the Wall's points, so that moving a door along its wall never pulls it out of the wall.
60. As an Author, I can rely on sliding a Portal being a single undo step however long the drag, so that undo returns it to where the drag began.
61. As an Author, I can drag a freestanding Portal to move it as I move a Prop, as one undo step, so that freestanding Portals behave like Props.
62. As an Author, I can flip the side a selected Portal faces, or a freestanding one's mirroring, with `X` or from the options strip, as one undo step, so that a door opening the wrong way is one key away from right.
63. As an Author, I can change a selected Portal's width in the options strip, its height following the image's proportions, as one undo step, so that a double door or a narrow window is the same image at another size.
64. As an Author, I am refused a width that is zero or less, with nothing changed, so that a Portal never vanishes through a typo.
65. As an Author, I can change a freestanding Portal's rotation in degrees in the options strip, as one undo step, so that it stands at any angle.
66. As an Author, I can press Delete with a Portal selected to remove it, and see the gap in its Wall close, so that removing a door restores the wall.
67. As an Author, I can rely on undoing that removal bringing the Portal back set into the same place of the same Wall, so that undo never loses an anchor.
68. As an Author, I am told in the status line that dragging a set Portal slides it along its Wall, so that I know what a drag will do.

### Freeing and setting

69. As an Author, I can free a selected Portal set into a Wall with `F` or from the options strip, and see it stay exactly where it stood, so that a door can be taken out of its wall without moving.
70. As an Author, I can rely on a freed Portal no longer following its former Wall, and on the Wall closing behind it, so that freeing means free.
71. As an Author, I can set a selected freestanding Portal into the nearest Wall within reach of its centre with `F` or from the options strip, at the nearest point on that Wall's line, so that a door dropped beside a wall snaps into it.
72. As an Author, I am told in the status line that no Wall is within reach when I set a Portal that stands far from every Wall, and nothing changes, so that a key press that cannot work says why.
73. As an Author, I can rely on setting and freeing each being one undo step that undo returns exactly to how the Portal stood, so that trying out a door costs me nothing.

### Portals through every Wall edit

74. As an Author, I can drag a point of a Wall and see the Portals set into its two segments move with the segments, each keeping its place along its segment, so that reshaping a room never leaves a door hanging.
75. As an Author, I can drag a point two segments away from a Portal and see that Portal not move at all, so that fixing one corner never disturbs doors elsewhere.
76. As an Author, I can bend or straighten a segment and see the Portals on it follow the curve, turned to its direction at their place, so that a curved wall keeps its doors.
77. As an Author, I can move a whole Wall and see its Portals move with it, so that a wall and its doors move as one.
78. As an Author, I can change a Wall's thickness and see its Portals and their gaps unchanged along the line, so that a thicker wall keeps its doors.
79. As an Author, I can add a point to a Wall, even on the segment a Portal sits on, and see every Portal stay exactly where it was, so that adding a corner never moves a door.
80. As an Author, I can remove a point of a Wall and see the Portals on the two segments it joined carried onto the joined segment, each at the same share of the way along, so that taking out a corner keeps the doors beside it.
81. As an Author, I can remove a point that a Portal covers and see that Portal removed with the point, and be told in the status line how many Portals went, so that a door is never left floating over a corner that no longer exists, and never silently.
82. As an Author, I can remove an end point of a Wall and see the Portals on its last segment removed with the segment and be told, so that a door goes when the stretch of wall it stood in goes.
83. As an Author, I can remove a Wall and see the Portals set into it removed with it, and be told how many, so that no door is left standing in no wall.
84. As an Author, I can undo any of these removals and get the Wall, its points, and every Portal back exactly, set where they were, so that undo restores the doors with the wall.
85. As an Author, I can rely on no Wall edit moving or removing a freestanding Portal, so that a freestanding Portal is independent of every Wall.
86. As an Author, I can rely on a Portal on a segment I dragged down to no length keeping the angle it had, so that collapsing a segment never spins a door.

### Painting Terrain

87. As an Author, I can choose the Paint tool from the viewport's tool strip, or press `B`, so that my next drags paint instead of placing or selecting.
88. As an Author, I can choose the image my Brush paints with in the browser, before or after choosing the Paint tool, so that any texture I own is ground I can paint.
89. As an Author, I can press on the Level, drag, and release to lay one stroke along the path my pointer took, so that painting is one gesture.
90. As an Author, I can press and release without moving to paint one round dab, so that a single spot of ground is one click.
91. As an Author, I can see a circle as large as my Brush following the pointer while the Paint tool is chosen, so that I know how much ground a stroke will cover before I press.
92. As an Author, I can see the stroke I am drawing as a translucent band as wide as my Brush, so that I see where it goes while I drag.
93. As an Author, I can see the painted ground in place of the band once I release, so that the result is visible the moment the stroke is done.
94. As an Author, I can keep painting stroke after stroke without choosing anything again, so that covering a room is a series of drags.
95. As an Author, I can paint more onto a Layer's Terrain with no Asset chosen, its own image being what the Brush paints with, so that coming back to a floor later needs no search in the browser.
96. As an Author, I am told in the status line to choose an Asset when I press with the Paint tool on a Layer that has no Terrain and none is chosen, and nothing is painted, so that a drag without a texture never leaves an invisible Element.
97. As an Author, I can read in the status line that a drag paints and Escape stops, and which Asset I paint with, so that I know what my next drag will do.
98. As an Author, I can paint across and beyond the Bounds, so that the Bounds never get in the way of composing.
99. As an Author, I can press Escape while drawing a stroke to throw it away and go back to selecting, so that a stroke started by accident costs nothing.
100. As an Author, I can choose the Wall or the Portal tool while painting and have the Paint tool give way, discarding a stroke being drawn, and choose the Paint tool while drawing a Wall and have the unfinished Wall thrown away, so that one thing is ever happening under my pointer.

### The Brush

101. As an Author, I can set the Brush's size in Grid cells in the tool's options, so that a broad stroke fills a hall and a narrow one traces a path.
102. As an Author, I can set the Brush's hardness, from a fully soft edge to a hard one, so that ground fades into the background or ends crisply as I choose.
103. As an Author, I can set the Brush's strength, so that a stroke shows its texture only partly, as worn ground does.
104. As an Author, I can rely on each stroke keeping the Brush settings it was laid with when I change them afterwards, so that turning the size down for the next stroke never shrinks the ones already laid.
105. As an Author, I can rely on the Brush starting at a sensible size, hardness, and strength in a new editor, so that the first stroke looks right without tuning.

### How Terrain looks

106. As an Author, I can see a stroke fully show its texture in the middle and fade to nothing at its edge, the fade as wide as the hardness says, so that painted ground blends into what is around it.
107. As an Author, I can see a stroke of less than full strength show its texture partly over what is below it, so that worn or thin ground is one setting.
108. As an Author, I can rely on a stroke never getting darker where it turns sharply, crosses itself, or loops back, so that a stroke looks as even as the Brush I set.
109. As an Author, I can rely on going over ground again never making it more than the strongest stroke there, so that I can paint without counting how often I passed.
110. As an Author, I can rely on a weaker stroke painted over a stronger one leaving the stronger one showing, so that touching up an edge never thins out a floor.
111. As an Author, I can see the texture repeated edge to edge across the Level at its natural size, the same as if it were placed as a Prop, so that a vendor's flagstones are as big as the vendor meant and line up from one stroke to the next.
112. As an Author, I can rely on two separate strokes of the same Terrain showing one continuous texture where they meet, so that a floor painted in pieces looks like one floor.
113. As an Author, I can see the Terrain under every Prop and Wall already on its Layer, so that painting a floor never covers the furniture.
114. As an Author, I can place Props and draw Walls after painting and see them over the Terrain, so that the floor stays the floor.
115. As an Author, I can see a Terrain whose image is missing or broken still show where its strokes lie, in the placeholder colour, so that a missing texture never hides the shape of my ground.

### Changing Terrain

116. As an Author, I can rely on one Terrain per Layer gathering every stroke I paint on that Layer, so that a floor is one Element however many strokes it took.
117. As an Author, I can change the image a Layer's Terrain shows to the Asset chosen in the browser from the tool's options, every stroke keeping its path and Brush settings, as one undo step, so that trying another floor texture costs no repainting.
118. As an Author, I am told in the status line, with nothing painted, when I paint with an image other than the one the Layer's Terrain shows, so that I am never surprised by a whole floor changing its look, and I know to change the Terrain's image instead.

### Terrain in history and selection

119. As an Author, I can undo a stroke and have exactly that stroke disappear and everything else stay as it was, so that undo is as fine-grained as my painting.
120. As an Author, I can undo the first stroke on a Layer and have its Terrain disappear with it, and redo it to have the same Terrain back, so that painting and unpainting a floor is one line of history.
121. As an Author, I can redo an undone stroke and have it back exactly as it was laid, so that undo is never a loss.
122. As an Author, I can rely on the ground looking exactly as it did before a stroke once I undo it, so that undo never leaves a trace.
123. As an Author, I can rely on changing the Brush's settings never being an undo step, so that tuning the Brush costs me nothing.
124. As an Author, I can rely on clicking where only Terrain lies selecting nothing, so that the floor never gets in the way of picking what stands on it.

### History and the view

125. As an Author, I can undo and redo with the usual shortcuts every step above, in the order I took them, whether it was a Wall, a Portal, a stroke, or a Prop, so that I never need a menu to take a step back and history stays one line.
126. As an Author, I can rely on a new action after undoing discarding the undone steps, so that history stays a single line I can reason about.
127. As an Author, I can rely on undo and redo, and flipping, freeing, or setting a Portal, waiting while I am drawing a Wall or a stroke, dragging, or holding an option of the tool strip while it changes, so that a step is never taken back while it is still being made.
128. As an Author, I can rely on selecting an Element, a point, or a control point, and hovering with the Portal tool, never being an undo step, so that looking at my work costs me nothing.
129. As an Author, I can pan and zoom the viewport without that showing up in undo, so that looking around never costs me a step.
130. As an Author, I can see a Prop or a Portal whose image cannot be loaded as a placeholder of the right size while the editor keeps running, so that a deleted or broken file never takes the editor down.
131. As an Author, I am told in the status line when a Command could not be carried out, with nothing changed, so that a failure costs me a retry and never a crash.

## Rules

### Placing and history

**A Project to start with**: the editor opens with a new, unsaved Project holding one Level named `Level 1` with one Layer named `Layer 1`.

**Placed where clicked**: with an Asset chosen and none of the Wall, the Portal, and the Paint tool chosen, a click on the Level places a Prop of that Asset on the current Layer, centred on the clicked point.

**Natural size**: a Prop's size in Grid cells is its image's pixel size divided by the Grid's pixels per cell, which is 256.
_Why_: the convention of the functional baseline, so its libraries place at the size their vendors meant.

**Placed on top**: a new Element is placed above every Element already on its Layer, except a Terrain made by a Paint, which is placed below them (Terrain goes under).

**Placement records a reference**: placing a Prop or a Portal records in the Project an Asset Reference holding the Asset's name, the Canonical Name of its Asset Folder, the place in that folder it was placed from as the first place it is known to sit, its byte size, its pixel size, and its content fingerprint; placing a second Element of the same Asset adds no second Asset Reference; a Paint that makes a Terrain, and an Edit Element setting a Terrain's Material, record the image's Asset Reference the same way, and no second one for an Asset already recorded. Follows from: A Project is device-independent.

**Placement records the folder**: the first Prop or Portal placed from an Asset Folder, and the first Terrain image taken from one, records that folder's Canonical Name and version in the Project; later Elements from the same folder add no second record. Follows from: A Missing Asset is always explainable.

**Anywhere on the Level**: an Element may lie outside the Bounds, a Prop placed there, a Wall with points there, and a Terrain with strokes there alike. Follows from: Bounds only decide what is exported.

**Many of the same**: several Props placed from the same Asset are independent Elements, each with its own ElementId. Follows from: No Element kind is limited to one per Level.

**Escape stops placing**: pressing Escape drops the chosen Asset and leaves the Wall tool, discarding a Wall being drawn, the Portal tool, or the Paint tool, discarding a stroke being drawn, and clicks select instead.

**Topmost is selected**: with the Select tool and no Asset chosen, a click selects the topmost Element under the pointer, a Prop by its rectangle, a Portal by its turned rectangle, and a Wall by its line outside the stretches its Portals cover, a Terrain never (Terrain is not picked); a click on empty space clears the selection; selecting an Element or one of its handles is never a history step.

**A drag is one step**: moving an Element by dragging records a single undo step however long the drag, and undo returns the Element to where the drag began.

**Removal is reversible in place**: undoing a Remove Element restores the Element with every property, its ElementId, and its place in the stacking order.

**Identity survives undo**: an Element removed by undoing a Place Element and brought back by redo has the ElementId it had before.

**Redo repeats exactly**: redoing a Place Element, Edit Element, Remove Element, Set Portal into Wall, Free Portal, or Paint leaves the Level as it was before the undo.

**A new step clears redo**: a Command applied after an undo discards the undone steps.

**One history**: Add Asset Folder, Place Element, Edit Element, Remove Element, Set Portal into Wall, Free Portal, and Paint are each one undo step, and undo walks back through them in the order they were applied whichever Manager handled them. Follows from: Every Command can be undone.

**Undo waits for the step being made**: undo and redo, from the keys or the menu, wait while a drag, a Wall or a stroke being drawn, or an option of the tool strip held while it changes is under way.

**View is not a step**: panning and zooming the viewport are not Commands and never appear in the history.

**A failed load is a placeholder**: an Element whose Asset cannot be loaded or decoded, an Element whose Asset is Missing, and an Element of a kind this editor does not know are drawn as the same placeholder of their recorded size, stay on their Layer, and the editor keeps running; a Terrain whose image cannot be loaded or decoded, or is Missing, is drawn as its coverage in the placeholder's colour, not as a box.

**A failed Command is reported**: a Place Element, Edit Element, Remove Element, Set Portal into Wall, Free Portal, or Paint that cannot be carried out is answered with the reason, places or changes nothing, and records no history step.

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

**Two points or none**: removing a point from a Wall of two points removes the Wall and the Portals set into it, as one history step that undoes to the Wall with both points and its Portals.

**Properties stay editable**: a Wall's thickness and colour are each changed through Edit Element, every change a step of its own. Follows from: Nothing is fixed at creation.

**Malformed Walls are refused**: a Place Element of a Wall with fewer than two points, a point that is not finite, or a thickness not above zero or not finite, and an Edit Element naming a point or segment the Wall does not have, adding a point not strictly between its segment's two points, putting the Wall, a point, or a control point where it is not finite, or setting such a thickness, are answered with the reason, change nothing, and record no history step.

**Drawn as a stroke**: a Wall is drawn centred on its line, as wide as its thickness, with round joins at its points and round caps at its ends, in its colour, at its place in the stacking order, except along the stretches its Portals cover (The Wall gives way). Follows from: Every Element that shows a surface is drawn with a Material.

### The Wall tool

**Drawing with the Wall tool**: with the Wall tool chosen, from the tool strip or with `W`, each click on the Level adds a point unless it lands within a few pixels of the last one; the Wall in progress is previewed with a segment from its last point to the pointer; Enter finishes it, a double-click finishes it at the point its first click added, a finished Wall of fewer than two points is discarded without a Command, the tool stays chosen for the next Wall, and undo and redo wait while a Wall is being drawn.

**One thing under the pointer**: choosing an Asset leaves the Wall tool and discards a Wall being drawn, unless the Portal tool or the Paint tool is chosen, which makes it the Portal's image or the image the Brush paints with and keeps that tool; choosing the Wall tool drops the chosen Asset and the selection and leaves the Portal tool or the Paint tool, discarding a stroke being drawn; choosing the Portal tool leaves the Wall tool or the Paint tool, discarding a Wall or a stroke being drawn, drops the selection, and keeps a chosen Asset as the Portal's image; choosing the Paint tool leaves the Wall tool or the Portal tool, discarding a Wall being drawn, drops the selection, and keeps a chosen Asset as the image the Brush paints with.

**Options follow the selection**: with a Wall selected, the tool's options show its thickness and colour and a change to either is sent as one Edit Element, a typed thickness of zero or less as typed, so that it is refused with the reason; with none selected, they set the thickness and colour the next Wall is drawn with, an eighth of a cell and a dark grey to start, and a typed thickness of zero or less leaves the next Wall's as it was; the thickness offered goes up to sixteen cells, dragged or typed, a selected Wall thicker than that shows its own thickness and sends nothing until changed, and a drag never takes it below a hundredth of a cell.

**Hit within the thickness**: a Wall is under the pointer when the pointer is no farther from its line than half its thickness or four screen pixels, whichever is more, and the nearest point of its line lies in no stretch a Portal covers; with a Wall selected, its handles are hit before any Element.

**Handles of the selected Wall**: the selected Wall shows a handle at each point, at each control point with guide lines to its segment's two points, and at the middle of each straight segment; dragging a point moves it, dragging a control point or a straight segment's middle handle puts that segment's control point under the pointer, and a double-click on the Wall's line adds a point at the nearest place on it.

**Delete acts on what is picked**: a click on a point or a control point selects it, and a click on a straight segment's middle handle, on the Wall's line, or on empty space lets it go, the middle handle being for dragging only; Delete removes the selected point, straightens the selected control point's segment, or, with no handle selected, removes the selected Element.

### The Portal

**A Portal is an image with a width**: a Portal shows an image Asset at a width in cells, with a height that keeps the image's proportions, and is either freestanding, with a position, a rotation, and whether it is mirrored, or set into a Wall, anchored by that Wall's ElementId, the index of one of its segments, a parameter along that segment from 0 to 1, and a side, left or right of the segment's direction from its first point to its second.

**Natural width**: a placed Portal's width and height are its image's pixel width and height divided by the Grid's 256 pixels per cell.

**Set Portals stand on the line**: a Portal set into a Wall is centred on the point of its segment at its parameter, with its width along the segment's direction there and its image's top facing its side: drawn as it is when it faces the left, mirrored across the Wall's line when it faces the right. Follows from: A Portal set into a Wall moves with it.

**A Portal covers its width**: a Portal set into a Wall covers the stretch of the Wall's line that reaches half its width either way from its centre, measured along the line and across the Wall's points, and stopping at the Wall's ends.

**The Wall gives way**: a Wall is not drawn along any stretch a Portal set into it covers; at each end of such a stretch the stroke ends squarely across the line, and an end of the Wall that a stretch reaches has no cap; overlapping stretches leave out what either covers.

**Freestanding like a Prop**: a freestanding Portal is drawn centred on its position, turned counter-clockwise by its rotation, and mirrored across its length when it is mirrored.

**Portals stack like Elements**: every Portal is drawn at its place in the stacking order, Portals that overlap each other included, and a Portal whose image cannot be drawn is a placeholder of its size, turned, mirrored, and covering its stretch as the image would. Follows from: Stacking order.

### Set Portal into Wall and Free Portal

**Placed into a Wall at once**: a Place Element of a Portal with an anchor places the Portal already set into the Wall, as one history step that undo takes away whole and redo brings back set into the same place. Follows from: Every Command can be undone.

**Set into a Wall**: a Set Portal into Wall anchors a Portal, freestanding or set into any Wall, to the given Wall, segment, parameter, and side, as one history step that undo returns to the anchor, position, rotation, and mirroring it had. Follows from: Every Command can be undone.

**Freed where it stands**: a Free Portal makes a Portal set into a Wall freestanding with the position, rotation, and mirroring it had, so nothing moves on the Level and its Wall is drawn whole again, as one history step that undo returns to the same anchor.

**Refused anchors**: a Set Portal into Wall or a Place Element of a Portal whose anchor names an Element that is not a Wall, a Wall on another Level, a segment the Wall does not have, or a parameter outside 0 to 1, a Free Portal of a freestanding Portal, and a Set Portal into Wall or Free Portal of an Element that is not a Portal are answered with the reason, change nothing, record no history step, and add no Asset Reference to the Project.

### Editing Portals

**Sliding along the Wall**: an Edit Element changing a set Portal's segment and parameter moves it along its Wall with its side kept, and a drag of it records a single history step however long, which undo returns to where the drag began.

**Portals stay editable**: a Portal's width, a set Portal's side, and a freestanding Portal's position, rotation, and mirroring are each changed through Edit Element, every change a step of its own outside a drag; a width not above zero, a segment or parameter its Wall does not have, a position, rotation, or mirroring of a set Portal, which follow its Wall, and a side or a place along a Wall of a freestanding Portal are refused with the reason, change nothing, and record no history step. Follows from: Nothing is fixed at creation.

**Removed Portals return set**: undoing the removal of a Portal set into a Wall restores it with its ElementId, its anchor, and its place in the stacking order, and its Wall gives way again.

### Portals through Wall edits

**Moves with its Wall**: moving a Wall, moving a point, setting or unsetting a control point, and changing the thickness or the colour change no Portal's segment, parameter, or side; each Portal set into the Wall stands at its parameter on its segment as the segment now is. Follows from: A Portal set into a Wall moves with it.

**Adding a point keeps Portals in place**: adding a point on segment *k* at parameter *s* moves a Portal on segment *k* at a parameter *t* below *s* to parameter *t* / *s* on segment *k*, one at or above *s* to parameter (*t* − *s*) / (1 − *s*) on segment *k* + 1, and every Portal on a later segment one segment on, in the same history step, so no Portal moves on the Level. Follows from: A Portal set into a Wall moves with it.

**Removing a point carries the Portals beside it**: removing an inner point moves each Portal on the two segments it joins whose stretch does not cover it onto the joined segment, at the parameter equal to the share of the two segments' combined length that lay before the Portal's centre, and every Portal on a later segment one segment back; removing the first point moves every Portal on a remaining segment one segment back; all in the same history step. Follows from: A Portal set into a Wall moves with it.

**Gone with its part of the Wall**: a Portal whose stretch covers an inner point being removed, a Portal on the segment that removing an end point takes away, and every Portal set into a Wall being removed, by Remove Element or by removing a point of a two-point Wall, are removed in the same history step, which undo restores whole; the Author is told in the status line how many Portals were removed. Follows from: A Portal set into a Wall moves with it.

**Freestanding Portals stay put**: no edit of any Wall moves, turns, or removes a freestanding Portal.

**No direction keeps the rotation**: a Portal set into a Wall at a place where its segment has no direction, a segment of no length or a curve whose control point lies on its end, keeps the rotation it had.

### The Portal tool

**Placing with the Portal tool**: with the Portal tool chosen, from the tool strip or with `P`, the Asset chosen before or while it is chosen is its image; with no Asset chosen a click places nothing and the status line asks for one; within reach of a Wall, no farther from its line than half a cell or half its thickness, whichever is more, the pointer shows a marker across the nearest such Wall's line at the nearest point on it, pointing to the side of the line the pointer is on, the topmost Wall winning a tie; a click there sends a Place Element of a Portal anchored at that point and side, a click out of reach sends one freestanding, unturned and centred on the click; the tool stays chosen.

**Picking Portals**: a Portal is under the pointer anywhere within its turned rectangle, and a Wall is not under the pointer where the nearest point of its line lies in a stretch a Portal covers.

**Dragging Portals**: dragging a selected Portal set into a Wall sends Edit Element changes of its segment and parameter to the nearest point on its own Wall's line, from press to release as one gesture, and the status line says a drag slides it along its Wall; dragging a freestanding Portal sends position changes as dragging a Prop does.

**Portal options and keys**: with a Portal selected, the options strip shows its width, its rotation in degrees when freestanding, a Flip button, and a Free Portal or Set into Wall button, a width or a rotation dragged or held while it changes sent as one gesture and a typed one as one step; `X` flips the side of a set Portal or the mirroring of a freestanding one; `F` frees a set Portal, or sets a freestanding one into the nearest Wall within reach of its centre, the topmost winning a tie, at the nearest point on that Wall's line and facing the right when it is mirrored or the left when it is not, and with no Wall within reach says so in the status line and sends nothing; `X` and `F` wait, as undo does, while a step is being made; Delete removes it.

### The Terrain

**Terrain is its strokes**: a Terrain holds one Material, which shows an image Asset, and an ordered list of strokes, each a path of one or more points in Grid cells with the Brush settings it was laid with: a size in cells above zero, the diameter the Brush covers; a hardness from 0 to 1; and a strength above 0 up to 1. Follows from: Nothing is fixed at creation.

**Shaped by a soft round Brush**: a stroke's coverage at a point, how much of the Material it shows there, is computed from the point's distance *d* to the nearest point of its path (the straight segments from each point to the next, or the single point of a one-point path), with radius *r* half its size and hardness *h*: its strength where *d* ≤ *h* × *r*; nothing where *d* ≥ *r* and *d* > *h* × *r*; and between them its strength × (1 − 3*t*² + 2*t*³), where *t* = (*d* − *h* × *r*) / (*r* − *h* × *r*).

**No build-up along a stroke**: a stroke's coverage at a point is the same however many of its segments pass near the point, at a joint, a self-crossing, or a loop alike.

**Strokes composite by the strongest**: a Terrain's coverage at a point is the largest coverage any of its strokes has there, so a later, weaker stroke never lowers it and passing over ground again never raises it past the strongest stroke.
_Why_: strength is how much of the Material a stroke shows, not how much it adds; and a maximum is exact in one pass on the GPU as well as on the CPU.

**Tiled at natural size**: a Terrain's Material shows its image repeated edge to edge across the Level, upright, each repetition the image's natural size in Grid cells (its pixel size divided by the Grid's 256 pixels per cell), one repetition with its lower-left corner at the Level's origin.

**Drawn masked by coverage**: a Terrain is drawn at its place in the stacking order as its Material's image over what lies below it, at each point as opaque as its coverage there times the image's own opacity: as the image where the coverage is full, not at all where there is none. Follows from: Every Element that shows a surface is drawn with a Material.

**The box follows the strokes**: a Terrain's position is the centre and its size the extent of the smallest box holding every point of every stroke grown by that stroke's radius on every side.

**Terrain goes under**: a Terrain made by a Paint is placed below every Element already on its Layer. Follows from: Stacking order.

**One Terrain per Layer**: a Paint on a Layer adds its stroke to the topmost Terrain on that Layer, and makes a Terrain only when the Layer has none.

### Paint

**A stroke is one step**: a Paint adds one stroke at the end of its Terrain's strokes as one history step; undo takes exactly that stroke away and redo puts it back, the same stroke at the same place in the order. Follows from: Every Command can be undone.

**The first stroke makes the Terrain**: a Paint on a Layer with no Terrain places a Terrain of the given image holding that stroke, in the same history step; undo takes the Terrain away whole and redo brings it back with the ElementId it had. Follows from: Every Command can be undone.

**Painted with its Material**: a Paint naming no image paints with its Terrain's Material and is refused on a Layer with no Terrain; a Paint naming an image other than the one its Terrain's Material shows is refused, with the reason naming both.

**Undo leaves no trace**: after undoing a stroke, every point of its Terrain has the coverage it had before the stroke was laid. Follows from: Every Command can be undone.

**Coverage is the strokes alone**: a Terrain's coverage is the same however its strokes came to be, laid one by one, undone and redone, or opened from a file. Follows from: Nothing is fixed at creation.

**Malformed strokes are refused**: a Paint with no point, a point that is not finite, a size not above zero or not finite, a hardness outside 0 to 1, or a strength not above 0 or above 1 is answered with the reason, changes nothing, and records no history step.

**The Material stays editable**: an Edit Element setting a Terrain's Material to another image Asset makes every stroke show that image, keeping every stroke's path and Brush settings, as one history step that undo returns to the image it had; one naming the image the Terrain already shows changes nothing and records no step. Follows from: Nothing is fixed at creation.

**Terrain changes only its Material**: an Edit Element of a Terrain that changes anything but its Material, and an Edit Element setting the Material of an Element that is not a Terrain, are answered with the reason, change nothing, and record no history step.

### The Paint tool

**Painting with the Paint tool**: with the Paint tool chosen, from the tool strip or with `B`, a press on the Level starts a stroke at the pointer, moving adds the pointer's path to it, and the release sends one Paint onto the current Layer with the path and the Brush's settings, naming the chosen Asset or, with none chosen, no image; with no Asset chosen and no Terrain on the current Layer, a press paints nothing and the status line asks for an Asset; the tool stays chosen; undo and redo wait while a stroke is being drawn.

**Seeing the stroke**: with the Paint tool chosen, a circle as large as the Brush's size follows the pointer over the Level, and a stroke being drawn is shown as a translucent band as wide as the Brush along its path, until the release.

**Brush options**: with the Paint tool chosen, the tool's options show the Brush's size in cells, its hardness and its strength as percentages, and the name of the image it paints with, the chosen Asset's or the current Layer's Terrain's, and the status line says that a drag paints and Escape stops, naming the chosen Asset as the one painted with; the Brush starts at a size of two cells, a hardness of 50 %, and a strength of 100 %; the size offered goes from a tenth of a cell to sixty-four cells, the strength from 1 % to 100 %; the Brush's settings are the Editor's, never a history step and never saved; and, when an Asset is chosen and the current Layer's Terrain shows another image, a button there sends the Edit Element that makes the Terrain show the chosen Asset.

**Terrain is not picked**: the Select tool never selects a Terrain; a click where only Terrain lies under the pointer clears the selection.

## Implementation Decisions

The technology the architecture fixes (Portal anchoring by Wall ElementId, segment index, parameter, and side, never by arc length, with the outline-changing Command removing a Portal whose part of the Wall vanishes in the same undo group; ShapeEngine's AnchorPortals and GenerateWalls; strokes as the truth of a painted region, coverage as the maximum over segments, and the CPU rasterizer as the golden reference; the tiled pixel cache opaque to all but PaintEngine and held by the Manager that calls ApplyStroke; one masked draw per Material; the Element kind registry; the generic field-setting and reflection-snapshot history commands; the deterministic maths functions) is used as written there and not restated.

- **The new Project** is created at startup by ProjectManager, the owner of the Project lifecycle: named `Untitled`, with one Level, one Layer, a Grid of 256 pixels per cell, and default Bounds of thirty by thirty cells from the origin, which are neither drawn nor edited.
- **Model**: the Project is an entity carrying its Grid, its Bounds, its Asset Reference table, and the resolution table that says where each Asset Reference loads from on this device, which ProjectManager alone writes and no file holds; its Levels are its children, each Level's Layers are that Level's children, and each Layer's Elements are that Layer's children in stacking order, the first drawn first. Every Element carries its kind, its position (its centre in Grid cells, `x` to the right and `y` upwards from the Level's origin), its size in cells, and an ElementId: a stable identity Commands and the history address it by, never the entity handle, which changes whenever an Element is respawned. The Element kind registry holds Prop, drawn as an image, which refers to its Asset by the Project-local row of the Asset Reference table; Wall, drawn as a stroked path; Portal, drawn as an image like a Prop; and Terrain, drawn as a painted surface. Whoever needs the Asset an Element shows, a Prop's, a Portal's, or a Terrain's image, reads its row through one query `model` offers for all three.
- **The Wall component** holds the points, one entry per segment with its optional control point, the thickness, and the colour, without alpha; points and control points are in Grid cells like every position in the model. It is a serialisable component on the Element tier under the stable name `wall`, and it says itself what makes a Wall malformed, so the Commands and the reading of a file refuse the same Walls for the same reasons.
- **The Portal component** holds the Asset Reference row of its image, its width in cells, its rotation in radians counter-clockwise, whether it is mirrored, and an optional anchor of four fields: `host`, the ElementId of the Element it is set into; `index`, the number of the part of that Element, a Wall's segment; `t`, the parameter along that part; and `side`, left or right. The fields name what the Portal is set into and its part, never a kind, so that a Room's edge fills them as a Wall's segment does. It is a serialisable component at version one on the Element tier under the stable name `portal`, and it says itself what makes a Portal malformed (a width not above zero or not finite, a rotation that is not finite, a parameter outside 0 to 1), so the Commands and the reading of a file refuse the same Portals. The Element's position is the Portal's centre and its size the width by the height the image's recorded pixel size gives. A set Portal's position, rotation, and mirroring are kept equal to what its anchor gives, so freeing it is clearing the anchor and nothing else, and the file holds the same values the editor draws.
- **The derived shape**: `model` also holds a Wall's derived shape, never saved: its line flattened into points, each tagged with the segment it lies on and the parameter along that segment; the stretches its Portals cover, as ranges along the line from a segment and parameter to a segment and parameter; and its stroke mesh as vertices in cells with indices and the arc length at each vertex, already leaving the stretches out. AuthoringManager derives it through ShapeEngine once per frame after every Manager has handled its Commands, Undo, and Redo, for every Wall whose points, segments, or thickness changed and for every Wall whose Portals were placed, edited, set, freed, or removed, so a placed, edited, undone, redone, or opened Wall has its shape before anything draws it or hit-tests it; a new colour keeps the shape. The same system sets the Element's position and size from the points, writes each set Portal's position, rotation, and mirroring where its anchor puts it whenever they differ, and sets every changed Portal's size from its width and its image's recorded pixel size. A Portal whose anchor names no Wall of its Level, or a segment its Wall lacks, is left as it is and adds no stretch. _Why_ derived in the model: the Editor hit-tests it and RenderEngine draws it, and neither may depend on ShapeEngine.
- **The Terrain component** holds the Material and the strokes. The Material is the built-in masked tiled image, so the component holds only what varies, the Asset Reference row of the image it tiles, saved under `image`; and the strokes in the order they were laid, each its points in Grid cells and its Brush settings. The Brush settings (size, hardness, strength) are a type of their own in `model`, BrushSettings, shared by the stroke and by the Editor's current Brush, so what the Editor sets is what a stroke records. Strokes are the truth: no pixel of a Terrain is saved or kept in the history. It is a serialisable component at version one on the Element tier under the stable name `terrain`, and it says itself what makes a Terrain malformed (Malformed strokes are refused), naming the stroke by its number, so the Paint and the reading of a file refuse the same strokes for the same reasons. A Terrain with no strokes, which only a file can hold, is well formed, draws nothing, and has a box of no size at the origin.
- **The derived coverage**: `model` also holds a Terrain's coverage, never saved and not reflected: tiles of 512 by 512 pixels at 32 pixels per cell (16 cells a side), keyed by their whole-number position in the Level's pixel plane, negative keys included, an absent tile being empty, one byte a pixel, row 0 at the tile's top, each with a revision that changes only when its pixels do and its pixels in a shared buffer. AuthoringManager derives it once per frame after every Manager has handled its Commands, Undo, and Redo, for every Terrain whose component changed, so a painted, undone, redone, restored, or opened Terrain has its coverage before anything draws it, and sets the Element's position and size from the strokes in the same pass; a new image leaves the coverage as it is. It holds PaintEngine's cache in a component of its own beside the Terrain, opaque to all but PaintEngine, and publishes the coverage again only when ApplyStroke says a tile changed, sharing the cache's buffers, so publishing copies no pixels. A Terrain restored by undoing its removal has no cache, and its coverage is derived afresh. _Why_ derived in the model: RenderEngine draws it and may not depend on PaintEngine. _Why_ the Manager holds the cache: an Engine is called and keeps nothing between calls, so the cache lives with the caller, and only PaintEngine reads what is inside it.
- **PaintEngine**: Rasterize takes strokes in order, a region given by its lower-left corner in cells, rounded to the nearest whole pixel, and its size in pixels, and a number of pixels per cell, and returns one byte a pixel, row 0 at the top. A pixel's coverage is computed at its centre, found from its whole-pixel index in the Level's pixel plane divided by the pixels per cell once per axis, so a pixel has the same value in every region that holds it, as the Export's tiles need; a stroke's coverage there is its stamp at the distance to the nearest of its segments, the maximum over segments, and strokes composite by the maximum; the value is the coverage times 255, rounded half up. Only the pixels inside each stroke's box grown by its radius are visited, and for each, only the segments whose own grown box holds it; distances use square roots and arithmetic only, so the result is the same on every machine. ApplyStroke brings a Terrain's cache up to its strokes and says whether any tile changed: the strokes the cache already holds from the first onwards are kept; strokes appended after them are composited onto the tiles they touch; when an earlier stroke changed or went, as an undo makes it go, only the tiles touched by the strokes that differ, before or after, are rasterized again, from the strokes whose box reaches each tile, so an undo of a stroke recomputes that stroke's tiles alone. A tile left with no coverage is dropped. The cache holds the resident base band at 32 pixels per cell alone, rasterized on the CPU, for the editor at every zoom; the view-following overlay bands and GPU rasterization the architecture describes are not built, so the viewport shows the base band magnified at high zoom, soft Brushes as the architecture measured them and hard edges as a ramp a few screen pixels wide. BlendWeights is not built: one Material needs no weights. Both are plain functions over model types, and PaintEngine uses no Bevy crate beyond `bevy_math`.
- **Paint**: Apply carries a Paint with the Layer, the stroke (its points and Brush settings), and optionally the chosen Asset (folder key and place). AuthoringManager refuses a target that is not a Layer, checks the stroke through the Terrain component's own check, and looks for the topmost Terrain among the Layer's children. With none, a Paint naming no Asset is refused as having nothing to paint with; one naming an Asset resolves it as a Prop's placement does (LoadAsset, the Asset Reference and folder rows recorded on the first application and never removed) and records a step of its own that spawns the Terrain with a fresh ElementId as the first child of the Layer, holding the stroke, which undo takes off by identity and redo spawns again first, beneath everything, because every later step was undone first. With one, a named Asset is found in its folder's index and looked up in the Project's Asset Reference table by its folder's Canonical Name and its place: one not recorded, or recorded under a row other than the Terrain's image, is another image, and the Paint is refused naming both. Otherwise it records a step of its own that appends the stroke and, on revert, takes it off the end, refusing to when the last stroke is not its own, keeping only that stroke, as the architecture's History bullet requires.
- **Edit Element of a Terrain**: the Element change gains a Terrain's Material, carrying the chosen Asset. Every other change sent for a Terrain, its position included, and a Material change for an Element that is not a Terrain, is a CommandFailed. A Material change naming the image the Terrain already shows, found in the Asset Reference table as a Paint finds it, records nothing; any other resolves the Asset as a placement does, recording the rows on the first application, and records a step of its own that swaps the Terrain's image row and back, the strokes untouched. Remove Element of a Terrain goes through the generic reflection snapshot like any Element's.
- **ShapeEngine**: GenerateWalls takes a Wall's points, control points, thickness, and the stretches its Portals cover, and returns the derived shape: the line is flattened with the chord never more than a thousandth of a cell off the curve, so that it is at most a pixel off at the highest export resolution, in at most 4096 chords a segment, past which only a control point tens of thousands of cells from its segment goes, and the stroke is tessellated by ShapeEngine's own stroker with round joins and caps at that tolerance, computed with no trigonometry, through correctly rounded square roots only, so that an Export is the same on every machine; the stroke leaves out the stretches, ends each run squarely across the line, and caps only the Wall's own ends that no stretch reaches. AnchorPortals takes a Wall and the segment, parameter, and width of each Portal set into it, and returns for each its centre, the angle of the line's direction there, or none where the segment has no direction, and the stretch it covers; given a point added at a segment and parameter, or a point removed, its variant through a point edit returns where each anchor goes: kept on a new segment and parameter, or gone. The centre and direction are evaluated on the exact curve; lengths along the line are measured over the flattened line, summing its chords, so the stretch, the share of length at a removed point, and the gap's ends use square roots and arithmetic alone and come out the same on every machine; the angle is computed through the deterministic maths functions. SplitWall splits a segment at a parameter strictly between its points, exactly to single precision, into two segments of the same joined shape (a straight segment into two straight ones, a quadratic by subdivision). All are plain functions over model types.
- **Place Element**: the Editor sends AuthoringManager Apply(Place Element) with the Layer and what to place: a Prop's position and chosen Asset (folder key and place); a Portal's chosen Asset, its centre when freestanding, and an optional anchor; or a Wall's points, thickness, and colour. For a Prop or a Portal, AuthoringManager checks a Portal's anchor first (the host a Wall on the Layer's Level with the segment named), then finds the Asset in the added folder's index, asks LibraryAccess to LoadAsset, which reads the file once for its byte size, pixel size, and fingerprint, checks the Portal it would place against the Portal's own check, and only then records the rows and spawns the Element with a fresh ElementId as the last child of the Layer, a Portal already standing where its anchor puts it; ProjectManager resolves the table's new row in the same frame, before anything draws it. A Wall resolves nothing from the library: it is placed by a step of its own that spawns the Element with its Wall component as the last child of the Layer, the same spawning on top as a Prop's. Undo despawns any of them; redo spawns it again with the same identity, appended, which is on top because every later step was undone first.
- **Edit Element**: an Edit Element carries one change: a position, or on a Wall a point moved (its index and new position), a control point set or unset (the segment and an optional position), a point added (the segment and the parameter along it), a point removed (its index), a thickness, or a colour, or on a Portal its width, a freestanding one's rotation or mirroring, or a set one's side or place along its Wall (segment and parameter). The Editor sends a drag as a sequence of Apply(Edit Element) messages, marked as the beginning, continuation, and end of one gesture from press to release, the end sent at the pointer's last position; AuthoringManager records them as one history Group of the generic field-setting command, so the step undoes to where the gesture began. A change marked as single is a step of its own. A position change on a Prop or a freestanding Portal sets its position; on a Wall it sets the whole Wall component, every point and control point translated by the difference between the new position and the current one. Moving a point, setting a control point, changing the thickness or the colour, and every Portal change set that field through the same command, a Portal's side and place along its Wall by setting its anchor. Adding and removing a point, the only changes that renumber segments, are a step of their own that keeps the whole Wall as it was, so that undo restores it exactly. Removing a point from a two-point Wall records the removal of the Wall and its Portals instead. AuthoringManager builds the Wall or the Portal each change would leave and refuses it for the reason its component gives before anything is recorded, and refuses a position, rotation, or mirroring of a set Portal and a side or place of a freestanding one. A press that moves the pointer less than a few pixels is a click, not a drag. Undo and redo, from the keys or the menu, wait while a drag, a Wall or a stroke being drawn, or a tool-strip option held while it changes is under way. _Why_: the step is still being made.
- **Set Portal into Wall and Free Portal**: Apply carries a Set Portal into Wall (the Portal and the anchor) and a Free Portal (the Portal), both handled by AuthoringManager. Set Portal into Wall checks the host as a placement does and is a reversible command of its own that remembers the anchor, position, rotation, and mirroring before it, so undo returns a freestanding Portal to the angle it stood at rather than the Wall's. Free Portal clears the anchor through the generic field-setting command, since the position, rotation, and mirroring already hold where it stood.
- **Wall edits carry their Portals**: the commands that add and remove a Wall's points ask AnchorPortals where the Portals set into that Wall go and record one history group holding the removal of each Portal that is gone, through Remove Element, then the reshaped Wall, then the remapped anchors through the generic field-setting command, leaving out an anchor that does not move; so revert restores the anchors, then the point, then the removed Portals. A Portal whose anchor names a segment the Wall lacks is moved one segment on by an added point, like a Portal on a later segment, so it never comes to name the new segment and goes on standing where it was saved; a removed point leaves it as it is. Remove Element of a Wall, and the removal of a two-point Wall by removing a point, record a group holding the removal of every Portal set into it before the Wall's. Moving the Wall, a point, or a control point records nothing for its Portals: deriving moves them. When a Command removes Portals, AuthoringManager answers with a PortalsRemoved message in `model` naming the Wall and the ElementIds of the Portals removed, and the Editor writes how many to the status line.
- **Remove Element**: the generic reflection-snapshot command, extended with the Element's index among its Layer's children; undo restores the entity, a Wall with its points and curves and a Portal with its anchor, and rebuilds the Layer's whole order with it at that index, so the Elements above it keep their places.
- **Steps of their own**: Place Element, Remove Element, Set Portal into Wall, Free Portal, Paint, a Terrain's Material change, Add Asset Folder, and adding or removing a point each close any gesture group left open before recording, so none joins a drag.
- **Failures**: an authoring Command that cannot be carried out (a chosen Asset in no added folder or not in its index, a file that cannot be read or is not an image, a target that is not a Layer or an Element, a change only a Wall, only a Portal, or only a Terrain has sent for another Element, a change other than its Material sent for a Terrain, a malformed Wall, Portal, or stroke, a Paint with nothing to paint with or naming another image than its Terrain's, a point added at either end of its segment, an anchor whose host is not a Wall, lies on another Level, or lacks the segment, a Free Portal of a freestanding Portal) is answered with a CommandFailed message carrying the reason, and nothing is recorded. An undo or redo that fails is answered with a HistoryFailed message, and the step stays where it was in the history. The Editor shows either in its status line.
- **Undo and redo** are handled by AuthoringManager for the one history, whichever Manager recorded the step: an Undo or Redo message takes back or repeats the most recent step, be it a placement or an added folder.
- **Ordering**: `model` orders every Manager's handling within a frame, Commands before Undo before Redo, so a Command and the Undo sent in the same frame apply in the order the Author gave them whichever Manager handles each. The Editor draws its overlays (the selection outline, the Wall tool's handles and preview, the Portal tool's marker, and the Paint tool's circle and band) after the last Manager set, so they show the frame's own state.
- **The Viewport** (the cell at the centre of the view, the zoom in logical pixels per cell between 4 and 1024, and the area of the window the Level is shown in) is presentation state in `model`, written only by the Editor and followed by RenderEngine's projection; it starts centred on the origin at 64 pixels per cell; its conversions between cells and screen points are the ones picking and drawing share. _Why_ it lives in `model`: the Editor may not depend on RenderEngine, so the type both use sits with the other shared contracts. Panning is the middle button, Space with the left button, or a plain trackpad scroll; zooming is the wheel, a pinch, or a scroll with Command or Control held, around the pointer.
- **Rendering**: RenderEngine draws each Element as its kind's descriptor in the Element kind registry says, through change detection, placed one unit apart along the camera's axis in stacking order through every Level, so a later Element is drawn over an earlier one. An Element drawn as an image keeps one sprite: a Prop's or a Portal's image is loaded through the `lib://` handle from the folder key and the place, spelled as on disk, that the Project's resolution table gives for its Asset Reference, at the Element's size and position; RenderEngine never looks a folder up by Canonical Name itself. A Portal's sprite is turned by its rotation, the turn built through the deterministic maths functions so the Export is the same on every machine, and flipped across its length when mirrored. An image that fails to load, a Missing Asset, a row not yet resolved, and an Element of a kind this editor does not know give the same flat coloured placeholder of the Element's size, a Portal's turned and flipped as its image would be. An Element drawn as a stroked path keeps one mesh from its derived shape's stroke, replaced when the shape changes, with a flat-colour Material shared by every Wall of that colour, blended rather than opaque so that it sorts with the sprites by depth; one without a derived shape yet is not drawn that frame. An Element drawn as a painted surface keeps one quad per tile of its derived coverage (Drawing Terrain). The projection is a 2D camera with one cell per world unit that follows the Viewport, over a mid grey that a Wall's default dark grey and the editor's dark panels both stand apart from.
- **Drawing Terrain**: RenderEngine draws an Element drawn as a painted surface through a Material type of its own, the masked tiled image: a plain WGSL Shader with no Bevy imports, a Bundled File compiled into RenderEngine and added to the shader assets under a fixed handle once every plugin is built, whichever order they were added in, with a warning that Terrain is not drawn when there are no shader assets. The Shader declares at group 2 its Params uniform (the image's natural size in cells, the lower-left corner and the extent in cells of the square drawn, whether the flat colour stands in for the image, and that colour), the image and its sampler, and the coverage and its sampler. It wraps the image's coordinates itself and samples the image with the image's own sampler, so a Prop and a Terrain share one loaded image. Each coverage tile is uploaded as an R8 image sampled linearly when its revision changes, its Material touched so the new image is bound, and drawn as one quad of 16 by 16 cells with that Material, alpha-blended at the Terrain's depth in the stacking order, so it sorts with the sprites and Wall meshes, on a render layer only the viewport's camera sees. The image is loaded through the resolution table and the `lib://` handle as a Prop's is; while it is loading, Missing, or failed, the tiles are drawn in the placeholder's flat colour, masked by the same coverage. A Terrain whose coverage has not been derived yet is not drawn that frame. A unit test parses and validates the Shader with naga, so a mistake in it shows without a GPU.
- **The tool**: the Editor shows a tool strip over the viewport with Select, Wall, Portal, and Paint, and the options of the tool or the selection: the Paint tool's Brush, the Wall tool's thickness and colour, or the selected Portal's width, its rotation in degrees while freestanding, a Flip button, and a Free Portal or Set into Wall button. The tool, the Wall being drawn, the next Wall's thickness and colour, and the option being changed are Editor state; the Wall being drawn is drawn by the Editor as a thin preview line through its points with a rubber band to the pointer. Enter sends one Apply(Place Element) with the points, a double-click sends it with the point its first click added, and fewer than two points sends nothing. An option changed on the selected Element by dragging, or while the colour picker is open, is sent as one gesture, ended when its Element leaves the strip. The Portal tool's image is the chosen Asset, kept when the tool is chosen and replaced when another Asset is chosen while it is; a click sends one Apply(Place Element) of a Portal, anchored where the marker shows or freestanding. No preview of the image is drawn before the click, since its size is known only once the Asset is loaded at placement. While the Paint tool is chosen the status line says that a drag paints and Escape stops, and names the chosen Asset as the one painted with.
- **The Paint tool**: its state, the stroke being drawn and the current Brush, lives in an Editor module of its own, as the Wall tool's does. A press starts the path at the pointer; while the button is held a point is added whenever the pointer is more than an eighth of the Brush's size from the last point added, and the release point ends the path unless it lies on the last one, so a press and release without movement is a one-point path; a release outside the window sends the path as it stands. The circle and the band are drawn by the Editor over the viewport, the band translucent with round ends; the painted result appears once AuthoringManager has derived it, in the frame after the release. The options are drag values for the size, the hardness, and the strength, changed in the Editor's state only, so no gesture is needed, then the name of the image the Brush paints with and, when the chosen Asset is not where the current Layer's Terrain's image resolves on this device, a button that sends one Edit Element of the Terrain's Material marked single. The Editor finds the current Layer's Terrain, its topmost, in one place, and its image's name through the Asset Reference table. The development-only input script's `describe` step logs the Paint tool, the Brush's settings, the stroke being drawn, and every Terrain with its ElementId, box, image row, and number of strokes.
- **Picking and selection** are the Editor's hit-testing: it maps pointer positions to cells through the Viewport and hit-tests the selected Wall's handles first (points, then control points, then the middles of straight segments), then every Element from the topmost Layer down and the last-drawn Element back, leaving out every Element whose kind's descriptor says it is drawn as a painted surface, a Prop by its rectangle, a Portal by mapping the pointer into its turned frame against its rectangle, and a Wall by the distance to the nearest flattened chord of its derived shape against half its thickness or four screen pixels converted to cells, whichever is more, leaving out the chords inside the derived shape's stretches. A double-click on a Wall's line inserts a point at the nearest chord's segment and parameter, interpolated along the chord and kept strictly between the segment's points. The Portal tool's marker, its click, and `F` find the Wall under the pointer the same way, over the Walls in stacking order so the topmost wins a tie: the nearest point on each Wall's flattened line within reach, the segment and parameter interpolated along the nearest chord, and the side from which side of the chord's direction the pointer lies on; the marker is a short line across the Wall with an arrowhead to the side. The selection holds an Element and optionally one of its points or control points; it lives in the Editor, is outlined on the Level, a Portal's turned as it is drawn, is dropped when its Element is gone, by Delete or by a Wall edit, when an Asset is chosen for placing, or when the Wall, Portal, or Paint tool is chosen, and is never in the history. A drag of a handle sends Edit Element as a gesture from press to release, as a Prop drag does, and selects the control point a middle handle's drag creates; a drag of the Wall's line or of a freestanding Portal sends position changes the same way, and a drag of a set Portal sends its segment and parameter at the nearest point of its own Wall's line. The current Layer is the Project's only Layer.
- **Keyboard**: Escape drops the chosen Asset and leaves the Wall, the Portal, or the Paint tool, discarding what is being drawn; `W` chooses the Wall tool, `P` the Portal tool, and `B` the Paint tool; Enter finishes the Wall being drawn; `X` flips and `F` frees or sets the selected Portal and do nothing otherwise; Delete, and Backspace on macOS too, removes or straightens what is selected; the platform's standard undo and redo shortcuts drive history, stated once so that the menu shows exactly the keys that work. Undo, redo, `X`, and `F` wait while a step is being made. Nothing happens while a text field has the keyboard.

## Test seams

The automated seam is a headless Bevy App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, and AuthoringManager over one fixture Asset Folder of images with known pixel sizes, a door image among them, driven by Apply, Undo, and Redo messages and asserted on the World (for Walls, the Wall component, the derived shape and its stretches, the Element box, the Layer's children, and the history; for Portals, the Portal and Element components and the answers; for Terrain, over a folder of two texture images, the Terrain and Element components, the derived coverage, its pixel values read from its tiles at chosen cells and whole tiles compared, the Layer's children, the answers, and the history); it has no window and no RenderEngine. `A Project to start with` runs ProjectManager alone over `model`. The drawing Rules run on the export seam, a headless App with RenderEngine under Bevy's default plugins without a window, asserting on the pixels of an exported PNG.

- **A Project to start with**: `crates/drs-project-manager/tests/new_project.rs::a_project_to_start_with`
- **Placed where clicked**: `crates/drs-app/tests/composing.rs::placed_where_clicked` (the Command's centring; the mapping from the click to cells and the tools that turn a click into something else are checked by hand)
- **Natural size**: `crates/drs-app/tests/composing.rs::natural_size`
- **Placed on top**: `crates/drs-app/tests/composing.rs::placed_on_top`, `crates/drs-app/tests/walls.rs::walls_are_placed_on_top`, `crates/drs-app/tests/terrain.rs::terrain_goes_under` (the exception)
- **Placement records a reference**: `crates/drs-app/tests/composing.rs::placement_records_a_reference`, `crates/drs-app/tests/portals.rs::portals_record_a_reference`, `crates/drs-app/tests/terrain.rs::painting_records_a_reference`
- **Placement records the folder**: `crates/drs-app/tests/composing.rs::placement_records_the_folder`, `crates/drs-app/tests/portals.rs::portals_record_the_folder`, `crates/drs-app/tests/terrain.rs::painting_records_the_folder`
- **Anywhere on the Level**: `crates/drs-app/tests/composing.rs::anywhere_on_the_level`, `crates/drs-app/tests/walls.rs::walls_lie_anywhere_on_the_level`, `crates/drs-app/tests/terrain.rs::strokes_lie_anywhere_on_the_level` (a stroke outside the Bounds and at negative cells, with its tiles)
- **Many of the same**: `crates/drs-app/tests/composing.rs::many_of_the_same`
- **Escape stops placing**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Topmost is selected**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **A drag is one step**: `crates/drs-app/tests/composing.rs::a_drag_is_one_step`, `crates/drs-app/tests/walls.rs::a_wall_drag_is_one_step`
- **Removal is reversible in place**: `crates/drs-app/tests/composing.rs::removal_is_reversible_in_place`, `crates/drs-app/tests/walls.rs::wall_removal_is_reversible_in_place`
- **Identity survives undo**: `crates/drs-app/tests/composing.rs::identity_survives_undo`
- **Redo repeats exactly**: `crates/drs-app/tests/composing.rs::redo_repeats_exactly`, `crates/drs-app/tests/portals.rs::portal_commands_redo_exactly`, `crates/drs-app/tests/terrain.rs::paint_redoes_exactly`
- **A new step clears redo**: `crates/drs-app/tests/composing.rs::a_new_step_clears_redo`
- **One history**: `crates/drs-app/tests/composing.rs::one_history`, `crates/drs-app/tests/portals.rs::portal_commands_share_the_history`, `crates/drs-app/tests/terrain.rs::paint_shares_the_history`
- **Undo waits for the step being made**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **View is not a step**: `crates/drs-app/tests/composing.rs::view_is_not_a_step` (a changed Viewport records nothing; the mapping from the pointer and the wheel to the Viewport is checked by hand)
- **A failed load is a placeholder**: `crates/drs-app/tests/projects.rs::missing_assets_stay`, `crates/drs-app/tests/projects.rs::unknown_kinds_are_kept` (the Element staying on its Layer; the drawing is by hand: no headless seam renders the viewport; verified by driving the editor with the dev-only input script), `crates/drs-app/tests/export.rs::a_missing_terrain_image_keeps_its_shape` (a Missing and an undecodable image: the placeholder's colour on the path, the background beyond the radius)
- **A failed Command is reported**: `crates/drs-app/tests/composing.rs::a_failed_command_is_reported` (Set Portal into Wall and Free Portal of an unknown Element, and a Paint of an Asset not in its folder, among the Commands), `crates/drs-app/tests/portals.rs::refused_anchors`, `crates/drs-app/tests/terrain.rs::malformed_strokes_are_refused`, `crates/drs-app/tests/terrain.rs::painted_with_its_material`
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
- **Two points or none**: `crates/drs-app/tests/walls.rs::two_points_or_none`, `crates/drs-app/tests/portals.rs::gone_with_a_two_point_wall`
- **Properties stay editable**: `crates/drs-app/tests/walls.rs::properties_stay_editable`
- **Malformed Walls are refused**: `crates/drs-app/tests/walls.rs::malformed_walls_are_refused`, `crates/drs-app/tests/walls.rs::a_point_at_a_segment_end_is_refused`
- **Drawn as a stroke**: `crates/drs-app/tests/export.rs::a_wall_is_drawn_as_a_stroke` (the colour along a straight Wall, just past its end within half the thickness, and the background beyond half the thickness beside it and at the corner a square cap would fill), `crates/drs-app/tests/export.rs::a_curved_wall_follows_its_curve` (the colour at the curve's middle and the background at the chord's middle), `crates/drs-app/tests/export.rs::a_wall_gives_way_to_its_portal` (the exception along a Portal's stretch); the viewport's drawing is checked by hand
- **Drawing with the Wall tool**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **One thing under the pointer**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Options follow the selection**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script (the refusal of a thickness not above zero is `crates/drs-app/tests/walls.rs::malformed_walls_are_refused`)
- **Hit within the thickness**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Handles of the selected Wall**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script (the Commands a drag and a double-click send are covered by `crates/drs-app/tests/walls.rs::a_handle_drag_is_one_step`, `crates/drs-app/tests/walls.rs::bending_keeps_the_points`, and `crates/drs-app/tests/walls.rs::adding_a_point_keeps_the_shape`)
- **Delete acts on what is picked**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script (the Commands Delete sends are covered by `crates/drs-app/tests/walls.rs::removing_a_point_joins_straight`, `crates/drs-app/tests/walls.rs::bending_keeps_the_points`, and `crates/drs-app/tests/walls.rs::wall_removal_is_reversible_in_place`)
- **A Portal is an image with a width**: `crates/drs-app/tests/portals.rs::a_portal_is_an_image_with_a_width`
- **Natural width**: `crates/drs-app/tests/portals.rs::natural_width`
- **Set Portals stand on the line**: `crates/drs-app/tests/portals.rs::set_portals_stand_on_the_line` (centre, rotation, and mirroring on a straight and a curved segment, on either side), `crates/drs-app/tests/export.rs::a_portal_faces_its_side` (an image of two colours, its top half's colour on the chosen side of a vertical Wall, swapped after a flip)
- **A Portal covers its width**: `crates/drs-app/tests/portals.rs::a_portal_covers_its_width` (a stretch inside a segment, across a point, and stopped at an end)
- **The Wall gives way**: `crates/drs-app/tests/export.rs::a_wall_gives_way_to_its_portal` (a Portal image shorter than the Wall's thickness: its colour at its centre, the background beside it inside the stretch, and the Wall's colour just past the stretch's end), `crates/drs-app/tests/export.rs::a_gap_follows_the_corner` (a Portal across a point: the background on both segments within the stretch), `crates/drs-app/tests/export.rs::no_cap_where_a_portal_reaches_the_end` (the background around the end point a stretch reaches), `crates/drs-app/tests/export.rs::overlapping_portals_stack` (the Wall left out along both stretches)
- **Freestanding like a Prop**: `crates/drs-app/tests/export.rs::a_freestanding_portal_is_turned` (a wide image turned a quarter turn covering a tall area), `crates/drs-app/tests/export.rs::a_mirrored_portal_is_flipped` (the image's halves swapped across its length)
- **Portals stack like Elements**: `crates/drs-app/tests/export.rs::overlapping_portals_stack` (the later Portal's colour where two overlap), `crates/drs-app/tests/export.rs::a_missing_portal_keeps_its_gap` (a Missing Asset's placeholder turned, set into its Wall, and the Wall giving way along it)
- **Placed into a Wall at once**: `crates/drs-app/tests/portals.rs::placed_into_a_wall_at_once`
- **Set into a Wall**: `crates/drs-app/tests/portals.rs::set_into_a_wall`
- **Freed where it stands**: `crates/drs-app/tests/portals.rs::freed_where_it_stands`
- **Refused anchors**: `crates/drs-app/tests/portals.rs::refused_anchors`
- **Sliding along the Wall**: `crates/drs-app/tests/portals.rs::sliding_along_the_wall`
- **Portals stay editable**: `crates/drs-app/tests/portals.rs::portals_stay_editable`
- **Removed Portals return set**: `crates/drs-app/tests/portals.rs::removed_portals_return_set`
- **Moves with its Wall**: `crates/drs-app/tests/portals.rs::moves_with_its_wall` (a point drag on the Portal's segment and two segments away, a bend and its straightening, a whole-Wall move, a thickness change)
- **Adding a point keeps Portals in place**: `crates/drs-app/tests/portals.rs::adding_a_point_keeps_portals_in_place` (on the Portal's segment either side of the new point, and upstream; the Portal's centre unchanged on a straight and a curved segment)
- **Removing a point carries the Portals beside it**: `crates/drs-app/tests/portals.rs::removing_a_point_carries_the_portals_beside_it`
- **Gone with its part of the Wall**: `crates/drs-app/tests/portals.rs::gone_with_a_covered_point`, `crates/drs-app/tests/portals.rs::gone_with_an_end_segment`, `crates/drs-app/tests/portals.rs::gone_with_the_wall`, `crates/drs-app/tests/portals.rs::gone_with_a_two_point_wall` (each with the answer naming the removed Portals and an undo restoring them; the status line is checked by hand)
- **Freestanding Portals stay put**: `crates/drs-app/tests/portals.rs::freestanding_portals_stay_put`
- **No direction keeps the rotation**: `crates/drs-app/tests/portals.rs::no_direction_keeps_the_rotation`
- **Placing with the Portal tool**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script (the Placement it sends is covered by `crates/drs-app/tests/portals.rs::placed_into_a_wall_at_once`)
- **Picking Portals**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Dragging Portals**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script (the Commands a slide sends are covered by `crates/drs-app/tests/portals.rs::sliding_along_the_wall`)
- **Portal options and keys**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script (the Commands the strip and the keys send are covered by `crates/drs-app/tests/portals.rs::portals_stay_editable`, `crates/drs-app/tests/portals.rs::set_into_a_wall`, and `crates/drs-app/tests/portals.rs::freed_where_it_stands`)
- **Terrain is its strokes**: `crates/drs-app/tests/terrain.rs::terrain_is_its_strokes`
- **Shaped by a soft round Brush**: `crates/drs-app/tests/terrain.rs::shaped_by_a_soft_round_brush` (full strength within the hardness, nothing beyond the radius, the falloff's value halfway between, and a one-point dab)
- **No build-up along a stroke**: `crates/drs-app/tests/terrain.rs::no_build_up_along_a_stroke` (the coverage at a sharp joint and at a self-crossing equals a single segment's there)
- **Strokes composite by the strongest**: `crates/drs-app/tests/terrain.rs::strokes_composite_by_the_strongest` (two overlapping half-strength strokes stay at half, a weaker stroke over a stronger leaves the stronger)
- **Tiled at natural size**: `crates/drs-app/tests/export.rs::terrain_is_tiled_at_natural_size` (a texture two cells a side whose quarters differ: each quarter's colour in the cells the repetitions from the origin put it in, upright, continuous across two strokes)
- **Drawn masked by coverage**: `crates/drs-app/tests/export.rs::a_stroke_shows_its_material` (the texture's colour on the path of a full-strength stroke, the background beyond its radius, a colour strictly between the two in its soft edge, and partly where the texture is half transparent), `crates/drs-app/tests/export.rs::a_weaker_stroke_shows_partly` (a half-strength stroke's middle strictly between the texture and the background); the viewport's drawing is checked by hand
- **The box follows the strokes**: `crates/drs-app/tests/terrain.rs::the_box_follows_the_strokes`
- **Terrain goes under**: `crates/drs-app/tests/terrain.rs::terrain_goes_under` (a Prop and a Wall placed first stay above the Terrain; a Prop placed after lands on top), `crates/drs-app/tests/export.rs::terrain_lies_under_props_and_walls`
- **One Terrain per Layer**: `crates/drs-app/tests/terrain.rs::one_terrain_per_layer` (and a file holding two Terrains on a Layer: the topmost takes the stroke)
- **A stroke is one step**: `crates/drs-app/tests/terrain.rs::a_stroke_is_one_step`
- **The first stroke makes the Terrain**: `crates/drs-app/tests/terrain.rs::the_first_stroke_makes_the_terrain`
- **Painted with its Material**: `crates/drs-app/tests/terrain.rs::painted_with_its_material`
- **Undo leaves no trace**: `crates/drs-app/tests/terrain.rs::undo_leaves_no_trace` (every tile of the coverage byte for byte as before the stroke, after undoing a stroke that overlaps two earlier ones)
- **Coverage is the strokes alone**: `crates/drs-app/tests/terrain.rs::coverage_is_the_strokes_alone` (strokes laid one by one, the same strokes after undoing back to the first and redoing, and the same strokes saved and opened, give identical tiles)
- **Malformed strokes are refused**: `crates/drs-app/tests/terrain.rs::malformed_strokes_are_refused`
- **The Material stays editable**: `crates/drs-app/tests/terrain.rs::the_material_stays_editable` (and naming the image already shown records no step), `crates/drs-app/tests/export.rs::the_material_stays_editable` (the new image in the Export)
- **Terrain changes only its Material**: `crates/drs-app/tests/terrain.rs::terrain_changes_only_its_material`
- **Painting with the Paint tool**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script (the Paint the release sends is covered by `crates/drs-app/tests/terrain.rs::a_stroke_is_one_step` and `crates/drs-app/tests/terrain.rs::painted_with_its_material`)
- **Seeing the stroke**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Brush options**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script (the Edit Element its button sends is covered by `crates/drs-app/tests/terrain.rs::the_material_stays_editable`)
- **Terrain is not picked**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script

ShapeEngine's own unit tests check what the Wall and Portal Rules rest on, as functions of the Engine alone, and are the coverage of no Rule: the flattening tolerance, the mesh covering the thickness, the exactness of a split, the centre and direction on a quadratic, the stretch measured across a point and stopped at an end, the remap through an added point leaving the centre where it was, and the stroke ending squarely at a stretch (`crates/drs-shape-engine/src/wall.rs::tests::a_straight_segment_flattens_to_its_ends`, `crates/drs-shape-engine/src/wall.rs::tests::the_chord_stays_within_tolerance`, `crates/drs-shape-engine/src/wall.rs::tests::the_mesh_covers_the_thickness`, `crates/drs-shape-engine/src/wall.rs::tests::a_split_keeps_the_shape`, `crates/drs-shape-engine/src/portal.rs::tests::the_centre_lies_on_the_curve`, `crates/drs-shape-engine/src/portal.rs::tests::a_stretch_crosses_points_and_stops_at_ends`, `crates/drs-shape-engine/src/portal.rs::tests::an_added_point_keeps_the_centre`, `crates/drs-shape-engine/src/wall.rs::tests::a_stretch_ends_the_stroke_squarely`).

PaintEngine's own unit tests check what the Terrain Rules rest on, as functions of the Engine alone, and are the coverage of no Rule: the stamp against its formula, the maximum over segments at a joint, a pixel the same in every region that holds it, ApplyStroke's appended result equal to a full Rasterize, and an undo rasterizing only the touched tiles (`crates/drs-paint-engine/src/rasterize.rs::tests::the_stamp_matches_its_formula`, `crates/drs-paint-engine/src/rasterize.rs::tests::a_joint_takes_the_maximum`, `crates/drs-paint-engine/src/rasterize.rs::tests::a_pixel_is_the_same_in_any_region`, `crates/drs-paint-engine/src/cache.rs::tests::appending_equals_rasterizing`, `crates/drs-paint-engine/src/cache.rs::tests::an_undo_touches_only_its_tiles`). RenderEngine's unit test `crates/drs-render-engine/src/terrain.rs::tests::the_shader_is_valid_wgsl` parses and validates the masked tiled image Shader without a GPU, and is the coverage of no Rule either.

## Not supported

- Panning, zooming, and selecting are never Commands and never history steps.
- An Element is never addressed by its entity handle across a Command or a history step; only its ElementId is stable.
- Removing a point never approximates the joined segments' curves: the joined segment is always straight.
- A Portal is never anchored by its distance along a Wall: only by the Wall's ElementId, a segment, a parameter, and a side.
- A Terrain never keeps pixels: its strokes are the truth, and no coverage is saved or kept in the history.
- The Brush's settings are never a history step and never saved; only the strokes laid with them are.

## Notes

- `A Project to start with` lives here because it is the Project the Author composes on; saving and reopening a Project are the projects capability's.
- Seventeen Rules (Escape stops placing, Topmost is selected, Undo waits for the step being made, Drawing with the Wall tool, One thing under the pointer, Options follow the selection, Hit within the thickness, Handles of the selected Wall, Delete acts on what is picked, Placing with the Portal tool, Picking Portals, Dragging Portals, Portal options and keys, Painting with the Paint tool, Seeing the stroke, Brush options, Terrain is not picked) have no automated test, against the requirement that every Rule has one; the viewport's drawing of A failed load is a placeholder, Drawn as a stroke, the Portal drawing Rules, Drawn masked by coverage, Tiled at natural size, and Terrain goes under, and the status line of Gone with its part of the Wall, are checked only by hand. They are behaviour of the egui interface and of the viewport's rendering, for which no headless seam exists. The accepted deviation is verification by hand, driving the editor with the development-only input script, whose `describe` step logs the Wall tool, every Wall and the stretches it gives way along, every Portal, the Paint tool with its Brush, and every Terrain, and which also confirms that a placed Prop, a drawn Wall, a Portal, and a Terrain are shown where and as large as the model says, a Portal turned, mirrored, and in its gap, a Terrain under the Props and Walls on its Layer, and a Terrain whose image is Missing keeping its shape in the placeholder's colour.
- Add Asset Folder changes device state, not Project state, yet it sits in the same history as composing steps because every Manager records into the one history. Undo therefore always reaches a Prop before the folder it came from.
- Adding and removing a point are the only changes that renumber a Wall's segments, and so the only Wall edits that record anything for its Portals.
- Overlapping Portals are allowed, on any Wall, and a Portal may straddle a point of a Wall, its gap following the stroke around it; only removing a point it covers removes it.
- A point removed beside Portals that do not cover it carries them onto the joined straight segment by share of length, so a straight Wall that gained and then lost such a point puts those Portals back exactly where they were.
- A Portal wider than its whole Wall leaves the Wall undrawn and unpickable by its line until the Portal is narrowed, slid, freed, or removed; the Wall stays an Element and its handles show when it is selected through undo or after the Portal changes.
- The band a stroke being drawn is shown as is translucent, and where its round ends overlap its line the translucency doubles; accepted as a detail of a preview that lasts until the release.
- Measured on the CPU in a development build: the first stroke on a Layer takes about 11 ms, appending a stroke 0.66 ms, undoing one 6.4 ms, and opening a Terrain of 200 strokes 103 ms; an undo on dense Terrain and opening a Project with much Terrain rasterize every stroke over the touched tiles and take longer.
- "Stroke" is used in its plain sense, one pass of a Brush from press to release, as "point" and "segment" are for Walls.
- Snapping Wall points or Portals to the Grid, splitting a Wall at a Portal or joining Walls, closed Walls, cubic curves and arcs, textured or translucent Walls, Walls or Portals blocking light or sight, open and closed Portals, dragging a set Portal off its Wall, a preview of a Portal's image before it is placed, rotating Props, drawing a Wall by dragging, selecting several Elements or points at once, erasing, moving, reshaping, or deleting a stroke once laid, more than one Material on a Terrain, Brush Presets, textured or shaped Brush tips, pressure, rotating, scaling, or offsetting a Terrain's texture, selecting, moving, or removing a Terrain from the viewport, Patches, Water, and Caves, and a painted result while a stroke is dragged do not exist.
