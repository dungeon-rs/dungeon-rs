# Walking skeleton: save, reopen, export

**Capabilities**:
- projects: none in this change. Save and Open are Project lifecycle operations, not domain Commands; the capability's Commands, Relink and Embed Asset, come later.
- export: Export Level

## Problem Statement

An Author can place Props on a Level and undo every step, but the work lives only as long as the editor runs. Nothing can be kept for tomorrow, handed to a collaborator, or put in front of players: there is no file to save, nothing to reopen, and no image to take to the table or a virtual tabletop.

## Solution

The Author saves the Project to a single file wherever they like and reopens it later, on this device or another. Every Prop comes back where it was, with its identity and its place in the stacking order. Assets are found again through the Canonical Name of their Asset Folder and their place in it, however that folder is located on this device; what cannot be found stays in the Project as a placeholder of the right size, and the Author is told in plain terms which Asset is missing, from which Asset Folder and version, and how many Elements use it. Saving again keeps everything, so a collaborator who lacks a folder never damages the Project. The Author also exports the Level as a PNG that covers exactly the Bounds at a resolution chosen in pixels per Grid cell, well beyond what the functional baseline allows.

## User Stories

### Saving

1. As an Author, I want to save the Project to a file I pick in my operating system's save dialog, so that my work survives closing the editor.
2. As an Author, I want the Project file extension added when I type a name without it, so that the file is recognised later without me remembering the extension.
3. As an Author, I want Save to write to the file the Project was last saved to or opened from without asking again, so that saving is one key.
4. As an Author, I want Save on a Project that has never been saved to ask me where, so that I am never asked for a location twice nor saved somewhere I did not choose.
5. As an Author, I want Save As to let me save under a new name and continue working in that file, so that I can keep versions.
6. As an Author, I want the editor's title to show the Project's name and whether it has unsaved changes, so that I know at a glance whether I can close it.
7. As an Author, I want a save to a read-only or vanished location to tell me why it failed and leave my work and the previous file as they were, so that a bad location costs me a retry, not a map.
8. As an Author, I want a save that fails halfway to leave the previous file intact, so that a full disk never leaves me with half a Project.
9. As an Author, I want a Project file whose path holds spaces, quotes, accents, non-Latin letters, or symbols to save and open like any other, so that my folder layout is never a problem.
10. As an Author, I want the file to hold nothing about my device, such as where my Asset Folders are, so that the file works on a collaborator's device.
11. As an Author, I want saving to keep every Asset Reference and every Element, including those whose Asset is missing on this device, so that opening a Project on the wrong device never loses anything.
12. As an Author, I want saving the same Project twice to produce the same file, so that a version control tool shows only real changes.
13. As an Author, I want saving to change nothing on screen and to add no undo step, so that saving is never something I have to undo around.
14. As an Author, I want undoing back to the state I saved to count as having no unsaved changes, so that the unsaved marker tells the truth.

### Reopening

15. As an Author, I want to open a Project file through my operating system's open dialog, filtered to Project files, so that I find my Projects quickly.
16. As an Author, I want the opened Project to replace what I was working on, with every Prop where it was, at its size, in its stacking order, so that I continue exactly where I left off.
17. As an Author, I want to be asked whether to save, discard, or cancel when I open a file while I have unsaved changes, so that I never lose work by opening another Project.
18. As an Author, I want the same question when I quit with unsaved changes, so that closing the window never loses work.
19. As an Author, I want cancelling at that question, or cancelling the save it leads to, to leave me exactly where I was, so that a wrong click costs nothing.
20. As an Author, I want an opened Project to start with an empty undo history, so that undo never reaches into a session that is gone.
21. As an Author, I want a file that is not a Project, is damaged, or cannot be read to be refused with the reason while my current Project stays untouched, so that a bad file never takes down what I have open.
22. As an Author, I want a Project saved by a newer version of the editor than mine to be refused with a plain explanation rather than opened with parts missing, so that I never unknowingly save a Project with parts of it dropped.
23. As an Author, I want a Project holding data this editor does not know, such as a property added by another version or a Plugin, to open and keep that data unchanged when I save, so that passing a file through my editor never strips it.
24. As an Author, I want an Element of a kind this editor does not know to stay in the Project as a placeholder, be mentioned when the Project opens, and be saved back untouched, so that a collaborator's Plugin content survives a round trip through my editor.
25. As an Author whose Asset Folder sits at a different path than on the device the Project was saved on, I want every Asset to be found through the folder's Canonical Name, so that the Project never cares where my library is.
26. As an Author on a device that lacks one of the Project's Asset Folders, I want the Props from it to stay as placeholders of the right size, and to be told which Assets are missing, from which Asset Folder and version, and how many Elements use each, so that I know what to install.
27. As an Author whose copy of an Asset Folder is an older version than the Project was saved against, I want an Asset that my version lacks to be reported with the version the Project recorded and the version I have, so that I know the gap is a version, not a mistake.
28. As an Author whose copy of an Asset Folder is a different version, I want Assets that are present in my copy to be used without comment, so that a version difference alone is never noise.
29. As an Author whose operating system spells an Asset's path differently in letter case or Unicode form than the device the Project was saved on, I want the Asset found anyway, so that moving between operating systems never produces false Missing Assets.
30. As an Author who touched up an image without renaming it, I want the Project to use the image at its place as it is now, so that an edit to my own art is picked up.
31. As an Author, I want the report of Missing Assets to be something I can read and dismiss, keeping the placeholders, so that I can work on the rest of the map meanwhile.
32. As an Author who opened a Project with Missing Assets, I want adding the Asset Folder under its Canonical Name to make them appear without reopening the Project, so that fixing a missing library is one step.
33. As an Author sharing a Project with a collaborator who lacks one of my Asset Folders, I want the Project to come back from them with every Asset Reference and Element intact, so that their saving never damages my map.
34. As an Author, I want a Project to open on a device with no Asset Folders at all, with every Prop a placeholder, so that I can at least look at the map and see what it needs.
35. As an Author, I want the Project to remember the file it was opened from, so that Save after Open goes to that file.

### Exporting

36. As an Author, I want to export the Level as a PNG image, so that I can take it to a virtual tabletop or a printer.
37. As an Author, I want to choose the resolution in pixels per Grid cell from common choices or type my own, so that the image fits the tabletop I use.
38. As an Author, I want a sensible resolution proposed, so that the common case is one confirmation.
39. As an Author, I want to see the size in pixels the image will have before I export, so that I know what I am about to produce.
40. As an Author, I want the image to cover exactly the Bounds, so that what I export is predictable and lines up with the Grid.
41. As an Author, I want Props outside the Bounds left out and Props straddling the edge cut at it, so that the image is exactly the Bounds and nothing else.
42. As an Author, I want to export well above 300 pixels per cell, so that a printed map is as sharp as my Assets allow.
43. As an Author, I want a resolution outside what the editor supports to be refused before anything is written, with the limits named, so that I never wait for an export that cannot finish.
44. As an Author, I want the export to look the same whatever the zoom or size of my window, so that the image is the map, not my screen.
45. As an Author, I want exporting to leave my view, my selection, and my Project as they were, so that exporting is a side step.
46. As an Author, I want an export to a read-only or vanished location to tell me why and leave no partial file, so that a bad location costs me a retry.
47. As an Author, I want to be told before exporting when the Level has Missing Assets or Elements of an unknown kind, so that I can decide whether placeholders in the image are acceptable.
48. As an Author, I want the exported image to show every Prop as the editor shows it, in the same stacking order, so that what I see is what I export.
49. As an Author, I want the export to wait until every image has loaded, so that nothing is exported half-drawn.
50. As an Author, I want exporting the same Level twice at the same resolution to produce the same file, so that I can trust a re-export to change only what I changed.
51. As an Author, I want the image to have no visible seams, so that a large export is one picture.
52. As an Author, I want a file name proposed from the Project's and Level's names, so that my exports are findable.
53. As an Author, I want the exported image to have an opaque background, so that it displays the same in every tabletop and printer.

## Rules

### Saving

**Saved as one file**: Save writes the whole Project into a single file with the `.dungeon` extension, which is added when the chosen name lacks it.

**Save remembers its file**: once a Project has been saved to or opened from a file, Save writes to that file without asking.

**Save without a file asks**: Save on a Project that has no file behaves as Save As.

**Save As moves the Project**: after Save As, the Project's file is the new one, and the previous file is left as it was.

**Named by its file**: the editor's title shows the Project file's name without its extension, or `Untitled` for a Project that has no file, with a marker while the Project has unsaved changes.

**Everything the Project is**: the file holds the Grid, the Bounds, every Level with its Layers, every Element with its ElementId, kind, position, size, and the properties of its kind, and the Asset Reference table with the Canonical Name and version of every Asset Folder recorded in it. Follows from: Project.

**Nothing of this device**: the file holds no path, folder key, or other value specific to this device. Follows from: A Project is device-independent.

**History is not saved**: the file holds nothing of the history, and an opened Project has an empty history. Follows from: Undo history is not part of a Project.

**Saving keeps every reference**: Save writes every Asset Reference and every Element, including Elements whose Asset is Missing and Elements of an unknown kind. Follows from: References are never dropped.

**Saving is a fixed point**: saving a Project, opening the file, and saving it again produces a byte-identical file.

**A failed save leaves the old file**: when a save cannot complete, the file that was at the chosen path before is unchanged, the Author is told the reason, and the Project keeps its unsaved changes and its remembered file.

**Any path works for Projects**: a Project file whose path holds spaces, quotes, non-ASCII letters, or symbols is saved and opened like any other.

**Saving is not a step**: Save adds nothing to the history and changes no Element, the selection, or the view.

**Unsaved means a step since the save**: a Project has unsaved changes exactly when the history's position differs from its position at the last save or open; undoing back to that position counts as having none.

### Reopening

**Opened as saved**: opening a saved file yields the same Grid, Bounds, Levels, Layers, and Elements, each Element with the ElementId, kind, position, size, and properties it was saved with, in the same stacking order, and the same Asset Reference table.

**Open replaces the Project**: the opened Project replaces the current one, and the opened file becomes the Project's file.

**Unsaved changes are asked about**: opening a file or quitting while the Project has unsaved changes asks the Author to save, discard, or cancel; cancelling, or cancelling or failing the save it leads to, leaves the Project, the history, and the editor as they were.

**A bad file is refused**: a file that is not a Project, cannot be read, or holds malformed data is refused with the reason, and the current Project and its history are untouched.
_Why_: a bad file must become a report, never a crash or a half-loaded Project.

**A newer file is refused**: a file whose format version, or any component version in it, is newer than this editor knows is refused, naming the version, and the current Project is untouched.
_Why_: opening it and saving would silently drop what the newer editor saved, against References are never dropped.

**Unknown components round-trip**: data in an Element under a component name this editor does not know is kept with the Element and written back unchanged on save. Follows from: References are never dropped.

**Unknown kinds are kept**: an Element whose kind is not registered on this editor stays on its Layer at its position in the stacking order, is drawn as a placeholder of its recorded size, is counted in the report shown after opening, and is written back unchanged on save. Follows from: References are never dropped.

**Resolved by Canonical Name and place**: an Asset Reference resolves to the file at its recorded place inside the Asset Folder on this device whose Canonical Name equals the recorded one, compared ignoring letter case and Unicode normalisation, wherever that folder sits on this device. Follows from: A Project is device-independent.

**Spelling differences resolve**: when no file sits at the exact recorded place, a file whose path differs from it only in letter case or Unicode normalisation resolves; when two or more such files exist, the Asset Reference is Missing.
_Why_: a plausible wrong Prop is worse than a placeholder.

**Same place, changed content**: a file at the recorded place resolves even when its byte size or content fingerprint differs from the recorded one.

**Versions do not block**: an Asset Folder whose version differs from the version recorded in the Project resolves by place like any other, and nothing is said about the difference for Assets that resolve.

**Missing Assets stay**: an Asset Reference that does not resolve is a Missing Asset; every Element that uses it stays on its Layer at its position in the stacking order and is drawn as a placeholder of its recorded pixel size. Follows from: References are never dropped.

**Missing Assets are explained**: after opening, the Author is shown, for each Missing Asset, its name, the Canonical Name of its Asset Folder, the version recorded in the Project, how many Elements use it, and whether that folder is absent on this device or present without the Asset; when present, the folder's version on this device is named beside the recorded one. Follows from: A Missing Asset is always explainable.

**Resolved when a folder arrives**: adding an Asset Folder, or redoing its addition, re-resolves the Asset References recorded against its Canonical Name; undoing its addition makes them Missing again. Follows from: Asset Folder Changed.

**Recorded versions stay**: opening and saving never change the Asset Folder versions recorded in the Project.

**Opened without folders**: a Project opens on a device with no Asset Folders, with every Asset Reference Missing.

**Identity survives the file**: an Element has the same ElementId after saving and reopening as before.

### Exporting

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

## Changes to existing behaviour

None of the pinned specs. Of the first walking skeleton, not yet pinned:

- composing — **A failed load is a placeholder**: extended; an Element whose Asset is Missing, and an Element of an unknown kind, are drawn as the same placeholder of their recorded size.
- asset-folders — the Implementation Decision that the Asset Folder Changed message is not sent yet is superseded: LibraryManager sends it, and ProjectManager listens.

## Implementation Decisions

The technology the architecture already fixes (the Project format principles of stable component names, per-component versions, and unknown data passed through; the asset identity table; the resolution order; the `lib://` source; the tiled export through OutputAccess; deterministic saves and output) is used as written there and not restated.

### The Project file

- **Shape**: one JSON document, pretty-printed in a fixed key order. It holds a format version; the Grid; the Bounds; the Asset Reference table (its Assets and its Asset Folders with Canonical Name and version); the Levels, each with its Layers, each Layer listing the ElementIds of its Elements in stacking order; and the Elements, keyed by ElementId written as a decimal string and sorted by it. Each Element is a map from stable component name to an envelope of a version and that component's data. The extension is `.dungeon`.
- **Serialisation registry**: `model` holds a registry of serialisable components, each with its stable name, current version, and how to read every version it has had and write the current one; each crate registers the components it owns when its plugin is built, and all components of this change are `model`'s. ProjectAccess's ReadProject and WriteProject consult the registry and never name a component themselves. An envelope whose name no entry knows is kept as raw data on the Element, under a component that holds unknown envelopes, and WriteProject writes those back verbatim. An envelope whose version is newer than its entry knows makes ReadProject fail, naming the component and the version; so does a format version newer than the editor's.
- **Atomic writes**: WriteProject writes to a temporary file beside the target and renames it over the target once complete, so a failed write leaves the previous file untouched and never a partial one.
- **Not in the file**: history, selection, viewport, folder keys, paths, and the resolution state. The Project's file and the history position at the last save are a resource owned by ProjectManager.

### Save, Open, and quit

- **Requests**: the Editor sends ProjectManager a Save message, with a path for Save As or none for Save, and an Open message with a path; ProjectManager answers with a message that says it succeeded or why it was refused, which the Editor shows. The Editor runs the native save and open dialogs on the main thread, filtered to Project files, and the save, discard, or cancel question; it asks for that question whenever the history position differs from the saved mark. Quitting is the Editor intercepting the window's close request and asking the same question.
- **Save**: ProjectManager gathers the Project from the World, asks ProjectAccess to WriteProject, and on success records the path and the history position as the saved mark. A refusal changes nothing.
- **Open**: ProjectManager asks ProjectAccess to ReadProject; on a refusal nothing changes. On success it despawns the current Project tree, spawns the opened one from the file, clears the history, records the file and the saved mark, and resolves every Asset Reference. Materialising a saved Project is lifecycle, not composing, so ProjectManager writes the Element components here while AuthoringManager keeps writing them during composing.
- **Resolution**: ProjectManager asks CatalogEngine to Resolve each Asset Reference against the Asset Folders in the World, matching Canonical Names the way uniqueness compares them and taking the first two steps of the architecture's order: the exact place, then a place differing only in letter case or Unicode normalisation, which is Missing when more than one candidate exists. The outcome is a resolution table on the Project, written only by ProjectManager, mapping each Asset Reference row to the folder key and place that loads it, or to Missing with its reason (folder absent, Asset absent in a folder of a named version, or ambiguous). RenderEngine reads the table to load through `lib://` or draw a placeholder; it no longer looks folders up by Canonical Name itself. The Editor reads the table and the Asset Reference table to build the report and the export warning.
- **Asset Folder Changed**: LibraryManager sends it after AddFolder succeeds, after undoing or redoing it, and at startup for each remembered folder, carrying the Canonical Name; ProjectManager re-resolves the Asset References recorded against that name.
- **Unknown kinds**: an Element whose kind is not in the registry is spawned with its common Element component, its unknown envelopes, and no kind component; RenderEngine draws it as the placeholder; the Editor counts it in the report.
- **Unsaved marker**: `history` exposes its position as a comparable value; ProjectManager keeps the saved mark; the Editor compares them for the title and the question.

### Export

- **Flow**: the Editor shows the export dialog (resolution choices and a typed value, the resulting pixel size, the placeholder warning), then the native save dialog proposing the file name, and sends ProjectManager an Export message with the Level, the pixels per cell, and the path. ProjectManager checks the limits and refuses outside them, asks OutputAccess to BeginImage, asks RenderEngine to RenderRegion for each tile of the Bounds, hands each to WriteTile, and ends with FinishImage; a failure before FinishImage makes OutputAccess remove the partial file. The outcome goes back to the Editor as a message.
- **RenderRegion**: renders a rectangle of the Level in cell coordinates at a given pixels per cell into a texture of a fixed tile size from one offscreen camera, through the same draw path and stacking depths as the viewport, with the clear colour opaque black, skipping the first frame of a fresh offscreen camera, and only once every Asset the Level uses has loaded or failed. Scaling between the Grid's pixels per cell and the export's uses linear filtering. The tile size is a parameter of the Export so a test can vary it.
- **Synchronous**: the editor waits for an Export in this change; progress and cancellation come later.

## Testing

Test references take the form `path/to/file.rs::test_fn`. Three seams.

- **Seam: the headless App saving and reopening.** A headless Bevy `App` built from the real plugins of `model`, `history`, LibraryAccess, CatalogEngine, LibraryManager, AuthoringManager, ProjectManager, and ProjectAccess, with no window and no RenderEngine, over fixture Asset Folders and Project files in temporary directories, driven by Save, Open, Apply, Undo, Redo, and AddFolder messages, and asserting on the World, the files written, and the messages answered. A second App over other directories plays the other device; renaming or removing the fixture folder plays a device that lacks it. Covers: Saved as one file, Save remembers its file, Save without a file asks (the refusal when no file is remembered; the dialog is by hand), Save As moves the Project, Everything the Project is, Nothing of this device, History is not saved, Saving keeps every reference, Saving is a fixed point, A failed save leaves the old file, Any path works for Projects, Saving is not a step, Unsaved means a step since the save, Opened as saved, Open replaces the Project, A bad file is refused, A newer file is refused, Unknown components round-trip, Unknown kinds are kept (all but the drawing), Resolved by Canonical Name and place, Spelling differences resolve, Same place, changed content, Versions do not block, Missing Assets stay (all but the drawing), Missing Assets are explained (the facts in the resolution table), Resolved when a folder arrives, Recorded versions stay, Opened without folders, Identity survives the file, Resolution within limits, A failed export is reported (the refusal and the absence of a file). Permission fixtures run on Linux and macOS only.
- **Seam: the offscreen Export.** The same headless App with RenderEngine and offscreen rendering added, exporting a fixture Level of solid-colour Props to a temporary file, then decoding the PNG and checking pixel facts: its size, the colour at a placed Prop's pixels, the background colour elsewhere, the absence of any non-opaque pixel, a Prop outside the Bounds leaving no trace, a straddling Prop's inside part, and byte identity between two exports and between two tile sizes. Covers: Bounds to start with, Exactly the Bounds, Clipped at the edge, Drawn as in the editor, Opaque background, Exported as PNG, Export waits for Assets, Same Level, same image, Tiles leave no seams, Export changes nothing. This seam needs a GPU adapter and fails, rather than skips, without one; CI provides a software adapter where a runner has none.
- **Checked by hand**, by the author or a verification agent driving the editor: Named by its file, Unsaved changes are asked about, Resolution in pixels per cell, Placeholders are warned about, Export name proposed, the dialogs and prompts, the Missing Asset report as shown, and the placeholders drawn for Missing Assets and unknown kinds.

## Out of Scope

- Relink and Embed Asset; the content-hash, Manifest-redirect, and format-twin steps of resolution; suggestions the Author confirms.
- Updating the Asset Folder versions a Project records; pruning Asset References no Element uses.
- Embedded Assets and the container the Project file would need for them.
- Migrations between format versions; this change writes the first version of every component, so the registry has no older version to read yet.
- A New Project command, recent files, autosave, backups, and more than one Project open at a time.
- Two editors or two Authors on the same Project file at once; locking.
- Export formats other than PNG; exporting more than one Level; choosing a Level, as there is one; hidden Layers, Layer Groups, and Trace Images in the Export, as none exist yet; the Ambient Light as the export background.
- Progress and cancellation of a long Export; exporting in the background; headless batch export from the command line.
- Editing or showing the Bounds; the Export uses the default Bounds.
- Reporting Missing Element Kinds by Plugin name and version; Plugins do not exist yet.

## Further Notes

- Export Level is a domain Command, and the domain's invariant Every Command can be undone admits no exception. An Export produces a file outside the Project and changes nothing inside it; this spec records no history step for it and leaves the question to the author: whether the invariant gains an exception for Commands whose only effect is output, or Export Level stops being called a Command.
- Unsaved means a step since the save counts Add Asset Folder, which changes device state and not the Project, as a step: adding a folder and quitting asks about saving. Accepted as the conservative side.
- Whether each hosted CI runner has a GPU adapter for the offscreen Export seam is found out when the seam is built; the architecture's note on software adapters (Mesa lavapipe) is the fallback.
