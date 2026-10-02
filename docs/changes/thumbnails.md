# Thumbnails

**Capabilities**:
- asset-folders: none (the browser over Asset Folders; no Command changes hands)

## Problem Statement

The browser lists Assets by name alone. A vendor library holds hundreds of thousands of files named `wall_stone_03`, `prop_crate_12`, and `tree_oak_a`, and the Author cannot tell a stone wall from a brick one, or a barrel from a crate, without placing it on the Level and looking. Finding an Asset by sight is how an Author actually works, and the functional baseline shows a picture for every Asset; today this editor shows none.

## Solution

Every Asset in the browser shows a thumbnail: a small picture of the image, fitted into a square cell with the Asset's name beneath, in a grid as wide as the panel allows. Thumbnails are generated in the background from the moment a folder is indexed, the rows the Author is looking at first, and appear as they are ready; until then a neutral placeholder holds the place. Once generated, a thumbnail is kept in a pack in the editor's cache directory and found again at every later start, so a library is thumbnailed once per device. A file that changes gets a new thumbnail; a file that is not an image gets a broken placeholder and never a crash; nothing is ever written into an Asset Folder. Scrolling stays smooth at any library size because the browser lays out only the visible rows, decodes thumbnails off the main thread, and keeps textures only for what is on screen.

## User Stories

### Seeing thumbnails

1. As an Author, I want the browser to show every Asset as a thumbnail with its name beneath, in a grid as wide as the panel, so that I recognise an Asset by sight instead of by name.
2. As an Author, I want thumbnails to appear in the browser as they are generated, without me doing anything, so that a freshly added folder fills in while I keep working.
3. As an Author, I want the rows I am looking at to get their thumbnails before the rest of the library, so that a 400,000-file folder is useful within seconds of being added.
4. As an Author, I want an Asset whose thumbnail is not yet generated to show a neutral placeholder in its cell, so that the grid holds its shape while the thumbnails fill in.
5. As an Author, I want a thumbnail that is generated but not yet on screen to show a placeholder of its own proportions for the brief moment before it appears, so that nothing jumps when it does.
6. As an Author, I want to scroll the grid as fast as I like and see placeholders at most briefly, with the visible rows filled a few frames after I stop, so that scrolling never waits for the thumbnails.
7. As an Author, I want the editor to stay as responsive as ever while thumbnails are being generated, whether I scroll, place Props, or pan the Level, so that generation is something I never notice.
8. As an Author, I want PNG, JPEG, and WebP Assets alike to show thumbnails, so that the whole library is covered whatever format a vendor ships.
9. As an Author, I want a thumbnail of an image with transparent parts to keep them transparent, so that a cut-out Prop looks like a cut-out and not like a square.
10. As an Author, I want a tall or wide image to keep its proportions inside the cell, neither stretched nor cropped, so that a long fence reads as a fence.
11. As an Author, I want an image smaller than the cell to be shown at its own size and not blown up, so that a 32-pixel token stays crisp.
12. As an Author, I want a file that is not a readable image to show a broken placeholder while the editor and the rest of the thumbnails carry on, so that one bad file never costs me the library or the session.
13. As an Author, I want to hover a cell and read the Asset's name in full, the Canonical Name of its folder, and its place in the folder, so that two Assets with the same name are told apart.
14. As an Author, I want to click a cell to choose the Asset to place, as I did in the list, so that the grid is the same browser with pictures.
15. As an Author, I want to see each Asset Folder named above the grid with how many of its Assets are shown, so that I still know which folders I have and that an empty one is empty.
16. As an Author, I want the filter to apply to the grid as it did to the list, so that typing part of a name narrows the pictures I see.

### Keeping thumbnails

17. As an Author, I want the thumbnails generated today to be there the next time I open the editor without being generated again, so that a library is thumbnailed once per device.
18. As an Author, I want a file that a vendor update replaced to get a new thumbnail at the next start, so that the picture is never of the old version.
19. As an Author, I want a remembered folder whose disk is unplugged to keep its thumbnails, so that they are back with the folder as soon as the disk is.
20. As an Author, I want undoing and redoing Add Asset Folder to keep the folder's thumbnails, so that undo is as cheap as any other step.
21. As an Author, I want the thumbnails kept in the editor's cache directory and never inside my Asset Folder, so that a read-only or cloud-synced folder stays untouched.
22. As an Author, I want to quit the editor while thumbnails are still being generated and have it quit at once, so that generation never holds me hostage.
23. As an Author, I want the thumbnails finished before I quit to be kept and generation to carry on from there at the next start, so that quitting never throws work away.
24. As an Author, I want a thumbnail cache the editor left incomplete, because it quit or crashed mid-write, to be used for what is complete and regenerated for the rest, so that a bad moment never means starting over.
25. As an Author, I want a thumbnail cache the editor cannot make sense of to be started afresh with no fuss, so that a damaged cache is an inconvenience and not an error.
26. As an Author, I want to be told when thumbnails cannot be kept, because the cache directory cannot be written, while the editor runs on with placeholders, so that a full disk never stops me from composing.
27. As an Author, I want a file that could not be read for a moment, because its disk was slow or its cloud copy was not yet local, to be tried again at the next start rather than marked broken for good, so that a transient fault never leaves a permanent hole.
28. As an Author, I want a file that is genuinely not an image to be remembered as broken, so that the editor does not try it again at every start.
29. As an Author with a 400,000-file library, I want opening the editor with every thumbnail kept to be as quick as opening it without them, so that the cache never becomes the thing I wait for.
30. As an Author with a 400,000-file library, I want the editor's memory to stay bounded however far I scroll, so that browsing a big library never costs more than a few dozen megabytes.

## Rules

### The grid

**A grid of thumbnails**: each Asset the browser shows is a cell holding its thumbnail fitted into a 128-pixel square with its name on one line beneath, in as many equal columns as the panel's width holds, at least one.

**Fitted, never enlarged**: a thumbnail keeps the image's proportions, is at most 128 pixels on its longer side, and is never larger than the image itself.

**Named and placed on hover**: hovering a cell shows the Asset's name in full, the Canonical Name of its Asset Folder, and its place in the folder.

**Chosen by a click**: clicking a cell chooses its Asset for placing, exactly as the list did.

**A placeholder until then**: an Asset whose thumbnail is not yet generated shows a neutral square placeholder; a generated thumbnail that is not yet decoded for display shows a neutral placeholder of the thumbnail's own proportions.

**Shown as generated**: a thumbnail appears in the browser within a few frames of being generated, without any action from the Author.

**Placeholders are brief**: rows revealed by scrolling faster than the prefetch covers show placeholders only while the scrolling lasts; the visible rows are filled within a few frames of the last scroll input.

**Textures only while visible**: a thumbnail is registered with the interface only while its row is laid out and is unregistered when it leaves; at most 512 decoded thumbnails are kept, the least recently shown dropped first.
_Why_: every registered texture costs a bind group per frame on Metal, and only dropping the decoded image frees its texture.

### Generation

**Generated in the background**: generating thumbnails never runs on the main thread; the browser, the viewport, and every Command stay responsive while it runs.

**The browser asks for its rows**: whenever the set of Assets in the browser's laid-out rows and the two rows either side changes, the browser names that set as wanted.

**Visible rows first**: the Assets last named as wanted that are still pending are generated before any other.

**Every image format**: a PNG, JPEG, or WebP Asset gets a thumbnail.

**Transparency is kept**: a thumbnail of an image with pixels that are not fully opaque keeps those pixels' transparency, and the panel shows through them.

**The first frame**: an animated image's thumbnail is of its first frame.

**As the viewport draws it**: a thumbnail shows the image as decoded, with no orientation metadata applied.
_Why_: the viewport draws the image the same way, so the thumbnail matches the Prop.

**Broken is a placeholder**: an Asset whose file cannot be decoded as an image shows a broken placeholder, the editor keeps running, and the remaining thumbnails are still generated.

**Broken is remembered**: a file that could not be decoded is recorded as broken and not tried again until its size or modification time changes; a file that could not be read is not recorded and is tried again at the next start.
_Why_: a cloud copy not yet downloaded or a disk that was slow is a fault of the moment, and marking it broken would hide the Asset until the vendor touched the file.

### The cache

**Kept across starts**: a thumbnail generated once is found at the next start and not generated again.

**A changed file gets a new thumbnail**: an Asset whose byte size or modification time differs from when its thumbnail was generated is generated again, and the old thumbnail is no longer served for it.

**Kept through undo and redo**: undoing and redoing Add Asset Folder keeps the folder's thumbnails, which are served again on redo without being generated again. Follows from: Every Command can be undone.

**Kept while a folder is away**: a remembered folder that cannot be scanned keeps its thumbnails, and they are served again when it can.

**Kept in the cache directory**: the pack and its index live in the editor's cache directory, and generating thumbnails creates or changes no file inside an Asset Folder.

**Append-only**: generating a thumbnail only appends to the pack and its index; no entry is removed or rewritten in this change.
_Why_: measured at 2.4 KB per opaque and 22 KB per transparent thumbnail, a pack serving 400,000 Assets stays within what a cache directory is for, and reclaiming entries no Asset refers to is a compaction left for later.

**Quitting stops generation**: quitting while thumbnails are generating ends the editor without waiting for the queue, and every thumbnail finished before then is kept.

**A torn record is skipped**: an index record that is incomplete or points beyond the end of the pack is skipped when the cache is opened, every complete record is served, and the skipped Asset is generated again.

**An unreadable cache starts afresh**: a pack or index the editor cannot make sense of is replaced by an empty one, every thumbnail is generated again, and nothing crashes.

**An unwritable cache is reported**: when the pack cannot be opened or written, the Author is told once in the status line, the browser shows placeholders, and the editor runs on.

## Changes to existing behaviour

- asset-folders — **Browsed across folders**: modified to "the browser shows the Assets of every added Asset Folder in one grid of thumbnails, ordered by the Canonical Name of their folder and then by place, and names each folder above the grid with how many of its Assets are shown", because the collapsing list per folder gives way to a grid whose rows must all be the same height, and the folder's Canonical Name moves to a line above the grid and to the cell's hover text.
- asset-folders — **Filtered by name**: unchanged in what it matches; it now narrows the grid instead of the list.
- asset-folders — **Nothing is written into the folder**: unchanged; the thumbnail pack and its index join the Manifest and the index cache in the editor's own directories, and **Kept in the cache directory** states it for thumbnails.
- asset-folders — **Scanning opens no file**: unchanged; generating thumbnails is a background job after the scan and is the first thing that opens an Asset's file, never the scan itself.

## Implementation Decisions

The design the architecture's Asset index and caches bullet settles (one append-only pack plus a fixed-record index in the cache directory, keyed by folder key, place, size, and modification time; positional reads, never mmap; decoding off the main thread with a prefetch of two rows; textures registered only while visible; an LRU of strong image handles) is used as written there and not restated, with two departures from that bullet's figures, both stated below and both to be folded into the bullet before this change is built: an index record is 32 bytes, not 16, because it carries the key's digest; and decoding for display runs in the asset system's load tasks, not on the async-compute pool, because the Editor reaches thumbnails only through the asset server.

### LibraryAccess: the Thumbnail contract

- **Open**: opens the pack and its index from a `thumbnails` directory under the cache directory, creating both when absent. The index is read whole; each 32-byte record holds a 16-byte digest of the key (folder key, place, byte size, modification time), the entry's offset and length in the pack, and the thumbnail's width and height. A record that is short or whose entry ends beyond the pack is skipped and logged; a file with the wrong magic or version is replaced by an empty one, and the Author is not told, since nothing is lost but time. An index that cannot be opened or a pack that cannot be opened for appending is an error the caller reports. _Why_ the digest in the record: the key is variable-length text, and a fixed record that carries the key's digest keeps the index self-contained and appendable, where recording entry numbers in the per-folder index cache would tie every thumbnail to a rewrite of that cache. At 400,000 records the index is 12.8 MB, twice the 6.1 MiB the spike read whole in 1.2 ms; the load time at this size is unmeasured.
- **Lookup**: given an Asset's key, says whether a thumbnail exists (with its width and height), is recorded as broken, or is absent. A broken entry is a record with a zero-length entry. When a key has more than one complete record, the last one appended is the one served.
- **Generate**: reads the file, decodes it with the `image` crate under its default memory limit, takes the first frame, downsamples it with a box filter so the longer side is at most 128 pixels and never larger than the image, and encodes it as PNG when any pixel is not fully opaque and as JPEG at quality 85 otherwise. The format is told from the bytes when read, so the record carries no tag. A decode failure appends a broken record; a read failure appends nothing and is logged.
- **Append**: the bytes go to the pack first, then the record to the index, so a crash between the two leaves at worst unreferenced bytes. Writes are buffered and flushed when the queue drains, every few hundred thumbnails, and when the generator stops; an operating-system crash may lose the last few thumbnails, which are generated again.
- **Read**: a positional read of one entry's bytes by its record.
- **The generator**: a queue of Assets to generate, served by a task pool of LibraryAccess's own with half the available cores, at least one thread, so that generation never competes with the asset system's decoding of thumbnails for display. The front of the queue is a set the caller can replace at any time (the browser's wanted Assets); the rest is served in the order enqueued. Dropping the generator stops its threads after the thumbnails in flight, flushes the pack, and returns within the time of one thumbnail. Completions (key, outcome) are handed back through a channel the caller drains each frame.
- **The `thumb://` source**: a second dynamic asset source, registered by the Host before the asset plugin builds like `lib://`, whose reader serves the encoded bytes of the thumbnail recorded for `thumb://<folder-key>/<place>` from a runtime table LibraryAccess keeps of which record serves each Asset at its current size and modification time; a path with no record is a missing asset, and the reader refuses any path `lib://` would refuse. The image loader the Host registers decodes the bytes in the asset system's load tasks, off the main thread, told by the Editor to guess the format from the bytes rather than the extension and to keep the pixels in the render world only. _Why_ through the asset server: the Editor depends on no ResourceAccess, so, as with a Prop's image through `lib://`, the asset server is its only way to a thumbnail's bytes, and it frees a texture when its last handle is dropped, which is what the LRU relies on.

### LibraryManager: driving generation

- After a folder is indexed (Add Asset Folder, Refresh at startup, redo) the Manager looks up every Asset's thumbnail through LibraryAccess, writes the outcome into a `model` component on the folder entity holding the thumbnail state of each Asset in the index's order (pending, ready with width and height, or broken), and enqueues the pending ones in place order. A folder that is undone or that cannot be scanned has nothing enqueued; its records stay in the pack.
- Each frame, in the Commands set and before any Command is handled, the Manager drains the generator's completions and updates the states, so a folder indexed in a frame shows every Asset pending at the end of that frame; it then handles the browser's wanted set: a `model` message carrying the Assets (folder key and place) the browser is laying out and prefetching, sent whenever that set changes, which the Manager passes to the generator as the front of its queue.
- The thumbnail cache is opened at startup before the remembered folders are restored; if it cannot be opened, or an append later fails, the Manager sends a `model` message with the reason once, and generation is not attempted again in this session.

### Editor: the grid

- The browser lays out the Assets the filter matches with egui's virtual rows at a fixed row height (the 128-pixel square, the name line, and the spacing), as many columns as the panel's width holds; how the filter finds those Assets is not changed here. Above the scroll area, one line per Asset Folder gives its Canonical Name and its count of matching Assets.
- For every laid-out row plus two rows either side, each Asset's state decides the cell: pending draws the square placeholder; broken draws the broken placeholder; ready loads `thumb://<folder-key>/<place>` through the asset server if no handle is held, draws a placeholder of the recorded proportions until the image is loaded, and draws the image once it is. Handles live in an LRU of 512 strong handles, never evicting what is laid out; the texture is registered with `EguiContexts::add_image` when first drawn and unregistered when its row leaves the laid-out range.
- The wanted message is sent when the set of laid-out and prefetched Assets differs from the last one sent.
- The cache failure message is reported in the status line, as Failure shown in the status line prescribes.

## Testing

The automated seam is the headless App of the real plugins of `model`, `history`, LibraryAccess, and LibraryManager over fixture folders in a temporary directory, with the editor's directories pointed at temporary ones, driven by messages and run until every thumbnail state has settled or a bound of frames is reached. Fixtures are written by the test with the `image` crate: an opaque PNG, a PNG with transparent pixels, a JPEG, a WebP, a tall image, a tiny image, and a file with an image extension that is not an image. Thumbnails are read back through LibraryAccess's Read and decoded in the test to assert on pixels.

- **Generated in the background**: `crates/drs-library-manager/tests/thumbnails.rs::generated_in_the_background` (the frame that indexes the folder returns with every state pending; later frames turn them ready)
- **Visible rows first**: `crates/drs-library-access/src/thumbnail.rs::tests::wanted_assets_come_first` (the generator's queue order, a function of LibraryAccess alone; completions on several threads arrive in no fixed order, so the seam cannot assert it)
- **Every image format**: `crates/drs-library-manager/tests/thumbnails.rs::every_image_format`
- **Transparency is kept**: `crates/drs-library-manager/tests/thumbnails.rs::transparency_is_kept`
- **Fitted, never enlarged**: `crates/drs-library-manager/tests/thumbnails.rs::fitted_never_enlarged` (the tall and the tiny fixture)
- **The first frame**: `crates/drs-library-manager/tests/thumbnails.rs::the_first_frame` (an animated PNG fixture)
- **As the viewport draws it**: by hand: a JPEG with orientation metadata shows unrotated
- **Broken is a placeholder**: `crates/drs-library-manager/tests/thumbnails.rs::broken_is_a_placeholder`
- **Broken is remembered**: `crates/drs-library-manager/tests/thumbnails.rs::broken_is_remembered` (a second App over the same directories finds the broken state without opening the file; an unreadable fixture is tried again)
- **Kept across starts**: `crates/drs-library-manager/tests/thumbnails.rs::kept_across_starts`
- **A changed file gets a new thumbnail**: `crates/drs-library-manager/tests/thumbnails.rs::a_changed_file_gets_a_new_thumbnail`
- **Kept through undo and redo**: `crates/drs-app/tests/thumbnails.rs::kept_through_undo_and_redo` (the Host's test App, which holds every Manager)
- **Kept while a folder is away**: `crates/drs-library-manager/tests/thumbnails.rs::kept_while_a_folder_is_away`
- **Kept in the cache directory**: `crates/drs-library-manager/tests/thumbnails.rs::kept_in_the_cache_directory`
- **Append-only**: `crates/drs-library-manager/tests/thumbnails.rs::append_only`
- **Quitting stops generation**: `crates/drs-library-manager/tests/thumbnails.rs::quitting_stops_generation` (the App is dropped mid-generation; the drop returns, and a second App serves what was finished)
- **A torn record is skipped**: `crates/drs-library-manager/tests/thumbnails.rs::a_torn_record_is_skipped` (the index is truncated mid-record and a record is pointed past the pack by the test)
- **An unreadable cache starts afresh**: `crates/drs-library-manager/tests/thumbnails.rs::an_unreadable_cache_starts_afresh`
- **An unwritable cache is reported**: `crates/drs-library-manager/tests/thumbnails.rs::an_unwritable_cache_is_reported` (Unix only; the cache directory is made read-only)
- **A grid of thumbnails**, **The browser asks for its rows**, **Named and placed on hover**, **Chosen by a click**, **A placeholder until then**, **Shown as generated**, **Placeholders are brief**, **Textures only while visible**, **Browsed across folders**: by hand: no automated seam for the egui interface; verified by driving the editor with the development-only input script and its screenshots

## Out of Scope

- Search at scale (the next change); the filter matches names as it does now.
- Dragging an Asset from the browser into the viewport.
- A thumbnail size preference, a size slider, or more than one thumbnail size.
- Prefabs, Materials, or any Asset Kind other than images as browser items.
- Compacting the pack, a size cap, or any eviction; the pack only grows in this change.
- Two editors sharing the thumbnail cache at once.
- A progress indicator for generation; the placeholders are the indicator.
- Pausing generation, for example on an online-only cloud folder; generating thumbnails reads every Asset once.
- Noticing an Asset changed on disk while the editor runs; a change is picked up at the next start.
- Using thumbnails anywhere but the browser.
- Honouring orientation metadata in images.

## Further Notes

- "Thumbnail" is used as plain English for a small picture of an image Asset, as the roadmap, the Needs, and the architecture already use it; it is presentation, not a domain concept, and carries no invariant.
- Generating thumbnails is the first job that opens every file in an Asset Folder. On an online-only cloud folder that means downloading the library once; the architecture accepts this for background jobs, and a way to pause is left for later.
- egui lays out scroll offsets in single-precision floats, which hold whole pixels only to about 16.7 million pixels of content: about 110,000 rows of this grid, or 660,000 Assets at six columns. Multi-million-Asset libraries are an accepted gap of this change.
- The cell is 128 pixels because that is what the spike measured (2.4 KB per opaque thumbnail, 60 µs to decode) and what the functional baseline shows at its default; the architecture's figures hold for it.
- The dev script's `describe` step should list the grid's cells with their rectangles so a verification agent can aim at them; that is tooling, not behaviour of the capability.
