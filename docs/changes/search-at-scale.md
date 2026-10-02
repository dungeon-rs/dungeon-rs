# Search at scale

**Capabilities**:
- asset-folders: none (the browser over Asset Folders; no Command changes hands)

## Problem Statement

A vendor library holds hundreds of thousands of Assets, and the Author looks for one by a piece of its name: `barrel`, `oak table`, `door`. Today the browser filters by checking every name on every frame, which is slow at that size, misses names whose letters fold differently or whose accents a file system spelled another way, takes the typed text as one piece so `table oak` finds nothing for `Table_Oak_Round`, and lists matches in folder order, so `bed` buries `Bed_Double` among every `Flowerbed`. The Author needs to find any Asset by name as fast as they type, whatever the letter case, with the likeliest matches first.

## Solution

The browser has a search field. Each keystroke searches every added Asset Folder at once and shows the matches, ranked, within the same moment: Assets in which every typed word begins a word of the name come first, then those that only contain it, each group in order of name. Several words narrow the matches; a word with a `/` in it matches the folder's Canonical Name and the subfolders as well, so `furniture/table` finds the tables in a furniture folder. The browser says how many Assets match, says so plainly when none do, and with nothing typed shows the whole library as it does without a search. The search follows the library: a folder added, undone, redone, or refreshed at start changes the matches at once, without the Author typing again. It is built in memory from the folders' indexes and never written to disk.

This change follows Thumbnails on the roadmap and builds on the browser that change leaves: one grid of thumbnails, a line per Asset Folder above it.

## User Stories

### Finding an Asset

1. As an Author, I want to type part of an Asset's name and see every Asset whose name contains it, so that I find an Asset without knowing which folder or subfolder it sits in.
2. As an Author, I want the search to ignore letter case, so that `barrel`, `Barrel`, and `BARREL` find the same Assets.
3. As an Author, I want letters that change length when their case changes to match as well, so that `STRASSE` finds `Straße`.
4. As an Author, I want an accented name to be found however its accent is stored, composed or decomposed, so that names unpacked from an archive or copied from another operating system match what I type.
5. As an Author, I want spaces before and after what I type to be ignored, so that a stray space never hides every match.
6. As an Author, I want to type several words and see only the Assets whose name contains every one of them, in any order, so that `table oak` narrows to oak tables.
7. As an Author, I want to type a word containing `/` to match the folder's Canonical Name and the subfolders an Asset sits in, so that `furniture/table` finds the tables in a furniture subfolder and `forgotten/barrel` the barrels of one vendor.
8. As an Author, I want Assets in which what I typed begins a word of the name listed before those in which it only appears inside a word, so that `bed` lists `Bed_Double` before `Flowerbed`.
9. As an Author, I want equally good matches listed in order of name, so that I can scan them like an index.
10. As an Author, I want two Assets with the same name in different Asset Folders listed in order of their folders' Canonical Names, and two in the same folder in order of place, so that the order never depends on chance.
11. As an Author who shares a library with a collaborator who keeps it at another path, I want the same text to list the same Assets in the same order on both our devices, so that "the third barrel" means the same Asset to both of us.

### Seeing the matches

12. As an Author, I want the matches to change with every keystroke, without pressing Enter, so that I see the effect of each letter as I type it.
13. As an Author, I want to see how many Assets match, so that I know whether to keep narrowing.
14. As an Author, I want to see how many Assets the library holds when nothing is typed, so that the count is always there.
15. As an Author, I want to see each Asset Folder above the grid with how many of its Assets match, so that I know where the matches come from.
16. As an Author, I want a message naming what I typed when nothing matches, so that I know the search ran and found nothing, rather than facing an empty panel.
17. As an Author, I want every Asset back, ordered by folder and place as without a search, when I clear the field, so that browsing is one keystroke away.
18. As an Author, I want a new search to show its matches from the top, so that the best matches are the ones I see first.
19. As an Author, I want to choose an Asset from the matches with one click, as from the whole library, so that searching is the way to pick an Asset.
20. As an Author, I want the Asset I chose to stay chosen while I change the text, even when it no longer matches, so that searching for the next Asset never loses the one I am placing.

### At scale

21. As an Author with 400,000 Assets, I want each keystroke's matches before I type the next, so that searching never feels slower than typing.
22. As an Author with 400,000 Assets, I want to scroll through every match of a broad search as smoothly as through the whole library, so that a search matching most of the library is no slower to browse.
23. As an Author with a large library, I want opening the editor to cost no more time for the search than indexing the folders already does, so that search never becomes the thing I wait for at start.

### Following the library

24. As an Author with text typed in the search field, I want the matching Assets of a folder I add to join the matches as soon as it is added, without typing again, so that a new folder is searchable at once.
25. As an Author, I want the Assets of a folder whose adding I undo to leave the matches at once, and to return when I redo it, so that the matches always show what is available.
26. As an Author, I want Assets added to or removed from a folder while the editor was closed to be found or gone at the next start, so that a vendor update is searchable.
27. As an Author, I want a remembered folder whose disk is unplugged to contribute no matches while it stays listed, so that the search never offers an Asset that cannot be placed.
28. As an Author with an empty library, I want the search field to tell me to add an Asset Folder rather than that nothing matches, so that the advice fits the situation.

## Rules

### Matching

**Words of the text**: the typed text is split at whitespace into words; whitespace before, between, and after them is ignored.

**Matched by part of a name**: an Asset matches a word without `/` when its name contains the word, both compared Unicode-normalised and fully case-folded, so that `barrel` finds `Old_BARREL` and `STRASSE` finds `Straße`.

**Accents however stored**: a word spelled with composed accents matches a name spelled with decomposed ones and the reverse; an unaccented letter does not match an accented one.

**Every word must match**: with several words, an Asset matches only when it matches each of them, in any order; two words may match overlapping parts of the name.

**A word with a slash matches the path**: a word that contains `/` is matched, as a name is, against the Asset's path in the library instead of its name: its folder's Canonical Name, a `/`, and its place without the extension.

**An empty text matches everything**: a text with no words matches every Asset of every added Asset Folder.

### Ranking

**Word starts rank first**: an Asset in which every word occurs at least once where a word of the name (or of the path, for a word with `/`) begins ranks above an Asset in which some word occurs only inside a word; a word of the name begins at its start or after a character that is neither a letter nor a digit.

**A deterministic order**: matches are ordered by rank, then by name, then by their folder's Canonical Name, then by place, each compared Unicode-normalised and case-folded first and as spelled to break a tie; the order depends on nothing but the text and the folders' Canonical Names and indexes, so it is the same on every device.

### Responding

**Every keystroke searches**: each change of the typed text is searched without the Author confirming it, and the matches for it are shown in the frame after the change.

**Answered within a keystroke**: over 400,000 Assets in the test build, building the search takes under 1 second and answering a text takes under 50 milliseconds.

### Following the library

**Matches follow the library**: the matches for the typed text include a folder's matching Assets from the frame Add Asset Folder is applied or redone, lose them from the frame it is undone, and reflect at start the Assets added to or removed from a remembered folder while the editor was closed, without the text being typed again. Follows from: Every Command can be undone.

**Only available Assets match**: a remembered folder that cannot be scanned contributes no matches.

### The browser

**Matches in the grid**: with text typed, the grid holds the matching Assets in the order A deterministic order gives, and lays out only the rows in view, however many match.

**Counted**: the browser shows how many Assets match the typed text, or, with nothing typed, how many Assets the library holds; the line above the grid for each Asset Folder gives how many of its Assets match.

**No match is said**: when Asset Folders are added and no Asset matches, the browser says that no Asset matches, quoting the typed text; with no Asset Folder added it says to add one, as it does today.

**A new text starts at the top**: when the typed text changes, the grid shows its matches from the first row.

**The chosen Asset stays chosen**: changing the typed text never changes which Asset is chosen for placing, whether or not it still matches.

## Changes to existing behaviour

- asset-folders — **Filtered by name**: removed, and replaced by Words of the text, Matched by part of a name, Accents however stored, Every word must match, A word with a slash matches the path, An empty text matches everything, Word starts rank first, and A deterministic order: the text was one piece compared lower-cased, and is now words that must all match, compared case-folded and Unicode-normalised, with ranked matches.
- asset-folders — **Browsed across folders** (as Thumbnails leaves it): modified so that its order, by the Canonical Name of the folder and then by place, holds when nothing is typed; with text typed, Matches in the grid gives the order, and the count beside each folder is of its matching Assets.

## Implementation Decisions

### CatalogEngine: the Search contract

- **Build** turns one Asset Folder's Canonical Name and index into that folder's search, plain data with no Bevy. It holds the folder's Assets in search order (name, then place, compared folded and then as spelled), and for each Asset its name and its path in the library, both Unicode-normalised (NFC) and fully case-folded as Resolve folds places, laid out in two contiguous texts, one for names and one for paths, with the offset of each row. Building one search per folder means a folder that changes rebuilds its own search only.
- **Search** answers a text over the searches of every folder: the matches in order, each as its folder and its position in that folder's index, and the count per folder. The text is split into words and each word folded as the names are. For each folder, the longest word is found by a SIMD substring scan of the names' or the paths' text (`memchr`'s `memmem`, as the architecture's Asset index bullet measured), each hit mapped back to its row, and the remaining words checked within that row; whether an occurrence begins a word is read from the character before it. Hits come out in the folder's search order, so each folder yields its matches already in order within each rank, and the folders' lists are merged by name and then Canonical Name, with no sort per keystroke. An empty text yields every Asset.
- **No fst in this change**: the rank comes from where the substring scan hits, which an fst over whole names cannot tell, and the measured fst substring stream costs 80–190 ms over 400,000 names. fst stays available to CatalogEngine for prefix lookups and an on-disk index, neither of which is needed here.
- **Cost** for 400,000 Assets: about 35 MiB in memory (two folded texts, their offsets, the search order); about a millisecond per text and well under 0.3 s per build in a release build. `memchr` becomes a direct dependency of CatalogEngine; it is already in the dependency graph.

### LibraryManager: keeping and asking the searches

- **One search per Asset Folder**, held in a component of LibraryManager's own on the folder's entity. It is built in the same step that writes the folder's index: Add Asset Folder applied or redone, and Refresh at start, which is the step that sends Asset Folder Changed for that folder. Undoing despawns the entity and its search with it. A folder whose scan fails at start has an empty index and therefore an empty search. _Why_ synchronous: the scan in the same step already costs more (about 1 s for 400,000 files), so building in the background would save little and leave frames of stale matches.
- **The search is never written to disk**: it is rebuilt from the folders' indexes at each start, so it adds no file to the editor's directories and can never disagree with the index.
- **Asking**: the browser sends LibraryManager a `model` message carrying the typed text whenever it changes. LibraryManager, in the Commands set, runs CatalogEngine's Search over every folder's search and writes the answer into a `model` resource written only by LibraryManager: the text it answers, the total count, the count per folder, and the matches as (folder entity, position in its index). The total count when nothing is typed is the sum of the folders' index sizes.
- **Answering again**: LibraryManager keeps the last text it was sent and runs it again in any frame in which a folder's search was built or dropped, in a system ordered after Redo, so a folder added, undone, redone, or refreshed changes the matches in that same frame.

### Editor: the browser

- The search field replaces the filter field; its text lives in the Editor's own state as the filter does today, and every change is sent as the message above, an empty text included.
- With text typed, the grid Thumbnails builds lays out the matches from the resource, in its order, through the same virtual rows, and the wanted set it sends for thumbnails is the matches in the laid-out and prefetched rows. Until the answer for the current text arrives, one frame later, the previous answer is shown. With nothing typed, the grid shows every Asset in folder and place order straight from the folders' indexes, as Thumbnails builds it.
- Above the grid, a line gives the count ("12 Assets match", "400,000 Assets"), and each folder's line its count of matches. No match gives the message quoting the text; with no folder added the existing message stays.
- A change of the text resets the grid's scroll offset to the top. The chosen Asset is kept in the Editor's state independently of the matches, as it is today.
- The Editor no longer matches names itself; the pinned Implementation Decision that says it filters by name itself no longer holds.

## Testing

Three seams. CatalogEngine's Search is tested directly over searches built from synthetic names, because ranking, folding, and order are a function of the Engine alone. The headless App of the real plugins of `model`, `history`, LibraryAccess, and LibraryManager over fixture folders in a temporary directory, with the editor's directories pointed at temporary ones, proves the matches follow the library; it sends the search message and asserts on the matches resource. Undo and redo run in the Host's test App, which holds every Manager, as Add Asset Folder is undoable does. The browser is checked by hand.

- **Words of the text**: `crates/drs-catalog-engine/tests/search.rs::words_of_the_text`
- **Matched by part of a name**: `crates/drs-catalog-engine/tests/search.rs::matched_by_part_of_a_name`
- **Accents however stored**: `crates/drs-catalog-engine/tests/search.rs::accents_however_stored`
- **Every word must match**: `crates/drs-catalog-engine/tests/search.rs::every_word_must_match`
- **A word with a slash matches the path**: `crates/drs-catalog-engine/tests/search.rs::a_word_with_a_slash_matches_the_path`
- **An empty text matches everything**: `crates/drs-catalog-engine/tests/search.rs::an_empty_text_matches_everything`
- **Word starts rank first**: `crates/drs-catalog-engine/tests/search.rs::word_starts_rank_first`
- **A deterministic order**: `crates/drs-catalog-engine/tests/search.rs::a_deterministic_order` (two folders built in either order give the same matches)
- **Answered within a keystroke**: `crates/drs-catalog-engine/tests/search.rs::answered_within_a_keystroke` (400,000 synthetic vendor-shaped names; the fastest of five runs of each of a narrow, a broad, a multi-word, and a slash text is bounded, so that a loaded runner does not fail it)
- **Every keystroke searches**: `crates/drs-library-manager/tests/search.rs::every_keystroke_searches` (a text sent in one frame is answered at the end of the frame that handles it), and by hand for the field sending each change
- **Matches follow the library**: `crates/drs-library-manager/tests/search.rs::matches_follow_an_added_folder`, `crates/drs-library-manager/tests/search.rs::matches_are_current_at_start`, and `crates/drs-app/tests/asset_folders.rs::matches_follow_undo_and_redo`
- **Only available Assets match**: `crates/drs-library-manager/tests/search.rs::only_available_assets_match`
- **Matches in the grid**, **Counted**, **No match is said**, **A new text starts at the top**, **The chosen Asset stays chosen**: by hand: no automated seam for the egui interface; verified by driving the editor with the development-only input script and its screenshots, over a generated folder of 400,000 image files for the scrolling and per-keystroke behaviour

## Out of Scope

- Tags and tag sets driving search.
- Fuzzy or typo-tolerant search (letters in order with gaps, edit distance).
- Accent-insensitive matching (`cafe` finding `Café`).
- Word starts inside a word by letter case (`OakTable` has one word for ranking).
- Searching the contents of a Project: Elements, Layers, or Levels.
- Saved searches, a search history, and filtering by Asset Kind.
- Highlighting the matched part of a name.
- Keeping the search on disk.

## Further Notes

- "Search", "match", and "search field" are plain English for presentation, not domain concepts, as "browser" already is; they carry no invariant.
- LibraryManager answering the browser's search message is a request operation its contract in the architecture does not yet list (AddFolder, RemoveFolder, RenameFolder, InstallPack, Refresh), as is the wanted set Thumbnails adds; and no call chain shows CatalogEngine's Search. The Editor reaches no Engine directly, so this is the only path to Search; the contract list and a call chain should name it before this change is built.
