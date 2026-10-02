# Paint Terrain with one Material

**Capabilities**:
- composing: Paint, Edit Element
- export: Export Level
- projects: none of its Commands; saving and opening Terrain

## Problem Statement

A dungeon has Walls and Props, but its ground is the bare background: there is no stone floor, no dirt, no grass. The functional baseline paints ground with a soft brush, but what it paints is frozen once it is down, it blends only a handful of textures, and its exports are capped. An Author who wants a floor today places a floor image as a Prop and lives with its hard rectangular edge.

## Solution

A Paint tool in the Editor. The Author chooses the Paint tool, or presses `B`, and an image Asset in the browser, such as a flagstone texture, and drags across the Level: a circle as large as the Brush follows the pointer, a translucent band shows where the stroke goes, and on release the ground under the band shows the texture, repeated at its natural size, full in the middle of the stroke and fading out at its soft edge. The first stroke on a Layer makes its Terrain, under everything already on the Layer; every later stroke adds to it. The Brush has a size, a hardness, and a strength in the tool's options. Every stroke is one undo step, and undo takes away exactly that stroke. The Terrain is its strokes, not its pixels: saving keeps every stroke with its path and its Brush settings, an opened Project draws the same ground, and the Export computes each stroke afresh at the Export's resolution, so a printed map's ground is as sharp as its Walls. The texture a Terrain shows can be changed at any time without repainting.

## User Stories

### Painting

1. As an Author, I can choose the Paint tool from the viewport's tool strip, or press `B`, so that my next drags paint instead of placing or selecting.
2. As an Author, I can choose the image my Brush paints with in the browser, before or after choosing the Paint tool, so that any texture I own is ground I can paint.
3. As an Author, I can press on the Level, drag, and release to lay one stroke along the path my pointer took, so that painting is one gesture.
4. As an Author, I can press and release without moving to paint one round dab, so that a single spot of ground is one click.
5. As an Author, I can see a circle as large as my Brush following the pointer while the Paint tool is chosen, so that I know how much ground a stroke will cover before I press.
6. As an Author, I can see the stroke I am drawing as a translucent band as wide as my Brush, so that I see where it goes while I drag.
7. As an Author, I can see the painted ground in place of the band once I release, so that the result is visible the moment the stroke is done.
8. As an Author, I can keep painting stroke after stroke without choosing anything again, so that covering a room is a series of drags.
9. As an Author, I can paint more onto a Layer's Terrain with no Asset chosen, its own image being what the Brush paints with, so that coming back to a floor later needs no search in the browser.
10. As an Author, I am told in the status line to choose an Asset when I press with the Paint tool on a Layer that has no Terrain and none is chosen, and nothing is painted, so that a drag without a texture never leaves an invisible Element.
11. As an Author, I can paint across and beyond the Bounds, so that the Bounds never get in the way of composing.
12. As an Author, I can press Escape while drawing a stroke to throw it away and go back to selecting, so that a stroke started by accident costs nothing.
13. As an Author, I can choose the Wall tool while painting and have the Paint tool give way, discarding a stroke being drawn, and choose the Paint tool while drawing a Wall and have the unfinished Wall thrown away, so that one thing is ever happening under my pointer.

### The Brush

14. As an Author, I can set the Brush's size in Grid cells in the tool's options, so that a broad stroke fills a hall and a narrow one traces a path.
15. As an Author, I can set the Brush's hardness, from a fully soft edge to a hard one, so that ground fades into the background or ends crisply as I choose.
16. As an Author, I can set the Brush's strength, so that a stroke shows its texture only partly, as worn ground does.
17. As an Author, I can rely on each stroke keeping the Brush settings it was laid with when I change them afterwards, so that turning the size down for the next stroke never shrinks the ones already laid.
18. As an Author, I can rely on the Brush starting at a sensible size, hardness, and strength in a new editor, so that the first stroke looks right without tuning.

### How Terrain looks

19. As an Author, I can see a stroke fully show its texture in the middle and fade to nothing at its edge, the fade as wide as the hardness says, so that painted ground blends into what is around it.
20. As an Author, I can see a stroke of less than full strength show its texture partly over what is below it, so that worn or thin ground is one setting.
21. As an Author, I can rely on a stroke never getting darker where it turns sharply, crosses itself, or loops back, so that a stroke looks as even as the Brush I set.
22. As an Author, I can rely on going over ground again never making it more than the strongest stroke there, so that I can paint without counting how often I passed.
23. As an Author, I can rely on a weaker stroke painted over a stronger one leaving the stronger one showing, so that touching up an edge never thins out a floor.
24. As an Author, I can see the texture repeated edge to edge across the Level at its natural size, the same as if it were placed as a Prop, so that a vendor's flagstones are as big as the vendor meant and line up from one stroke to the next.
25. As an Author, I can rely on two separate strokes of the same Terrain showing one continuous texture where they meet, so that a floor painted in pieces looks like one floor.
26. As an Author, I can see the Terrain under every Prop and Wall already on its Layer, so that painting a floor never covers the furniture.
27. As an Author, I can place Props and draw Walls after painting and see them over the Terrain, so that the floor stays the floor.
28. As an Author, I can see a Terrain whose image is missing or broken still show where its strokes lie, in the placeholder colour, so that a missing texture never hides the shape of my ground.

### Changing Terrain

29. As an Author, I can rely on one Terrain per Layer gathering every stroke I paint on that Layer, so that a floor is one Element however many strokes it took.
30. As an Author, I can change the image a Layer's Terrain shows to the Asset chosen in the browser from the tool's options, every stroke keeping its path and Brush settings, as one undo step, so that trying another floor texture costs no repainting.
31. As an Author, I am told in the status line, with nothing painted, when I paint with an image other than the one the Layer's Terrain shows, so that I am never surprised by a whole floor changing its look, and I know to change the Terrain's image instead.

### History

32. As an Author, I can undo a stroke and have exactly that stroke disappear and everything else stay as it was, so that undo is as fine-grained as my painting.
33. As an Author, I can undo the first stroke on a Layer and have its Terrain disappear with it, and redo it to have the same Terrain back, so that painting and unpainting a floor is one line of history.
34. As an Author, I can redo an undone stroke and have it back exactly as it was laid, so that undo is never a loss.
35. As an Author, I can rely on the ground looking exactly as it did before a stroke once I undo it, so that undo never leaves a trace.
36. As an Author, I can undo and redo strokes with the usual shortcuts, in the order I took them, among my Walls and Props, so that history stays one line.
37. As an Author, I can rely on undo and redo waiting while I am drawing a stroke, so that a step is never taken back while it is still being made.
38. As an Author, I can rely on changing the Brush's settings never being an undo step, so that tuning the Brush costs me nothing.
39. As an Author, I can rely on clicking where only Terrain lies selecting nothing, so that the floor never gets in the way of picking what stands on it.

### Export, saving, and sharing

40. As an Author, I can export a Level and see its Terrain as the editor shows it, under its Props and Walls, so that the ground I paint is the ground I export.
41. As an Author, I can export at a high resolution and see the Terrain's edges as sharp as that resolution allows, never blown up from what the screen showed, so that a printed map's ground is crisp.
42. As an Author, I can export the same painted Level twice and get the same file, and get the same image whatever the tiles it is assembled from, so that a large painted export has no seams.
43. As an Author, I can save a Project with Terrain and reopen it with every stroke, its path, and its Brush settings as they were, and the ground drawn the same, so that my painting survives closing the editor.
44. As an Author, I can rely on a reopened Terrain still being its strokes, not a flattened picture, so that what later changes to strokes allow works on ground painted today.
45. As an Author sharing a Project with a collaborator whose editor does not know Terrain, I can rely on their editor keeping my Terrain as a placeholder of its extent and saving it back untouched, so that a round trip through an older editor never loses my ground.
46. As an Author, I can rely on a Terrain whose image is missing on another device keeping every stroke and its image's Asset Reference, so that the floor comes back as soon as the image is found.

## Rules

### The Terrain

**Terrain is its strokes**: a Terrain holds one Material, which shows an image Asset, and an ordered list of strokes, each a path of one or more points in Grid cells with the Brush settings it was laid with: a size in cells above zero, the diameter the Brush covers; a hardness from 0 to 1; and a strength above 0 up to 1. Follows from: Nothing is fixed at creation.

**Shaped by a soft round Brush**: a stroke's coverage at a point, how much of the Material it shows there, is computed from the point's distance *d* to the nearest point of its path (the straight segments from each point to the next, or the single point of a one-point path), with radius *r* half its size and hardness *h*: its strength where *d* ≤ *h* × *r*; nothing where *d* ≥ *r* and *d* > *h* × *r*; and between them its strength × (1 − 3*t*² + 2*t*³), where *t* = (*d* − *h* × *r*) / (*r* − *h* × *r*).

**No build-up along a stroke**: a stroke's coverage at a point is the same however many of its segments pass near the point, at a joint, a self-crossing, or a loop alike.

**Strokes composite by the strongest**: a Terrain's coverage at a point is the largest coverage any of its strokes has there, so a later, weaker stroke never lowers it and passing over ground again never raises it past the strongest stroke.
_Why_: strength is how much of the Material a stroke shows, not how much it adds; and a maximum is exact in one pass on the GPU, which painting moves to.

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

**The Material stays editable**: an Edit Element setting a Terrain's Material to another image Asset makes every stroke show that image, keeping every stroke's path and Brush settings, as one history step that undo returns to the image it had. Follows from: Nothing is fixed at creation.

**Terrain changes only its Material**: an Edit Element of a Terrain that changes anything but its Material, and an Edit Element setting the Material of an Element that is not a Terrain, are answered with the reason, change nothing, and record no history step.

### The Paint tool

**Painting with the Paint tool**: with the Paint tool chosen, from the tool strip or with `B`, a press on the Level starts a stroke at the pointer, moving adds the pointer's path to it, and the release sends one Paint onto the current Layer with the path and the Brush's settings, naming the chosen Asset or, with none chosen, no image; with no Asset chosen and no Terrain on the current Layer, a press paints nothing and the status line asks for an Asset; the tool stays chosen; undo and redo wait while a stroke is being drawn.

**Seeing the stroke**: with the Paint tool chosen, a circle as large as the Brush's size follows the pointer over the Level, and a stroke being drawn is shown as a translucent band as wide as the Brush along its path, until the release.

**Brush options**: with the Paint tool chosen, the tool's options show the Brush's size in cells, its hardness and its strength as percentages, and the name of the image it paints with, the chosen Asset's or the current Layer's Terrain's; the Brush starts at a size of two cells, a hardness of 50 %, and a strength of 100 %; the size offered goes from a tenth of a cell to sixty-four cells, the strength from 1 % to 100 %; the Brush's settings are the Editor's, never a history step and never saved; and, when an Asset is chosen and the current Layer's Terrain shows another image, a button there sends the Edit Element that makes the Terrain show the chosen Asset.

**Terrain is not picked**: the Select tool never selects a Terrain; a click where only Terrain lies under the pointer clears the selection.

### Export (export capability)

**Terrain at the Export's resolution**: each pixel of the Export shows a Terrain's coverage at that pixel's centre as its strokes give it at the Export's pixels per cell, never enlarged from what the editor shows, and the same in every tile the image is assembled from. Follows from: Nothing is fixed at creation.

### Saving (projects capability)

**Saved as its strokes**: a saved Terrain holds its image's Asset Reference and every stroke in order with its points, size, hardness, and strength, and reopens the same, with the same coverage; an editor that does not know the Terrain kind keeps it as a placeholder of its box and writes it back unchanged. Follows from: References are never dropped.

## Changes to existing behaviour

The composing Rules named here are those the composing spec holds now. Where Portals lands first, its modifications of the same Rules stand and the clauses below are added to them.

- composing — **Placed where clicked**: modified to "with an Asset chosen and neither the Wall tool nor the Paint tool chosen, a click on the Level places a Prop of that Asset on the current Layer, centred on the clicked point", because a press with the Paint tool paints.
- composing — **Placed on top**: modified to "a new Element is placed above every Element already on its Layer, except a Terrain made by a Paint, which is placed below them (Terrain goes under)", because ground lies under what stands on it.
- composing — **Placement records a reference**: modified to add "a Paint that makes a Terrain, and an Edit Element setting a Terrain's Material, record the image's Asset Reference the same way, and no second one for an Asset already recorded", because a Terrain's image is an Asset like a Prop's.
- composing — **Placement records the folder**: modified to add "the first Terrain image taken from an Asset Folder records the folder the same way", for the same reason.
- composing — **Anywhere on the Level**: modified to add "and a Terrain with strokes there", because strokes may lie outside the Bounds.
- composing — **Escape stops placing**: modified to add "or leaves the Paint tool, discarding a stroke being drawn", because the Paint tool is another way of not selecting.
- composing — **Topmost is selected**: modified to add "a Terrain is never selected (Terrain is not picked)", because the ground must not get in the way of what stands on it.
- composing — **One history**: modified to add Paint to the Commands that are each one undo step in the one history, because strokes join the history.
- composing — **Redo repeats exactly**: modified to add Paint, for the same reason.
- composing — **A failed Command is reported**: modified to add Paint, for the same reason.
- composing — **A failed load is a placeholder**: modified to add "a Terrain whose image cannot be loaded or decoded, or is Missing, is drawn as its coverage in the placeholder's colour, not as a box", because a box would hide the shape of the ground and cover the whole extent of its strokes.
- composing — **One thing under the pointer**: modified to add "choosing an Asset while the Paint tool is chosen makes it the image the Brush paints with and keeps the Paint tool; choosing the Paint tool leaves the Wall tool, discarding a Wall being drawn, drops the selection, and keeps a chosen Asset as the image the Brush paints with; choosing the Wall tool leaves the Paint tool, discarding a stroke being drawn", because the Paint tool joins the tool strip and takes its image from the browser.
- export — **Drawn as in the editor**: modified to add "a Terrain as its Material masked by its coverage, under the Elements above it, its coverage computed at the Export's resolution (Terrain at the Export's resolution)", because the Export computes strokes afresh rather than enlarging what the editor shows.
- projects — **A bad file is refused**: modified to add "a Terrain among it with a stroke of no point, a point that is not finite, a size not above zero or not finite, a hardness outside 0 to 1, or a strength not above 0 or above 1", because the file and the Command refuse the same strokes.

## Implementation Decisions

The technology the architecture fixes (strokes as the truth of a painted region, coverage as the maximum over segments, the CPU rasterizer as the golden reference, the tiled pixel cache private to PaintEngine, one masked draw per Material, the Shader contract, the generic history commands, the serialisation registry, and the deterministic maths) is used as written there and not restated.

- **What this change builds of Painting**: the CPU rasterizer only, and of the tiled pixel cache only the resident base band at 32 pixels per cell in 512-pixel tiles, for the editor at every zoom; the view-following overlay bands and GPU rasterization are the change that moves strokes to the GPU. The viewport therefore shows the base band magnified at high zoom, soft Brushes as the architecture measured them and hard edges as a ramp a few screen pixels wide; the Export does not use the cache. BlendWeights is not built: one Material needs no weights.
- **The Terrain descriptor**: `model` gains the Terrain kind in the Element kind registry under the stable name `terrain`, drawn as a painted surface, a new way of drawing beside images and stroked paths. Its component holds the Material, here the Asset Reference row of the image that the built-in tiled-image Shader shows, and the strokes in order, each its points in Grid cells and its Brush settings. The Brush settings (size, hardness, strength) are a type of their own in `model`, shared by the stroke and by the Editor's current Brush, so that what the Editor sets is what a stroke records. The component says itself what makes it malformed (Malformed strokes are refused), so the Paint, the Edit Element, and the reading of a file refuse the same Terrains for the same reasons. It is a serialisable component at version one on the Element tier; a later Material per stroke, for blending, is a version two with a migration. A Terrain with no strokes, which only a file can hold, is well formed, draws nothing, and has a box of no size at the origin.
- **The derived coverage**: `model` holds a Terrain's coverage, never saved and not reflected, as tiles of 512 by 512 pixels at 32 pixels per cell (16 cells a side), keyed by their whole-number position in the Level's pixel plane, negative keys included, an absent tile being empty, one byte a pixel, row 0 at the tile's top, each with a revision that changes only when its pixels do. AuthoringManager writes it, with the Terrain's box, in its once-per-frame derivation after every Manager has handled its Commands, Undo, and Redo, so a painted, undone, redone, or opened Terrain has its coverage before anything draws it. It keeps PaintEngine's cache in a component of its own and publishes the cache's tiles as shared buffers, so publishing copies no pixels. _Why_ derived in the model: RenderEngine draws it and may not depend on PaintEngine.
- **PaintEngine**: Rasterize takes strokes in order, a region given by its lower-left corner in cells and its size in pixels, and a number of pixels per cell, and returns one byte a pixel, row 0 at the top. A pixel's coverage is computed at its centre, found from its whole-pixel index counted from the region's corner and divided by the pixels per cell once per axis, so a pixel has the same value in every region that holds it, as the Export's tiles need; the value is the coverage times 255, rounded half up. Only the pixels inside each stroke's box grown by its radius are visited, and for each, only the segments whose own grown box holds it; distances use square roots and arithmetic only, so the result is the same on every machine. ApplyStroke brings the cache up to a Terrain's strokes: strokes appended since it last ran are composited onto the tiles they touch, and when earlier strokes changed or went, as an undo makes them, only the tiles touched by the strokes that differ are rasterized again from all strokes, so an undo of a stroke recomputes that stroke's tiles alone. Both are plain functions over model types; the cache is a value AuthoringManager owns, and PaintEngine uses no Bevy beyond the narrow ECS crates.
- **Paint**: Apply gains a Paint carrying the Layer, the stroke (its points and Brush settings), and optionally the chosen Asset (folder key and place). AuthoringManager checks the stroke through the Terrain component's own check and looks for the topmost Terrain among the Layer's children. With none, it resolves the Asset as a Prop's placement does (LoadAsset, the Asset Reference and folder rows recorded on the first application and never removed) and records a step that spawns the Terrain with a fresh ElementId as the first child of the Layer, holding the stroke, which undo takes off by identity and redo spawns again first. With one, a named Asset is looked up in the Project's Asset Reference table by its folder and place: one not recorded, or recorded under a row other than the Terrain's, is another image and the Paint is refused. Otherwise it records a reversible step of its own that appends the stroke and, on revert, removes it from the end, keeping only that stroke, as the architecture's History bullet requires. Each Paint is a step of its own that closes any gesture group left open.
- **Edit Element**: the Element change gains a Terrain's Material, carrying the chosen Asset; AuthoringManager resolves it as a placement does, recording the rows on the first application, and records a step of its own that swaps the Terrain's image row and back, the strokes untouched. Every other change sent for a Terrain, its position included, is a CommandFailed: moving and reshaping strokes is the change that edits strokes. Remove Element of a Terrain goes through the generic reflection snapshot like any Element's; its coverage is derived again when it is restored.
- **Rendering**: RenderEngine draws a Terrain through a Material type of its own, the masked tiled image: a plain WGSL Shader with no Bevy imports, a Bundled File compiled into RenderEngine and added to the shader assets at startup, declaring its Params uniform (the image's natural size in cells and the tile's origin in cells), the image and its sampler, and the coverage tile and its sampler at group 2. The Shader wraps the image's coordinates itself and samples it with the image's own sampler, so a Prop and a Terrain can share one loaded image. Each coverage tile is uploaded as an R8 image when its revision changes and drawn as one quad of 16 by 16 cells with that Material, alpha-blended at the Terrain's stacking depth, so it sorts with the sprites and Wall meshes by depth. The image is loaded through the resolution table and the `lib://` handle as a Prop's is; while it is loading, Missing, or failed, the tiles are drawn in the placeholder's flat colour, masked by the same coverage. A capture of a region waits for a Terrain's image as it waits for a Prop's.
- **Export**: ProjectManager, for each tile of the Export, asks PaintEngine to Rasterize every Terrain of the Level over that tile's region at the Export's pixels per cell, and hands those coverages to RenderEngine with the RenderRegion request. RenderEngine draws them for that capture alone, one quad of the tile's size per Terrain with the same Material, in place of the cache's tiles, which the offscreen camera does not see; the coverage's pixels then fall on the Export's pixels one to one, so linear sampling returns them unchanged. This needs the dependency the Further Notes name.
- **The tool**: the tool strip gains Paint, `B` choosing it and Escape returning to Select; its state (the stroke being drawn and the current Brush settings) lives in an Editor module of its own, as the Wall tool's does. While a stroke is drawn, a point is added to its path whenever the pointer is more than an eighth of the Brush's size from the last point added, and the release point ends the path unless it lies on the last one; a press and release without movement is a one-point path. The circle and the band are drawn by the Editor over the viewport, the band translucent with round ends; the painted result appears once AuthoringManager has derived it, in the frame after the release. The options follow the Wall tool's: drag values for the size, the hardness, and the strength, changed in the Editor's state only, so no gesture is needed; the button that changes the Terrain's image sends one Edit Element marked single. The Editor finds the current Layer's Terrain and its image's name through the model and the resolution table. Picking leaves every Terrain out. The development-only input script's `describe` step logs the Paint tool, the Brush's settings, and every Terrain with its ElementId, box, image row, and number of strokes.
- **Project file**: ProjectAccess changes nothing: the Terrain component travels through the serialisation registry under `terrain`; an editor without the Terrain kind keeps its envelope on the Element and writes it back. Opening a Project rasterizes every Terrain's strokes into the base band on the CPU in the frame after Open.

## Testing

- **Composing seam**: a headless App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, and AuthoringManager, with no window and no RenderEngine, over a fixture Asset Folder holding two texture images of known pixel size, driven by Apply, Undo, and Redo messages and asserted on the Terrain and Element components, the derived coverage (pixel values read from its tiles at chosen cells, and whole tiles compared), the Layer's children, the answers, and the history.
  - **Terrain is its strokes**: `crates/drs-app/tests/terrain.rs::terrain_is_its_strokes`
  - **Shaped by a soft round Brush**: `crates/drs-app/tests/terrain.rs::shaped_by_a_soft_round_brush` (full strength within the hardness, nothing beyond the radius, the falloff's value halfway between, and a one-point dab)
  - **No build-up along a stroke**: `crates/drs-app/tests/terrain.rs::no_build_up_along_a_stroke` (the coverage at a sharp joint and at a self-crossing equals a single segment's there)
  - **Strokes composite by the strongest**: `crates/drs-app/tests/terrain.rs::strokes_composite_by_the_strongest` (two overlapping half-strength strokes stay at half, a weaker stroke over a stronger leaves the stronger)
  - **The box follows the strokes**: `crates/drs-app/tests/terrain.rs::the_box_follows_the_strokes`
  - **Terrain goes under**: `crates/drs-app/tests/terrain.rs::terrain_goes_under` (a Prop and a Wall placed first stay above the Terrain; a Prop placed after lands on top)
  - **One Terrain per Layer**: `crates/drs-app/tests/terrain.rs::one_terrain_per_layer`
  - **A stroke is one step**: `crates/drs-app/tests/terrain.rs::a_stroke_is_one_step`
  - **The first stroke makes the Terrain**: `crates/drs-app/tests/terrain.rs::the_first_stroke_makes_the_terrain`
  - **Painted with its Material**: `crates/drs-app/tests/terrain.rs::painted_with_its_material`
  - **Undo leaves no trace**: `crates/drs-app/tests/terrain.rs::undo_leaves_no_trace` (every tile of the coverage byte for byte as before the stroke, after undoing a stroke that overlaps two earlier ones)
  - **Coverage is the strokes alone**: `crates/drs-app/tests/terrain.rs::coverage_is_the_strokes_alone` (strokes laid one by one, the same strokes after undoing back to the first and redoing, and the same strokes saved and opened, give identical tiles)
  - **Malformed strokes are refused**: `crates/drs-app/tests/terrain.rs::malformed_strokes_are_refused`
  - **The Material stays editable**: `crates/drs-app/tests/terrain.rs::the_material_stays_editable`
  - **Terrain changes only its Material**: `crates/drs-app/tests/terrain.rs::terrain_changes_only_its_material`
  - The Terrain cases of the modified composing Rules: **Placed on top** by `terrain_goes_under` above, **Placement records a reference** `crates/drs-app/tests/terrain.rs::painting_records_a_reference`, **Placement records the folder** `crates/drs-app/tests/terrain.rs::painting_records_the_folder`, **Anywhere on the Level** `crates/drs-app/tests/terrain.rs::strokes_lie_anywhere_on_the_level` (a stroke outside the Bounds and at negative cells, with its tiles), **One history** `crates/drs-app/tests/terrain.rs::paint_shares_the_history`, **Redo repeats exactly** `crates/drs-app/tests/terrain.rs::paint_redoes_exactly`, and **A failed Command is reported** by `malformed_strokes_are_refused` and `painted_with_its_material` above.
- **Offscreen export seam**: the existing headless App with RenderEngine under Bevy's default plugins without a window, exporting to a temporary PNG and asserting pixels.
  - **Drawn masked by coverage** and the Terrain part of the modified **Drawn as in the editor**: `crates/drs-app/tests/export.rs::a_stroke_shows_its_material` (the texture's colour on the path of a full-strength stroke, the background beyond its radius, and a colour strictly between the two in its soft edge), `crates/drs-app/tests/export.rs::a_weaker_stroke_shows_partly` (a half-strength stroke's middle strictly between the texture and the background)
  - **Tiled at natural size**: `crates/drs-app/tests/export.rs::terrain_is_tiled_at_natural_size` (a texture two cells wide whose halves differ: each half's colour in the cells the repetitions from the origin put it in, continuous across two strokes)
  - **Terrain goes under** as drawn: `crates/drs-app/tests/export.rs::terrain_lies_under_props_and_walls`
  - **Terrain at the Export's resolution**: `crates/drs-app/tests/export.rs::terrain_at_the_exports_resolution` (a hard stroke exported at 200 pixels per cell: the texture's colour at the pixel whose centre lies just inside the radius and the background at the one just outside, which the editor's 32 pixels per cell could not give), `crates/drs-app/tests/export.rs::painted_tiles_leave_no_seams` (the existing Rule **Tiles leave no seams** for a painted Level: two tile sizes byte-identical)
  - **Same Level, same image** for a painted Level: `crates/drs-app/tests/export.rs::painted_levels_export_the_same`
  - **A failed load is a placeholder** as modified: `crates/drs-app/tests/export.rs::a_missing_terrain_image_keeps_its_shape` (the placeholder's colour on the path, the background beyond the radius)
- **Projects seam**: the existing headless App saving and reopening a Project with a Terrain of several strokes, and a second App whose registry lacks the Terrain kind opening and saving the same file.
  - **Saved as its strokes**: `crates/drs-app/tests/projects.rs::terrain_is_saved_as_its_strokes` (the strokes, their order, their Brush settings, the image's Asset Reference, and the coverage after reopening), `crates/drs-app/tests/projects.rs::unknown_terrain_round_trips`
  - **A bad file is refused** as modified: `crates/drs-app/tests/projects.rs::a_malformed_terrain_is_refused`
- **By hand**: **Painting with the Paint tool**, **Seeing the stroke**, **Brush options**, and **Terrain is not picked**, with the modified **Placed where clicked**, **Escape stops placing**, **Topmost is selected**, and **One thing under the pointer**: no automated seam for the egui interface, the accepted deviation of the composing spec; verified by driving the editor with the development-only input script, which also confirms that the viewport draws the Terrain where the model says, under the Props and Walls on its Layer, and that a Missing image keeps its shape in the placeholder's colour.
- PaintEngine's own unit tests check what the Rules above rest on, as functions of the Engine alone, and are the coverage of no Rule: the stamp against its analytic values, the maximum over segments at a joint, a pixel the same in every region that holds it, ApplyStroke's appended result equal to a full Rasterize, and an undo rasterizing only the touched tiles (`crates/drs-paint-engine/src/rasterize.rs::tests::the_stamp_matches_its_formula`, `crates/drs-paint-engine/src/rasterize.rs::tests::a_joint_takes_the_maximum`, `crates/drs-paint-engine/src/rasterize.rs::tests::a_pixel_is_the_same_in_any_region`, `crates/drs-paint-engine/src/cache.rs::tests::appending_equals_rasterizing`, `crates/drs-paint-engine/src/cache.rs::tests::an_undo_touches_only_its_tiles`).

## Out of Scope

- Erasing, and moving, reshaping, or deleting a stroke after it is laid: the change that makes an erase a stroke and every stroke editable.
- GPU rasterization and the view-following bands at 64, 128, and 256 pixels per cell: the change that moves strokes to the GPU; until then the viewport magnifies the 32-pixel base.
- More than one Material on a Terrain, and blending them: the blending change; until then a stroke with another image is refused.
- Saving and choosing Brush Presets, Brushes from Plugins, textured or shaped Brush tips, pressure, jitter, and scatter.
- Materials from Assets and author Shaders: the Terrain's Material is the built-in tiled image.
- Rotating, scaling, or offsetting the texture's repetition.
- Selecting, moving, or removing a Terrain from the viewport, and changing the Brush's size with keys.
- Patches, Water, and Caves, the other painted Element kinds.
- A painted live result while dragging, before the release.

## Further Notes

- **Architecture check**: within the architecture. ProjectManager calls PaintEngine's Rasterize per Export tile and hands the coverages to RenderEngine's RenderRegion, as the Export a Level call chain shows; RenderEngine's own Material type uses `bevy_shader`, which the render stack may use. PaintEngine needs no render stack in this change, since its CPU rasterizer returns plain bytes and RenderEngine turns them into images; the Brush's settings live in `model` as a shared type, the current Brush in the Editor; the derived coverage follows the Derived model component guideline. The Shader contract's single texture and sampler is for author Shaders; how an author Shader is multiplied by its weight mask is for the custom Materials change.
- **Domain**: "stroke" is used in its plain sense, one pass of a Brush from press to release, as "point" and "segment" are for Walls. The next change makes strokes editable one by one, which may warrant a glossary entry (for example, "Stroke: one pass of a Brush along a path, laid by one Paint; a painted Element's shape is its strokes, in order").
- **Cost on the CPU**: a new stroke rasterizes only its own pixels, but an undo on dense Terrain, and opening a Project with much Terrain, rasterize every stroke over the touched tiles and can take a noticeable fraction of a second to seconds in a development build, as the architecture's CPU figures show; the GPU change removes this. A large painted Export rasterizes on the CPU and takes longer than an unpainted one.
- Brush spacing is not a setting: coverage is continuous along a stroke's segments, so there are no dabs to space; the Editor only samples the pointer's path finely enough for the Brush's size.
