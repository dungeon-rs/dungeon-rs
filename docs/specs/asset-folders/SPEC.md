# Asset Folders

**Commands**: Add Asset Folder, Remove Asset Folder, Rename Canonical Name, Install Asset Pack

## Purpose

An Author owns folders full of Assets, from vendors and of their own making, and uses them where they already are. This capability lets the Author tell the editor where such a folder is, gives it the Canonical Name Projects know it by, indexes its Assets without touching the folder, and shows them as a grid of thumbnails so an Asset can be found by sight or by name. Folders added once stay available on this device and follow what changes in them, and their thumbnails are generated once per device and kept outside the folders.

## User Stories

### Adding an Asset Folder

1. As an Author, I can pick an Asset Folder in my operating system's folder dialog, so that I use my Assets where they already are.
2. As an Author, I can confirm the folder's Canonical Name with the folder's own name proposed, so that the common case is one confirmation.
3. As an Author, I can add a folder whose path holds spaces, quotes, accents, non-Latin letters, or symbols like any other, so that my library's layout is never a problem.
4. As an Author, I can add a read-only folder like any other, so that a folder I am not allowed to change is still usable.
5. As an Author with my library in a cloud-synced folder, I can rely on the editor writing nothing into that folder, so that the sync tool has nothing to clobber or re-upload.
6. As an Author who shares a library with a collaborator who keeps it at a different path, I can rely on my Project knowing the folder by its Canonical Name and never by its path, so that the Project can travel.
7. As an Author, I am refused a Canonical Name already in use on this device, told which folder holds it, and asked again, so that no two folders are confused.
8. As an Author, I am refused a blank Canonical Name, so that a folder never ends up nameless.
9. As an Author, I can cancel at the folder dialog or at the name prompt and have nothing recorded, so that a false start leaves no trace.
10. As an Author, I am refused a folder I already added and told the Canonical Name it already has, so that I do not end up with the same Assets twice.
11. As an Author, I am refused a folder inside an added Asset Folder, or one containing it, with the reason, so that Assets are never counted twice under two names.
12. As an Author, I am refused a folder that cannot be read, with the reason and nothing recorded, so that the editor neither crashes nor pretends.
13. As an Author, I can add a folder with no Assets and see it as empty, so that a folder that will be filled by a vendor's sync is ready when the files arrive.
14. As an Author, I can add a large folder with only the folder's listing and file metadata read, never the files, so that adding stays quick on slow disks and online-only cloud folders.
15. As an Author, I can rely on image files being found whatever the letter case of their extension, so that `Table.PNG` counts.
16. As an Author, I can rely on a file with a name the editor cannot read, or a subfolder it cannot list, being skipped while the rest of the folder is indexed, and I am told how many were skipped, so that one odd entry never costs me a library.
17. As an Author, I can rely on hidden files and folders being ignored, so that system clutter does not show up as Assets.

### Browsing

18. As an Author, I can see the Assets of every added Asset Folder in one grid of thumbnails, with each folder named above the grid and how many of its Assets are shown, so that I can find an Asset without knowing where it lives and still know which folders I have and that an empty one is empty.
19. As an Author, I can type part of a name and see only the Assets whose name contains it, ignoring case, so that I find an Asset among hundreds.
20. As an Author, I can hover a cell and read the Asset's name in full, the Canonical Name of its folder, and its place in the folder, so that two files with the same name in different subfolders are told apart.
21. As an Author, I can click a cell to choose the Asset to place, so that the grid is where placing starts.
22. As an Author, I can find the folders I added the next time I open the editor, so that I add each folder once per device.
23. As an Author, I can rely on Assets added to or removed from a folder while the editor was closed appearing or disappearing at the next start, so that a vendor update is picked up.
24. As an Author, I can see a remembered folder whose disk is unplugged still listed, empty and with a report, so that it is back as soon as the disk is.
25. As an Author, I can undo adding a folder and redo it without being asked its Canonical Name again, so that adding a folder is a step like any other.

### Seeing thumbnails

26. As an Author, I can see every Asset in the browser as a thumbnail with its name beneath, in a grid as wide as the panel, so that I recognise an Asset by sight instead of by name.
27. As an Author, I can watch thumbnails appear in the browser as they are generated, without doing anything, so that a freshly added folder fills in while I keep working.
28. As an Author, I get the rows I am looking at thumbnailed before the rest of the library, so that a 400,000-file folder is useful within seconds of being added.
29. As an Author, I see a neutral placeholder in the cell of an Asset whose thumbnail is not yet generated, so that the grid holds its shape while the thumbnails fill in.
30. As an Author, I see a placeholder of a thumbnail's own proportions for the brief moment before a generated thumbnail appears, so that nothing jumps when it does.
31. As an Author, I can scroll the grid as fast as I like and see placeholders at most briefly, with the visible rows filled a few frames after I stop, so that scrolling never waits for the thumbnails.
32. As an Author, I find the editor as responsive as ever while thumbnails are being generated, whether I scroll, place Props, or pan the Level, so that generation is something I never notice.
33. As an Author, I get thumbnails for PNG, JPEG, and WebP Assets alike, so that the whole library is covered whatever format a vendor ships.
34. As an Author, I see the transparent parts of an image stay transparent in its thumbnail, so that a cut-out Prop looks like a cut-out and not like a square.
35. As an Author, I see a tall or wide image keep its proportions inside the cell, neither stretched nor cropped, so that a long fence reads as a fence.
36. As an Author, I see an image smaller than the cell at its own size and not blown up, even on a dense display, so that a 32-pixel token stays crisp.
37. As an Author, I see a broken placeholder for a file that is not a readable image while the editor and the rest of the thumbnails carry on, so that one bad file never costs me the library or the session.

### Keeping thumbnails

38. As an Author, I find the thumbnails generated today there the next time I open the editor without their being generated again, so that a library is thumbnailed once per device.
39. As an Author, I get a new thumbnail at the next start for a file that a vendor update replaced, so that the picture is never of the old version.
40. As an Author, I keep the thumbnails of a remembered folder whose disk is unplugged, so that they are back with the folder as soon as the disk is.
41. As an Author, I keep a folder's thumbnails through undoing and redoing Add Asset Folder, so that undo is as cheap as any other step.
42. As an Author, I can rely on the thumbnails being kept in the editor's cache directory and never inside my Asset Folder, so that a read-only or cloud-synced folder stays untouched.
43. As an Author, I can quit the editor while thumbnails are still being generated and have it quit at once, so that generation never holds me hostage.
44. As an Author, I keep the thumbnails finished before I quit, and generation carries on from there at the next start, so that quitting never throws work away.
45. As an Author, I can rely on a thumbnail cache the editor left incomplete, because it quit or crashed mid-write, being used for what is complete and regenerated for the rest, so that a bad moment never means starting over.
46. As an Author, I can rely on a thumbnail cache the editor cannot make sense of being started afresh with no fuss, so that a damaged cache is an inconvenience and not an error.
47. As an Author, I am told when thumbnails cannot be kept, because the cache directory cannot be written, while the editor runs on with placeholders, so that a full disk never stops me from composing.
48. As an Author, I can rely on a file that could not be read for a moment, because its disk was slow or its cloud copy was not yet local, being tried again at the next start rather than marked broken for good, so that a transient fault never leaves a permanent hole.
49. As an Author, I can rely on a file that is genuinely not an image being remembered as broken, so that the editor does not try it again at every start.
50. As an Author with a 400,000-file library, I open the editor with every thumbnail kept as quickly as without them, so that the cache never becomes the thing I wait for.
51. As an Author with a 400,000-file library, I can rely on the editor's memory staying bounded however far I scroll, so that browsing a big library never costs more than a few dozen megabytes.

## Rules

### Adding an Asset Folder

**Folder name proposed**: the Canonical Name prompt proposes the folder's own name, which the Author may change.

**Blank names are refused**: a Canonical Name that is empty once leading and trailing whitespace is removed is refused, and nothing is recorded.

**Names are unique on this device**: a Canonical Name equal to one already in use on this device, compared ignoring letter case and Unicode normalisation, is refused with the folder that holds it named, and nothing is recorded. Follows from: Canonical Names are unique on a device.

**The same folder is refused**: a folder already added on this device, however its path is spelled (letter case where the file system ignores it, a trailing separator, `.` components, or a symbolic link to it), is refused and the Author is told the Canonical Name it already has.

**Nested folders are refused**: a folder inside an added Asset Folder, or one containing an added Asset Folder, is refused with the reason, and nothing is recorded.

**Unreadable folders are refused**: a folder that does not exist or cannot be listed is refused with the reason, and nothing is recorded.

**Any path works**: a folder whose path holds spaces, quotes, non-ASCII letters, or symbols is added, and its Assets load.

**Nothing is written into the folder**: adding a folder or generating its thumbnails creates or changes no file inside it, so a read-only folder is added like any other; the Manifest, the index cache, and the thumbnail pack and its index live in the editor's own directories.
_Why_: Asset Folders may be read-only, cloud-synced, or overwritten by the next vendor release.

**Scanning opens no file**: adding a folder reads directory entries and file metadata only; a file the editor is not allowed to read is indexed like any other.

**Images are Assets**: a file whose extension is `png`, `webp`, `jpg`, or `jpeg`, in any letter case, is an Asset of the image Asset Kind; every other file is not an Asset.

**Hidden entries are skipped**: files and folders whose name begins with a dot are not scanned.

**Links are not followed**: a symbolic link inside the folder is neither indexed nor descended into.
_Why_: a link can point outside the folder or form a loop.

**Unreadable names are skipped**: a file whose name is not valid Unicode is not an Asset, it is counted as skipped, and the rest of the folder is still indexed.

**Unlistable subfolders are skipped**: a subfolder that cannot be listed is counted as skipped, and the rest of the folder is still indexed.

**Known by name and place**: an Asset's name is its file name without the extension; two files with the same name in different subfolders are two Assets, told apart by their place in the folder.

**An empty folder is added**: a folder holding no Assets is added with its Manifest and shown as holding none.

**Cancelling records nothing**: cancelling the folder dialog or the name prompt leaves no Manifest, no index, and no history step.

**Remembered across starts**: a folder added on this device is available at the next start without being added again.

**Current at start**: Assets added to or removed from a remembered folder while the editor was closed appear in or disappear from the browser at the next start.

**A remembered folder that cannot be scanned stays known**: a remembered Asset Folder whose path cannot be read at start stays listed with no Assets, keeps its Manifest, and is reported to the Author.

**Add Asset Folder is undoable**: undoing makes the folder's Assets unavailable on this device and removes its Manifest; redoing restores it with the same Canonical Name without asking. Follows from: Every Command can be undone.

### Browsing

**Browsed across folders**: the browser shows the Assets of every added Asset Folder in one grid of thumbnails, ordered by the Canonical Name of their folder and then by place, and names each folder above the grid with how many of its Assets are shown.

**Filtered by name**: the browser shows only the Assets whose name contains the typed text, ignoring letter case and surrounding whitespace; an empty filter shows all.

**A grid of thumbnails**: each Asset the browser shows is a cell holding its thumbnail in a 128-point square with its name on one line beneath, in as many equal columns as the panel's width holds, at least one.

**Never enlarged on screen**: a thumbnail is drawn at most one physical pixel of the display per pixel of the thumbnail, keeping its proportions, and is scaled down only as far as it must be to fit the cell's square.

**Named and placed on hover**: hovering a cell shows the Asset's name in full, the Canonical Name of its Asset Folder, and its place in the folder.

**Chosen by a click**: clicking a cell chooses its Asset for placing.

**A placeholder until then**: an Asset whose thumbnail is not yet generated shows a neutral square placeholder; a generated thumbnail that is not yet decoded for display shows a neutral placeholder of the thumbnail's own proportions; a generated thumbnail that cannot be decoded for display shows the broken placeholder and is not loaded again until it is dropped from the decoded thumbnails kept.

**Shown as generated**: a thumbnail appears in the browser within a few frames of being generated, without any action from the Author.

**Placeholders are brief**: rows revealed by scrolling faster than the prefetch covers show placeholders only while the scrolling lasts; the visible rows are filled within a few frames of the last scroll input.

**Textures only while visible**: a thumbnail is registered with the interface only while its row is laid out, and is unregistered when its row leaves and in every frame the browser is not drawn, as when its tab is behind another; at most 512 decoded thumbnails are kept, the least recently shown dropped first, never one that is laid out.
_Why_: every registered texture costs a bind group per frame on Metal, and only dropping the decoded image frees its texture.

### Generating thumbnails

**Generated in the background**: generating thumbnails runs on threads of its own, never on the main thread; the browser, the viewport, and every Command stay responsive while it runs.

**The browser asks for its rows**: whenever the set of Assets in the browser's laid-out rows and the two rows either side changes, the browser names that set as wanted.

**Visible rows first**: the Assets last named as wanted that are still pending are generated before any other.

**Every image format**: a PNG, JPEG, or WebP Asset gets a thumbnail.

**Fitted, never enlarged**: a thumbnail keeps the image's proportions, is at most 128 pixels on its longer side, and is never larger than the image itself.

**Transparency is kept**: a thumbnail of an image with pixels that are not fully opaque keeps those pixels' transparency, and the panel shows through them.

**The first frame**: an animated image's thumbnail is of its first frame.

**As the viewport draws it**: a thumbnail shows the image as decoded, with no orientation metadata applied.
_Why_: the viewport draws the image the same way, so the thumbnail matches the Prop.

**Broken is a placeholder**: an Asset whose file cannot be decoded as an image, including one whose decoder panics, shows a broken placeholder, the editor keeps running, and the remaining thumbnails are still generated.

**Broken is remembered**: a file that could not be decoded is recorded as broken and not tried again until its size or modification time changes; a file that could not be read is not recorded and is tried again at the next start.
_Why_: a cloud copy not yet downloaded or a disk that was slow is a fault of the moment, and marking it broken would hide the Asset until the vendor touched the file.

### The thumbnail cache

**Kept across starts**: a thumbnail generated once is found at the next start and not generated again.

**A changed file gets a new thumbnail**: an Asset whose byte size or modification time differs from when its thumbnail was generated is generated again, and the old thumbnail is no longer served for it.

**Kept through undo and redo**: undoing and redoing Add Asset Folder keeps the folder's thumbnails, which are served again on redo without being generated again. Follows from: Every Command can be undone.

**Kept while a folder is away**: a remembered folder that cannot be scanned keeps its thumbnails, and they are served again when it can.

**Kept in the cache directory**: the thumbnail pack and its index live in the editor's cache directory, and generating thumbnails creates or changes no file inside an Asset Folder.

**Append-only**: generating a thumbnail only appends to the thumbnail pack and its index, and no entry is removed or rewritten; the one exception is opening the cache, which writes the index again without its torn records, replacing the file whole.
_Why_: measured at 2.4 KB per opaque and 22 KB per transparent thumbnail, a thumbnail pack serving 400,000 Assets stays within what a cache directory is for.

**Quitting stops generation**: quitting while thumbnails are generating ends the editor without waiting for the queue, and every thumbnail finished before then is kept.

**A torn record is skipped**: an index record that is incomplete or points beyond the end of the thumbnail pack is skipped when the cache is opened and the index is written again without it, so that every complete record is served, the skipped Asset is generated again, and what is appended afterwards is served at every later start.

**An unreadable cache starts afresh**: a thumbnail pack or index the editor cannot make sense of is replaced, with the other, by an empty one, every thumbnail is generated again, the Author is not told, and nothing crashes.

**An unwritable cache is reported**: when the thumbnail pack or its index cannot be opened or written, the Author is told once in the status line, with the file that failed, no thumbnail is generated for the rest of the session, the Assets without a thumbnail keep their placeholders, and the editor runs on.

## Implementation Decisions

- **Flow**: the Editor shows the platform's native folder dialog and then the Canonical Name prompt, and sends LibraryManager one AddFolder message carrying the path and the name. LibraryManager trims the name and checks, in this order: the name is not blank; the folder exists and can be listed once symbolic links, `.` components, and letter case the file system ignores are resolved; it is not already added and neither inside nor around an added folder; the name is not in use on this device, compared through CatalogEngine's Canonical Name equality, Unicode-normalised and in lower case. _Why_ the folder before the name: re-adding a folder under its own name is then told that the folder is already added, not that the name is taken. The step is then applied and recorded in the history, and answered with a FolderAdded message carrying what the scan skipped, or a FolderRefused message with its reason; a step whose Manifest cannot be written or whose scan fails after the checks passed is refused with the reason too, and its Manifest is forgotten again, so nothing is recorded. The Editor keeps the prompt open with the reason for a refusal about the name; any other refusal closes it and is shown in the status line, as is the count of skipped entries. The browser's one other request is a Browse message carrying the Assets it wants thumbnails for first, each named by its folder key and place (the model's AssetAddress, which also names the Asset a Place Element places); nothing answers it but the thumbnail states it changes.
- **The Manifest** lives in the editor's configuration directory: one JSON file per Asset Folder, named by its folder key, holding the key, the folder's path as the Author gave it, the Canonical Name, the version, and the renames (none). The version is the UTC day the folder was added. A Manifest is written through a sibling temporary file and a rename, so a reader never sees half of one. The set of folders in use on this device is the set of Manifests in that directory; forgetting a folder removes its Manifest and stops serving it. A JSON file in that directory that does not hold a Manifest, or whose name is not the key inside it, is logged and skipped; any other file there is ignored.
- **The folder key** is a device-local identifier minted when the folder is added. It names the Manifest, the index cache, and the `lib://` and `thumb://` source entries, and it is never written into a Project; Projects know a folder by its Canonical Name.
- **Scanning** is a synchronous, stat-only walk through LibraryAccess: directory entries and file metadata are read, nothing is opened. Hidden entries and symbolic links are left out silently; names that are not valid Unicode, subfolders that cannot be listed, and entries whose metadata cannot be read are counted as skipped; the folder itself being unlistable is an error. The scan lists every file found, Asset or not, ordered by place, and is diffed against the folder's index cache (added, removed, changed by size or modification time), after which the cache is rewritten. Generating thumbnails is a background job after the scan and is the first thing that opens an Asset's file. _Why_ every file: the cache knows files by their metadata alone; which of them are Assets is decided afterwards by classifying them.
- **The index cache** lives in the editor's cache directory under the folder key, as JSON. It is only a cache: one that is missing or cannot be read counts as empty, and undoing Add Asset Folder leaves it in place.
- **Classification** is CatalogEngine's Classify with one built-in Indexing Rule: the image extensions, compared ignoring case, give the image Asset Kind. A further rule is another Indexing Rule, not a change to the Engine. An indexed Asset carries its name (the file stem), its place (its relative path with `/` separators, spelled as on disk), its Asset Kind, its byte size, and its modification time.
- **The `lib://` source**: every Asset is loaded through one dynamic asset source, `lib://<folder-key>/<place>`, whose reader looks the key up in a table LibraryAccess changes at runtime as folders are added and forgotten; scanning registers the folder, forgetting removes it. The reader refuses any path that is not a plain relative path below a registered folder. The Host registers the source before the asset plugin builds, because asset sources freeze then, and turns `.meta` lookups off, since Asset Folders never hold them.
- **Directories**: LibraryAccess resolves the platform's configuration and cache directories; a resource in `model` overrides either, so tests and development builds point them at other directories.
- **Startup**: LibraryManager opens the thumbnail cache, then reads every Manifest, spawns an Asset Folder entity for each with an empty index, and Refreshes it, which is the same scan diffed against its cache. A folder whose scan fails keeps its entity and its Manifest and is reported with a FolderUnavailable message, which the Editor shows in the status line. Asset Folder Changed is sent as a message carrying the Canonical Name for each remembered folder at startup, whether or not its scan succeeded, after Add Asset Folder is applied, and after it is undone or redone; ProjectManager listens and resolves again the Asset References recorded against that name.
- **In the World**, each Asset Folder is an entity carrying its Canonical Name, folder key, path, version, index of Assets, and the counts of what the last scan skipped, written only by LibraryManager, and a `Thumbnails` component, also written only by LibraryManager, holding each Asset's thumbnail state in the index's order: pending, ready with the thumbnail's width and height in pixels, or broken. The Editor reads both to fill the browser and filters by name itself: folders are listed above the grid by Canonical Name with how many of their Assets match, and each Asset is a cell named by its name, with its folder's Canonical Name and its place on hover.
- **Undo and redo**: the recorded step holds the Manifest. Undoing forgets the Manifest, withdraws the folder's waiting Assets from the thumbnail generator, and despawns the entity; redoing writes the same Manifest again (same key, name, and version) and scans the folder again, which serves its thumbnails from the cache.
- **Ordering**: LibraryManager handles its Commands in the set `model` orders for every Manager, Commands before Undo before Redo within a frame, so a Command and the Undo sent in the same frame apply in the order the Author gave them.
- **LibraryAccess's Thumbnail contract**: the thumbnail cache, opened over the cache directory and the table the `thumb://` source reads; its `serve`, which makes the source serve an Asset's thumbnail as the Asset is now and says whether it is ready, broken, or pending; the generator, started over the cache, whose queue the caller enqueues to, puts wanted Assets at the front of, and withdraws a folder from, and which hands back completions; and the table's `read`, which gives the encoded thumbnail the source serves for an Asset. Opening it creates a `thumbnails` directory under the cache directory holding the thumbnail pack and its index.
- **The thumbnail pack and its index**: each starts with a 16-byte header of a magic and a version. The thumbnail pack holds the encoded thumbnails back to back; the index holds one 32-byte record per thumbnail generated: a 16-byte digest of the key (the folder key, the place, the byte size, and the modification time, hashed with BLAKE3), the entry's offset and length in the thumbnail pack, and the thumbnail's width and height. A record with a zero length says the Asset is broken. Both files are created when absent. The index is read whole when the cache opens; when a key has more than one record, the last one appended is served. A file whose header is not the one expected is replaced, with the other, by an empty one, and that is logged at `info`. A record that is incomplete or whose entry ends beyond the thumbnail pack is skipped and logged at `warn`, and the index is then written again through a sibling temporary file and a rename, as a Manifest is, so later appends line up and a skipped record can never come to point at a later entry. Entries are read with positional reads, never mapped into memory. _Why_ the digest in the record: the key is variable-length text, and a fixed record that carries its digest keeps the index self-contained and appendable, so the per-folder index cache is never rewritten for a thumbnail.
- **Making a thumbnail**: the file is read and decoded with the `image` crate under its default memory limit, its format told from the bytes; the first frame is taken, with no orientation applied, and downsampled with a box filter so its longer side is at most 128 pixels and never larger than the image; it is encoded as PNG when any pixel is not fully opaque and as JPEG at quality 85 otherwise. A file that is read but not decoded, or whose decoding panics, is broken and gets a broken record; a file that cannot be read gets no record, is logged at `warn`, and stays pending. Each broken file is logged at `debug` and their count at `info` when the queue drains.
- **The generator** runs on threads of its own, half the available cores and at least one, named `thumbnails-` and their number, so that generation never competes with the asset system's decoding of thumbnails for display; failing to start a single one is an error the caller reports. Its queue has a front the caller replaces at any time, served first, and the rest in the order enqueued; no Asset is handed out twice, and one with a record already served is passed over. Each thread makes one thumbnail at a time under `catch_unwind`, marked through the model's CaughtPanics resource as catching its own panics, so that a decoder's panic makes that Asset broken, is logged at `warn`, and leaves the thread serving the rest without a crash report. Dropping the generator stops its threads after the thumbnails in flight and writes out what they made.
- **Appending**: a thread appends a thumbnail's bytes and its record in memory, and they are written out when the queue drains, every 256 thumbnails, when the oldest unwritten one has waited 100 milliseconds, and when the generator stops. A write puts the bytes into the thumbnail pack, syncs it to disk, and only then writes the records to the index, so a crash leaves at worst bytes nothing refers to. Only after the write are the thumbnails served and their completions handed back, so a state the browser shows as ready is on disk. _Why_ 100 milliseconds: Shown as generated asks for a thumbnail within a few frames, and a write held back until 256 thumbnails would keep the first ones of a small folder waiting. The first failed write is handed back once as the cache's failure; the writer then writes nothing more, since the thumbnail pack's length on disk is no longer known and a later record could point at the wrong bytes, and the thumbnails not yet written are lost and generated again at a later start.
- **The `thumb://` source**: a second dynamic asset source, registered by the Host before the asset plugin builds, after `lib://`, whose reader serves the encoded bytes of the thumbnail served for `thumb://<folder-key>/<place>` from the table that `serve` and the generator keep of which record serves each Asset at its current size and modification time. A path with no thumbnail, a broken one included, is a missing asset, and the reader refuses any path `lib://` would refuse. The Editor loads a thumbnail with the image loader told to guess the format from the bytes and to keep the pixels in the render world only, so it is decoded in the asset system's load tasks, off the main thread. _Why_ through the asset server: the Editor depends on no ResourceAccess, so, as with a Prop's image through `lib://`, the asset server is its only way to a thumbnail's bytes, and it frees a texture when its last handle is dropped, which is what the decoded thumbnails kept rely on.
- **LibraryManager drives generation**: the thumbnail cache is opened and the generator started at startup before the remembered folders are restored. After a folder is indexed (Add Asset Folder, Refresh at startup, redo), the Manager serves every Asset's thumbnail through `serve`, writes the states into the folder's `Thumbnails`, and enqueues the pending ones in place order; a folder that cannot be scanned has nothing enqueued, and one that is undone is withdrawn, and their records stay in the thumbnail pack. Each frame, in the Commands set and before any Command is handled, the Manager drains the generator's completions into the states, only for an Asset still indexed with the size and modification time it was generated for, and then carries out the frame's last Browse, passing the wanted Assets that are still pending to the generator as the front of its queue; an earlier Browse of the same frame names Assets the browser no longer shows and is passed over. When the cache cannot be opened, the generator cannot start, or a write fails, the Manager sends one ThumbnailsUnavailable message with the reason and generates nothing more in the session. When the editor is asked to quit, the Manager drops the generator in the same frame.
- **The Editor's grid**: the browser lays out the Assets the filter matches in egui's virtual rows of one fixed height (the 128-point square and the name line), as many columns as the panel's width holds. Above the grid, one line per Asset Folder gives its Canonical Name and its count of matching Assets, with a word when the folder holds none or none match. The cell's state decides what it draws: pending draws the square placeholder; broken draws the broken placeholder; ready loads `thumb://<folder-key>/<place>` through the asset server if no handle is held, draws a placeholder of the recorded proportions until the image is loaded, the broken placeholder if it failed to load, and the image once it is loaded, registered with egui when it is first drawn. A thumbnail is drawn at the smaller of one point per physical pixel of the display and what fits the square. The two rows either side of those laid out are loaded ahead without being registered, and the laid-out and prefetched Assets are sent as Browse whenever they differ from those last sent. Handles live in a set of at most 512, the least recently shown dropped first, never one that is laid out; a texture is unregistered when its row leaves the laid-out range and, for all of them, in a frame in which the Assets panel is not drawn. ThumbnailsUnavailable is shown in the status line, as Failure shown in the status line prescribes.

## Test seams

The automated seam is a headless Bevy App of the real plugins of `model`, `history`, LibraryAccess, and LibraryManager over fixture folders in a temporary directory, with the editor's directories pointed at temporary ones, driven by messages and asserted on the World, the configuration directory, and the cache directory. `Add Asset Folder is undoable` and `Kept through undo and redo` run in the Host's test App, which also holds AuthoringManager and ProjectManager. _Why_ two Apps: the workspace check counts development dependencies, so the LibraryManager seam must not depend on another Manager. The thumbnail fixtures are written by the tests with the `image` crate, and thumbnails are read back through the table the `thumb://` source reads and decoded to assert on pixels.

- **Folder name proposed**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Blank names are refused**: `crates/drs-library-manager/tests/asset_folders.rs::blank_names_are_refused`
- **Names are unique on this device**: `crates/drs-library-manager/tests/asset_folders.rs::names_are_unique_on_this_device`
- **The same folder is refused**: `crates/drs-library-manager/tests/asset_folders.rs::the_same_folder_is_refused` (the symbolic link on Unix only; the letter case on macOS only, where the file system folds it)
- **Nested folders are refused**: `crates/drs-library-manager/tests/asset_folders.rs::nested_folders_are_refused`
- **Unreadable folders are refused**: `crates/drs-library-manager/tests/asset_folders.rs::unreadable_folders_are_refused`
- **Any path works**: `crates/drs-library-manager/tests/asset_folders.rs::any_path_works` (indexing), `crates/drs-app/tests/composing.rs::any_path_works` (placing and the asset path), `crates/drs-app/tests/projects.rs::any_path_works_for_assets` (saving and resolving on another device), `crates/drs-app/tests/export.rs::any_path_works_in_the_export` (loading and drawing)
- **Nothing is written into the folder**: `crates/drs-library-manager/tests/asset_folders.rs::nothing_is_written_into_the_folder`, `crates/drs-library-manager/tests/thumbnails.rs::kept_in_the_cache_directory` (generating thumbnails)
- **Scanning opens no file**: `crates/drs-library-manager/tests/asset_folders.rs::scanning_opens_no_file` (Unix only; skipped where the process may read a sealed file)
- **Images are Assets**: `crates/drs-library-manager/tests/asset_folders.rs::images_are_assets`
- **Hidden entries are skipped**: `crates/drs-library-manager/tests/asset_folders.rs::hidden_entries_are_skipped`
- **Links are not followed**: `crates/drs-library-manager/tests/asset_folders.rs::links_are_not_followed` (Unix only)
- **Unreadable names are skipped**: `crates/drs-library-manager/tests/asset_folders.rs::unreadable_names_are_skipped` (Linux only: Apple's file system refuses such a name, and Windows paths are always Unicode)
- **Unlistable subfolders are skipped**: `crates/drs-library-manager/tests/asset_folders.rs::unlistable_subfolders_are_skipped` (Unix only; skipped where the process may list a sealed folder)
- **Known by name and place**: `crates/drs-library-manager/tests/asset_folders.rs::known_by_name_and_place`
- **An empty folder is added**: `crates/drs-library-manager/tests/asset_folders.rs::an_empty_folder_is_added`
- **Cancelling records nothing**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Remembered across starts**: `crates/drs-library-manager/tests/asset_folders.rs::remembered_across_starts`
- **Current at start**: `crates/drs-library-manager/tests/asset_folders.rs::current_at_start`
- **A remembered folder that cannot be scanned stays known**: `crates/drs-library-manager/tests/asset_folders.rs::a_remembered_folder_that_cannot_be_scanned_stays_known`
- **Add Asset Folder is undoable**: `crates/drs-app/tests/asset_folders.rs::add_asset_folder_is_undoable`
- **Browsed across folders**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Filtered by name**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **A grid of thumbnails**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Never enlarged on screen**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script and its screenshots
- **Named and placed on hover**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Chosen by a click**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **A placeholder until then**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Shown as generated**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Placeholders are brief**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Textures only while visible**: by hand: no automated seam for the egui UI; verified with the dev-only input script, whose description of the grid counts the thumbnails kept and registered
- **Generated in the background**: `crates/drs-library-manager/tests/thumbnails.rs::generated_in_the_background` (the frame that indexes the folder returns with every state pending, later frames turn them ready, and a file is decoded on a generator thread; the thread check is left out when another logger already holds the process)
- **The browser asks for its rows**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Visible rows first**: `crates/drs-library-access/src/thumbnail.rs::tests::wanted_assets_come_first` (the generator's queue order, a function of LibraryAccess alone; completions on several threads arrive in no fixed order, so the App seam cannot assert it)
- **Every image format**: `crates/drs-library-manager/tests/thumbnails.rs::every_image_format`
- **Fitted, never enlarged**: `crates/drs-library-manager/tests/thumbnails.rs::fitted_never_enlarged` (a tall and a tiny fixture)
- **Transparency is kept**: `crates/drs-library-manager/tests/thumbnails.rs::transparency_is_kept`
- **The first frame**: `crates/drs-library-manager/tests/thumbnails.rs::the_first_frame` (an animated PNG fixture)
- **As the viewport draws it**: by hand: a JPEG with orientation metadata shows unrotated
- **Broken is a placeholder**: `crates/drs-library-manager/tests/thumbnails.rs::broken_is_a_placeholder`, `crates/drs-library-access/src/thumbnail.rs::tests::a_decoder_panic_is_broken_and_the_rest_carry_on` (a decoder that panics, on a thread marked as catching it)
- **Broken is remembered**: `crates/drs-library-manager/tests/thumbnails.rs::broken_is_remembered` (a second App over the same directories finds the broken state without opening the file; the half about a file that cannot be read runs on Unix only, and is skipped where the process may read a sealed file)
- **Kept across starts**: `crates/drs-library-manager/tests/thumbnails.rs::kept_across_starts`
- **A changed file gets a new thumbnail**: `crates/drs-library-manager/tests/thumbnails.rs::a_changed_file_gets_a_new_thumbnail`
- **Kept through undo and redo**: `crates/drs-app/tests/thumbnails.rs::kept_through_undo_and_redo`
- **Kept while a folder is away**: `crates/drs-library-manager/tests/thumbnails.rs::kept_while_a_folder_is_away`
- **Kept in the cache directory**: `crates/drs-library-manager/tests/thumbnails.rs::kept_in_the_cache_directory`
- **Append-only**: `crates/drs-library-manager/tests/thumbnails.rs::append_only`, `crates/drs-library-manager/tests/thumbnails.rs::a_torn_record_is_skipped` (the exception on opening)
- **Quitting stops generation**: `crates/drs-library-manager/tests/thumbnails.rs::quitting_stops_generation` (the editor is asked to quit mid-generation and stops within a bound of the time one thumbnail took, and a second App serves what was finished)
- **A torn record is skipped**: `crates/drs-library-manager/tests/thumbnails.rs::a_torn_record_is_skipped` (the index is cut mid-record and a record is pointed past the thumbnail pack by the test; a third start serves what the second appended)
- **An unreadable cache starts afresh**: `crates/drs-library-manager/tests/thumbnails.rs::an_unreadable_cache_starts_afresh` (a damaged index and a damaged thumbnail pack)
- **An unwritable cache is reported**: `crates/drs-library-manager/tests/thumbnails.rs::an_unwritable_cache_is_reported` (opening; Unix only, and skipped where the process may write a sealed folder), `crates/drs-library-manager/tests/thumbnails.rs::a_failed_write_is_reported_once` (a write after opening, in a child process whose files may not grow; Unix only)

## Not supported

- The editor never writes into an Asset Folder: no Manifest, cache, thumbnail, or sidecar file of any kind.
- The folder key never leaves the device: a Project never records it.
- Thumbnails honour no orientation metadata in an image, as the viewport does not.

## Notes

- Fourteen Rules (Folder name proposed, Cancelling records nothing, Browsed across folders, Filtered by name, A grid of thumbnails, Never enlarged on screen, Named and placed on hover, Chosen by a click, A placeholder until then, Shown as generated, Placeholders are brief, Textures only while visible, The browser asks for its rows, As the viewport draws it) have no automated test, against the requirement that every Rule has one. All but the last are behaviour of the egui interface alone, for which no headless seam exists, and the last depends on an image the tests do not write; the accepted deviation is verification by hand, driving the editor with the development-only input script and its screenshots.
- Development builds take the folder from an environment variable instead of opening the dialog, drive the editor from a script of input steps whose description lists the grid's cells with what each shows and its rectangle, save a screenshot on request, and keep the editor's own files under a directory of choice. That is tooling for verification, not behaviour of the capability.
- "Thumbnail" is plain English for a small picture of an image Asset; it is presentation, not a domain concept, and carries no invariant. "Thumbnail pack" is the cache file and is not an Asset Pack.
- Generating thumbnails opens every file of an Asset Folder once; on an online-only cloud folder that means downloading the library once, which the architecture accepts for background jobs.
- egui lays out scroll offsets in single-precision floats, which hold whole pixels only to about 16.7 million pixels of content: about 110,000 rows of the grid, or 660,000 Assets at six columns. Larger libraries scroll less precisely; that is an accepted limitation.
- The cell is 128 points because that is what the spike measured (2.4 KB per opaque thumbnail, 60 µs to decode) and what the functional baseline shows at its default; the architecture's figures hold for it.
