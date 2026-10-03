# Export

**Commands**: Export Level

## Purpose

A map is made to be put in front of players, at the table or on a virtual tabletop. This capability turns the Level into a PNG that covers exactly the Bounds at a resolution the Author chooses in pixels per Grid cell, drawn as the editor draws it and the same every time, well beyond the resolution the functional baseline allows.

## User Stories

1. As an Author, I can export the Level as a PNG image, so that I can take it to a virtual tabletop or a printer.
2. As an Author, I can choose the resolution in pixels per Grid cell from common choices or type my own, so that the image fits the tabletop I use.
3. As an Author, I am proposed a sensible resolution, so that the common case is one confirmation.
4. As an Author, I can see the size in pixels the image will have before I export, so that I know what I am about to produce.
5. As an Author, I can rely on the image covering exactly the Bounds, so that what I export is predictable and lines up with the Grid.
6. As an Author, I can rely on Elements outside the Bounds being left out and those straddling the edge being cut at it, so that the image is exactly the Bounds and nothing else.
7. As an Author, I can export well above 300 pixels per cell, so that a printed map is as sharp as my Assets allow.
8. As an Author, I am refused a resolution outside what the editor supports before anything is written, with the limits named, so that I never wait for an export that cannot finish.
9. As an Author, I can rely on the export looking the same whatever the zoom or size of my window, so that the image is the map, not my screen.
10. As an Author, I can rely on exporting leaving my view, my selection, and my Project as they were, so that exporting is a side step.
11. As an Author, I am told why an export to a read-only or vanished location failed, with no partial file left, so that a bad location costs me a retry.
12. As an Author, I am told before exporting when the Level has Missing Assets or Elements of an unknown kind, so that I can decide whether placeholders in the image are acceptable.
13. As an Author, I can rely on the exported image showing every Prop, Portal, Wall, Room, and Terrain as the editor shows it, a Wall at its thickness and colour, a Room's floor under its Walls, a Portal turned and mirrored as in the editor and each Wall and each Room's Walls left out along its Portals, a Terrain's painted ground under the Props and Walls above it, with every erase and every edited stroke as the editor shows them, in the same stacking order, so that the doors and the ground I see are the doors and the ground I export.
14. As an Author, I can export at a high resolution and see the edges of painted ground as sharp as that resolution allows, never blown up from what the screen showed, so that a printed map's ground is crisp.
15. As an Author, I can rely on a Portal whose Asset is Missing being exported as a placeholder still standing in its gap, and a Terrain whose image is Missing as its painted shape in the placeholder's colour, so that the doorway and the floor show even where their images do not.
16. As an Author, I can rely on the Export showing none of the editor's own marks, such as the selection outline, the Portal tool's marker, or a selected stroke's band and handles, so that I need not clear the selection before exporting.
17. As an Author, I can rely on the export waiting until every image has loaded, so that nothing is exported half-drawn.
18. As an Author, I can export the same Level twice at the same resolution and get the same file, so that I can trust a re-export to change only what I changed.
19. As an Author, I can rely on the image having no visible seams, so that a large export is one picture.
20. As an Author, I am proposed a file name from the Project's and Level's names, so that my exports are findable.
21. As an Author, I can rely on the exported image having an opaque background, so that it displays the same in every tabletop and printer.

## Rules

**Bounds to start with**: a new Project's Bounds are thirty by thirty cells with their lower-left corner at the Level's origin.

**Exactly the Bounds**: the Export is as many pixels wide as the Bounds' width in cells times the resolution, and as many high as the height times the resolution, and shows exactly the Bounds. Follows from: Bounds only decide what is exported.

**Resolution in pixels per cell**: the Author chooses a whole number of pixels per Grid cell, offered as 50, 100, 200, and 300 or typed, with 100 proposed.

**Resolution within limits**: a resolution below 1 or above 1024 pixels per cell is refused before anything is written, with the limits named.

**Clipped at the edge**: an Element wholly outside the Bounds appears nowhere in the Export; an Element straddling the edge appears only where it lies inside.

**Drawn as in the editor**: every Element on the Level appears in the Export as the editor draws it, at its position and size in cells scaled to the resolution, a Prop as its image, a Portal as its image turned and mirrored, a Wall as its stroke left out along the stretches its Portals cover, with no cap where a stretch reaches an end of the Wall, a Room as its floor under its Walls, the Walls left out along the stretches its Portals cover, and a Terrain as its Material masked by its coverage, its erases and its edited strokes as they stand, under the Elements above it, its coverage computed at the Export's resolution (Terrain at the Export's resolution), in stacking order, placeholders included, a Missing Portal's placeholder turned, set into its Wall, and keeping its gap, and a Missing Terrain image's placeholder colour masked by the coverage. Follows from: Stacking order.

**Terrain at the Export's resolution**: each pixel of the Export shows a Terrain's coverage at that pixel's centre as its strokes give it at the Export's pixels per cell, never enlarged from what the editor shows, and the same in every tile the image is assembled from. Follows from: Nothing is fixed at creation.

**Only the Level is exported**: the Export shows nothing the editor draws over the Level: no selection outline, handle, guide line, Wall or Room preview, Portal marker, Brush circle, stroke band, or selected stroke.

**Opaque background**: every pixel no Element covers is opaque black, and the Export holds no transparent or translucent pixel.

**Exported as PNG**: the Export is written as a PNG file at the path the Author chose, with `.png` added when the name lacks it.

**Placeholders are warned about**: when the Level has Elements using Missing Assets or of an unknown kind, the Author is told how many before the Export starts, and those Elements are exported as their placeholders.

**Export waits for Assets**: the Export is captured only once every Asset the Level uses has loaded or failed to load.

**Same Level, same image**: two Exports of the same Level at the same resolution are byte-identical.

**Tiles leave no seams**: the Export is the same image whatever the size of the tiles it is assembled from.

**A failed export is reported**: when the Export cannot be written, the Author is told the reason and no partial file remains.

**Export changes nothing**: an Export changes no Element, the selection, or the view, and the Project has no more unsaved changes after it than before.

**Export name proposed**: the save dialog proposes the Project's name, a dash, and the Level's name as the file name.

## Implementation Decisions

The technology the architecture fixes (the tiled export through OutputAccess, deterministic output, the CPU rasterizer as the golden reference for the Export) is used as written there and not restated.

- **The dialog**: the Editor shows a modal Export dialog naming the Level and the Bounds in cells, with the four resolutions as one click each and a field the Author types into, which is not clamped: a value outside the limits is refused in words naming them and the Export button is disabled until it is within. The dialog shows the image size the resolution gives and, when the Level has Elements drawn as placeholders (Missing Assets or unknown kinds, counted from the resolution table and the Element kind registry), how many will be exported as shown. Export then runs the platform's save dialog proposing `<Project> - <Level>.png` and sends ProjectManager an Export Level message with the Level, the pixels per cell, the path (`.png` added when the name lacks it in any letter case), and the tile size the Editor always passes, 1024 pixels. The Level is the Project's only one.
- **The flow**: ProjectManager refuses, before any file exists, a resolution outside 1 to 1024 pixels per cell, naming the limits; a tile size outside what RenderEngine renders (1 to 8192 pixels a side); an entity that is not a Level or has no Project; and Bounds whose pixel size cannot be counted. It then asks OutputAccess to BeginImage as many pixels as the Bounds are cells times the resolution, has RenderEngine RenderRegion each tile of the Bounds with the tile's Terrain coverages, hands each to WriteTile, and ends with FinishImage, answering with the image written (its path and pixel size) or the reason it was refused or failed. The Export draws everything on every Level of the Project; a Project has one Level, so the Level in the request selects nothing.
- **A job over frames**: rendering a tile and reading its pixels back from the GPU takes a few frames, so an Export is a job that the request starts and a system advances every frame: it asks for the next tile as soon as RenderEngine can take a request, keeps a few requests in flight, writes each tile as its pixels arrive, in row-major order from the top-left corner of the image, and finishes once the last tile is written. Exports run one at a time in the order they were asked for. A failure anywhere drops the image writer, which removes the partial file, and is answered with its reason; opening another Project abandons every Export in progress the same way, answered as refused because the Project was replaced, and no partial file is left. Nothing is recorded in the history: an Export changes nothing in the Project. Open is handled before the Exports each frame, so an Export of a Project being replaced never asks for another tile.
- **Terrain per tile**: before it requests a tile, ProjectManager asks PaintEngine to Rasterize every Terrain of the Level whose box meets the tile over the tile's region at the Export's pixels per cell, leaves out a coverage that covers nothing there, and hands the rest to RenderEngine with the RenderRegion request, so painted ground, every erase and edited stroke included, is computed afresh at the Export's resolution through the same Rasterize as the editor's tiles and the same in every tile, never taken from the editor's 32 pixels per cell. A request refused because the previous region still waits keeps its coverages for the next try; an accepted one takes them without copying. A Terrain whose box misses the tile costs no rasterizing.
- **Waiting**: while an Export runs the Editor ignores the pointer and the keys over the viewport and offers neither Undo, Redo, nor another Export, and its status line says the viewport waits for the Export; the frames keep running. Its overlays (the selection outline, the selected Wall's or Room's handles and guide lines, the Wall or the Room being drawn, the Portal tool's marker, and the Paint tool's circle, band, and selected stroke with its handles) are not drawn and the selection is left as it is, so none reaches a tile. Progress and cancellation do not exist.
- **RenderRegion**: one offscreen camera draws a square of the Level, given by its lower-left corner in cells, into a texture of the tile size at the chosen pixels per cell, through the same sprites, Wall and Room meshes, and stacking depths as the viewport, over an opaque black clear colour rather than the viewport's grey, drawn after the viewport's camera. The operation is a request and a poll: a request points the camera at the region and a later poll yields its pixels once the GPU has handed them back, two or three frames later; one region is captured per frame, and a request is refused while the previous region still waits for its frame. The request's Terrain coverages are drawn for that capture alone, one quad of the region's size per Terrain with the viewport's masked tiled image Material, at the Terrain's place in the stacking order, on a render layer only the offscreen camera sees, while the viewport's coverage tiles sit on one only the viewport's camera sees; each coverage is an R8 image sampled nearest, its texels falling on the region's pixels one to one, so the Export shows exactly the coverage Rasterize computed. A region is captured only once no sprite or Terrain is still loading its image and its Terrains' quads are in place, a frame after they were drawn, a mesh having nothing to load, and never in the frame the camera was spawned in, whose render is not yet the camera's own, nor before its target exists on the GPU. The readback is RenderEngine's own, through a staging buffer copied after the frame is drawn and mapped before the next extraction, so the Engine is called and never notified. The camera is spawned when the tile size or resolution differs from the current camera's and released when the Export ends. The sprites keep the sampler they are drawn with in the viewport, linear filtering, so an image scales between the Grid's pixels per cell and the Export's by linear interpolation, as on screen. Without a renderer, as in a headless editor without Bevy's render plugins, a request says so instead of drawing.
- **Tiles**: a tile's corner is counted in whole pixels from the Bounds' lower-left corner and divided by the resolution once per axis, so every tile's edge lands where the image's pixel grid says whether or not the resolution divides the tile size; the right and bottom tiles may reach past the Bounds, and OutputAccess clips them to the image. The tile size is a parameter of the request so a test can vary it.
- **OutputAccess**: BeginImage opens a PNG of the final size as a temporary file beside the chosen path, under the path's name and a `.part` suffix; WriteTile assembles the tiles of one row into a band as high as a tile and as wide as the image and streams each completed band into the encoder, so an image of twenty thousand pixels a side costs one band of memory; FinishImage closes the encoder, flushes the file to the disk, and renames it over the path. The PNG is 8-bit RGBA with fixed compression and filter settings and an sRGB chunk, so the same tiles always give the same bytes. A tile out of order, of another height than its band, or of the wrong size is refused; a failure, or dropping the writer unfinished, removes the temporary file and leaves whatever was at the path untouched. The file is created as readable as any file the Author makes. OutputAccess uses no Bevy.

## Test seams

The automated seam is a headless Bevy App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, AuthoringManager, and RenderEngine under Bevy's default plugins without a window, over one fixture Asset Folder of solid-colour images, an image of four coloured quarters, a half-transparent one, and one that cannot be decoded among them, exporting to temporary files, decoding the PNG, and asserting on pixel facts (the size, the colour at a placed Prop's cells and elsewhere, the colour along and beside a Wall and within a Portal's gap, a Room's floor and Walls, a Portal's colours on either side of its Wall, a Terrain's colour on a stroke's path, in its soft edge, and beyond its radius, the absence of any non-opaque pixel, a Prop outside the Bounds leaving no trace, a straddling Prop's inside part, byte identity between two Exports and between two tile sizes) and on the messages answered. It draws on the GPU and fails, rather than skips, where no adapter exists. Where a test needs small Bounds to export at a high resolution, its fixture sets them in the World directly, because no Command resizes the Bounds. The image writer alone runs as unit tests of OutputAccess.

- **Bounds to start with**: `crates/drs-app/tests/export.rs::bounds_to_start_with`
- **Exactly the Bounds**: `crates/drs-app/tests/export.rs::exactly_the_bounds`
- **Resolution in pixels per cell**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Resolution within limits**: `crates/drs-app/tests/export.rs::resolution_within_limits`
- **Clipped at the edge**: `crates/drs-app/tests/export.rs::clipped_at_the_edge`, `crates/drs-app/tests/export.rs::a_wall_is_clipped_at_the_edge`, `crates/drs-app/tests/export.rs::a_room_is_clipped_at_the_edge`, `crates/drs-output-access/src/lib.rs::tests::tiles_overhanging_the_edges_are_clipped`
- **Drawn as in the editor**: `crates/drs-app/tests/export.rs::drawn_as_in_the_editor` (the Props, their stacking, and a Missing Asset's placeholder at its size), `crates/drs-app/tests/export.rs::any_path_works_in_the_export`, `crates/drs-app/tests/export.rs::walls_stack_with_props` (a Wall over one Prop and under another), `crates/drs-app/tests/export.rs::a_wall_is_drawn_as_a_stroke`, `crates/drs-app/tests/export.rs::a_curved_wall_follows_its_curve`, `crates/drs-app/tests/export.rs::a_wall_gives_way_to_its_portal`, `crates/drs-app/tests/export.rs::a_gap_follows_the_corner`, `crates/drs-app/tests/export.rs::a_portal_faces_its_side` (turned to a vertical Wall and mirrored after a flip), `crates/drs-app/tests/export.rs::a_freestanding_portal_is_turned`, `crates/drs-app/tests/export.rs::a_mirrored_portal_is_flipped`, `crates/drs-app/tests/export.rs::overlapping_portals_stack`, `crates/drs-app/tests/export.rs::no_cap_where_a_portal_reaches_the_end`, `crates/drs-app/tests/export.rs::a_missing_portal_keeps_its_gap`, `crates/drs-app/tests/export.rs::a_room_fills_its_floor`, `crates/drs-app/tests/export.rs::a_curved_room_fills_its_curve`, `crates/drs-app/tests/export.rs::a_room_is_walled_all_round`, `crates/drs-app/tests/export.rs::a_rooms_floor_lies_under_its_walls` (a Room over one Prop and under another), `crates/drs-app/tests/export.rs::a_room_wall_gives_way_to_its_portal`, `crates/drs-app/tests/export.rs::a_stroke_shows_its_material`, `crates/drs-app/tests/export.rs::terrain_lies_under_props_and_walls`, `crates/drs-app/tests/export.rs::a_missing_terrain_image_keeps_its_shape`, `crates/drs-app/tests/export.rs::an_erase_shows_what_lies_below` (the background on a full-strength erase's path), `crates/drs-app/tests/export.rs::an_erase_leaves_no_build_up` (a half-strength erase's joint as its straight stretch), `crates/drs-app/tests/export.rs::an_edited_stroke_exports_as_edited` (a moved stroke at its new place, an erase turned to painting); the placeholder of an unknown kind, drawn by the same code as a Missing Asset's, is checked by hand
- **Terrain at the Export's resolution**: `crates/drs-app/tests/export.rs::terrain_at_the_exports_resolution` (a hard dab exported at 200 pixels per cell: the image's colour at the pixel whose centre lies just inside the radius and the background at the one just outside, which the editor's 32 pixels per cell could not tell apart), `crates/drs-app/tests/export.rs::painted_tiles_leave_no_seams`
- **Only the Level is exported**: by hand: no automated seam draws the Editor's overlays; verified by driving the editor with the dev-only input script, exporting with a Portal selected and the Portal tool's marker showing, with the Paint tool's circle and a stroke's band showing, and with a stroke selected in Edit strokes
- **Opaque background**: `crates/drs-app/tests/export.rs::opaque_background`
- **Exported as PNG**: `crates/drs-app/tests/export.rs::exported_as_png`
- **Placeholders are warned about**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Export waits for Assets**: `crates/drs-app/tests/export.rs::export_waits_for_assets`
- **Same Level, same image**: `crates/drs-app/tests/export.rs::same_level_same_image`, `crates/drs-app/tests/export.rs::painted_levels_export_the_same`, `crates/drs-output-access/src/lib.rs::tests::the_same_tiles_give_the_same_bytes`
- **Tiles leave no seams**: `crates/drs-app/tests/export.rs::tiles_leave_no_seams`, `crates/drs-app/tests/export.rs::painted_tiles_leave_no_seams` (a painted Level at two tile sizes, byte-identical)
- **A failed export is reported**: `crates/drs-app/tests/export.rs::a_failed_export_is_reported`, `crates/drs-app/tests/export.rs::an_open_refuses_a_running_export`, `crates/drs-output-access/src/lib.rs::tests::a_dropped_writer_leaves_no_file`, `crates/drs-output-access/src/lib.rs::tests::an_incomplete_image_is_refused_and_removed`
- **Export changes nothing**: `crates/drs-app/tests/export.rs::export_changes_nothing` (the Elements, the view, the history, and the saved mark; the selection is the Editor's and is checked by hand)
- **Export name proposed**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script

## Not supported

- An Export is never a history step and never changes the Project.
- An Export never leaves a partial file: the image appears at its path whole or not at all.

## Notes

- Four Rules (Resolution in pixels per cell, Placeholders are warned about, Only the Level is exported, Export name proposed) have no automated test, against the requirement that every Rule has one, and the unknown kind's placeholder of Drawn as in the editor and the selection clause of Export changes nothing are checked only by hand. They are behaviour of the egui interface and of the Editor's overlays, for which no headless seam exists. The accepted deviation is verification by hand, driving the editor with the development-only input script, which stands in for the save dialog through an environment variable.
- A painted Export rasterizes every Terrain over every tile it reaches on the CPU and takes longer than an unpainted one: measured in a development build, 61 by 61 cells at 100 pixels per cell in 0.85 s.
- The offscreen seam needs a GPU adapter and fails without one; whether every hosted CI runner has one, and a software adapter as the fallback, is an open question of the architecture.
- The Export is a PNG of the Project's one Level with the default Bounds; other formats, choosing a Level, hidden Layers, Layer Groups, the Ambient Light as the background, progress, cancellation, and headless batch export do not exist.
