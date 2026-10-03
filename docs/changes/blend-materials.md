# Blend any number of Materials

**Capabilities**:
- composing: Paint, Edit Element
- export: none of its Commands; the Export draws a Terrain from the weights of its Materials
- projects: none of its Commands; saving and opening a Terrain whose strokes each show their own image

## Problem Statement

A Terrain shows one image. To lay flagstones with a path of dirt through them and moss creeping in at the edges, the Author has to give up: painting with another image on the same Layer is refused, and changing the Terrain's image changes the whole floor at once. Ground in a real map is never one texture; it is several, fading into each other where they meet. The functional baseline blends Terrain, but only in four to eight slots per Level, and an Author who wants a ninth texture has to choose which one to give up.

## Solution

Every stroke paints with the image the Author chose for it, and one Terrain blends every image its strokes show, however many. Where a stroke of one image is laid over ground of another, the new image takes its share as the stroke's strength and soft edge say, and the ground beneath gives way by as much, so that ground fully painted stays fully painted whatever mix of images it holds. A stroke laid later takes its share over a stroke laid earlier, so the order of painting decides what lies on top, as it does for paint and erase today; painting the same ground again never builds up, and an erase takes from every image alike. Choosing the image is what it is today, the Asset chosen in the browser; with none chosen, the Brush paints with the image of the Terrain's latest stroke. A stroke's image stays editable, and an image can be replaced by another across the whole Terrain in one step, so trying another floor texture still costs no repainting. The Export blends the same images by the same rule at its own resolution, and an image missing on one device leaves its share of the ground in the placeholder's colour while every other image shows.

## User Stories

### Painting with several images

1. As an Author, I can choose another image in the browser while painting and lay the next stroke with it on the same Layer, so that a floor of flagstones gets its path of dirt without a second Layer.
2. As an Author, I can rely on a stroke of a new image joining the Layer's Terrain rather than starting another, so that the ground of a Layer stays one Element however many images it holds.
3. As an Author, I can paint with as many different images as I like on one Terrain, ten or twenty, without being told to give one up, so that the ground never runs out of slots.
4. As an Author, I can rely on each stroke keeping the image it was laid with when I choose another image afterwards, so that choosing moss for the next stroke never turns the flagstones into moss.
5. As an Author, I can keep painting with no Asset chosen and have the Brush paint with the image of the Terrain's latest stroke, so that coming back to the ground I was painting needs no search in the browser.
6. As an Author, I can read in the tool's options which image my next stroke paints with, the chosen Asset's or the latest stroke's, so that I know what the next drag lays down.
7. As an Author, I can start a Terrain on a Layer with any image I choose, and paint every further image over it, so that the first image is not special.

### How images blend

8. As an Author, I can paint a full-strength stroke of one image over ground of another and see only the new image along its middle, so that a path is a path and not a tint.
9. As an Author, I can see a soft stroke's edge fade from the new image into the image beneath it, with no gap of the background between them, so that two textures meet as smoothly as a soft Brush says.
10. As an Author, I can paint a half-strength stroke of one image over fully painted ground of another and see half of each, so that worn or mixed ground is one setting.
11. As an Author, I can rely on ground that was fully painted staying fully painted whatever images I paint over it, so that blending never lets the background show through a floor.
12. As an Author, I can paint the image beneath back over the one I laid on top and see it take over again, so that the latest stroke always says what lies on top.
13. As an Author, I can rely on passing over the same ground again with the same image never making it show more than the strongest stroke of that image there, so that I can paint without counting how often I passed.
14. As an Author, I can rely on a weaker stroke of an image over a stronger stroke of the same image leaving the stronger one showing, so that touching up an edge never thins out a floor.
15. As an Author, I can rely on a stroke of one image never getting darker or showing more where it turns sharply, crosses itself, or loops back over another image, so that a stroke looks as even over other ground as over bare ground.
16. As an Author, I can paint three or more images over the same spot and see each of them in it in proportion to how strongly it was painted there, so that a muddy, mossy, cracked patch is a few strokes.
17. As an Author, I can paint an image over bare ground beside another image and see it alone there, so that blending only happens where images meet.
18. As an Author, I can rely on a Terrain painted with one image looking exactly as it did before images could blend, so that my existing floors are untouched.
19. As an Author, I can see each image repeated edge to edge across the Level at its natural size, every image's repetitions starting at the Level's origin, so that two textures of the same size line up where they meet.

### Erasing among images

20. As an Author, I can erase across ground of several images and see every one of them taken away alike, so that an erase is a hole in the ground, not in one texture.
21. As an Author, I can erase with less than full strength across blended ground and see all of it thinned, never below what the strength leaves, so that worn patches work on mixed ground too.
22. As an Author, I can paint an image over ground I erased and see that image alone there, as on bare ground, so that an erased patch never brings the old texture back when I repaint it.
23. As an Author, I can erase from a Terrain of several images whatever Asset is chosen in the browser, so that erasing never asks for an image.

### Changing images afterwards

24. As an Author, I can select a stroke with Edit strokes and read in the options strip which image it shows, so that I know which texture each stroke laid.
25. As an Author, I can set the selected stroke to show the Asset chosen in the browser, as one undo step, every other stroke keeping its image, so that one stroke laid with the wrong texture is fixed without repainting it.
26. As an Author, I can replace the selected stroke's image with the chosen Asset on every stroke of the Terrain that shows it, as one undo step, so that trying another floor texture costs no repainting, however many strokes it took.
27. As an Author, I can rely on replacing an image keeping every stroke's path, Brush settings, erasing, and place in the order, so that only the texture changes.
28. As an Author, I can replace one image with another the Terrain already shows and see the two become one, so that merging two textures I no longer want apart is one step.
29. As an Author, I can turn an erase into painting and see it paint with the image it carries, the image of the stroke laid before it unless I set another, so that turning an erase around never paints with a texture I did not expect.
30. As an Author, I can rely on setting a stroke to the image it already shows, or replacing an image with itself, never being an undo step, so that a click that changes nothing costs nothing.
31. As an Author, I can undo and redo every change of a stroke's image and every replacement, and see the ground exactly as it was, so that trying a texture is never a risk.

### Undo and the same ground everywhere

32. As an Author, I can undo a stroke of one image over another and see both images exactly as they were before it, so that undo never leaves a trace in a blend.
33. As an Author, I can rely on blended ground being the same whether I reached its strokes by painting, erasing, editing, removing, undoing, or opening a file, so that what I see is always what the strokes say.
34. As an Author, I can move, reshape, retune, or remove a stroke of one image over others and see the blend follow, so that every stroke stays editable on mixed ground.
35. As an Author, I can zoom in on blended ground and see the images meet as sharply or softly as the Brush says at every zoom, so that close-up work on mixed ground is as reliable as on one image.
36. As an Author, I can paint, edit, zoom, and pan over ground of six images without the editor stuttering, so that a richly painted floor is as quick to work on as a plain one.
37. As an Author, I can rely on images whose strokes lie far out of view costing no memory for the sharp ground of the view, so that a Level of many textures stays light.

### Saving, sharing, and missing images

38. As an Author, I can save a Project with blended Terrain and reopen it with every stroke's image as it was and the ground drawn the same, so that my blends survive closing the editor.
39. As an Author, I can open a Project saved before images could blend and find every stroke showing the image its Terrain showed, so that older maps open unchanged.
40. As an Author sharing a Project with a collaborator whose editor predates blending, I can rely on their editor refusing it as newer, naming the version, rather than opening it with every stroke in one image, so that a round trip through an older editor never flattens my textures.
41. As an Author sharing a Project with a collaborator on a different computer, I can rely on their editor showing the same blend mine shows, so that we discuss the same ground.
42. As an Author on a device that lacks one of a Terrain's images, I can see that image's share of the ground in the placeholder's colour while every other image shows as it should, so that one missing texture never hides the rest of the floor.
43. As an Author on a device that lacks some of a Terrain's images, I am told which images are missing with the Terrain counted among what uses each, so that I know what to install.

### Export

44. As an Author, I can export a Level of blended Terrain and see every image in the image file exactly where and as strongly as the editor shows it, so that the map I hand out is the map I painted.
45. As an Author, I can export at a resolution finer than anything the editor shows and see the images meet as sharply as that resolution allows, so that a printed map's ground is crisp where textures meet.
46. As an Author sharing a Project with a collaborator on a different computer, I can rely on the same Level exporting to the same image on both, however many images its Terrain blends, so that a re-export by either of us changes only what we changed.
47. As an Author, I can export a Terrain one of whose images is missing and see that image's share in the placeholder's colour beside the other images, so that the floor's shape survives a missing texture in the image too.

## Rules

A stroke's coverage at a point is as Shaped by a soft round Brush says. A Terrain's coverage at a point, how much of the Terrain shows there, is composited from all its strokes in order as Strokes composite by the strongest and Erasing caps what remains say, whatever images they show.

### Materials

**Each stroke has its Material**: every stroke of a Terrain holds, beside its path, its Brush settings, and whether it erases, the image Asset its Material shows; strokes showing the same image paint with the same Material, and a Terrain blends one Material for each image its strokes that paint show, however many. Follows from: Every Element that shows a surface is drawn with a Material.

**No limit on Materials**: a Paint, a change of a stroke's image, and a replacement of an image are never refused for the number of Materials the Terrain already blends, and every Material of a Terrain of twelve or more shows as the weight Rules say.

**Painted with the image it names**: a Paint that paints and names an image adds a stroke showing that image to the topmost Terrain on its Layer, whatever images its other strokes show, or makes the Terrain when the Layer has none; one that names no image adds a stroke showing the image of its Terrain's last stroke, and is refused on a Layer with no Terrain.

**An erase keeps the image before it**: a Paint that erases adds a stroke showing the image of its Terrain's last stroke, whatever image it names, or none.

**Coverage takes every Material**: a Terrain's coverage is composited from all its strokes in the order they were laid, a stroke of any image counting as it does on a Terrain of one image, so painting over ground of another image never raises the coverage past the strongest stroke there.

### Weights

**A Material's claim**: a Material's claim at a point starts at none and is composited from the Terrain's strokes in the order they were laid: a stroke of that Material that paints raises it to the stroke's own coverage there where that is higher; a stroke of another Material that paints, and every erase, lower it to one minus the stroke's own coverage there where that is lower.

**Weights share the coverage**: a Material's weight at a point, how much of it the Terrain shows there, is the Terrain's coverage there times the Material's claim divided by the sum of every Material's claim there, and none where every claim is none; so the weights at a point sum to exactly the coverage there and never to more than one.
_Why_: a claim is a maximum or a minimum, so it is exact in one pass and never builds up, while sharing the coverage by the claims keeps fully painted ground fully painted whatever images meet on it, which no weight computed for each Material alone can promise.

**A later Material takes its share**: where a stroke of one Material with coverage *c* is laid over ground the strokes before it cover fully with another Material alone, the new Material's weight is *c* and the other's 1 − *c*; within a full-strength stroke's hardness, its Material alone shows.

**Painting the same Material again changes nothing**: a stroke of a Material laid again where that Material's claim is already as strong, and a weaker stroke of a Material over a stronger one of the same Material, change no weight; a stroke's weights at a joint, a self-crossing, or a loop over other Materials are the same as a single segment's.

**One Material is as before**: on a Terrain whose strokes that paint all show one image, that Material's weight at every point is the Terrain's coverage there, so the Terrain is drawn exactly as a Terrain of one image is drawn without blending.

**An erase takes from every Material**: an erase lowers the Terrain's coverage and every Material's claim alike, so a full-strength erase leaves no Material within its hardness, an erase of strength *s* leaves at most 1 − *s* of all of them together, and a stroke laid after it over ground it emptied shows its own Material alone. Follows from: Erasing caps what remains.

**Weights are the strokes alone**: every Material's weight at every point is the same however the strokes came to be, laid one by one, edited, removed, undone and redone, or opened from a file, and after undoing a stroke, an edit of one, or a change of images, every point has the weights it had before the step was taken. Follows from: Nothing is fixed at creation; Every Command can be undone.

### Drawing

**Drawn blended by weight**: a Terrain is drawn at its place in the stacking order showing, at each point, every Material's image at its weight there and what lies below the Terrain at one minus its coverage there, for images with no transparency of their own; where the coverage is full, nothing below shows, whatever Materials share it. Follows from: Every Element that shows a surface is drawn with a Material.

**Every Material tiled at natural size**: each Material of a Terrain shows its image repeated edge to edge across the Level, upright, each repetition the image's natural size in Grid cells, one repetition with its lower-left corner at the Level's origin.

**A missing Material keeps its share**: a Material whose image is Missing, or cannot be loaded or decoded, is drawn as its weight in the placeholder's colour, every other Material of its Terrain as its image. Follows from: References are never dropped.

### Changing images

**A stroke's image stays editable**: an Edit Element setting a stroke's image to an image Asset makes that stroke show it and changes nothing else of it or of any other stroke, as one history step that undo returns to the image it had; one naming the image the stroke already shows changes nothing and records no step. Follows from: Nothing is fixed at creation.

**An image can be replaced**: an Edit Element replacing an image a Terrain's strokes show with an image Asset makes every stroke of that Terrain that showed it show the new one, keeping every stroke's path, Brush settings, erasing, and place in the order, as one history step that undo returns to the strokes' images as they were; one naming the same image for both changes nothing and records no step, and one naming an image no stroke of the Terrain shows is answered with the reason, changes nothing, and records no step. Follows from: Nothing is fixed at creation.

**An image change records a reference**: setting a stroke's image or replacing an image with one the Project has not recorded records its Asset Reference and its Asset Folder's Canonical Name and version as placing a Prop of it would, and no second one for an Asset already recorded. Follows from: A Project is device-independent.

### The Paint tool

**The image the Brush paints with**: with the Paint tool painting, the image the next stroke paints with is the chosen Asset or, with none chosen, the image of the last stroke of the current Layer's Terrain; the options strip names it, and asks for an Asset to paint with when there is neither.

**Images of the selected stroke**: with the Paint tool editing strokes and a stroke selected, the options strip names the image the stroke shows and, when an Asset is chosen that is not that image, offers a button that sets the stroke's image to it and a button that replaces the stroke's image with it on the whole Terrain, each sending one Edit Element marked single.

### The same ground at every band

These extend the composing spec's Rules of painted ground at every zoom to every Material.

**Every Material at every band matches the reference**: at every band, every Material's weight that the tiles the editor rasterizes on the GPU give at a pixel is within 3/255 of the weight PaintEngine's CPU computation gives there for the Terrain's strokes over that tile's region at that band, on a Terrain of two and of five Materials, for soft and hard strokes, a half-strength erase, and strokes of one Material crossing another's.
_Why_: the GPU's claims and coverage are each within 1/255 of the CPU's, and a weight is a ratio of them.

**Materials hold only the tiles they reach**: a Material holds tiles only where some stroke of it that paints reaches, at the base and at the active band alike, so a Material whose strokes lie out of the view holds no tile of an overlay band, and a Terrain never holds more overlay tiles for its coverage, nor for each Material's mask or claim, than the bound for the view's area that Overlay tiles are let go out of reach gives.

**Only the touched tiles blend again**: a Paint, an edit, a removal, a change of a stroke's image, a replacement of an image, an undo, or a redo computes afresh the weights of every Material only on the tiles of the base and of the active band that the reach of a changed stroke meets, as it was or as it is; every other tile of every Material keeps its revision.

### Cost

**Many Materials cost the main thread little**: on the Terrain of Painting costs the main thread little (200 strokes of 40 segments each over a Level of 60 by 60 cells, one in five erasing, a view of 4096 by 4096 screen pixels at a zoom of 256) with its strokes that paint showing six images in turn, PaintEngine's main-thread work to bring the tiles up to one new stroke takes under 1 ms, up to one moved stroke under 2 ms, a switch from the base to the band of 256 under 6 ms, and a pan of 512 screen pixels at that band under 3 ms, in the build the Author runs.

### Export

**The Export blends by the same weights**: the Export computes every Material's weight of every Terrain with PaintEngine's CPU computation at the Export's resolution and never from the editor's tiles, so each pixel of the Export shows every Material's image at the weight its strokes give at that pixel's centre at the Export's pixels per cell, the same in every tile the image is assembled from, and two Exports of the same Level at the same resolution are byte-identical. Follows from: Nothing is fixed at creation.

### Saving

**Saved with each stroke's image**: a saved Terrain holds every stroke in order with its image's Asset Reference beside its points, Brush settings, and whether it erases, and reopens with the same strokes and the same weights; a Terrain saved before strokes had images of their own opens with every stroke showing the image the Terrain showed, and one saved before strokes could erase also with every stroke painting; the next save writes the current version, and an editor that knows only the version before it refuses it as newer. Follows from: References are never dropped.

## Changes to existing behaviour

- composing — **Terrain is its strokes**: modified to "a Terrain holds an ordered list of strokes, each a path of one or more points in Grid cells with the Brush settings it was laid with (a size in cells above zero, the diameter the Brush covers; a hardness from 0 to 1; and a strength above 0 up to 1), whether it paints or erases, and the image Asset its Material shows (Each stroke has its Material)", because a Terrain no longer holds one Material.
- composing — **Tiled at natural size**: replaced by **Every Material tiled at natural size**, because a Terrain shows several images.
- composing — **Drawn masked by coverage**: replaced by **Drawn blended by weight**; on a Terrain of one image the two say the same (One Material is as before).
- composing — **Painted with its Material**: replaced by **Painted with the image it names**; a Paint naming another image than the Terrain's strokes show is no longer refused.
- composing — **An erase needs no image**: modified to "a Paint that erases adds its stroke to the topmost Terrain on its Layer whatever image it names, or none, the stroke showing the image of the Terrain's last stroke (An erase keeps the image before it)".
- composing — **The first stroke makes the Terrain**: modified to "a Paint on a Layer with no Terrain places a Terrain holding that stroke, showing the image it names, in the same history step; …" (the rest unchanged), because the image belongs to the stroke.
- composing — **The Material stays editable**: removed, and replaced by **A stroke's image stays editable** and **An image can be replaced**.
- composing — **Terrain changes only its Material and its strokes**: modified to "an Edit Element of a Terrain that changes anything but one of its strokes or replaces an image its strokes show, its position and every change only a Wall, a Room, or a Portal has included, and an Edit Element replacing an image or changing a stroke of an Element that is not a Terrain, are answered with the reason, change nothing, and record no history step".
- composing — **Placement records a reference**: modified so that "a Paint that makes a Terrain, and an Edit Element setting a Terrain's Material" reads "a Paint that paints with an image, an Edit Element setting a stroke's image, and one replacing an image (An image change records a reference)", because every stroke now names its image.
- composing — **Undo leaves no trace**: modified to "after undoing a stroke, an edit or the removal of one, a change of a stroke's image, or a replacement of an image, every point of its Terrain has the coverage and the weight of every Material it had before the step was taken".
- composing — **Coverage is the strokes alone**: extended by **Weights are the strokes alone** to every Material's weight.
- composing — **A failed load is a placeholder**: modified so that "a Terrain whose image cannot be loaded or decoded, or is Missing, is drawn as its coverage in the placeholder's colour" reads "a Material of a Terrain whose image cannot be loaded or decoded, or is Missing, is drawn as its weight in the placeholder's colour (A missing Material keeps its share)".
- composing — **Brush options**: modified so that the name shown is that of the image the Brush paints with (The image the Brush paints with), the current Layer's Terrain's last stroke's when no Asset is chosen, and the button that made the Terrain show the chosen Asset is removed.
- composing — **Stroke options follow the selection**: modified to name the selected stroke's image and offer its two buttons (Images of the selected stroke).
- composing — **Every band matches the reference**: extended by **Every Material at every band matches the reference**; the Terrain's coverage itself is still matched within 1/255.
- composing — **The base is resident**, **An overlay covers the view**, and **Overlay tiles are let go out of reach**: extended by **Materials hold only the tiles they reach**, each Material holding its own tiles at the base and at the active band where its strokes that paint reach, under the same bound each.
- composing — **Only the touched tiles recompute**: extended by **Only the touched tiles blend again**.
- composing — **Without a renderer, the base on the CPU**: modified so that the base on the CPU holds every Material's weights as well as the coverage.
- export — **The Export keeps the CPU rasterizer**: extended by **The Export blends by the same weights**.
- export — **Drawn as in the editor**: modified so that "a Terrain as its Material masked by its coverage" reads "a Terrain as its Materials blended by their weights (Drawn blended by weight)" and "a Missing Terrain image's placeholder colour masked by the coverage" reads "a Missing Material's placeholder colour at its weight".
- export — **Terrain at the Export's resolution**: modified so that each pixel shows a Terrain's coverage and every Material's weight at that pixel's centre as its strokes give them at the Export's pixels per cell (The Export blends by the same weights).
- projects — **Saved as its strokes**: replaced by **Saved with each stroke's image**.

## Implementation Decisions

The technology the architecture fixes (its Terrain blending bullet: one masked draw per Material, each with its own Shader, multiplied by an editable per-Material weight mask held in tiled textures, cost scaling with the Materials visible; its Painting bullet and the mask tile bands; the Project format's per-component versions) is used as written there and not restated. The decisions of the composing spec for Terrain, among them those of rasterizing on the GPU (bands, the tiles on the GPU, the stroke pipeline in a paint and an erase variant, the render-world pass, ApplyStroke's comparison from both ends, the base on the CPU without a renderer), are extended, not restated. Every Material of a Terrain today is the built-in masked tiled image, so a Material is told apart by the image it shows alone.

- **The Terrain component** in `model` no longer holds an image; each stroke holds its image's Asset Reference row under `image`, written for every stroke, an erase's too. The Terrain offers its Materials: the distinct rows its strokes that paint show, in ascending row order, which is the order the Project first recorded their images and the order they are drawn in. The component goes to version 3. As the component-migration guideline says, version two's shape is kept as private shapes of its own (a Terrain with an image and strokes without one), migrated into the current one with every stroke taking the Terrain's image; version one's migrates the same way with every stroke painting as well; `read` matches versions 1, 2, and the current one, a version above 3 is refused as newer, and the next save writes version 3. Which images a stroke shows is never malformed; a row the Asset Reference table lacks is treated as a Prop's is.
- **The Assets an Element shows**: the one query `model` offers for the Asset an Element shows yields every row it shows, one for a Prop or a Portal and each distinct row of a Terrain's strokes, erases included, so the Open report counts a Terrain once among the Elements using each of its images and the Editor's and RenderEngine's readers stay in one place.
- **Paint**: Apply carries a Paint as today. A stroke that paints naming an Asset resolves it as a Prop's placement does, whether or not the Layer has a Terrain, recording the Asset Reference and folder rows on the first application and never removing them; its step appends the stroke with that row, or spawns the Terrain holding it as today. A stroke that paints naming none takes the row of the Terrain's last stroke. A stroke that erases takes the row of the Terrain's last stroke, with no lookup of the image it names. The refusal of another image is removed.
- **Edit Element of a Terrain**: the Element change's Material of a Terrain becomes a replacement, carrying the row of the image to replace and the chosen Asset; and a change of one stroke gains its image set, carrying the chosen Asset. Both resolve the Asset as a placement does, recording its rows on the first application. Setting a stroke's image sets that field of the stroke through the generic field-setting command, as a single step, and records nothing when the row is the stroke's own. A replacement is a reversible step of its own that keeps the numbers of the strokes it changed and swaps their rows and back, and is one of the steps that close any gesture group left open; it records nothing when the two rows are the same, and is a CommandFailed naming the image when no stroke of the Terrain shows the row to replace. The routing of a Terrain's own changes stays the match that names every Element change.
- **PaintEngine's contract**: Rasterize stays one operation over strokes that paint or erase, unchanged. BlendWeights takes a Terrain's strokes in order, each with its Material (its image's row), a region in cells, and a number of pixels per cell, and gives the Terrain's coverage and, for each Material in drawing order, its mask: one byte a pixel saying how opaque that Material is laid over the Materials drawn before it and what lies below. A Material's claim is Rasterize of the strokes with every stroke that paints another Material taken as an erase, so claims use the paint and erase pixel rules unchanged. From the coverage byte *t*, the claim bytes *m*, their sum *s*, and the sum *p* of the claims of the Materials up to and including this one in drawing order, the mask byte is 255 × *t* × *m* ÷ (*s* × (255 − *t*) + *t* × *p*) rounded half up in integer arithmetic, and none where the divisor is zero. Laid in drawing order with the usual alpha blending, these masks give each Material exactly the weight *t* × *m* ÷ *s* and what lies below 1 − *t*, before the rounding of each mask; where one Material alone has a claim, its mask is the coverage itself, so a Terrain of one image is drawn from its coverage exactly as today. BlendWeights is a plain function over model types on the CPU, the reference for the GPU and the Export's path.
- **The cache**: the tiled cache of a Terrain keeps, for each tile key, the coverage tile and, for each Material whose strokes that paint reach the tile, its mask tile, and its claim tile where two or more Materials reach that tile; where one Material alone reaches a tile, its claim and its mask are the coverage, and no other tile is held for it. ApplyStroke's comparison from both ends counts a change of a stroke's image as a change of that stroke, so a replacement recomputes only the tiles the replaced strokes reach. On the GPU, after the coverage and the claims of a dirty tile are drawn with the paint and erase variants of the stroke pipeline, PaintEngine's render-world pass computes the masks in a pass per Material in drawing order, accumulating the sums of claims in floating-point scratch tiles, so each pass binds the same few textures whatever the number of Materials and no slot count exists. The per-tile job names each stroke reaching the tile once, with its Material, and the render world expands it into the passes, so the main thread's work does not grow with the Materials. The base and the overlays follow the composing spec's bands for each Material's tiles; without a renderer, the base on the CPU holds the coverage, the claims, and the masks as pixels.
- **The derived coverage** in `model` holds, beside its band, each Material's row and the tiles of its mask, in drawing order, each tile with its revision and either its pixels or its image on the GPU. It offers each Material's weight at a pixel, worked out from the masks in drawing order, which the seams read.
- **RenderEngine** draws, for each tile, one quad per Material with the masked tiled image Material unchanged, showing that Material's image and masked by its mask tile, alpha-blended, the quads of a tile spread within the Terrain's place on the camera's axis in drawing order, so they sort in that order and stay between the Elements below and above the Terrain. A Material whose image is loading, Missing, or failed is drawn in the placeholder's flat colour through the same mask. RenderRegion draws, for each Terrain of the region, one quad per Material in drawing order from the masks the Export hands it, sampled nearest.
- **The Export**: before it requests a tile, ProjectManager asks PaintEngine to BlendWeights every Terrain whose box meets the tile over the tile's region at the Export's pixels per cell, leaves out a Material whose mask covers nothing there, and hands the rest to RenderEngine with the RenderRegion request, as the export spec decides for the coverage today.
- **The Paint tool**: the name of the image the Brush paints with is the chosen Asset's or, with none, that of the last stroke of the current Layer's Terrain, found in the Editor's view of the Level through the Asset Reference table. With a stroke selected, the options strip shows its image's name after Paints and Erases and, when the chosen Asset is not where that image resolves on this device, two buttons: one sends an Edit Element setting the stroke's image to the chosen Asset, the other one replacing the stroke's image with it on the Terrain, each marked single. The button that set the Terrain's Material is removed. The development-only input script's `describe` step logs each stroke's image row and, for every Terrain, its Materials in drawing order with how many tiles each holds at the base and at the active band.

## Testing

- **Composing seam**: the headless App with no window and no RenderEngine, the base on the CPU, over the Terrain fixture's folder grown to twelve texture images, asserting on the Terrain component, each Material's weight at chosen cells read from the derived coverage, the coverage, whole tiles compared byte for byte, the answers, and the history. Every existing Terrain and stroke test stays as it is but those of the Rules this change replaces, which follow it.
  - **Each stroke has its Material**: `crates/drs-app/tests/materials.rs::each_stroke_has_its_material` (strokes of three images on one Terrain, each holding its row, the Terrain's Materials in the order their rows were recorded)
  - **No limit on Materials**: `crates/drs-app/tests/materials.rs::no_limit_on_materials` (twelve images painted side by side and over each other, each showing its weight)
  - **Painted with the image it names**: `crates/drs-app/tests/terrain.rs::painted_with_the_image_it_names` (another image appended to the same Terrain; no image taking the last stroke's; no image and no Terrain refused)
  - **An erase keeps the image before it**: `crates/drs-app/tests/strokes.rs::an_erase_keeps_the_image_before_it` (an erase naming another image and one naming none, both showing the last stroke's image, and the image they carry painted once turned to painting)
  - **Coverage takes every Material**: `crates/drs-app/tests/materials.rs::coverage_takes_every_material` (the coverage of strokes of three images equal to that of the same strokes all of one image)
  - **A Material's claim** and **Weights share the coverage**: `crates/drs-app/tests/materials.rs::weights_share_the_coverage` (at chosen cells, each weight equal to the coverage times its claim over the sum of claims, worked out from known stroke coverages, and the weights summing to the coverage)
  - **A later Material takes its share**: `crates/drs-app/tests/materials.rs::a_later_material_takes_its_share` (a full-strength stroke's middle, its soft edge at a known coverage, a half-strength stroke, and the earlier Material painted back over the later)
  - **Painting the same Material again changes nothing**: `crates/drs-app/tests/materials.rs::painting_the_same_material_again_changes_nothing` (a stroke laid twice, a weaker stroke over a stronger, and a sharp joint and a self-crossing over another Material)
  - **One Material is as before**: `crates/drs-app/tests/materials.rs::one_material_is_as_before` (every tile of a Terrain of one image the same mask as its coverage, and the same as the coverage before this change)
  - **An erase takes from every Material**: `crates/drs-app/tests/materials.rs::an_erase_takes_from_every_material` (a full and a half-strength erase over three blended images, and a stroke painted after it showing alone)
  - **Weights are the strokes alone**: `crates/drs-app/tests/materials.rs::weights_are_the_strokes_alone` (strokes of four images painted, erased, edited, re-imaged, replaced, removed, undone, and redone giving the same tiles as their final strokes painted afresh and as the same strokes saved and opened; every tile as before after each undo)
  - **A stroke's image stays editable**: `crates/drs-app/tests/materials.rs::a_strokes_image_stays_editable` (one step, undone and redone, the other strokes untouched, the stroke's own image recording no step)
  - **An image can be replaced**: `crates/drs-app/tests/materials.rs::an_image_can_be_replaced` (every stroke of it changed, an erase among them, the rest untouched; merging into an image already shown; the same image recording no step; an image no stroke shows refused)
  - **An image change records a reference**: `crates/drs-app/tests/materials.rs::an_image_change_records_a_reference`; `crates/drs-app/tests/terrain.rs::painting_records_a_reference` (a second image painted onto an existing Terrain recording its row)
  - **Terrain changes only its Material and its strokes**, as modified: `crates/drs-app/tests/terrain.rs::terrain_changes_only_its_material_and_strokes` (a replacement and a stroke's image sent for a Prop refused)
- **Offscreen seam**: the headless editor of the export and mask tile tests, with RenderEngine and PaintEngine's plugin, over the export fixture's solid-colour images. These tests draw on the GPU and fail, rather than skip, where no adapter exists.
  - **Every Material at every band matches the reference**: `crates/drs-app/tests/mask_tiles.rs::every_material_matches_the_reference` (Terrains of two and of five Materials, soft and hard strokes crossing each other's Materials and a half-strength erase, read back at each of the four bands, every Material's weight at every pixel within 3/255 of BlendWeights on the CPU, and the coverage within 1/255)
  - **Materials hold only the tiles they reach**: `crates/drs-app/tests/mask_tiles.rs::materials_hold_only_the_tiles_they_reach` (a Material painted far from the view holding base tiles and no overlay tile; a pan bringing its tiles in; each Material's tiles within the bound)
  - **Only the touched tiles blend again**: `crates/drs-app/tests/mask_tiles.rs::only_the_touched_tiles_blend_again` (a stroke of a new image, a stroke's image changed, a replacement, and their undo changing the revisions of exactly the tiles their reach meets, for every Material)
  - **Drawn blended by weight** and **One Material is as before**: `crates/drs-app/tests/export.rs::two_materials_blend` (one solid colour alone, another over it in a full-strength stroke's middle, in its soft edge, and at half strength: each pixel within 2/255 of the two colours mixed by the weights BlendWeights gives, as the drawing mixes them, with no background where the coverage is full), `crates/drs-app/tests/export.rs::five_materials_blend` (five solid colours overlapping with soft strokes and an erase: each chosen pixel within 5/255 of the reference blend), the existing Terrain tests of the export file passing unchanged; the viewport's drawing is checked by hand
  - **Every Material tiled at natural size**: `crates/drs-app/tests/export.rs::every_material_is_tiled_at_natural_size` (two four-quartered textures of different sizes, each quarter's colour where its repetitions from the origin put it)
  - **A missing Material keeps its share**: `crates/drs-app/tests/export.rs::a_missing_material_keeps_its_share` (one of two images Missing and one undecodable: the placeholder's colour at its weight, the other image beside it)
  - **The Export blends by the same weights**: `crates/drs-app/tests/export.rs::the_export_blends_by_the_same_weights` (a hard stroke of one image ending inside another exported at 200 pixels per cell, the pixels either side of its radius; two tile sizes and two views byte-identical)
  - **A stroke's image stays editable** and **An image can be replaced**, in the image: `crates/drs-app/tests/export.rs::changed_images_export_as_changed`
- **Projects seam**: the existing App over temporary directories.
  - **Saved with each stroke's image**: `crates/drs-app/tests/projects.rs::terrain_is_saved_as_its_strokes` (strokes of two images, an erase among them, written at version 3 with each stroke's row, reopened with the same weights), `crates/drs-app/tests/projects.rs::a_terrain_before_blending_opens` (a Terrain at version 2 opening with every stroke showing its image and the same coverage, then saved at version 3), `crates/drs-app/tests/projects.rs::an_older_terrain_opens` (version 1 opening with every stroke painting and showing the Terrain's image, saved at version 3), `crates/drs-app/tests/projects.rs::a_newer_file_is_refused` (a Terrain at version 4), `crates/drs-app/tests/projects.rs::a_malformed_terrain_is_refused` (every malformed stroke at versions 1, 2, and 3), `crates/drs-app/tests/projects.rs::unknown_terrain_round_trips` (strokes of two images)
  - **A missing Material keeps its share**, in the report: `crates/drs-app/tests/projects.rs::missing_assets_stay` (a Terrain of two images, one Missing, staying with every stroke and counted once among the Elements using the Missing one)
- **Timed budget**: PaintEngine's main-thread work timed through its contract over a generated Terrain, following the timed-budget guideline, in the `timed` group.
  - **Many Materials cost the main thread little**: `crates/drs-paint-engine/tests/budget.rs::many_materials_cost_the_main_thread_little`
- **By hand**: **The image the Brush paints with**, **Images of the selected stroke**, and the viewport's half of **Drawn blended by weight** and **A missing Material keeps its share**, the egui UI and the viewport having no headless seam; verified by driving the editor with the development-only input script: painting three images over each other, painting with no Asset chosen, selecting strokes and using both buttons, undoing, and zooming over the blend, while `describe` logs the strokes' images and each Material's tiles; the Edit Elements the buttons send are covered by `crates/drs-app/tests/materials.rs::a_strokes_image_stays_editable` and `crates/drs-app/tests/materials.rs::an_image_can_be_replaced`.
- PaintEngine's own unit tests check what the Rules rest on, as functions of the Engine alone, and are the coverage of no Rule: the mask formula against the weights it lays (masks laid in order giving *t* × *m* ÷ *s*), a lone Material's mask equal to the coverage, the claims never summing below the coverage, BlendWeights on a Terrain of one image equal to Rasterize, and the cache's masks after appending, editing, re-imaging, and replacing equal to BlendWeights over the same strokes.

## Out of Scope

- Materials other than the built-in masked tiled image: author Shaders, a Material's parameters, and a Material Asset chosen for a stroke (the Richer maps milestone's custom Materials and Shaders); a Material is told apart by its image until then.
- A panel listing a Terrain's Materials, ordering them, hiding one, or giving one an opacity of its own.
- Painting an image underneath the others, height- or texture-driven blending, and blend modes between Materials.
- Replacing an image without first selecting a stroke that shows it.
- Spreading a band switch's recompute over several frames, and freeing the tiles of Terrains on hidden Layers or other Levels: still the architecture's Accepted gaps, inside PaintEngine.
- Measuring the GPU time of the weights passes automatically, and on platforms other than the one the architecture measured.

## Further Notes

- **Architecture**: the Terrain blending bullet states BlendWeights' shape (one mask per Material, the opacity it is laid with over the Materials drawn before it), the cache per Terrain with per-Material masks and claims, the Export call chain asking PaintEngine to BlendWeights, and the memory where Materials meet; the per-Material GPU passes are unmeasured until this change builds them.
- "Weight" and "claim", like "coverage", are defined by the Rules themselves (A Material's claim, Weights share the coverage) and are not in the glossary.
- With the sum of claims dividing the coverage, a stroke laid over ground where two or more other Materials already meet, or over ground not fully covered, shows somewhat less than its coverage there; laid over ground fully covered by one other Material, it shows exactly its coverage (A later Material takes its share).
- An image with transparency of its own lets through what its Material is laid over: the Materials drawn before it, in the order the Project first recorded their images, and what lies below the Terrain.
- The order Materials are drawn in changes the masks but not the weights, so it is invisible for images without transparency; it follows the rows of the Asset Reference table, which every device holds the same.
- An erase carries an image only so that turning it to painting has one to paint with; it shows nothing, and its image is counted among those its Terrain uses.
- A Terrain whose strokes that paint have all been turned to erasing blends no Material and shows nothing until a stroke paints again.
- The Export rasterizes the coverage and the claims of every Material meeting each tile on the CPU, so a painted Export of blended Terrain takes longer than one of a single image, roughly once more per Material meeting there.
