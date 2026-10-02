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
6. As an Author, I can rely on Props outside the Bounds being left out and Props straddling the edge being cut at it, so that the image is exactly the Bounds and nothing else.
7. As an Author, I can export well above 300 pixels per cell, so that a printed map is as sharp as my Assets allow.
8. As an Author, I am refused a resolution outside what the editor supports before anything is written, with the limits named, so that I never wait for an export that cannot finish.
9. As an Author, I can rely on the export looking the same whatever the zoom or size of my window, so that the image is the map, not my screen.
10. As an Author, I can rely on exporting leaving my view, my selection, and my Project as they were, so that exporting is a side step.
11. As an Author, I am told why an export to a read-only or vanished location failed, with no partial file left, so that a bad location costs me a retry.
12. As an Author, I am told before exporting when the Level has Missing Assets or Elements of an unknown kind, so that I can decide whether placeholders in the image are acceptable.
13. As an Author, I can rely on the exported image showing every Prop as the editor shows it, in the same stacking order, so that what I see is what I export.
14. As an Author, I can rely on the export waiting until every image has loaded, so that nothing is exported half-drawn.
15. As an Author, I can export the same Level twice at the same resolution and get the same file, so that I can trust a re-export to change only what I changed.
16. As an Author, I can rely on the image having no visible seams, so that a large export is one picture.
17. As an Author, I am proposed a file name from the Project's and Level's names, so that my exports are findable.
18. As an Author, I can rely on the exported image having an opaque background, so that it displays the same in every tabletop and printer.

## Rules

**Bounds to start with**: a new Project's Bounds are thirty by thirty cells with their lower-left corner at the Level's origin.

**Exactly the Bounds**: the Export is as many pixels wide as the Bounds' width in cells times the resolution, and as many high as the height times the resolution, and shows exactly the Bounds. Follows from: Bounds only decide what is exported.

**Resolution in pixels per cell**: the Author chooses a whole number of pixels per Grid cell, offered as 50, 100, 200, and 300 or typed, with 100 proposed.

**Resolution within limits**: a resolution below 1 or above 1024 pixels per cell is refused before anything is written, with the limits named.

**Clipped at the edge**: an Element wholly outside the Bounds appears nowhere in the Export; an Element straddling the edge appears only where it lies inside.

**Drawn as in the editor**: every Element on the Level appears in the Export at its position and size in cells scaled to the resolution, in stacking order, placeholders included. Follows from: Stacking order.

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

The technology the architecture fixes (the tiled export through OutputAccess, deterministic output) is used as written there and not restated.

- **The dialog**: the Editor shows a modal Export dialog naming the Level and the Bounds in cells, with the four resolutions as one click each and a field the Author types into, which is not clamped: a value outside the limits is refused in words naming them and the Export button is disabled until it is within. The dialog shows the image size the resolution gives and, when the Level has Elements drawn as placeholders (Missing Assets or unknown kinds, counted from the resolution table and the Element kind registry), how many will be exported as shown. Export then runs the platform's save dialog proposing `<Project> - <Level>.png` and sends ProjectManager an Export Level message with the Level, the pixels per cell, the path, and the tile size the Editor always passes, 1024 pixels. The Level is the Project's only one.
- **The flow**: ProjectManager refuses, before any file exists, a resolution outside 1 to 1024 pixels per cell, naming the limits; a tile size outside what RenderEngine renders (1 to 8192 pixels a side); an entity that is not a Level or has no Project; and Bounds whose pixel size cannot be counted. It then asks OutputAccess to BeginImage as many pixels as the Bounds are cells times the resolution, has RenderEngine RenderRegion each tile of the Bounds, hands each to WriteTile, and ends with FinishImage, answering with the image written (its path and pixel size) or the reason it was refused or failed. The Export draws everything on every Level of the Project; a Project has one Level, so the Level in the request selects nothing.
- **A job over frames**: rendering a tile and reading its pixels back from the GPU takes a few frames, so an Export is a job that the request starts and a system advances every frame: it asks for the next tile as soon as RenderEngine can take a request, keeps a few requests in flight, writes each tile as its pixels arrive, in row-major order from the top-left corner of the image, and finishes once the last tile is written. Exports run one at a time in the order they were asked for. A failure anywhere drops the image writer, which removes the partial file, and is answered with its reason; opening another Project abandons every Export in progress the same way, answered as refused because the Project was replaced, and no partial file is left. Nothing is recorded in the history: an Export changes nothing in the Project. Open is handled before the Exports each frame, so an Export of a Project being replaced never asks for another tile.
- **Waiting**: while an Export runs the Editor ignores the pointer and the keys over the viewport and offers neither Undo, Redo, nor another Export, and its status line says the viewport waits for the Export; the frames keep running. Progress and cancellation do not exist.
- **RenderRegion**: one offscreen camera draws a square of the Level, given by its lower-left corner in cells, into a texture of the tile size at the chosen pixels per cell, through the same sprites and stacking depths as the viewport, over an opaque black clear colour, drawn after the viewport's camera. The operation is a request and a poll: a request points the camera at the region and a later poll yields its pixels once the GPU has handed them back, two or three frames later; one region is captured per frame, and a request is refused while the previous region still waits for its frame. A region is captured only once no sprite is still loading its image and never in the frame the camera was spawned in, whose render is not yet the camera's own, nor before its target exists on the GPU. The readback is RenderEngine's own, through a staging buffer copied after the frame is drawn and mapped before the next extraction, so the Engine is called and never notified. The camera is spawned when the tile size or resolution differs from the current camera's and released when the Export ends. The sprites keep the sampler they are drawn with in the viewport, linear filtering, so an image scales between the Grid's pixels per cell and the Export's by linear interpolation, as on screen. Without a renderer, as in a headless editor without Bevy's render plugins, a request says so instead of drawing.
- **Tiles**: a tile's corner is counted in whole pixels from the Bounds' lower-left corner and divided by the resolution once per axis, so every tile's edge lands where the image's pixel grid says whether or not the resolution divides the tile size; the right and bottom tiles may reach past the Bounds, and OutputAccess clips them to the image. The tile size is a parameter of the request so a test can vary it.
- **OutputAccess**: BeginImage opens a PNG of the final size as a temporary file beside the chosen path, under the path's name and a `.part` suffix; WriteTile assembles the tiles of one row into a band as high as a tile and as wide as the image and streams each completed band into the encoder, so an image of twenty thousand pixels a side costs one band of memory; FinishImage closes the encoder, flushes the file to the disk, and renames it over the path. The PNG is 8-bit RGBA with fixed compression and filter settings and an sRGB chunk, so the same tiles always give the same bytes. A tile out of order, of another height than its band, or of the wrong size is refused; a failure, or dropping the writer unfinished, removes the temporary file and leaves whatever was at the path untouched. The file is created as readable as any file the Author makes. OutputAccess uses no Bevy.

## Test seams

The automated seam is a headless Bevy App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, AuthoringManager, and RenderEngine under Bevy's default plugins without a window, over one fixture Asset Folder of solid-colour images, exporting to temporary files, decoding the PNG, and asserting on pixel facts (the size, the colour at a placed Prop's cells and elsewhere, the absence of any non-opaque pixel, a Prop outside the Bounds leaving no trace, a straddling Prop's inside part, byte identity between two Exports and between two tile sizes) and on the messages answered. It draws on the GPU and fails, rather than skips, where no adapter exists. The image writer alone runs as unit tests of OutputAccess.

- **Bounds to start with**: `crates/drs-app/tests/export.rs::bounds_to_start_with`
- **Exactly the Bounds**: `crates/drs-app/tests/export.rs::exactly_the_bounds`
- **Resolution in pixels per cell**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Resolution within limits**: `crates/drs-app/tests/export.rs::resolution_within_limits`
- **Clipped at the edge**: `crates/drs-app/tests/export.rs::clipped_at_the_edge`, `crates/drs-output-access/src/lib.rs::tests::tiles_overhanging_the_edges_are_clipped`
- **Drawn as in the editor**: `crates/drs-app/tests/export.rs::drawn_as_in_the_editor`, `crates/drs-app/tests/export.rs::any_path_works_in_the_export` (the Props and their stacking; the placeholders in the Export are checked by hand)
- **Opaque background**: `crates/drs-app/tests/export.rs::opaque_background`
- **Exported as PNG**: `crates/drs-app/tests/export.rs::exported_as_png`
- **Placeholders are warned about**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Export waits for Assets**: `crates/drs-app/tests/export.rs::export_waits_for_assets`
- **Same Level, same image**: `crates/drs-app/tests/export.rs::same_level_same_image`, `crates/drs-output-access/src/lib.rs::tests::the_same_tiles_give_the_same_bytes`
- **Tiles leave no seams**: `crates/drs-app/tests/export.rs::tiles_leave_no_seams`
- **A failed export is reported**: `crates/drs-app/tests/export.rs::a_failed_export_is_reported`, `crates/drs-app/tests/export.rs::an_open_refuses_a_running_export`, `crates/drs-output-access/src/lib.rs::tests::a_dropped_writer_leaves_no_file`, `crates/drs-output-access/src/lib.rs::tests::an_incomplete_image_is_refused_and_removed`
- **Export changes nothing**: `crates/drs-app/tests/export.rs::export_changes_nothing`
- **Export name proposed**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script

## Not supported

- An Export is never a history step and never changes the Project.
- An Export never leaves a partial file: the image appears at its path whole or not at all.

## Notes

- Three Rules (Resolution in pixels per cell, Placeholders are warned about, Export name proposed) have no automated test, against the requirement that every Rule has one, and the placeholders of Drawn as in the editor are checked only by hand. They are behaviour of the egui interface, for which no headless seam exists. The accepted deviation is verification by hand, driving the editor with the development-only input script, which stands in for the save dialog through an environment variable.
- The offscreen seam needs a GPU adapter and fails without one; whether every hosted CI runner has one, and a software adapter as the fallback, is an open question of the architecture.
- The Export is a PNG of the Project's one Level with the default Bounds; other formats, choosing a Level, hidden Layers, Layer Groups, the Ambient Light as the background, progress, cancellation, and headless batch export do not exist.
