# Asset Folders

**Commands**: Add Asset Folder, Remove Asset Folder, Rename Canonical Name, Install Asset Pack

## Purpose

An Author owns folders full of Assets, from vendors and of their own making, and uses them where they already are. This capability lets the Author tell the editor where such a folder is, gives it the Canonical Name Projects know it by, indexes its Assets without touching the folder, and shows them as a grid of thumbnails so an Asset can be found by sight, or by searching the whole library for a piece of its name as fast as the Author types. Folders added once stay available on this device and follow what changes in them, and their thumbnails are generated once per device and kept outside the folders.

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
19. As an Author, I can hover a cell and read the Asset's name in full, the Canonical Name of its folder, and its place in the folder, so that two files with the same name in different subfolders are told apart.
20. As an Author, I can click a cell to choose the Asset to place, so that the grid is where placing starts.
21. As an Author, I can find the folders I added the next time I open the editor, so that I add each folder once per device.
22. As an Author, I can rely on Assets added to or removed from a folder while the editor was closed appearing or disappearing at the next start, so that a vendor update is picked up.
23. As an Author, I can see a remembered folder whose disk is unplugged still listed, empty and with a report, so that it is back as soon as the disk is.
24. As an Author, I can undo adding a folder and redo it without being asked its Canonical Name again, so that adding a folder is a step like any other.

### Searching

25. As an Author, I can type part of an Asset's name and see every Asset whose name contains it, so that I find an Asset without knowing which folder or subfolder it sits in.
26. As an Author, I can rely on the search ignoring letter case, so that `barrel`, `Barrel`, and `BARREL` find the same Assets.
27. As an Author, I can rely on letters that change length when their case changes matching as well, so that `STRASSE` finds `Straße`.
28. As an Author, I can find an accented name however its accent is stored, composed or decomposed, so that names unpacked from an archive or copied from another operating system match what I type.
29. As an Author, I can rely on spaces before and after what I type being ignored, so that a stray space never hides every match.
30. As an Author, I can type several words and see only the Assets whose name contains every one of them, in any order, so that `table oak` narrows to oak tables.
31. As an Author, I can type a word containing `/` to match the folder's Canonical Name and the subfolders an Asset sits in, so that `furniture/table` finds the tables in a furniture subfolder and `forgotten/barrel` the barrels of one vendor.
32. As an Author, I see Assets in which what I typed begins a word of the name listed before those in which it only appears inside a word, so that `bed` lists `Bed_Double` before `Flowerbed`.
33. As an Author, I see equally good matches listed in order of name, so that I can scan them like an index.
34. As an Author, I see two Assets with the same name in different Asset Folders listed in order of their folders' Canonical Names, and two in the same folder in order of place, so that the order never depends on chance.
35. As an Author who shares a library with a collaborator who keeps it at another path, I can rely on the same text listing the same Assets in the same order on both our devices, so that "the third barrel" means the same Asset to both of us.
36. As an Author, I see the matches change with every keystroke, without pressing Enter, so that I see the effect of each letter as I type it.
37. As an Author, I can see how many Assets match, so that I know whether to keep narrowing.
38. As an Author, I can see how many Assets the library holds when nothing is typed, so that the count is always there.
39. As an Author, I can see each Asset Folder above the grid with how many of its Assets match, so that I know where the matches come from.
40. As an Author, I see a message naming what I typed when nothing matches, so that I know the search ran and found nothing, rather than facing an empty panel.
41. As an Author, I get every Asset back, ordered by folder and place as without a search, when I clear the field, so that browsing is one keystroke away.
42. As an Author, I see a new search's matches from the top, so that the best matches are the ones I see first.
43. As an Author, I can choose an Asset from the matches with one click, as from the whole library, so that searching is the way to pick an Asset.
44. As an Author, I keep the Asset I chose while I change the text, even when it no longer matches, so that searching for the next Asset never loses the one I am placing.
45. As an Author with 400,000 Assets, I get each keystroke's matches before I type the next, so that searching never feels slower than typing.
46. As an Author with 400,000 Assets, I can scroll through every match of a broad search as smoothly as through the whole library, so that a search matching most of the library is no slower to browse.
47. As an Author with a large library, I open the editor with no more time spent on the search than indexing the folders already takes, so that search never becomes the thing I wait for at start.
48. As an Author with text typed in the search field, I see the matching Assets of a folder I add join the matches as soon as it is added, without typing again, so that a new folder is searchable at once.
49. As an Author, I see the Assets of a folder whose adding I undo leave the matches at once, and return when I redo it, so that the matches always show what is available.
50. As an Author, I can find the Assets added to a folder while the editor was closed, and no longer find those removed, at the next start, so that a vendor update is searchable.
51. As an Author, I get no matches from a remembered folder whose disk is unplugged while it stays listed, so that the search never offers an Asset that cannot be placed.
52. As an Author with an empty library, I am told by the browser to add an Asset Folder rather than that nothing matches, so that the advice fits the situation.

### Seeing thumbnails

53. As an Author, I can see every Asset in the browser as a thumbnail with its name beneath, in a grid as wide as the panel, so that I recognise an Asset by sight instead of by name.
54. As an Author, I can watch thumbnails appear in the browser as they are generated, without doing anything, so that a freshly added folder fills in while I keep working.
55. As an Author, I get the rows I am looking at thumbnailed before the rest of the library, so that a 400,000-file folder is useful within seconds of being added.
56. As an Author, I see a neutral placeholder in the cell of an Asset whose thumbnail is not yet generated, so that the grid holds its shape while the thumbnails fill in.
57. As an Author, I see a placeholder of a thumbnail's own proportions for the brief moment before a generated thumbnail appears, so that nothing jumps when it does.
58. As an Author, I can scroll the grid as fast as I like and see placeholders at most briefly, with the visible rows filled a few frames after I stop, so that scrolling never waits for the thumbnails.
59. As an Author, I find the editor as responsive as ever while thumbnails are being generated, whether I scroll, place Props, or pan the Level, so that generation is something I never notice.
60. As an Author, I get thumbnails for PNG, JPEG, and WebP Assets alike, so that the whole library is covered whatever format a vendor ships.
61. As an Author, I see the transparent parts of an image stay transparent in its thumbnail, so that a cut-out Prop looks like a cut-out and not like a square.
62. As an Author, I see a tall or wide image keep its proportions inside the cell, neither stretched nor cropped, so that a long fence reads as a fence.
63. As an Author, I see an image smaller than the cell at its own size and not blown up, even on a dense display, so that a 32-pixel token stays crisp.
64. As an Author, I see a broken placeholder for a file that is not a readable image while the editor and the rest of the thumbnails carry on, so that one bad file never costs me the library or the session.

### Keeping thumbnails

65. As an Author, I find the thumbnails generated today there the next time I open the editor without their being generated again, so that a library is thumbnailed once per device.
66. As an Author, I get a new thumbnail at the next start for a file that a vendor update replaced, so that the picture is never of the old version.
67. As an Author, I keep the thumbnails of a remembered folder whose disk is unplugged, so that they are back with the folder as soon as the disk is.
68. As an Author, I keep a folder's thumbnails through undoing and redoing Add Asset Folder, so that undo is as cheap as any other step.
69. As an Author, I can rely on the thumbnails being kept in the editor's cache directory and never inside my Asset Folder, so that a read-only or cloud-synced folder stays untouched.
70. As an Author, I can quit the editor while thumbnails are still being generated and have it quit at once, so that generation never holds me hostage.
71. As an Author, I keep the thumbnails finished before I quit, and generation carries on from there at the next start, so that quitting never throws work away.
72. As an Author, I can rely on a thumbnail cache the editor left incomplete, because it quit or crashed mid-write, being used for what is complete and regenerated for the rest, so that a bad moment never means starting over.
73. As an Author, I can rely on a thumbnail cache the editor cannot make sense of being started afresh with no fuss, so that a damaged cache is an inconvenience and not an error.
74. As an Author, I am told when thumbnails cannot be kept, because the cache directory cannot be written, while the editor runs on with placeholders, so that a full disk never stops me from composing.
75. As an Author, I can rely on a file that could not be read for a moment, because its disk was slow or its cloud copy was not yet local, being tried again at the next start rather than marked broken for good, so that a transient fault never leaves a permanent hole.
76. As an Author, I can rely on a file that is genuinely not an image being remembered as broken, so that the editor does not try it again at every start.
77. As an Author with a 400,000-file library, I open the editor with every thumbnail kept as quickly as without them, so that the cache never becomes the thing I wait for.
78. As an Author with a 400,000-file library, I can rely on the editor's memory staying bounded however far I scroll, so that browsing a big library never costs more than a few dozen megabytes.

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

**Browsed across folders**: with nothing typed in the search field, the browser shows the Assets of every added Asset Folder in one grid of thumbnails, ordered by the Canonical Name of their folder (compared Unicode-normalised and case-folded first, and as spelled to break a tie) and then by place, and names each folder above the grid, in the same order, with how many of its Assets are shown.

**A grid of thumbnails**: each Asset the browser shows is a cell holding its thumbnail in a 128-point square with its name on one line beneath, in as many equal columns as the panel's width holds, at least one.

**Never enlarged on screen**: a thumbnail is drawn at most one physical pixel of the display per pixel of the thumbnail, keeping its proportions, and is scaled down only as far as it must be to fit the cell's square.

**Named and placed on hover**: hovering a cell shows the Asset's name in full, the Canonical Name of its Asset Folder, and its place in the folder.

**Chosen by a click**: clicking a cell chooses its Asset for placing.

**A placeholder until then**: an Asset whose thumbnail is not yet generated shows a neutral square placeholder; a generated thumbnail that is not yet decoded for display shows a neutral placeholder of the thumbnail's own proportions; a generated thumbnail that cannot be decoded for display shows the broken placeholder and is not loaded again until it is dropped from the decoded thumbnails kept.

**Shown as generated**: a thumbnail appears in the browser within a few frames of being generated, without any action from the Author.

**Placeholders are brief**: rows revealed by scrolling faster than the prefetch covers show placeholders only while the scrolling lasts; the visible rows are filled within a few frames of the last scroll input.

**Textures only while visible**: a thumbnail is registered with the interface only while its row is laid out, and is unregistered when its row leaves and in every frame the browser is not drawn, as when its tab is behind another; at most 512 decoded thumbnails are kept, the least recently shown dropped first, never one that is laid out.
_Why_: every registered texture costs a bind group per frame on Metal, and only dropping the decoded image frees its texture.

### Searching

**Words of the text**: the typed text is split at whitespace into words; whitespace before, between, and after them is ignored.

**Matched by part of a name**: an Asset matches a word without `/` when its name contains the word, both compared Unicode-normalised and fully case-folded, so that `barrel` finds `Old_BARREL` and `STRASSE` finds `Straße`.

**Accents however stored**: a word spelled with composed accents matches a name spelled with decomposed ones and the reverse; an unaccented letter does not match an accented one, whether the accent is composed with the letter or a combining mark after it.

**Every word must match**: with several words, an Asset matches only when it matches each of them, in any order; two words may match overlapping parts of the name.

**A word with a slash matches the path**: a word that contains `/` is matched, as a name is, against the Asset's path in the library instead of its name: its folder's Canonical Name, a `/`, and its place without the extension.

**An empty text matches everything**: a text with no words matches every Asset of every added Asset Folder.

**Word starts rank first**: an Asset in which every word occurs at least once where a word of the name (or of the path, for a word with `/`) begins ranks above an Asset in which some word occurs only inside a word; a word of the name begins at its start or after a character that is neither a letter nor a digit, a combining mark counting as part of the letter before it.

**A deterministic order**: matches are ordered by rank, then by name, then by their folder's Canonical Name, then by place, each compared Unicode-normalised and case-folded first and as spelled to break a tie; the order depends on nothing but the text and the folders' Canonical Names and indexes, so it is the same on every device.

**Every keystroke searches**: each change of the typed text is searched without the Author confirming it, and the matches for it are shown in the frame after the change.

**Answered within a keystroke**: over 400,000 Assets in the test build, building the search and answering the first text takes under 1 second, and answering each text after it under 50 milliseconds.

**Matches follow the library**: the matches for the typed text include a folder's matching Assets from the frame Add Asset Folder is applied or redone, lose them from the frame it is undone, and reflect at start the Assets added to or removed from a remembered folder while the editor was closed, without the text being typed again. Follows from: Every Command can be undone.

**Only available Assets match**: a remembered folder that cannot be scanned contributes no matches.

**Matches in the grid**: with text typed, the grid holds the matching Assets in the order A deterministic order gives, and lays out only the rows in view, however many match.

**Counted**: the browser shows how many Assets match the typed text, or, with nothing typed, how many Assets the library holds; the line above the grid for each Asset Folder gives how many of its Assets match.

**No match is said**: when Asset Folders are added and no Asset matches, the browser says that no Asset matches, quoting the typed text; with no Asset Folder added it says to add one.

**A new text starts at the top**: when the typed text changes, the grid shows its matches from the first row.

**The chosen Asset stays chosen**: changing the typed text never changes which Asset is chosen for placing, whether or not it still matches.

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

**Broken is remembered**: a file that could not be decoded is recorded as broken and not tried again until its size or modification time changes; a file that could not be read is not recorded, is not read again in the same session, and is tried again at the next start.
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

- **Flow**: the Editor shows the platform's native folder dialog and then the Canonical Name prompt, and sends LibraryManager one AddFolder message carrying the path and the name. LibraryManager trims the name and checks, in this order: the name is not blank; the folder exists and can be listed once symbolic links, `.` components, and letter case the file system ignores are resolved; it is not already added and neither inside nor around an added folder; the name is not in use on this device, compared through CatalogEngine's Canonical Name equality, Unicode-normalised and in lower case. _Why_ the folder before the name: re-adding a folder under its own name is then told that the folder is already added, not that the name is taken. The step is then applied and recorded in the history, and answered with a FolderAdded message carrying what the scan skipped, or a FolderRefused message with its reason; a step whose Manifest cannot be written or whose scan fails after the checks passed is refused with the reason too, and its Manifest is forgotten again, so nothing is recorded. The Editor keeps the prompt open with the reason for a refusal about the name; any other refusal closes it and is shown in the status line, as is the count of skipped entries. The browser's one other request is a Browse message carrying the text typed in its search field and the Assets it wants thumbnails for first, each named by its folder key and place (the model's AssetAddress, which also names the Asset a Place Element places); the text is answered in the search's matches, and the wanted Assets change only which pending Assets are generated first.
- **The Manifest** lives in the editor's configuration directory: one JSON file per Asset Folder, named by its folder key, holding the key, the folder's path as the Author gave it, the Canonical Name, the version, and the renames (none). The version is the UTC day the folder was added. A Manifest is written through a sibling temporary file and a rename, so a reader never sees half of one. The set of folders in use on this device is the set of Manifests in that directory; forgetting a folder removes its Manifest and stops serving it. A JSON file in that directory that does not hold a Manifest, or whose name is not the key inside it, is logged and skipped; any other file there is ignored.
- **The folder key** is a device-local identifier minted when the folder is added. It names the Manifest, the index cache, and the `lib://` and `thumb://` source entries, and it is never written into a Project; Projects know a folder by its Canonical Name.
- **Scanning** is a synchronous, stat-only walk through LibraryAccess: directory entries and file metadata are read, nothing is opened. Hidden entries and symbolic links are left out silently; names that are not valid Unicode, subfolders that cannot be listed, and entries whose metadata cannot be read are counted as skipped; the folder itself being unlistable is an error. The scan lists every file found, Asset or not, ordered by place, and is diffed against the folder's index cache (added, removed, changed by size or modification time), after which the cache is rewritten. Generating thumbnails is a background job after the scan and is the first thing that opens an Asset's file. _Why_ every file: the cache knows files by their metadata alone; which of them are Assets is decided afterwards by classifying them.
- **The index cache** lives in the editor's cache directory under the folder key, as JSON. It is only a cache: one that is missing or cannot be read counts as empty, and undoing Add Asset Folder leaves it in place.
- **Classification** is CatalogEngine's Classify with one built-in Indexing Rule: the image extensions, compared ignoring case, give the image Asset Kind. A further rule is another Indexing Rule, not a change to the Engine. An indexed Asset carries its name (the file stem), its place (its relative path with `/` separators, spelled as on disk), its Asset Kind, its byte size, and its modification time.
- **The `lib://` source**: every Asset is loaded through one dynamic asset source, `lib://<folder-key>/<place>`, whose reader looks the key up in a table LibraryAccess changes at runtime as folders are added and forgotten; scanning registers the folder, forgetting removes it. The reader refuses any path that is not a plain relative path below a registered folder. The Host registers the source before the asset plugin builds, because asset sources freeze then, and turns `.meta` lookups off, since Asset Folders never hold them.
- **Directories**: LibraryAccess resolves the platform's configuration and cache directories; a resource in `model` overrides either, so tests and development builds point them at other directories.
- **Startup**: LibraryManager opens the thumbnail cache, then reads every Manifest, spawns an Asset Folder entity for each with an empty index, and Refreshes it, which is the same scan diffed against its cache. A folder whose scan fails keeps its entity, its Manifest, and an empty search, and is reported with a FolderUnavailable message, which the Editor shows in the status line. Asset Folder Changed is sent as a message carrying the Canonical Name for each remembered folder at startup, whether or not its scan succeeded, after Add Asset Folder is applied, and after it is undone or redone; ProjectManager listens and resolves again the Asset References recorded against that name.
- **In the World**, each Asset Folder is an entity carrying its Canonical Name, folder key, path, version, index of Assets, and the counts of what the last scan skipped, written only by LibraryManager, and a `Thumbnails` component, also written only by LibraryManager, holding each Asset's thumbnail state in the index's order: pending, ready with the thumbnail's width and height in pixels, or broken. The Editor reads both to fill the browser, and the search's matches for what the browser shows with text typed; it never matches names itself. Each Asset is a cell named by its name, with its folder's Canonical Name and its place on hover.
- **Undo and redo**: the recorded step holds the Manifest. Undoing forgets the Manifest, withdraws the folder's waiting Assets from the thumbnail generator, and despawns the entity; redoing writes the same Manifest again (same key, name, and version) and scans the folder again, which serves its thumbnails from the cache.
- **Ordering**: LibraryManager handles its Commands, and Browse, in the set `model` orders for every Manager, Commands before Undo before Redo within a frame, so a Command and the Undo sent in the same frame apply in the order the Author gave them; it answers the search text in a system ordered after Redo.
- **LibraryAccess's Thumbnail contract**: the thumbnail cache, opened over the cache directory and the table the `thumb://` source reads; its `serve`, which makes the source serve an Asset's thumbnail as the Asset is now and says whether it is ready, broken, or pending; the generator, started over the cache, whose queue the caller enqueues to, puts wanted Assets at the front of, and withdraws a folder from, and which hands back completions; and the table's `read`, which gives the encoded thumbnail the source serves for an Asset. Opening it creates a `thumbnails` directory under the cache directory holding the thumbnail pack and its index.
- **The thumbnail pack and its index**: each starts with a 16-byte header of a magic and a version. The thumbnail pack holds the encoded thumbnails back to back; the index holds one 32-byte record per thumbnail generated: a 16-byte digest of the key (the folder key, the place, the byte size, and the modification time, hashed with BLAKE3), the entry's offset and length in the thumbnail pack, and the thumbnail's width and height. A record with a zero length says the Asset is broken. Both files are created when absent. The index is read whole when the cache opens; when a key has more than one record, the last one appended is served. A file whose header is not the one expected is replaced, with the other, by an empty one, and that is logged at `warn` when either file was there. A record that is incomplete or whose entry ends beyond the thumbnail pack is skipped and logged at `warn`, and the index is then written again through a sibling temporary file and a rename, as a Manifest is, so later appends line up and a skipped record can never come to point at a later entry. Entries are read with positional reads, never mapped into memory. _Why_ the digest in the record: the key is variable-length text, and a fixed record that carries its digest keeps the index self-contained and appendable, so the per-folder index cache is never rewritten for a thumbnail.
- **Making a thumbnail**: the file is read and decoded with the `image` crate under its default memory limit, its format told from the bytes; the first frame is taken, with no orientation applied, and downsampled with a box filter so its longer side is at most 128 pixels and never larger than the image; it is encoded as PNG when any pixel is not fully opaque and as JPEG at quality 85 otherwise. A file that is read but not decoded, or whose decoding panics, is broken and gets a broken record; a file that cannot be read gets no record, is logged at `warn`, and stays pending; the generator passes it over for the rest of the session, however often it is enqueued or named as wanted, so it is read again at the next start and not before. Each broken file is logged at `debug` and their count at `info` when the queue drains and when generation stops.
- **The generator** runs on threads of its own: half the available cores and at least one, named `thumbnails-` and their number, that make thumbnails, so that generation never competes with the asset system's decoding of thumbnails for display, and one more, `thumbnails-timer`, that writes out what they made when it is due. Starting not a single thread that makes thumbnails is an error the caller reports, and those that do start serve the queue; the timer thread failing to start is logged at `warn`, and what is made is then written out only as each thumbnail finishes. Its queue has a front the caller replaces at any time, served first, and the rest in the order enqueued; no Asset is handed out twice, and one with a record already served is passed over. Each thread makes one thumbnail at a time catching any panic, marked through the model's CaughtPanics resource as catching its own panics, so that a decoder's panic makes that Asset broken, is logged at `warn`, and leaves the thread serving the rest without a crash report. Dropping the generator stops its threads after the thumbnails in flight and writes out what they made.
- **Appending**: a thread appends a thumbnail's bytes and its record in memory, and they are written out when the queue has drained, when 256 are unwritten, or when the oldest unwritten one has waited 100 milliseconds; they are also written out when the generator stops. Each thread that makes thumbnails decides this when it finishes one, and the timer thread sleeps until the oldest unwritten thumbnail is due, or while none is unwritten until a thread appends one, so a thumbnail finished while every other thread is busy with a slow file is written out on time. A write puts the bytes into the thumbnail pack, syncs it to disk, and only then writes the records to the index, so a crash leaves at worst bytes nothing refers to. Only after the write are the thumbnails served and their completions handed back, so a state the browser shows as ready is on disk. _Why_ 100 milliseconds: Shown as generated asks for a thumbnail within a few frames, and a write held back until 256 thumbnails would keep the first ones of a small folder waiting. The first failed write is handed back once as the cache's failure; the writer then writes nothing more, since the thumbnail pack's length on disk is no longer known and a later record could point at the wrong bytes, and the thumbnails not yet written are lost and generated again at a later start.
- **The `thumb://` source**: a second dynamic asset source, registered by the Host before the asset plugin builds, after `lib://`, whose reader serves the encoded bytes of the thumbnail served for `thumb://<folder-key>/<place>` from the table that `serve` and the generator keep of which record serves each Asset at its current size and modification time. A path with no thumbnail, a broken one included, is a missing asset, and the reader refuses any path `lib://` would refuse. The Editor loads a thumbnail with the image loader told to guess the format from the bytes and to keep the pixels in the render world only, so it is decoded in the asset system's load tasks, off the main thread. _Why_ through the asset server: the Editor depends on no ResourceAccess, so, as with a Prop's image through `lib://`, the asset server is its only way to a thumbnail's bytes, and it frees a texture when its last handle is dropped, which is what the decoded thumbnails kept rely on.
- **LibraryManager drives generation**: the thumbnail cache is opened and the generator started at startup before the remembered folders are restored. After a folder is indexed (Add Asset Folder, Refresh at startup, redo), the Manager serves every Asset's thumbnail through `serve`, writes the states into the folder's `Thumbnails`, and enqueues the pending ones in place order; a folder that cannot be scanned has nothing enqueued, and one that is undone is withdrawn, and their records stay in the thumbnail pack. Each frame, in the Commands set and before it handles any of its own requests, the Manager drains the generator's completions into the states, only for an Asset still indexed with the size and modification time it was generated for, and then carries out the frame's last Browse, passing the wanted Assets that are still pending to the generator as the front of its queue; an earlier Browse of the same frame names Assets the browser no longer shows and is passed over. When the cache cannot be opened, the Manager sends one ThumbnailsUnavailable message with the reason and neither serves nor generates a thumbnail in the session; when the cache opens but no thread of the generator starts, it sends one with the reason, keeps the cache, and serves the thumbnails it holds while generating none in the session; when a write fails, it sends one with the reason and generates nothing more in the session, while the thumbnails already served stay served. When the editor is asked to quit, the Manager drops the generator in the same frame.
- **The Editor's grid**: the browser lays out the Assets the grid holds (every Asset with nothing typed, the matches with text typed) in egui's virtual rows of one fixed height (the 128-point square and the name line), as many columns as the panel's width holds. Above the grid, one line per Asset Folder, in the order the search gives Canonical Names, gives its Canonical Name and its count of Assets shown, with a word when the folder holds none or none match. The cell's state decides what it draws: pending draws the square placeholder; broken draws the broken placeholder; ready loads `thumb://<folder-key>/<place>` through the asset server if no handle is held, draws a placeholder of the recorded proportions until the image is loaded, the broken placeholder if it failed to load, and the image once it is loaded, registered with egui when it is first drawn. A thumbnail is drawn at one physical pixel of the display per pixel of the thumbnail, scaled down, keeping its proportions, only when that would not fit the square. The two rows either side of those laid out are loaded ahead without being registered, and the laid-out and prefetched Assets are sent as Browse whenever they differ from those last sent. Handles live in a set of at most 512, the least recently shown dropped first, never one that is laid out; a texture is unregistered when its row leaves the laid-out range and, for all of them, in a frame in which the Assets panel is not drawn. ThumbnailsUnavailable is shown in the status line, as Failure shown in the status line prescribes.
- **CatalogEngine's Search contract**: one `LibrarySearch` value holds the search of every added Asset Folder. Its Build turns one folder's key, Canonical Name, and index into that folder's search, in place of the one it had, and Remove drops it; neither touches another folder's search. A folder's search holds its Assets in search order (by name, then by place, each compared folded and then as spelled), and for each its name and its path in the library (the Canonical Name, a `/`, and the place without the extension), both Unicode-normalised (NFC), fully case-folded, and normalised again, as Resolve folds places, laid out back to back in two contiguous texts, one for names and one for paths, each row ended by a NUL byte, with each row's start and each Asset's position in the index as `u32`. The folders are kept in the order of their Canonical Names, folded and then as spelled. A folder whose folded names or paths would pass 4 GiB is searched as far as they fit. Plain data with no Bevy.
- **Search** answers a text: whether it holds a word, each folder with its count of matches, in Canonical Name order, and the matches in order, each as the folder's position among them and the Asset's position in its index. Each word is folded as names are. In each folder the longest word is found by a SIMD substring scan of the names' or the paths' text (`memchr`'s `memmem`), each hit mapped back to its row, and the other words looked for within that row; whether an occurrence begins a word is read from the character before it, passing over combining marks, which belong to the letter they follow, and an occurrence followed by a combining mark ends inside a letter and does not count, so `x` never matches an `x` with a combining acute. A combining mark is any of Unicode's marks, the combining grapheme joiner (U+034F) included, although it combines with nothing and its combining class of zero leaves it out of composing. Hits come out in the folder's search order, so each folder yields its matches already in order within each rank. When more than one folder has matches of a rank, their lists are merged through a heap of their heads by the place of each Asset among those of every folder; that order, by name folded, then as spelled (read from the folders' indexes, which the caller passes in, rather than kept), then by Canonical Name, then in search order, is worked out by the same kind of heap merge on the first search that needs it after a folder's search is built or removed, and kept until the next. A text without words lists no match and counts every Asset of each folder, so no list of the whole library is built. A word holding a NUL byte matches nothing. _Why_ no fst: the rank comes from where the substring scan hits, which an fst over whole names cannot tell, and its substring stream measured 80–190 ms over 400,000 names.
- **Cost** for 400,000 Assets: about 30 MiB in memory (the two folded texts, their row starts, the positions, and, once worked out, the order). In the test build on the development machine, building the search of 400,000 vendor-shaped names and answering the first text took about 0.5 s; a narrow text was answered in about 0.2 ms, a multi-word, a slash, and an accented text in 1–2 ms, and a one-letter text matching 264,000 Assets in about 18 ms.
- **LibraryManager keeps the search**: one `LibrarySearch` in a resource of LibraryManager's own, beside the text the browser last sent. A folder's search is built in the same step that writes its index: Add Asset Folder applied or redone, and Refresh at start; undoing drops it as the entity is despawned. A remembered folder whose scan fails at start has an empty index and therefore an empty search. The search is never written to disk: it is built again from the indexes at each start, so it adds no file to the editor's directories and can never disagree with an index. _Why_ synchronous: the scan in the same step already costs more (about 1 s for 400,000 files), and building in the background would leave frames of stale matches.
- **Answering**: handling Browse in the Commands set, LibraryManager takes the search text it carries; in a system ordered after Redo, in any frame in which the text changed or a folder's search was built or dropped, it runs Search over the folders' indexes and writes the answer into SearchMatches, a `model` resource written only by LibraryManager: the text answered, whether it holds a word, the total count, each folder's entity with its count of matches in Canonical Name order, and the matches as the folder's entity and the Asset's position in its index. With a text of no words the total is the sum of the folders' index sizes and no match is listed. Answering after Redo makes a folder added, undone, redone, or refreshed change the matches in that same frame.
- **The Editor's search field** keeps its text in the Editor's own state and sends it with every Browse, which the browser sends whenever the text or the wanted Assets differ from what it last sent, an empty text included. With a word typed and an answer holding a word, the grid lays out the matches of SearchMatches in its order, through the same virtual rows, and the wanted set is the matches in the laid-out and prefetched rows; until the answer to a changed text arrives, a frame later, the previous answer is shown, or the whole library when there was none. With nothing typed, the grid shows every Asset straight from the folders' indexes, in the order the search gives Canonical Names and then by place. Above the folder lines a line gives the count (`12 Assets match`, `400,000 Assets`) or `No Asset matches “…”`, quoting the answered text trimmed; with no folder added the advice to add one stays. A typed change, or a different answered text shown, resets the grid's scroll offset to the top. The chosen Asset is kept in the Editor's state independently of the matches.

## Test seams

The automated seam is a headless Bevy App of the real plugins of `model`, `history`, LibraryAccess, and LibraryManager over fixture folders in a temporary directory, with the editor's directories pointed at temporary ones, driven by messages and asserted on the World, the configuration directory, and the cache directory. `Add Asset Folder is undoable` and `Kept through undo and redo` run in the Host's test App, which also holds AuthoringManager and ProjectManager. _Why_ two Apps: the workspace check counts development dependencies, so the LibraryManager seam must not depend on another Manager. The thumbnail fixtures are written by the tests with the `image` crate, and thumbnails are read back through the table the `thumb://` source reads and decoded to assert on pixels. The search seam of that App sends Browse with a text and asserts on SearchMatches; `Matches follow the library` runs its undo and redo in the Host's test App. Matching, ranking, folding, and order are a function of CatalogEngine alone, so they are tested on its Search directly over searches built from synthetic names; `Answered within a keystroke` runs alone, with no other test beside it, so that the bound measures the code and not the machine's load. What the generator alone decides (the queue's order, a panicking decoder, writing out on time, an unreadable file) is tested as unit tests of LibraryAccess, and serving a kept cache without a generator as a unit test of LibraryManager, since no App can make a thread fail to start.

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
- **A grid of thumbnails**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Never enlarged on screen**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script and its screenshots
- **Named and placed on hover**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Chosen by a click**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **A placeholder until then**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Shown as generated**: `crates/drs-library-access/src/thumbnail.rs::tests::written_out_while_every_thread_is_busy` (a thumbnail finished while the only thread is busy with a slow file is handed back without waiting for it); the browser's drawing by hand, driving the editor with the dev-only input script
- **Placeholders are brief**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Textures only while visible**: by hand: no automated seam for the egui UI; verified with the dev-only input script, whose description of the grid counts the thumbnails kept and registered
- **Words of the text**: `crates/drs-catalog-engine/tests/search.rs::words_of_the_text`
- **Matched by part of a name**: `crates/drs-catalog-engine/tests/search.rs::matched_by_part_of_a_name`
- **Accents however stored**: `crates/drs-catalog-engine/tests/search.rs::accents_however_stored` (an `x` and a `j` followed by combining marks among the names, the combining grapheme joiner among them)
- **Every word must match**: `crates/drs-catalog-engine/tests/search.rs::every_word_must_match`
- **A word with a slash matches the path**: `crates/drs-catalog-engine/tests/search.rs::a_word_with_a_slash_matches_the_path`
- **An empty text matches everything**: `crates/drs-catalog-engine/tests/search.rs::an_empty_text_matches_everything`, `crates/drs-library-manager/tests/search.rs::every_keystroke_searches` (an empty text counts every Asset)
- **Word starts rank first**: `crates/drs-catalog-engine/tests/search.rs::word_starts_rank_first` (a word after an `x` with a combining acute, and after one with the combining grapheme joiner, ranking as inside a word)
- **A deterministic order**: `crates/drs-catalog-engine/tests/search.rs::a_deterministic_order` (two folders built in either order give the same matches; the tie-breaks as spelled are not exercised)
- **Every keystroke searches**: `crates/drs-library-manager/tests/search.rs::every_keystroke_searches` (a text sent in one frame is answered at the end of the frame that handles it); the field sending each change by hand
- **Answered within a keystroke**: `crates/drs-catalog-engine/tests/search.rs::answered_within_a_keystroke` (400,000 vendor-shaped names, accented ones among them; the fastest of three builds and of five runs of each of a narrow, a broad, a multi-word, an accented, and a slash text is bounded, so that a loaded runner does not fail it)
- **Matches follow the library**: `crates/drs-library-manager/tests/search.rs::matches_follow_an_added_folder`, `crates/drs-library-manager/tests/search.rs::matches_are_current_at_start`, `crates/drs-app/tests/asset_folders.rs::matches_follow_undo_and_redo`
- **Only available Assets match**: `crates/drs-library-manager/tests/search.rs::only_available_assets_match`
- **Matches in the grid**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script over a generated folder of 400,000 image files
- **Counted**: `crates/drs-library-manager/tests/search.rs::every_keystroke_searches`, `crates/drs-app/tests/asset_folders.rs::matches_follow_undo_and_redo` (the total the browser shows, as answered); the per-folder counts and the browser's line by hand, driving the editor with the dev-only input script, whose description of the grid gives the line
- **No match is said**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script, whose description of the grid gives the line
- **A new text starts at the top**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script and its screenshots
- **The chosen Asset stays chosen**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Generated in the background**: `crates/drs-library-manager/tests/thumbnails.rs::generated_in_the_background` (the frame that indexes the folder returns with every state pending, later frames turn them ready, and a file is decoded on a generator thread; the thread check is left out when another logger already holds the process)
- **The browser asks for its rows**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Visible rows first**: `crates/drs-library-access/src/thumbnail.rs::tests::wanted_assets_come_first` (the generator's queue order, a function of LibraryAccess alone; completions on several threads arrive in no fixed order, so the App seam cannot assert it)
- **Every image format**: `crates/drs-library-manager/tests/thumbnails.rs::every_image_format`
- **Fitted, never enlarged**: `crates/drs-library-manager/tests/thumbnails.rs::fitted_never_enlarged` (a tall and a tiny fixture)
- **Transparency is kept**: `crates/drs-library-manager/tests/thumbnails.rs::transparency_is_kept`
- **The first frame**: `crates/drs-library-manager/tests/thumbnails.rs::the_first_frame` (an animated PNG fixture)
- **As the viewport draws it**: by hand: a JPEG with orientation metadata shows unrotated
- **Broken is a placeholder**: `crates/drs-library-manager/tests/thumbnails.rs::broken_is_a_placeholder`, `crates/drs-library-access/src/thumbnail.rs::tests::a_decoder_panic_is_broken_and_the_rest_carry_on` (a decoder that panics, on a thread marked as catching it)
- **Broken is remembered**: `crates/drs-library-manager/tests/thumbnails.rs::broken_is_remembered` (a second App over the same directories finds the broken state without opening the file; the half about a file that cannot be read runs on Unix only, and is skipped where the process may read a sealed file), `crates/drs-library-access/src/thumbnail.rs::tests::an_unreadable_file_is_read_once_a_session` (a file that cannot be read, enqueued and wanted again, is read once)
- **Kept across starts**: `crates/drs-library-manager/tests/thumbnails.rs::kept_across_starts`, `crates/drs-library-manager/src/thumbnails.rs::tests::kept_thumbnails_are_served_without_a_generator` (a start whose generator cannot start still serves what was kept, and says once that thumbnails cannot be generated)
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

- Remove Asset Folder, Rename Canonical Name, and Install Asset Pack are owned here and not implemented.
- Sixteen Rules (Folder name proposed, Cancelling records nothing, Browsed across folders, A grid of thumbnails, Never enlarged on screen, Named and placed on hover, Chosen by a click, A placeholder until then, Placeholders are brief, Textures only while visible, The browser asks for its rows, As the viewport draws it, Matches in the grid, No match is said, A new text starts at the top, and The chosen Asset stays chosen) have no automated test, against the requirement that every Rule has one. All but As the viewport draws it are behaviour of the egui interface alone, for which no headless seam exists, and that one depends on an image the tests do not write; the accepted deviation is verification by hand, driving the editor with the development-only input script and its screenshots, over a generated folder of 400,000 image files for the search's scrolling and per-keystroke behaviour.
- Nine Rules are tested in part: the responsiveness of Generated in the background is checked by hand, and its thread check only when the test holds the process's logger; that the panel shows through a transparent thumbnail (Transparency is kept) and the placeholder drawn for a broken Asset (Broken is a placeholder) are the egui interface's and checked by hand; the status line of An unwritable cache is reported is the Editor's and checked by hand, and only the failed write's report is checked to name the file; Visible rows first is tested on the generator's queue alone, the browser's wanted set reaching it through Browse being checked by hand; Shown as generated is tested on the generator's writing out alone, the browser drawing what is handed back being checked by hand; Every keystroke searches is tested from Browse onwards, the field sending each change being checked by hand; Counted is tested on the total LibraryManager answers, the per-folder counts and the browser's line being checked by hand; and A deterministic order is tested on rank, name folded, Canonical Name, and place folded, its tie-breaks as spelled having no fixture that needs them. The stories that a start with every thumbnail kept is as quick as one without, and that the search adds no time worth waiting for at start, have no Rule of their own: the first rests on the architecture's measured figures, the second on the build that Answered within a keystroke bounds.
- Development builds take the folder from an environment variable instead of opening the dialog, drive the editor from a script of input steps whose description lists the grid's cells with what each shows and its rectangle, save a screenshot on request, and keep the editor's own files under a directory of choice. That is tooling for verification, not behaviour of the capability.
- "Search", "match", and "search field" are plain English for presentation, not domain concepts, as "browser" is; they carry no invariant.
- "Thumbnail" is plain English for a small picture of an image Asset; it is presentation, not a domain concept, and carries no invariant. "Thumbnail pack" is the cache file and is not an Asset Pack.
- Generating thumbnails opens every file of an Asset Folder once; on an online-only cloud folder that means downloading the library once, which the architecture accepts for background jobs.
- egui lays out scroll offsets in single-precision floats, which hold whole pixels only to about 16.7 million pixels of content: about 110,000 rows of the grid, or 660,000 Assets at six columns. Larger libraries scroll less precisely; that is an accepted limitation.
- The cell is 128 points because that is what the spike measured (2.4 KB per opaque thumbnail, 60 µs to decode) and what the functional baseline shows at its default; the architecture's figures hold for it.
