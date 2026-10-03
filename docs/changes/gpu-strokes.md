# Strokes rasterize on the GPU

**Capabilities**:
- composing: Paint, Edit Element
- export: none of its Commands; the Export keeps computing Terrain on the CPU, whatever the editor shows

## Problem Statement

The editor computes a Terrain's coverage on the CPU at 32 pixels per cell and shows that, magnified, at every zoom. Close up, a hard-edged stroke turns into a blurred ramp several screen pixels wide, so the Author cannot judge a crisp edge where it matters most. And every change pays the CPU's price: on a Terrain of a couple of hundred strokes, undoing one, removing one, or dragging one with Edit strokes recomputes every stroke over the tiles it touches, tens to hundreds of milliseconds a frame, so the editor stutters exactly when the Author is shaping ground. The functional baseline stutters on busy Layers too; this product promises to be fast.

## Solution

Painting stays smooth and sharp at any zoom. The editor rasterizes strokes on the GPU into the mask tiles a Terrain is drawn from, so a new stroke, an erase, an edit, an undo, and a dragged stroke show the next frame at a cost the Author never notices. The ground is kept at a coarse resolution for the whole Level, always ready for an overview, and at the resolution the zoom calls for wherever the Author is looking, switching between a few fixed resolutions only when the zoom has moved well past one, so zooming and panning never stutter and a pinch that wobbles never flickers. Only the tiles a change touches are recomputed, and tiles far out of view are let go, so memory stays bounded however far the Author pans. What the GPU computes is the same ground the CPU computes, to the last level of a byte, and the Export keeps the CPU's exact computation at its own resolution, so the image is the same whatever zoom the Author worked at.

## User Stories

### Painting close up

1. As an Author, I can paint at any zoom and see the stroke's ground in the frame after I release it, so that painting close up is as immediate as painting from afar.
2. As an Author, I can erase at any zoom and see the ground gone in the frame after I release, so that erasing close up is as immediate as painting.
3. As an Author, I can zoom in on a hard-edged stroke and see its edge as crisp as the zoom allows, at most a pixel or two of ramp on screen, so that I can judge a sharp edge where I look closely.
4. As an Author, I can see a soft stroke's edge fade as smoothly at every zoom as the Brush says, so that zooming never makes soft ground look stepped.
5. As an Author, I can rely on an erase showing no build-up at its joints, crossings, and overlaps at every zoom, so that an erase looks as even close up as from afar.
6. As an Author, I can paint across and beyond the Bounds at any zoom and see the ground there as anywhere else, so that the Bounds never get in the way of composing.

### Editing dense Terrain

7. As an Author, I can drag a stroke of a Terrain holding hundreds of strokes and see its ground follow the pointer every frame without the editor stuttering, so that reshaping ground feels direct.
8. As an Author, I can drag a point of a stroke, change a stroke's Brush settings, turn it to erasing, or remove it on dense Terrain without a pause, so that fixing an old stroke costs no waiting.
9. As an Author, I can undo and redo strokes and stroke edits on dense Terrain without a pause, so that stepping through history stays quick however much I painted.
10. As an Author, I can rely on a change to one stroke recomputing only the ground near that stroke, so that editing stays quick however large the Terrain grows.
11. As an Author, I can rely on the ground looking exactly as it did before a change once I undo it, at every zoom, so that undo never leaves a trace close up either.

### Zooming and panning

12. As an Author, I can zoom in and out over painted ground without the editor stuttering at every step, so that looking closer never interrupts my work.
13. As an Author, I can see the ground sharpen in a few steps as I zoom in, rather than recomputed at every step, so that zooming costs only what the sharper view needs.
14. As an Author, I can wobble the zoom slightly, as a trackpad pinch does, without the ground switching back and forth between resolutions, so that the ground never flickers under my fingers.
15. As an Author, I can pan at a high zoom and see the ground entering the view already drawn, so that I never see a gap where ground should be.
16. As an Author, I can zoom out to the whole Level and see all of its ground at once, without waiting, so that an overview is always one gesture away.
17. As an Author, I can rely on the view never showing a Terrain missing, half drawn, or drawn twice over while I zoom or pan, so that what I see is always the whole ground.
18. As an Author, I can pan back and forth over the same place at a high zoom without the editor recomputing the ground it showed a moment ago, so that looking around stays cheap.

### Memory

19. As an Author, I can pan across a large Level at a high zoom for as long as I like without the editor's memory growing, so that a long session over a big map stays as light as a short one.
20. As an Author, I can rely on the sharp ground of one zoom being let go when I zoom to another, so that zooming in and out never piles up memory.

### The same ground everywhere

21. As an Author, I can rely on the ground shown at every zoom being the ground my strokes say, to within a level no eye can see, so that no zoom shows a stroke, an erase, or an edge the others do not.
22. As an Author sharing a Project with a collaborator on a different computer and graphics card, I can rely on their editor showing the same ground mine shows, to within that same invisible level, so that we discuss the same map.
23. As an Author, I can open a Project with much Terrain and see all of it drawn without a long pause, so that opening a painted map is as quick as opening a bare one.

### Export

24. As an Author, I can export a Level and get the same image whatever zoom I was working at and whatever the editor was showing, so that the image is the map, not my screen.
25. As an Author, I can export at a resolution finer than anything the editor shows and see the edges of painted ground as sharp as that resolution allows, so that a printed map's ground is crisp.
26. As an Author sharing a Project with a collaborator on a different computer, I can rely on the same Level exporting to the same image on both, so that a re-export by either of us changes only what we changed.

## Rules

A Terrain's coverage (Shaped by a soft round Brush, Strokes composite by the strongest, Erasing caps what remains) is shown in the editor from square tiles of 512 by 512 pixels at a band: a number of pixels per Grid cell, one of 32, 64, 128, and 256. The band of 32 is the base; the others are overlays. The view is the rectangle of cells the Viewport shows: its area around its centre at its zoom.

### Bands

**The band follows the zoom**: a Terrain's coverage is shown at one band at a time, its active band: the floor band of the zoom when its coverage is first derived, the floor band being the largest of 64, 128, and 256 not above the zoom, or the base when the zoom is below 64; at zooms above 512 the band of 256 is shown magnified more than twice, up to four times at the highest zoom.

**The band holds through a wobble**: once a band is active it stays active while the zoom is at least 0.9 times the band, the base excepted, and at most 2.2 times the band, the band of 256 excepted; when the zoom leaves that range the active band becomes the floor band of the zoom.
_Why_: a pinch wobbles the zoom by several per cent, and without this margin the band would switch back and forth every few frames, recomputing the whole view each time.

**The base is resident**: a Terrain holds its base tiles wherever its strokes reach, over the whole Level and beyond the Bounds, whatever band is active, and brings them up to its strokes with every change, so the base is ready the moment the zoom drops back to it.

**An overlay covers the view**: while an overlay band is active, a Terrain holds that band's tiles that meet the view and that some stroke's reach meets, each holding every stroke, and rasterizes a tile the moment it comes to meet the view.

**Overlay tiles are let go out of reach**: an overlay tile is dropped once no part of it lies within one tile's width of the view, and every tile of an overlay band is dropped when another band becomes active; so a Terrain never holds more overlay tiles than (⌈*W* ÷ 460.8⌉ + 3) × (⌈*H* ÷ 460.8⌉ + 3) for a view *W* by *H* screen pixels in area, 48 tiles (12 MiB) for a view of 1920 by 1080.

### The same ground

**Every band matches the reference**: every pixel of every tile the editor rasterizes on the GPU, at every band, holds the value PaintEngine's CPU rasterization gives at that pixel for the Terrain's strokes over that tile's region at that band, to within 1/255, for paint and erase alike, at a joint, a self-crossing, and an overlap of erases as on a straight stretch. Follows from: Coverage is the strokes alone.

**Only the touched tiles recompute**: a Paint rasterizes onto what they hold only the tiles of the base and of the active band that its stroke's reach meets; an edit, a removal, an undo, or a redo of strokes rasterizes afresh only the tiles of the base and of the active band that the reach of a changed stroke meets, as it was or as it is; every other tile is left as it is and keeps its revision.

**A change shows in its frame**: the tiles a Paint, a stroke edit, an undo, a redo, an opened Project, a band switch, or a pan touches are rasterized before anything draws the frame in which AuthoringManager derives the change, so a stroke shows in the frame after its release and a dragged stroke's ground follows every frame of the drag.

**No hole while zooming or panning**: in every frame, the Terrain is drawn from its active band alone, every tile drawn holding every stroke, so the view never shows the Terrain missing, partly drawn, drawn from two bands at once, or from tiles not yet rasterized.

**Without a renderer, the base on the CPU**: an editor running without rendering derives each Terrain's coverage at the base band alone, rasterized on the CPU, its tiles holding their pixels, whatever the zoom.

### Cost

**Painting costs the main thread little**: on a Terrain of 200 strokes of 40 segments each over a Level of 60 by 60 cells, one in five erasing, with a view of 4096 by 4096 screen pixels at a zoom of 256, PaintEngine's main-thread work to bring the tiles up to one new stroke takes under 1 ms, and up to one moved stroke under 2 ms, in the build the Author runs.

**Zooming and panning cost the main thread little**: on that Terrain and view, PaintEngine's main-thread work for a switch from the base to the band of 256 takes under 4 ms, and for a pan of 512 screen pixels at that band under 2 ms, in the build the Author runs.

### Export

**The Export keeps the CPU rasterizer**: the Export computes every Terrain's coverage with PaintEngine's CPU rasterization at the Export's resolution (Terrain at the Export's resolution) and never reads the editor's tiles, so two Exports of the same Level at the same resolution are byte-identical whatever band and view the editor shows. Follows from: Same Level, same image.

## Implementation Decisions

The technology the architecture fixes (its Painting bullet: GPU rasterization inside PaintEngine into 512-pixel R8 mask tiles that are render-world images, a pass per dirty tile that clears it and draws one instanced capsule quad per segment with a maximum blend for paint and a minimum blend for erase, runs of same-kind strokes sharing a draw; the tiled cache keyed by band and tile in the Level's pixel plane with a resident base and view-following overlays, the floor rule and its hysteresis; the CPU rasterizer as the golden reference and the Export's path; the cache opaque to all but PaintEngine and held by AuthoringManager; RenderEngine drawing the masked Material over the active band's tiles unchanged; and its Accepted gaps) is used as written there and not restated. The composing spec's decisions for Terrain, and those of the erase-and-reshape change (an erase as the minimum, Rasterize compositing in order, ApplyStroke's comparison of the strokes from both ends), are extended, not restated; every Rule of that change holds at every band.

- **Rasterize** stays one operation: strokes in order, a region in cells, a number of pixels per cell, and a target. The target is either a buffer on the CPU, exactly as today, which serves the Export's tiles and the cache without a renderer, or a GPU tile, whose pixels PaintEngine's render-world pass draws; both follow the same pixel rules (a pixel's value taken at its centre from its whole-pixel index in the Level's pixel plane, row 0 at the tile's top), so a GPU tile matches the CPU result within the rounding of the R8 format.
- **PaintEngine's render plugin**: PaintEngine gains a plugin the Host registers. In the render app it owns the stroke pipeline in a paint and an erase variant, its Shader a Bundled File of plain WGSL with a fixed handle, as the built-in Material guideline adds a Shader, so the `ci` tool's naga check validates it; the per-frame job extracted from the main world (each dirty tile, whether it is cleared or appended to, and the strokes reaching it in order with their segments); and one system in the render graph's Begin set, which runs before any camera renders, encoding one pass per dirty tile through the render context. Ordering by that set needs only `bevy_render`, so PaintEngine needs no `bevy_core_pipeline`. A job that finds its pipeline not compiled yet is kept for the next frame, never dropped, so the tiles are complete once the pipeline is ready. The plugin uses no events, messages, or observers.
- **The tiles on the GPU** are R8 images created for the render world alone, with no copy kept on the CPU, usable as a render attachment and as a copy source (so a test can read one back), sampled linearly. They are created when a tile is first needed and freed when the cache drops them, with the Terrain, or when the band they belong to stops being active.
- **ApplyStroke** brings a Terrain's cache up to its strokes and to the view, and says whether the tiles a reader sees changed. Beside the strokes it takes the Viewport's zoom and view and whether tiles are rasterized on the GPU or the CPU. Within the base and the active band it applies the erase-and-reshape change's comparison from both ends; it chooses the active band by the floor rule and the hysteresis, remembering the band in the cache, so each Terrain keeps its own; on a switch it drops the previous overlay band and queues every tile of the new one that meets the view; on a pan it queues the tiles coming to meet the view and drops those out of reach; and it hands the GPU work to PaintEngine's render plugin through a PaintEngine system parameter the Manager's deriving system takes, never through an event. The base on the GPU is a whole-Level band like any other, rasterized on the GPU when a renderer exists, so dragging a stroke on dense Terrain costs GPU time at every zoom and no CPU rasterization at all. A GPU tile's revision changes whenever it is rasterized afresh, since its pixels never come back to the CPU to be compared; a CPU tile keeps the erase-and-reshape change's rule of changing only when its pixels do.
- **Without a renderer**: PaintEngine's plugin says, in the main world, whether a render app exists; without one (a headless editor without Bevy's render plugins, as the composing seam is), ApplyStroke keeps the base band alone on the CPU, as the composing spec decides today, and ignores the view.
- **The derived coverage** in `model` gains the band its tiles are at and, per tile, its revision and either its pixels, when rasterized on the CPU, or its image on the GPU, when rasterized there; it holds the tiles of the Terrain's active band that the cache holds: every base tile while the base is active, the overlay's tiles while one is. A tile's corner and extent in cells follow from its key and band. AuthoringManager's deriving system runs, as today, for every Terrain whose component changed, and also for every Terrain when the Viewport's zoom, centre, or area changed, and publishes only when ApplyStroke says something a reader sees changed.
- **RenderEngine** draws the published tiles with the masked tiled image Material unchanged, one quad per tile of 512 ÷ band cells a side at the tile's corner, on the viewport's render layer at the Terrain's place in the stacking order: a GPU tile's image is bound as it is, with no upload, and a tile rasterized afresh needs no new bind group, its image being the same; a CPU tile is uploaded when its revision changes, as today; a tile no longer published loses its quad and its Material. The offscreen camera of RenderRegion keeps drawing the coverages the Export hands it, never the published tiles.
- **The Export** is unchanged: ProjectManager asks PaintEngine to Rasterize each Terrain over each export tile at the Export's resolution into a CPU buffer, as the export spec decides.
- **The development-only input script**: its `describe` step logs, for every Terrain, its active band and how many tiles it holds at the base and at the active band, so a script can check a switch, a pan, and the tiles let go.
- **Crates**: PaintEngine uses `bevy_render`, `bevy_image`, and `bevy_shader` from the Restricted external dependencies table and the narrow ECS crates; the Shader's fixed handle, the tile images, and binding them by identity need `bevy_asset` as well (Further Notes).

## Testing

- **Offscreen seam**: the export tests' headless editor with RenderEngine and PaintEngine's render plugin under Bevy's default plugins without a window, its builder moved into the shared test support so a second file uses it, driven by Apply, Undo, and Redo messages and by writing the Viewport's zoom, centre, and area as the Editor would, and asserting on the published coverage (its band, its tiles' keys and revisions) and on GPU tiles read back from the GPU and compared pixel by pixel with PaintEngine's CPU Rasterize of the Terrain's strokes over the same region at the same band. These tests draw on the GPU and fail, rather than skip, where no adapter exists.
  - **The band follows the zoom**: `crates/drs-app/tests/mask_tiles.rs::the_band_follows_the_zoom` (a fresh Terrain at zooms of 20, 40, 64, 100, 128, 300, and 1024 taking the bands 32, 32, 64, 64, 128, 256, and 256)
  - **The band holds through a wobble**: `crates/drs-app/tests/mask_tiles.rs::the_band_holds_through_a_wobble` (a sequence of zooms: from 64 down to 58 keeping 64 and to 57 taking 32, up to 70 keeping 32 and to 71 taking 64, up to 140 keeping 64 and to 141 taking 128, and ± 6 % around 64 for many frames switching never)
  - **The base is resident**: `crates/drs-app/tests/mask_tiles.rs::the_base_is_resident` (a stroke painted outside the view at a zoom of 256, then the zoom dropped to 40: the base's tiles hold the stroke and match the reference, and none was rasterized by the drop)
  - **An overlay covers the view**: `crates/drs-app/tests/mask_tiles.rs::an_overlay_covers_the_view` (the overlay's tiles exactly those meeting the view that some stroke reaches, including one at negative cells; a pan bringing new tiles in, each matching the reference in its first frame)
  - **Overlay tiles are let go out of reach**: `crates/drs-app/tests/mask_tiles.rs::overlay_tiles_are_let_go_out_of_reach` (a pan of half a tile keeping every tile, a pan far away dropping the old ones, a long pan over painted ground never holding more than the bound for the view's area, and a band switch dropping the previous band)
  - **Every band matches the reference**: `crates/drs-app/tests/mask_tiles.rs::every_band_matches_the_reference` (soft and hard strokes, a one-point dab, a half-strength erase with a sharp joint and a self-crossing, two overlapping erases, and a stroke beyond the Bounds, read back at each of the four bands: every pixel within 1/255), `crates/drs-app/tests/mask_tiles.rs::edited_tiles_match_the_reference` (after a point moved, a stroke moved, an erase turned to painting, a stroke removed, and each undone and redone, at the base and at an overlay)
  - **Only the touched tiles recompute**: `crates/drs-app/tests/mask_tiles.rs::only_the_touched_tiles_recompute` (a new stroke, a moved stroke, and its undo changing the revisions of exactly the tiles their reach meets, at the base and at the active band)
  - **The Export keeps the CPU rasterizer**: `crates/drs-app/tests/export.rs::the_export_keeps_the_cpu_rasterizer` (the same Level exported with the view at the base and at the band of 256, byte-identical)
- **Composing seam**: the existing headless App with no window and no RenderEngine. Every existing Terrain and stroke test stays as it is and keeps passing, since without a renderer the coverage is the base on the CPU as today.
  - **Without a renderer, the base on the CPU**: `crates/drs-app/tests/terrain.rs::without_a_renderer_the_base_is_on_the_cpu` (the zoom set to 256: the coverage at the base band, its tiles holding pixels equal to before)
- **Timed budget**: PaintEngine's main-thread work timed through its contract over a generated Terrain of the architecture's measured shape, following the timed-budget guideline (PaintEngine at the `dev` build's optimisation in the `fast` profile, named in the architecture's Profiles bullet; the tests in the `timed` group). GPU time is not measured automatically: Metal's render-pass timestamps are unusable and Bevy waits for no GPU work, so a headless frame's time leaves the GPU's out; the architecture's measured GPU figures stand for it.
  - **Painting costs the main thread little**: `crates/drs-paint-engine/tests/budget.rs::painting_costs_the_main_thread_little`
  - **Zooming and panning cost the main thread little**: `crates/drs-paint-engine/tests/budget.rs::zooming_and_panning_cost_the_main_thread_little`
- **By hand**: **A change shows in its frame** and **No hole while zooming or panning**, the viewport's drawing having no headless seam (the accepted deviation of the composing spec); verified by driving the editor with the development-only input script over a Terrain of a few hundred strokes at zooms of 40, 100, and 300, painting, erasing, dragging a stroke, undoing, zooming in steps and with a wobble, and panning, while the development build logs frame times and the `describe` step logs each Terrain's band and tiles; the half of A change shows in its frame that the model holds, the touched tiles' new revisions in the frame of the change, is covered by `crates/drs-app/tests/mask_tiles.rs::only_the_touched_tiles_recompute`.
- PaintEngine's own unit tests check what the Rules rest on, as functions of the Engine alone, and are the coverage of no Rule: the floor band and the hysteresis as a function of the zoom and the band before, the tiles meeting a view at a band, the tiles out of reach, and the job of an edit naming exactly the tiles its old and new reach meet with the strokes reaching each in order.

## Out of Scope

- More than one Material on a Terrain, blend weights, and the per-Material cost of the bands: the next change.
- Spreading a band switch's recompute over several frames while the base stands in magnified, and freeing the tiles of Terrains that are hidden or on another Level: the architecture's Accepted gaps leave these for later, inside PaintEngine.
- Bands of 8 and 16 pixels per cell for the lowest zooms, and mipmapped tiles: the base is shown minified there.
- Rasterizing the Export on the GPU.
- Bands chosen by the screen's physical pixels rather than the Viewport's logical zoom.
- Measuring GPU time in automated tests, and on platforms other than the one the architecture measured.

## Further Notes

- **Architecture**: PaintEngine uses `bevy_asset` for its stroke shader's fixed handle and its GPU tile images, as the Restricted external dependencies table allows; `model` names each GPU tile by a plain identity that RenderEngine maps to the tile's image, so `model` stays on the narrow crates.
- `bevy_core_pipeline` is not needed: the render graph's sets come from `bevy_render`, and Begin runs before the set the cameras render in.
- The band and the tile are the editor's presentation of coverage, defined by the Rules themselves as coverage is by the Terrain Rules, and are not domain terms.
- Each Terrain remembers its own active band, so a Terrain painted first, or restored by undo, while the zoom lies within the hysteresis margin of another band starts at the floor band of the zoom, and may show a different band from the Terrains around it until the zoom next leaves the margin; with one Terrain per Layer and one Layer today, this cannot yet be seen.
- Until the stroke pipeline has compiled, in the first frames after the editor starts, the GPU tiles are empty and Terrain is not drawn; on platforms that compile pipelines in the background this may last a few frames.
- A band switch rasterizes every tile of the new band in the view in one frame: about 6–8 ms of GPU work for a 4096-pixel view on the measured machine, a single frame's hitch at most, once per switch.
- Hard-edged strokes still show a ramp of about three screen pixels when the active band is magnified twice, as the architecture measured; at zooms above 512 the band of 256 is magnified up to four times.
