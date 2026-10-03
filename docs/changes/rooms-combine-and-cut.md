# Rooms combine and cut

**Capabilities**:
- composing: Place Element, Edit Element, Remove Element, Set Portal into Wall, Free Portal
- projects: none of its Commands; saving and opening Rooms that cut
- export: none of its Commands; exporting combined and cut Rooms

## Problem Statement

Rooms stand on their own: two Rooms that overlap are both drawn whole, so a Wall runs across the middle of what the Author meant as one L-shaped hall, and the floor of the later Room hides half of the earlier Room's Wall where they meet. There is no way to take floor away, so a pillar, a pit, a courtyard, or a recess in a wall cannot be drawn at all. The functional baseline merges overlapping rooms, but it does so by redrawing their shape once, so the outline it leaves can no longer be taken apart, and the doors set into the walls that moved are dropped or corrupted.

## Solution

Rooms on the same Layer combine. Wherever Rooms overlap, their Walls run only round the outside of the floor they cover together; no Wall crosses open floor, and each Room keeps its own floor colour, the later one drawn over the earlier. Rooms that meet exactly edge to edge are two rooms with one Wall between them, and a door set into that Wall stays as long as the Wall is there. A Room can cut: the Author switches Cut on in the Room tool's options before drawing a Room, or for a selected Room, and the Room then takes floor away from the Rooms before it on its Layer, with Walls round the hole; a Room drawn after a cut fills it back in. Every Room keeps the outline the Author drew: its points stay handles, its hidden edges stay editable, and the combination is worked out again after every step and at every frame of a drag, so nothing is ever baked. Portals stay set into the edge of the Room they were set into and never move when another Room changes; a Portal whose Wall another Room or a cut takes away is removed in the same undo step, and the Author is told. The Export and the saved Project show and keep exactly this, the Project holding only each Room's own outline and whether it cuts.

## User Stories

### Overlapping Rooms

1. As an Author, I can draw a Room that overlaps another on the same Layer and see one combined room, its Walls running round the outside of both and no Wall across the floor where they overlap, so that an L-shaped hall is two rectangles.
2. As an Author, I can see each combined Room keep its own floor colour, the later Room's floor drawn over the earlier one's where they overlap, so that combining never repaints my floors.
3. As an Author, I can draw a Room entirely inside a larger one and see it as a patch of floor in its own colour with no Walls of its own, so that a dais or a tiled area inside a hall needs no Wall.
4. As an Author, I can chain several Rooms, each overlapping the next, and see one outline round them all, so that a winding hall is as many Rooms as it has bends.
5. As an Author, I can combine Rooms with curved edges and see each Wall follow its curve exactly up to where another Room begins, so that a round tower joined to a square keep is drawn round.
6. As an Author, I can combine Rooms of different wall thickness and colour and see each stretch of Wall keep the thickness and colour of the Room whose edge it runs along, so that combining never repaints my Walls.
7. As an Author, I can see a rounded corner wherever one Room's Wall runs into another's, so that a join looks built, not cut.
8. As an Author, I can see every Wall of combined Rooms drawn over every floor of the combination, so that a later floor never hides half of an earlier Room's Wall.
9. As an Author, I can see an Element I placed between two Rooms of a combination drawn over the earlier floor and under the combination's Walls, and one placed after the last of them drawn over everything, so that the stacking order still decides what I see.
10. As an Author, I can draw Rooms that lie apart on the Layer and see each walled on its own, so that combining happens only where Rooms meet.
11. As an Author, I can put a Room on another Layer over one on this Layer and see both walled whole, so that a separate building on its own Layer keeps its own Walls.

### Rooms that meet edge to edge

12. As an Author, I can place a Room whose edge lies exactly on another Room's edge, from the outside, and see one Wall between them, drawn once, so that two rooms side by side share their wall.
13. As an Author, I can see that shared Wall drawn in the thickness and colour of the later of the two Rooms, so that the Wall between them looks like one Wall.
14. As an Author, I can share only part of an edge and see the Wall between the Rooms only along the shared part, the rest of each edge still walled on the outside, so that a small room off a long hall shares just its own width.
15. As an Author, I can place two overlapping Rooms whose edges run together along the outside, such as two rectangles with the same top line, and see one Wall there, so that an aligned outline is walled once.
16. As an Author, I can rely on Rooms whose edges lie a hair apart being walled each on their own, and Rooms that overlap by a hair being combined with no Wall between them, so that only edges lying exactly on one another share a Wall.
17. As an Author, I can place a Room inside another so that one of its edges lies on the other's edge, beside a third Room sharing that edge from the outside, and see the Wall between the outer Rooms kept, so that nesting a patch of floor against a shared Wall changes nothing about the Wall.

### Cutting

18. As an Author, I can switch Cut on in the Room tool's options before drawing, so that the next Room I draw takes floor away instead of adding it.
19. As an Author, I can draw a cut inside a Room and see the floor taken away under it and Walls run round the hole, so that a pillar, a pit, or a courtyard is one Room drawn over another.
20. As an Author, I can draw a cut across a Room's outline and see a notch, the Room's Walls stopping where the cut begins and following the cut's edge round the notch, so that a recess in a wall is one gesture.
21. As an Author, I can see what lies under a hole, the Elements placed before the Rooms and the background, show through it, so that a hole is a hole.
22. As an Author, I can see the Walls round a cut's hole drawn in the cut's own wall thickness and colour, so that a pit's rim can differ from the room's Walls.
23. As an Author, I can draw a Room after a cut, over its hole, and see it fill the hole back in, so that what I draw last is what I see.
24. As an Author, I can rely on a cut taking floor away only from Rooms before it on its own Layer, so that a cut never reaches into another Layer or undoes what I drew after it.
25. As an Author, I can cut across the Wall between two Rooms that share an edge and see that Wall gone where the cut lies, so that a cut opens through any Wall.
26. As an Author, I can see every cut's outline as a thin guide line in the editor, also where it lies over no Room and takes nothing away, and never in the Export, so that I can find a cut I cannot otherwise see.
27. As an Author, I can select a Room and switch Cut on or off for it in the options, as one undo step, keeping its outline, thickness, and colours, so that whether a Room cuts is never fixed.

### Editing combined Rooms

28. As an Author, I can drag a point or a control point of one Room into or out of another and see the combined outline and its Walls follow at every frame of the drag, so that I see the shape I will get before I let go.
29. As an Author, I can move a whole Room into, out of, or across others and see the combination follow it, as one undo step, so that rearranging a dungeon is dragging its Rooms.
30. As an Author, I can add or remove a point, bend or straighten an edge, or change a thickness or colour of a combined Room and see the combination follow, so that every edit I can make to one Room works in a combination.
31. As an Author, I can select a Room and see its whole outline, the edges hidden inside other Rooms' floors included, with every handle on it, so that I can still reshape the parts of a Room that carry no Wall.
32. As an Author, I can double-click any edge of a selected Room, a hidden one included, to add a point there, so that a hidden edge is as editable as a walled one.
33. As an Author, I can click a Room's floor where it shows, or a Wall that runs along its edges, to select it, and click inside a cut to select the cut, so that every Room is picked where it is seen.
34. As an Author, I can remove a Room from a combination and see the other Rooms walled and floored as they would be without it, so that taking a Room away never leaves a hole in a Wall.
35. As an Author, I can undo and redo every step above and see the combination exactly as it was before and after the step, so that combining never makes undo approximate.

### Portals in combined Rooms

36. As an Author, I can set a Portal into any Wall of combined Rooms, the Wall round a cut's hole and the Wall between two Rooms that share an edge included, so that every Wall I see takes a door.
37. As an Author, I can rely on the Portal tool's marker snapping only to Walls that are drawn, never to an edge hidden inside a floor or one a cut took away, so that I never place a door in open floor.
38. As an Author, I can set a door into the Wall between two Rooms that share an edge and see it stay when I edit either Room, as long as the Wall between them is there, so that the door between two rooms survives reshaping them.
39. As an Author, I can edit, place, or remove any other Room of the Layer and see a Portal never move, turn, or flip, so that a door only ever moves with its own Room.
40. As an Author, I can draw or move a Room over a door, or draw a cut over it, and see the door removed with its Wall in the same undo step and be told in the status line how many Portals were removed, so that a door is never left standing in open floor or over a hole, and never silently.
41. As an Author, I can undo that step and get every removed Portal back exactly, set where it was, so that undo restores the doors with the Rooms.
42. As an Author, I can drag a Room over a door and back out again in one drag and keep the door, so that passing over a door on the way costs nothing.
43. As an Author, I can see a door whose centre still lies on a Wall kept when another Room or a cut takes away the Wall beside it, its gap following the Wall round the new corner, and a door whose centre lies where the Wall is gone removed, so that what happens to a door straddling a cut is predictable: its centre decides.
44. As an Author, I can place a door near where two Rooms' Walls meet and see its gap run round onto the other Room's Wall, so that a door at a corner of a combination is drawn as it would be built.
45. As an Author, I can slide a door along its Room's Walls only where they run, so that sliding never pulls a door into open floor.
46. As an Author, I am refused, with the reason, a Portal set into or slid onto a place on a Room's edge where no Wall runs, so that a door is never set into nothing.

### Export and saving

47. As an Author, I can export a Level with combined and cut Rooms and see in the image what the editor shows: one outline, each floor in its colour, holes showing what lies below, and shared Walls with their doors, so that what I see is what I export.
48. As an Author, I can save a Project with combined and cut Rooms and reopen it with every Room's own outline and whether it cuts as they were, and the floors, Walls, and doors exactly as before, so that nothing about combining is lost or baked in.
49. As an Author, I can open a Project saved before Rooms could cut and see every Room in it as one that does not cut, so that older Projects open as they were drawn.
50. As an Author opening a Project from an editor that left doors inside another Room's floor, as an editor that does not combine Rooms does, I can see those doors standing where they were saved, kept, saved back unchanged, free to be freed, and set into their Wall again when an edit brings it back, so that opening a Project never drops a door.
51. As an Author sharing a Project with a collaborator whose editor does not know Rooms that cut, I can rely on their editor refusing the Project as one saved by a newer editor, rather than opening it with my holes filled in, so that a round trip never silently loses a cut.

### Speed

52. As an Author, I can drag a point of a Room on a Layer of twenty combined Rooms with curved edges, shared Walls, and doors, and see the combination follow every frame, so that combining never makes the editor stutter.

## Rules

### Combining

**Rooms of a Layer combine**: taken in stacking order, every Room on a Layer that does not cut adds the place its outline winds around to the Layer's combined floor, and every Room that cuts takes the place its outline winds around away from what the Rooms before it added; a Room after a cut adds to the combined floor as any other.

**Walls run where the floor ends**: Walls run along every stretch of a Room's edge that has the Layer's combined floor on one side and not on the other, and along shared edges (Shared edges keep their Wall); every other stretch of a Room's edge, inside the combined floor or outside it, carries no Wall.

**Shared edges keep their Wall**: where an edge of one Room lies exactly on an edge of another Room of the same Layer, neither of them cutting, with the places their outlines wind around on opposite sides, the stretch they share carries a Wall wherever the combined floor lies on both sides of it, whatever other Rooms lie over it.
_Why_: Rooms that overlap open into each other, while Rooms that only meet edge to edge are two rooms with a wall between them, which is where a door belongs.

**Walled once**: a stretch where edges of two or more Rooms lie exactly on one another and a Wall runs carries one Wall, drawn in the thickness and colour of the last of those Rooms in the stacking order.

**Each Wall in its Room's look**: every stretch of Wall not walled once is drawn in the thickness and colour of the Room whose edge it runs along, a Room that cuts included.

**The Walls close round the combination**: the Walls along the edge of the combined floor are drawn centred on it, with a round join at every point of it, each point where one Room's edge meets another's included, and the Wall of a shared edge is drawn centred on it with round ends where it meets the others; along the stretches Portals cover, Walls end squarely across the line instead.

**Each Room keeps its floor**: every Room that does not cut is drawn with its own floor, in its own floor colour, over the place its outline winds around except what a Room that cuts after it on its Layer takes away; where two floors overlap, the later one is drawn over the earlier; a Room that cuts has no floor. Follows from: Every Element that shows a surface is drawn with a Material.

**Walls over the combination**: Rooms of one Layer whose outlines overlap or touch, directly or through other Rooms of that Layer, cuts included, form one combination, whose Walls are all drawn over every floor of the combination, at the place in the stacking order of its last Room; an Element between two Rooms of the combination is drawn over the earlier Room's floor and under the Walls, and an Element after its last Room over them.
_Why_: drawn at each Room's own place, a later floor would cover the inner half of an earlier Room's Wall where the two meet.

**Layers keep their Rooms apart**: a Room combines only with the Rooms of its own Layer, and a Room that cuts takes floor away only from them; a Room on another Layer is floored and walled as if no Room of this Layer were there.
_Why_: a Layer is hidden, faded, and blended as a whole, so a Wall shared between two Layers could follow neither.

**Cuts are shown in the editor**: the viewport draws every Room that cuts with a thin guide line along its whole outline, whether or not it takes floor away; the Export never shows it.

### Through every edit

**Combined after every step**: after every Place Element, Edit Element, or Remove Element of a Room, every undo and redo of one, and every opening of a Project, every Room of the Layer has the floor and Walls the Rules above give for the Layer's Rooms as they now are, and every Room of every other Layer is unchanged. Follows from: Nothing is fixed at creation.

**Followed through a drag**: while a point, a control point, or a whole Room is dragged, every Room of its Layer has, after each change of the drag, the floor and Walls the Rules above give for that moment, and the drag stays one history step.

**Whether a Room cuts stays editable**: a Place Element of a Room says whether it cuts, and an Edit Element changes it, as a step of its own that undo returns; the Room's points, edges, wall thickness, and colours are kept either way, its floor colour included. Follows from: Nothing is fixed at creation.

**Nothing combined is saved**: a saved Room holds only its own points, edges, wall thickness, wall colour, floor colour, and whether it cuts; the combined floor and its Walls are derived again when the Project opens, and come out as they were. Follows from: Nothing is fixed at creation.

**Older Rooms do not cut**: a Room saved by an editor that did not yet know Rooms that cut opens as a Room that does not cut, and is saved back with whether it cuts.

### Portals in combined Rooms

**Portals go where a Wall runs**: a Portal is set into a Room only at a place on one of the Room's edges where a Wall runs; a Place Element of a Portal, a Set Portal into Wall, or an Edit Element sliding a Portal whose anchor names a place on a Room's edge where no Wall runs is refused with the reason, changes nothing, and records no history step. Follows from: A Portal set into a Wall moves with it.

**Other Rooms never move a Portal**: placing, editing, or removing any Room other than the one a Portal is set into changes none of the Portal's anchor, centre, rotation, or mirroring; it can only remove the Portal (Gone with its Wall). Follows from: A Portal set into a Wall moves with it.

**The centre decides**: a Portal set into a Room stays set as long as a Wall runs at its centre, however much of its width now lies where the Wall is gone. Follows from: A Portal set into a Wall moves with it.

**Gone with its Wall**: a Place Element, Edit Element, or Remove Element of a Room that leaves no Wall at the centre of a Portal set into a Room of its Layer, where one ran before it, removes that Portal, whichever Room it is set into, in the same history step, which undo restores whole with the Portal set where it was; the Author is told in the status line how many Portals were removed. Follows from: A Portal set into a Wall moves with it.

**A drag decides at its end**: while a drag is under way, a Portal left with no Wall at its centre stands where it stood and makes no Wall give way, and is set into its Wall again as soon as the drag brings the Wall back; when the drag ends with no Wall at its centre, the Portal is removed in the drag's history step.
_Why_: passing over a door on the way to somewhere else should not cost the door.

**A stretch follows the Walls**: a Portal set into a Room covers the stretch that reaches half its width either way from its centre, measured along the Wall it stands in: round the edge of the combined floor, across every point, those where one Room's edge meets another's included, and onto the other Room's Wall; or along the Wall of a shared edge, stopping at its ends. Every Wall the stretch covers gives way, whichever Room's edge it runs along.

**A Portal without its Wall stands**: a Portal set into an edge of a Room on its Level where no Wall runs at its centre, which outside a drag no Command of this editor leaves but a Project from an editor that does not combine Rooms may hold, is drawn at its saved position, rotation, and mirroring, makes no Wall give way, is saved back unchanged, can be freed, and is not removed for lacking a Wall; an edit that brings a Wall back at its centre sets it into that Wall again. Follows from: References are never dropped.

### Speed

**Recombined within a frame**: in the test build, deriving the floors, the Walls, and the Portals' places of a Layer of twenty Rooms, each with two curved edges, with sixteen shared edges and ten Portals, takes under 4 milliseconds.
_Why_: a drag derives its Layer again at every frame, and this leaves three quarters of a 60 Hz frame for the rest.

### The Room tool

**Cut in the Room options**: the Room tool's options strip holds a Cut switch beside the wall thickness and the colours; with a Room selected it shows whether that Room cuts, and a change is sent as one Edit Element; with none selected it sets whether the next Room cuts, off to start.

**Picking combined Rooms**: a Room that does not cut is under the pointer inside its floor as it is drawn, a Room that cuts anywhere inside its outline, and either wherever a Wall drawn in its look is under the pointer, outside the stretches Portals cover; a combination's Walls are hit at the place of its last Room, so ahead of every floor of the combination.

**The whole outline is handled**: the selected Room is outlined along every edge, including those that carry no Wall, and shows its handles on all of them; a double-click on any of its edges adds a point at the nearest place on it.

## Changes to existing behaviour

The Rules named here are those the composing and projects specs hold now that Walls, Portals, and Rooms have landed, which this change builds on.

- composing — **A Room is its outline**: modified to add "and whether it cuts" to what a Room is, because a Room may take floor away.
- composing — **A Room is placed as one step**: modified to "a Place Element of a Room puts on the given Layer a Room with the given points, wall thickness, wall colour, floor colour, and whether it cuts, every edge straight, together with the removal of every Portal it leaves without a Wall (Gone with its Wall), as one history step that undo takes away whole and redo brings back whole", because placing a Room can take another Room's Wall away.
- composing — **Room properties stay editable**: modified to "a Room's wall thickness, wall colour, floor colour, and whether it cuts are each changed through Edit Element, every change a step of its own", because whether a Room cuts is a property like any other.
- composing — **The floor fills the outline**: modified to "a Room's floor covers every place its outline winds around, including every part of an outline whose edges cross, up to the outline's line, except what a Room that cuts after it on its Layer takes away; a Room that cuts has no floor", because cuts take floor away (Each Room keeps its floor).
- composing — **The Walls close around the floor**: replaced by Walls run where the floor ends, Shared edges keep their Wall, Walled once, Each Wall in its Room's look, and The Walls close round the combination, because a Room's Walls now run only where the combined floor ends.
- composing — **The floor lies under its Walls**: modified to "a Room's floor is drawn at its place in the stacking order and its Walls with its combination's (Walls over the combination); every Element before the Room is drawn under its floor, and every Element after the last Room of its combination over its floor and Walls", because a combination's Walls lie over all of its floors.
- export — **Drawn as in the editor**: modified so that a Room appears "as its floor and its Walls are drawn with its combination (Walls over the combination), the Walls left out along the stretches Portals cover", because a combination's Walls lie over all of its floors.
- projects — **Saved as its outline**: modified to "a saved Room holds its points, which edges are curved and their control points, its wall thickness, its wall colour, its floor colour, and whether it cuts, and reopens the same; an editor that does not know the Room kind keeps it as a placeholder of its size and writes it back unchanged", because whether a Room cuts is saved.
- composing — **Portals set into Rooms**: modified so that a Portal "can be set into a Room's Walls at a place on its edges where a Wall runs (Portals go where a Wall runs)", because a Room's edge carries a Wall only where the combined floor ends.
- composing — **A stretch runs round the Room**: replaced by A stretch follows the Walls, because a Portal's Wall may now continue onto another Room's edge, and a shared edge's Wall has ends.
- composing — **Gone with its part of the Room**: modified to add "and every Portal whose Wall a Command takes away (Gone with its Wall)", because another Room's edit can take a Wall away.
- composing — **Refused anchors**: modified to add "a place on a Room's edge where no Wall runs" to the anchors refused, because a Portal is set only into a Wall that is drawn.
- projects — **A lost Wall leaves the Portal standing**: modified to add "or a place on a Room's edge where no Wall runs at its centre (A Portal without its Wall stands)" to the anchors that leave a Portal standing, because an editor that does not combine Rooms may leave a Portal inside another Room's floor.
- composing — **Picking Rooms**: replaced by Picking combined Rooms, because a Room's floor may be cut away and its Walls drawn at another Room's place.
- composing — **Handles of the selected Room**: modified so that a double-click "on any of its edges" adds a point, and the Room is outlined along every edge (The whole outline is handled), because an edge without a Wall stays editable.
- composing — **Room options follow the selection**: modified to add the Cut switch (Cut in the Room options), because whether a Room cuts is set in the tool.
- composing — **Picking Portals**: modified to "a Room's Walls are not under the pointer where the nearest point of the line lies in a stretch any Portal covers, whichever Room it is set into", because a Portal's stretch may cover another Room's Wall.
- composing — **Placing with the Portal tool**: modified so that the marker snaps only to Walls that are drawn, a Room's Wall anchoring the Portal to the edge of the Room whose look it is drawn in (on a stretch walled once for several Rooms, the last of them), because a Room's hidden edges take no door.
- composing — **Dragging Portals**: modified so that a Portal set into a Room slides "to the nearest point of the Walls that run along its own Room's edges", because a slide never leaves its Wall.
- composing — **Portal options and keys**: modified so that `F` sets a freestanding Portal "into the nearest Wall within reach of its centre that is drawn, a Wall's line or a Room's Wall, at the nearest point on it", for the same reason.

## Implementation Decisions

The technology the architecture fixes (Rooms keeping their editable source outlines as the truth and deriving the combined outline, cached; i_overlay's `EdgeOverlay` behind ShapeEngine's own adapter, every output edge a piece of one Room's edge, curved edges flattened for the boolean and rebuilt as exact sub-segments of their source curves with the parameter recovered by nearest point; party walls kept from the cancelled coincident records and clipped to the derived outline; Portal anchoring by the Room's ElementId, edge, parameter, and side over that provenance, and the removal of a Portal whose part of the Wall vanishes by the outline-changing Command in the same undo group) is used as written there, and the Walls, Portals, and Rooms changes' decisions (the derived shape in `model`, the once-per-frame deriving, the reversible point commands, the answer naming removed Portals) are extended, not restated.

- **The Room component**: gains whether the Room cuts, and goes to version two under `room`; reading version one gives a Room that does not cut, so an older Project opens as drawn and is written back at version two. Place Element's Room payload gains whether it cuts, and the Element change gains it, set through the generic field-setting command by reflect path like the colours. An editor that knows only version one refuses a version-two file as newer, the projects capability's existing behaviour.
- **The derived shape**: a Room's derived shape, never saved, now holds its floor's fill mesh after the cuts that follow it, the walled pieces of its edges (each a range of parameters on one edge, flattened into a line tagged with the edge and parameter, for hit-testing, the Portal tool, and slides), the stroke mesh of the Walls drawn in its look, the ElementId of the Room at whose place they are drawn (its combination's last), and the stretches Portals cover on its pieces, whichever Room those Portals are set into. Its whole source outline stays derived as a closed line too, for the selection outline, the handles, and double-clicks on hidden edges. Pieces are kept per Room rather than per combination, so that RenderEngine colours each Room's Walls with that Room's Material and the Editor knows which Room a Wall picks.
- **Deriving**: the once-per-frame system that derives Rooms after every Manager has handled its Commands, Undo, and Redo derives a whole Layer at once: when a Room on it is placed, edited, or removed, a Room joins or leaves it, its Elements' order changes, or a Portal set into one of its Rooms changes, every Room of that Layer is derived again through ShapeEngine, and each derived shape is written only where it differs, as Derived model component prescribes. A Portal set into a Room with no Wall at its centre is left where it stands and adds no stretch, which serves the drag in progress and the Portal from another editor alike.
- **ShapeEngine**: CombineOutlines takes a Layer's Rooms in stacking order (each outline and whether it cuts) and returns, for each Room, its floor, the walled pieces of its edges, and its combination with that combination's last Room. It flattens every edge with each vertex on the curve at the Walls' tolerance, and folds the Rooms in stacking order through `EdgeOverlay`, one pass per run of Rooms between cuts, each pass a union of the run onto what came before and each cut a difference, so that provenance carries through every pass; it keeps the cancelled coincident records of each pass as shared edges, clipped to the final combined floor, and the pieces where several Rooms' edges coincide along the outline with every source they came from, giving each such stretch to the last of its Rooms. Each Room's pieces are collapsed into runs of one edge and rebuilt as exact sub-segments of the source curve, their ends moved onto the cut vertex they meet. A Room's floor is its own outline less the cuts after it that overlap it, filled by lyon_tessellation under the non-zero rule as before. Combinations are the Rooms whose outlines overlap or touch, joined transitively, found from their boxes first. GenerateWalls strokes each Room's runs with round joins inside a run, round ends where a run meets another Room's or a shared edge meets the outline, and square ends at the stretches. AnchorPortals locates each Portal by its Room, edge, and parameter on the walled pieces and the shared edges, places its centre and direction on the exact source curve, measures its stretch along the Wall it stands in (round a contour across other Rooms' pieces, or along a shared edge to its ends), and reports a Portal with no Wall at its centre as such. All stay plain functions over model types.
- **Commands that take Walls away**: before recording a step that places, edits, or removes a Room, or changes whether it cuts, AuthoringManager works out the Layer's combination before and after the step through ShapeEngine and records, in the same history group, the removal through the existing Remove Element command of every Portal set into a Room of that Layer with a Wall at its centre before and none after. For a gesture, it keeps the Layer's Portals with a Wall when the gesture began, and on the gesture's end works out the combination as the gesture leaves it and appends those removals to the gesture's group before closing it. Placing, setting, and sliding a Portal are checked against the combination as it stands when the Command is handled, worked out the same way rather than read from a derived shape that may not yet include this frame's steps. The answer naming removed Portals lists each with the Room or Wall it was set into, and the Editor writes their total to the status line.
- **Rendering**: RenderEngine draws each Room's floor at the Room's own stacking depth and the Walls mesh of each Room at the depth of the Room its derived shape names, a fraction of a unit above that Room's floor, the meshes of one combination ordered among themselves by their Rooms' stacking order, each with the flat-colour Material of its Room's wall colour, in the viewport and in RenderRegion alike. A Room that cuts has no floor mesh. Capturing a region waits for nothing new.
- **The tool**: the Room tool's options strip gains the Cut switch, Editor state for the next Room when none is selected. The Editor draws each cut's whole outline as a thin guide line in the viewport, as it draws the selection outline, never through RenderEngine, so no Export holds it. Picking uses the derived floor and walled pieces; the selected Room's outline and handles come from the whole source outline. The Portal tool snaps to walled pieces only, a stretch walled once giving the anchor to the last of its Rooms; a slide follows the walled pieces of the Portal's own Room; `F` looks for the nearest walled piece or Wall line.
- **Project file**: ProjectAccess changes nothing: the Room component travels through the serialisation registry at its new version, and nothing derived is written.
- **The timed test** follows Timed budget test: it builds the Layer of Recombined within a frame in ShapeEngine's own test, asserts its size before timing, bounds the fastest of a few runs, runs in the `timed` group, and the crates on its path (ShapeEngine, i_overlay and the crates it is built from, kurbo, lyon_tessellation) gain the `fast` profile overrides that give them the levels `dev` builds them at, named in the architecture's Profiles bullet.

## Testing

- **Composing seam**: a headless App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, and AuthoringManager over a fixture Asset Folder holding a door image of known pixel size, with no window and no RenderEngine, driven by Apply, Undo, and Redo messages and asserted on the Room, Portal, and Element components, each Room's derived shape (its floor mesh, its walled pieces, the Room its Walls are drawn at, its stretches), the Layer's children, the answers, and the history. Every Room's points are given exactly, so shared edges coincide exactly. A second Layer comes from opening a fixture Project file of two Layers, since the editor has no Add Layer yet.
  - **Rooms of a Layer combine**: `crates/drs-app/tests/rooms_combine.rs::rooms_of_a_layer_combine` (two overlapping rectangles, a chain of three, a Room inside another), `crates/drs-app/tests/rooms_combine.rs::a_cut_takes_floor_away` (a hole, a notch, a cut over no Room, and a cut across a shared edge), `crates/drs-app/tests/rooms_combine.rs::a_room_after_a_cut_fills_it`
  - **Walls run where the floor ends**: `crates/drs-app/tests/rooms_combine.rs::walls_run_where_the_floor_ends` (the walled pieces of two overlapping Rooms, one of them curved, ending where the other's outline crosses, and none inside the other's floor)
  - **Shared edges keep their Wall**: `crates/drs-app/tests/rooms_combine.rs::shared_edges_keep_their_wall` (a whole shared edge, a partly shared one, edges a hair apart and a hair overlapping, and a Room lying over the shared stretch), `crates/drs-app/tests/rooms_combine.rs::a_nested_room_on_a_shared_edge` (three Rooms on one line)
  - **Walled once**: `crates/drs-app/tests/rooms_combine.rs::walled_once` (a shared edge and two outline edges running together, each walled by the later Room only)
  - **Each Wall in its Room's look**, as derived: `crates/drs-app/tests/rooms_combine.rs::each_wall_in_its_rooms_look` (each Room's stroke mesh as wide as its own thickness)
  - **Each Room keeps its floor**, as derived: `crates/drs-app/tests/rooms_combine.rs::each_room_keeps_its_floor` (each floor mesh covering its own outline, a cut Room's absent, a floor under a later cut not covering the hole)
  - **Walls over the combination**, as derived: `crates/drs-app/tests/rooms_combine.rs::walls_over_the_combination` (the last Room named for every Room of a combination, through a chain and through a cut; a Room apart naming itself)
  - **Layers keep their Rooms apart**: `crates/drs-app/tests/rooms_combine.rs::layers_keep_their_rooms_apart` (overlapping Rooms and a cut on two Layers of an opened Project, each Room walled and floored whole across the Layers)
  - **Combined after every step**: `crates/drs-app/tests/rooms_combine.rs::combined_after_every_step` (placing, moving, a point moved, added, and removed, a bend, a thickness, the cut switch, removal, and undo and redo of each, each derived shape equal to that of the same Rooms placed afresh)
  - **Followed through a drag**: `crates/drs-app/tests/rooms_combine.rs::followed_through_a_drag`
  - **Whether a Room cuts stays editable**: `crates/drs-app/tests/rooms_combine.rs::whether_a_room_cuts_stays_editable`
  - **Portals go where a Wall runs**: `crates/drs-app/tests/rooms_combine.rs::portals_go_where_a_wall_runs` (placing, setting, and sliding onto a hidden edge refused; onto a shared edge's Wall and a hole's Wall accepted)
  - **Other Rooms never move a Portal**: `crates/drs-app/tests/rooms_combine.rs::other_rooms_never_move_a_portal`
  - **The centre decides**: `crates/drs-app/tests/rooms_combine.rs::the_centre_decides` (a cut straddling a Portal on either side of its centre)
  - **Gone with its Wall**: `crates/drs-app/tests/rooms_combine.rs::gone_under_a_placed_room`, `crates/drs-app/tests/rooms_combine.rs::gone_under_a_cut`, `crates/drs-app/tests/rooms_combine.rs::gone_when_the_cut_switch_changes`, `crates/drs-app/tests/rooms_combine.rs::gone_when_another_room_moves` (each with the answer naming the removed Portals and their Rooms, and an undo restoring them set)
  - **A drag decides at its end**: `crates/drs-app/tests/rooms_combine.rs::a_drag_decides_at_its_end` (over a door and back out, kept; ending over it, removed in the drag's one step; undo restoring it)
  - **A stretch follows the Walls**: `crates/drs-app/tests/rooms_combine.rs::a_stretch_follows_the_walls` (across a point where one Room's edge meets another's, onto the other Room's pieces, and stopping at a shared edge's ends)
  - The combination cases of the modified composing Rules: **A Room is placed as one step** by `gone_under_a_placed_room` above, **Room properties stay editable** by `whether_a_room_cuts_stays_editable` above, **The floor fills the outline** by `each_room_keeps_its_floor` above, **Refused anchors** by `portals_go_where_a_wall_runs` above, **Gone with its part of the Room** by the four `gone_` tests above.
- **Offscreen export seam**: the existing headless App with RenderEngine under Bevy's default plugins without a window, exporting to a temporary PNG and asserting pixels.
  - **Rooms of a Layer combine**, **Walls run where the floor ends**, and **Each Room keeps its floor**: `crates/drs-app/tests/export.rs::overlapping_rooms_share_one_floor` (the later floor's colour in the overlap, the floor colour where the earlier Room's edge runs inside the later one's floor, the wall colour on the outside)
  - **Shared edges keep their Wall** and **A stretch follows the Walls**: `crates/drs-app/tests/export.rs::a_shared_edge_keeps_its_wall_and_door` (the wall colour along the shared edge, a narrow Portal image's colour at its centre, and each Room's floor colour on its own side of the line inside the stretch)
  - **Each Room keeps its floor** and **Each Wall in its Room's look**, with a cut: `crates/drs-app/tests/export.rs::a_cut_leaves_a_hole` (the background in the hole beyond half the cut's thickness, the cut's wall colour on its outline, the Room's floor colour between)
  - **Each Wall in its Room's look**: `crates/drs-app/tests/export.rs::each_wall_keeps_its_rooms_colour`
  - **Walls over the combination** and **The Walls close round the combination**: `crates/drs-app/tests/export.rs::walls_lie_over_the_combinations_floors` (the earlier Room's wall colour inside the later Room's floor within half the thickness of the point where the outlines cross, and a Prop placed between the two Rooms showing over the earlier floor and under the Wall)
- **Projects seam**: the existing headless App saving and reopening a Project with overlapping, adjacent, and cutting Rooms and Portals on a shared edge and a hole's Wall; opening a fixture file holding a version-one Room; and opening a fixture file, as an editor that does not combine Rooms writes it, holding a Portal set into an edge inside another Room's floor.
  - **Nothing combined is saved**: `crates/drs-app/tests/projects.rs::combined_rooms_reopen_the_same` (the file's Room data holding only the Room's own fields; every derived shape and Portal the same after reopening; saving again byte-identical)
  - **Older Rooms do not cut**: `crates/drs-app/tests/projects.rs::older_rooms_do_not_cut`
  - **A Portal without its Wall stands**, with the modified **A lost Wall leaves the Portal standing**: `crates/drs-app/tests/projects.rs::a_portal_without_its_wall_stands` (drawn at its saved place with no stretch, kept through an edit of another Room, written back unchanged, Free Portal accepted, and set into its Wall again when the Room over it is moved away)
- **Timed**: **Recombined within a frame**: `crates/drs-shape-engine/tests/combine.rs::recombined_within_a_frame`
- **By hand**: **Cuts are shown in the editor**, **Cut in the Room options**, **Picking combined Rooms**, and **The whole outline is handled**, with the modified **Handles of the selected Room**, **Room options follow the selection**, **Picking Portals**, **Placing with the Portal tool**, **Dragging Portals**, and **Portal options and keys**: no automated seam for the egui interface, the accepted deviation of the composing spec; verified by driving the editor with the development-only input script, which also confirms that combined floors and Walls, holes, and shared Walls with their doors are drawn in the viewport where the model says, and that the combination follows a point drag smoothly on a Layer of twenty Rooms.
- ShapeEngine's own unit tests check what the Rules above rest on, as functions of the Engine alone, and are the coverage of no Rule: provenance through a union and a difference, curved pieces rebuilt on their curve with their ends on the cut vertex, provenance carried through a second pass after a cut, a shared edge clipped by a later cut, and a Portal's stretch crossing from one Room's pieces to another's (`crates/drs-shape-engine/src/combine.rs::tests::pieces_keep_their_source_through_a_cut`, `crates/drs-shape-engine/src/combine.rs::tests::curved_pieces_are_rebuilt_on_their_curve`, `crates/drs-shape-engine/src/combine.rs::tests::a_room_after_a_cut_keeps_its_source`, `crates/drs-shape-engine/src/combine.rs::tests::a_shared_edge_is_clipped_by_a_cut`, `crates/drs-shape-engine/src/portal.rs::tests::a_stretch_crosses_onto_another_room`).

## Out of Scope

- Opening a shared edge without a Portal, or keeping a Wall across an overlap: a shared edge always carries its Wall, and overlapping Rooms are always open into each other.
- Freeing a Portal instead of removing it when its Wall is taken away: Free Portal stays the Author's own Command.
- Combining Rooms across Layers, with drawn Walls, or with Caves; a cut that takes away from Rooms after it, from other Layers, or from Terrain, Props, or drawn Walls.
- Restacking Rooms, and moving a Room to another Layer: they come with Layers.
- A floor taking the colour of the combination, blended floors, and a Material other than the flat colours.
- Joining, splitting, or baking Rooms into one Room, and turning a combination into Walls.
- A key or modifier that draws a cut while held.
- Mitred or blended joins where Walls of different thickness meet.
- Layers of hundreds of Rooms beyond the measured case: deriving a whole Layer at every change is linear in its Rooms' edges.

## Further Notes

- **Architecture check**: no new component, contract operation, dependency direction, or restricted crate is needed. CombineOutlines, GenerateWalls, and AnchorPortals take the Layer's Rooms instead of one; the derived shape stays the `model` component AuthoringManager writes and the Editor and RenderEngine read; the Commands that change Rooms record the Portal removals in their own history groups, as the architecture prescribes for a Portal whose piece of Wall vanishes. The architecture's accepted gap, party walls verified for two coincident Room edges only, is met here by Rules that speak of floors on each side rather than of how many edges coincide; `a_nested_room_on_a_shared_edge` and `walled_once` are the tests that settle it, and a merge order of `EdgeOverlay` that cannot meet them is an architecture question. Two further cases the spike did not run are risks of the same kind: edges of two Rooms coinciding along the outline with their floors on the same side, which must keep the provenance of both, and the passes after a cut, which must carry provenance and coincident records through.
- **Settled open question**: a Portal that straddles where a cut or another Room takes its Wall away stays as long as a Wall runs at its centre; its stretch then follows the Wall round the new corner, and it is removed when its centre lies where the Wall is gone.
- A Portal's side stays relative to its own edge's direction, so a combination, whose pieces may run reversed, never changes which way a Portal faces.
- A Room whose floor is entirely cut away, or hidden under a later Room's floor, draws nothing and is picked nowhere until the Rooms over it change; it stays an Element, and it is selected again through undo or by moving what covers it.
- An Element placed between two Rooms of one combination is under the combination's Walls even where it lies far from the later Room, because a combination is drawn as one.
- Shared edges need exact coincidence: two edges whose points differ by any amount are either apart, and each walled, or overlapping, and open into each other. Snapping puts a point placed or dragged exactly on a Grid corner or on another Wall's or Room's point, which is how the Author reaches a shared edge; a point placed freely meets another only when it is given exactly alike.
- An editor that knows Rooms but not cutting refuses a Project holding version-two Rooms as saved by a newer editor; the alternative, a separate component that such an editor would keep and write back unread, was set aside because it would draw the Author's holes filled in and let the collaborator edit around them unseen.
