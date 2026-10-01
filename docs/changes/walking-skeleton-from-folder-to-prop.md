# Walking skeleton: from folder to Prop

**Capabilities**:
- asset-folders: Add Asset Folder
- composing: Place Element, Edit Element, Remove Element

## Problem Statement

An Author owns folders full of Assets and an editor that cannot see them. There is no way to tell the editor where a folder is, no list to find an Asset in, no Level to put it on, and nothing to take back a wrong move. Until a Prop can travel from a folder on disk to a Level on screen and back out again, nothing else in the roadmap can be tried end to end.

## Solution

The editor opens on a new, unsaved Project with one Level and one Layer. The Author adds an Asset Folder: picks it in a native dialog, gives it a Canonical Name, and the editor creates its Manifest, scans and indexes the folder without touching it, and lists its Assets by name. The Author picks an Asset, clicks on the Level to place a Prop at its natural size, drags it to move it, and removes it. Every one of those steps, the folder included, can be undone and redone.

## User Stories

### Adding an Asset Folder

1. As an Author, I want to pick an Asset Folder in my operating system's folder dialog, so that I use my Assets where they already are.
2. As an Author, I want to be asked for the folder's Canonical Name with the folder's own name proposed, so that the common case is one confirmation.
3. As an Author, I want a folder whose path holds spaces, quotes, accents, non-Latin letters, or symbols to work like any other, so that my library's layout is never a problem.
4. As an Author, I want a read-only folder to be added like any other, so that a folder I am not allowed to change is still usable.
5. As an Author with my library in a cloud-synced folder, I want the editor to write nothing into that folder, so that the sync tool has nothing to clobber or re-upload.
6. As an Author who shares a library with a collaborator who keeps it at a different path, I want my Project to know the folder by its Canonical Name and never by its path, so that the Project can travel later.
7. As an Author, I want a Canonical Name already in use on this device to be refused with the folder that holds it named, and to be asked again, so that no two folders are confused.
8. As an Author, I want a blank Canonical Name to be refused, so that a folder never ends up nameless.
9. As an Author, I want to cancel at the folder dialog or at the name prompt and have nothing recorded, so that a false start leaves no trace.
10. As an Author, I want adding a folder I already added to be refused and told the Canonical Name it already has, so that I do not end up with the same Assets twice.
11. As an Author, I want a folder inside an added Asset Folder, or one containing it, to be refused with the reason, so that Assets are never counted twice under two names.
12. As an Author, I want a folder that cannot be read to be refused with the reason and nothing recorded, so that the editor neither crashes nor pretends.
13. As an Author, I want a folder with no Assets to be added and shown as empty, so that a folder that will be filled by a vendor's sync is ready when the files arrive.
14. As an Author, I want adding a large folder to read only the folder's listing and file metadata, never the files, so that adding stays quick on slow disks and online-only cloud folders.
15. As an Author, I want image files to be found whatever the letter case of their extension, so that `Table.PNG` counts.
16. As an Author, I want a file with a name the editor cannot read to be skipped while the rest of the folder is indexed, so that one odd file never costs me a library.
17. As an Author, I want hidden files and folders to be ignored, so that system clutter does not show up as Assets.

### Browsing

18. As an Author, I want to see the Assets of every added Asset Folder in one list, each with its name and the Canonical Name of its folder, so that I can find an Asset without knowing where it lives.
19. As an Author, I want to type part of a name and see only the Assets whose name contains it, ignoring case, so that I find an Asset among hundreds.
20. As an Author, I want two files with the same name in different subfolders to be two Assets told apart by their place in the folder, so that neither hides the other.
21. As an Author, I want the folders I added to be there the next time I open the editor, so that I add each folder once per device.
22. As an Author, I want Assets added to or removed from a folder while the editor was closed to appear or disappear at the next start, so that a vendor update is picked up.
23. As an Author, I want to undo adding a folder and redo it without being asked its Canonical Name again, so that adding a folder is a step like any other.

### Composing

24. As an Author, I want the editor to open on a new, unsaved Project with one Level and one Layer, so that I can place something at once.
25. As an Author, I want to choose an Asset and click on the Level to place a Prop centred where I clicked, so that placing is one gesture.
26. As an Author, I want a placed Prop to appear at its natural size in Grid cells, so that a vendor's table is as big as the vendor meant.
27. As an Author, I want each new Prop to land on top of the Props already on the Layer, so that what I place last is what I see.
28. As an Author, I want to place the same Asset many times, so that a room gets as many barrels as it needs.
29. As an Author, I want to place a Prop outside the Bounds, so that the Bounds never get in the way of composing.
30. As an Author, I want to press Escape to stop placing and go back to selecting, so that I never place by accident.
31. As an Author, I want to click a Prop to select it, with the topmost one winning when they overlap, so that I always get the one I see.
32. As an Author, I want to drag a selected Prop to move it and have the whole drag be a single undo step, so that undo takes the Prop back to where the drag began, not one pixel back.
33. As an Author, I want to press Delete to remove the selected Prop, so that removing is one key.
34. As an Author, I want to undo and redo with the usual shortcuts, so that I never need a menu to take a step back.
35. As an Author, I want an undone removal to bring the Prop back exactly where it was, including its place in the stacking order, so that undo never reshuffles my map.
36. As an Author, I want a new action after undoing to discard the undone steps, so that history stays a single line I can reason about.
37. As an Author, I want to pan and zoom the viewport without that showing up in undo, so that looking around never costs me a step.
38. As an Author, I want a Prop whose image cannot be loaded to show as a placeholder of the right size while the editor keeps running, so that a deleted or broken file never takes the editor down.

## Rules

### Adding an Asset Folder

**Folder name proposed**: the Canonical Name prompt proposes the folder's own name, which the Author may change.

**Blank names are refused**: a Canonical Name that is empty once leading and trailing whitespace is removed is refused, and nothing is recorded.

**Names are unique on this device**: a Canonical Name equal to one already in use on this device, compared ignoring letter case and Unicode normalisation, is refused with the folder that holds it named, and nothing is recorded. Follows from: Canonical Names are unique on a device.

**The same folder is refused**: a folder already added on this device, however its path is spelled (letter case, trailing separator, or a symbolic link to it), is refused and the Author is told the Canonical Name it already has.

**Nested folders are refused**: a folder inside an added Asset Folder, or one containing an added Asset Folder, is refused with the reason, and nothing is recorded.

**Unreadable folders are refused**: a folder that does not exist or cannot be listed is refused with the reason, and nothing is recorded.

**Any path works**: a folder whose path holds spaces, quotes, non-ASCII letters, or symbols is added, and its Assets load.

**Nothing is written into the folder**: adding a folder creates or changes no file inside it, so a read-only folder is added like any other; the Manifest and the index cache live in the editor's own directories.
_Why_: Asset Folders may be read-only, cloud-synced, or overwritten by the next vendor release.

**Scanning opens no file**: adding a folder reads directory entries and file metadata only; a file the editor is not allowed to read is indexed like any other.

**Images are Assets**: a file whose extension is `png`, `webp`, `jpg`, or `jpeg`, in any letter case, is an Asset of the image Asset Kind; every other file is not an Asset.

**Hidden entries are skipped**: files and folders whose name begins with a dot are not scanned.

**Links are not followed**: a symbolic link inside the folder is neither indexed nor descended into.
_Why_: a link can point outside the folder or form a loop.

**Unreadable names are skipped**: a file whose name is not valid Unicode is not an Asset, and the rest of the folder is still indexed.

**Known by name and place**: an Asset's name is its file name without the extension; two files with the same name in different subfolders are two Assets, told apart by their place in the folder.

**An empty folder is added**: a folder holding no Assets is added with its Manifest and shown as holding none.

**Cancelling records nothing**: cancelling the folder dialog or the name prompt leaves no Manifest, no index, and no history step.

**Remembered across starts**: a folder added on this device is available at the next start without being added again.

**Current at start**: Assets added to or removed from a remembered folder while the editor was closed appear in or disappear from the browser at the next start.

**Add Asset Folder is undoable**: undoing makes the folder's Assets unavailable on this device and removes its Manifest; redoing restores it with the same Canonical Name without asking. Follows from: Every Command can be undone.

### Browsing

**Browsed across folders**: the browser lists the Assets of every added Asset Folder, each with its name and its folder's Canonical Name.

**Filtered by name**: the browser shows only the Assets whose name contains the typed text, ignoring letter case; an empty filter shows all.

### Composing

**A Project to start with**: the editor opens with a new, unsaved Project holding one Level named `Level 1` with one Layer named `Layer 1`.

**Placed where clicked**: with an Asset chosen, a click on the Level places a Prop of that Asset on the current Layer, centred on the clicked point.

**Natural size**: a Prop's size in Grid cells is its image's pixel size divided by 256 pixels per cell.
_Why_: the convention of the functional baseline, so its libraries place at the size their vendors meant.

**Placed on top**: a new Prop is placed above every Element already on its Layer.

**Placement records a reference**: placing a Prop records in the Project an Asset Reference holding the Asset's name, the Canonical Name of its Asset Folder, its place in that folder, its byte size, its pixel size, and its content fingerprint; placing a second Prop of the same Asset adds no second Asset Reference. Follows from: A Project is device-independent.

**Placement records the folder**: the first Prop placed from an Asset Folder records that folder's Canonical Name and version in the Project; later Props from the same folder add no second record. Follows from: A Missing Asset is always explainable.

**Anywhere on the Level**: a Prop may be placed outside the Bounds. Follows from: Bounds only decide what is exported.

**Many of the same**: several Props placed from the same Asset are independent Elements, each with its own ElementId. Follows from: No Element kind is limited to one per Level.

**Escape stops placing**: pressing Escape drops the chosen Asset, and clicks select instead of place.

**Topmost is selected**: a click with no Asset chosen selects the topmost Prop under the pointer; a click on empty space clears the selection; selection is never a history step.

**A drag is one step**: moving a Prop by dragging records a single undo step however long the drag, and undo returns the Prop to where the drag began.

**Removal is reversible in place**: undoing a Remove Element restores the Prop with every property, its ElementId, and its place in the stacking order.

**Identity survives undo**: an Element removed by undoing a Place Element and brought back by redo has the ElementId it had before.

**Redo repeats exactly**: redoing a Place Element, Edit Element, or Remove Element leaves the Level as it was before the undo.

**A new step clears redo**: a Command applied after an undo discards the undone steps.

**One history**: Add Asset Folder, Place Element, Edit Element, and Remove Element are each one undo step, and undo walks back through them in the order they were applied whichever Manager handled them. Follows from: Every Command can be undone.

**View is not a step**: panning and zooming the viewport are not Commands and never appear in the history.

**A failed load is a placeholder**: an Element whose Asset cannot be loaded or decoded is drawn as a placeholder of its recorded size, stays on its Layer, and the editor keeps running.

## Changes to existing behaviour

None. There are no pinned specs yet.

## Implementation Decisions

The technology the architecture already fixes (Manifest contents, Canonical Name, the asset identity table, the index cache, the dynamic `lib://` asset source, the history design keyed by ElementId with reflection-based commands) is used as written there and not restated here.

### Asset Folders

- **Flow**: the Editor shows the native folder dialog and the Canonical Name prompt, then sends LibraryManager one AddFolder message carrying the path and the name. LibraryManager checks the folder (readable, not already added, not nested) and the name (not blank, unique on this device), asks LibraryAccess to WriteManifest and ScanFolder, asks CatalogEngine to Classify the scanned files, writes the Asset Folder into the World, and records the step in history. A refusal is reported back to the Editor with its reason; the Editor shows it and, for a name refusal, keeps the prompt open.
- **The Manifest lives in the editor's configuration directory**, one file per Asset Folder named by its folder key, holding the folder's path as the Author gave it, the Canonical Name, the version, and the renames (none yet). The version is recorded as the day the folder was added until a later change lets the Author set it. The index cache lives in the editor's cache directory under the same key. This settles the open question of where the Manifest lives: never inside the folder.
- **The folder key** is a device-local identifier minted when the folder is added. It names the Manifest, the index cache, and the `lib://` source entry, and it is never written into a Project; Projects know a folder by its Canonical Name.
- **The set of folders in use on this device** is the set of Manifests in the configuration directory. Forgetting a folder removes its Manifest; WriteManifest covers writing a Manifest and removing it. Undoing Add Asset Folder forgets the folder; redoing writes the Manifest again from the recorded path and name and scans again. The index cache is a cache and is left in place.
- **Scanning** is a stat-only walk diffed against the index cache, as the architecture describes; hidden entries and symbolic links are skipped, non-Unicode names are skipped and counted. Scanning is synchronous in this change: the editor waits for it. ScanFolder also registers the folder key with the `lib://` source so its Assets can be loaded.
- **Classification** is CatalogEngine's Classify with one built-in Indexing Rule: the image extensions, compared ignoring case, give the image Asset Kind. Plugins contribute further Indexing Rules later.
- **Startup**: LibraryManager reads every Manifest through LibraryAccess and Refreshes each folder, which is the same scan diffed against its cache. The Asset Folder Changed message is not sent yet; nothing listens.
- **Directories**: LibraryAccess resolves the platform's configuration and cache directories by default; a resource in `model` overrides both so tests point them at temporary directories.
- **In the World**, each Asset Folder is an entity carrying its Canonical Name, folder key, path, and its index of Assets (name, place in the folder, Asset Kind, byte size, modification time), written only by LibraryManager. The Editor reads it to fill the browser and filters by name itself.

### Composing

- **The new Project** is created at startup by ProjectManager, the owner of the Project lifecycle: one Level, one Layer, a Grid of 256 pixels per cell, and default Bounds that this change neither shows nor edits.
- **Model**: Project, Level, Layer, Element, and the Asset Reference table are `model` components. A Layer holds its Elements in stacking order. An Element carries its ElementId, its kind, its position in cells (its centre), and its size in cells. Prop is the first descriptor in the Element kind registry; its Material is the default that shows its image, and it refers to its Asset by the Project-local row of the Asset Reference table.
- **Place Element**: the Editor sends AuthoringManager Apply(Place Element) with the Layer, the position, and the chosen Asset. AuthoringManager asks LibraryAccess to LoadAsset, which yields the Asset's byte size, pixel size, and content fingerprint besides the handle, adds the Asset Reference row if the Asset has none, spawns the Element with a new ElementId on top of the Layer, and records a Place command. Undo despawns it; redo respawns it with the same ElementId at the same place in the order.
- **Edit Element**: a drag is sent as a sequence of Apply(Edit Element) messages for the position, marked as one gesture from press to release; AuthoringManager records them as one history Group of the generic SetField command, so the step undoes to the position at press.
- **Remove Element**: the generic reflection-snapshot command, extended with the Element's place in its Layer's order so undo re-inserts it there.
- **Rendering**: RenderEngine's viewport systems draw one image per Prop through change detection, loading it through the `lib://` handle at the Prop's natural size, and swap in a placeholder of the recorded size when loading fails. The viewport's centre and zoom are presentation state owned by the Editor, which RenderEngine's projection follows. Author Shaders and CompileMaterial are not touched.
- **Picking and selection** are the Editor's: it maps pointer positions to Level coordinates through the viewport projection and hit-tests the Props' rectangles in stacking order. The selection lives in the Editor and never in the history.
- **Keyboard**: Escape leaves placing; Delete (and Backspace on macOS) removes the selection; the platform's standard undo and redo shortcuts drive history.

## Testing

Test references take the form `path/to/file.rs::test_fn`. Two seams, both a headless Bevy `App` built from the real plugins of the crates involved (`model`, `history`, LibraryAccess, CatalogEngine, LibraryManager, AuthoringManager, ProjectManager), with no window and no RenderEngine, driven by sending Command messages and asserting on the World, the configuration directory, and the cache directory.

- **Seam: the headless App with fixture Asset Folders.** Each test creates its fixture folder in a temporary directory so it can hold unusual names, permissions, hidden entries, and links, and points the editor's directories at temporary ones. Covers: Blank names are refused, Names are unique on this device, The same folder is refused, Nested folders are refused, Unreadable folders are refused, Any path works (as far as indexing; loading is checked by hand), Nothing is written into the folder, Scanning opens no file, Images are Assets, Hidden entries are skipped, Links are not followed, Unreadable names are skipped, Known by name and place, An empty folder is added, Remembered across starts (a second App over the same directories), Current at start, Add Asset Folder is undoable. Permission and non-Unicode fixtures run on Linux and macOS only.
- **Seam: the headless App composing on a fixture Asset.** One fixture folder with images of known pixel size. Covers: A Project to start with, Placed where clicked, Natural size, Placed on top, Placement records a reference, Placement records the folder, Anywhere on the Level, Many of the same, A drag is one step, Removal is reversible in place, Identity survives undo, Redo repeats exactly, A new step clears redo, One history.
- **Checked by hand**, by the author or by a verification agent driving the editor: Folder name proposed, Cancelling records nothing, Browsed across folders, Filtered by name, Escape stops placing, Topmost is selected, View is not a step, A failed load is a placeholder, and that a placed Prop is drawn where and as large as the model says. Rendering correctness has no automated test in this change.

## Out of Scope

- Remove Asset Folder, Rename Canonical Name, and Install Asset Pack; editing the Manifest's version or its renames.
- Saving, reopening, and exporting a Project; resolving Asset References; Missing Assets; pruning Asset References no Element uses.
- Thumbnails, search at scale, and rendering only the visible rows of a very long browser list.
- Noticing changes to an Asset Folder while the editor runs (folder watching), and the Asset Folder Changed message.
- Scanning in the background; the editor waits for a scan in this change.
- Two editors open on the same Asset Folders at once.
- More than one Level or Layer, Layer properties, Restack, Resize Bounds, drawing the Grid or the Bounds, snapping.
- Properties of a Prop other than its position (rotation, scale, Material); a property panel; the simple and advanced views.
- Drag and drop from the operating system or from the browser into the viewport; multiple selection.
- Any Element kind other than Prop; Plugins and their Indexing Rules.

## Further Notes

- Add Asset Folder changes device state, not Project state, yet it sits in the same history as composing steps because every Manager records into the one history. Undo therefore always reaches a Prop before the folder it came from.
- The headless seams leave RenderEngine out because headless rendering still needs a GPU adapter; that question stays open in `docs/specs/OPEN-QUESTIONS.md`.
