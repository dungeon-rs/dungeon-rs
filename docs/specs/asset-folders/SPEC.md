# Asset Folders

**Commands**: Add Asset Folder, Remove Asset Folder, Rename Canonical Name, Install Asset Pack

## Purpose

An Author owns folders full of Assets, from vendors and of their own making, and uses them where they already are. This capability lets the Author tell the editor where such a folder is, gives it the Canonical Name Projects know it by, indexes its Assets without touching the folder, and lists them so an Asset can be found by name. Folders added once stay available on this device and follow what changes in them.

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

18. As an Author, I can see the Assets of every added Asset Folder in one list, each with its name under the Canonical Name of its folder, so that I can find an Asset without knowing where it lives.
19. As an Author, I can type part of a name and see only the Assets whose name contains it, ignoring case, so that I find an Asset among hundreds.
20. As an Author, I can tell two files with the same name in different subfolders apart by their place in the folder, so that neither hides the other.
21. As an Author, I can find the folders I added the next time I open the editor, so that I add each folder once per device.
22. As an Author, I can rely on Assets added to or removed from a folder while the editor was closed appearing or disappearing at the next start, so that a vendor update is picked up.
23. As an Author, I can see a remembered folder whose disk is unplugged still listed, empty and with a report, so that it is back as soon as the disk is.
24. As an Author, I can undo adding a folder and redo it without being asked its Canonical Name again, so that adding a folder is a step like any other.

## Rules

### Adding an Asset Folder

**Folder name proposed**: the Canonical Name prompt proposes the folder's own name, which the Author may change.

**Blank names are refused**: a Canonical Name that is empty once leading and trailing whitespace is removed is refused, and nothing is recorded.

**Names are unique on this device**: a Canonical Name equal to one already in use on this device, compared ignoring letter case and Unicode normalisation, is refused with the folder that holds it named, and nothing is recorded. Follows from: Canonical Names are unique on a device.

**The same folder is refused**: a folder already added on this device, however its path is spelled (letter case where the file system ignores it, a trailing separator, `.` components, or a symbolic link to it), is refused and the Author is told the Canonical Name it already has.

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

**Browsed across folders**: the browser lists the Assets of every added Asset Folder, each with its name and its folder's Canonical Name.

**Filtered by name**: the browser shows only the Assets whose name contains the typed text, ignoring letter case and surrounding whitespace; an empty filter shows all.

## Implementation Decisions

- **Flow**: the Editor shows the platform's native folder dialog and then the Canonical Name prompt, and sends LibraryManager one AddFolder message carrying the path and the name. LibraryManager trims the name and checks, in this order: the name is not blank; the folder exists and can be listed once symbolic links, `.` components, and letter case the file system ignores are resolved; it is not already added and neither inside nor around an added folder; the name is not in use on this device, compared Unicode-normalised and in lower case. _Why_ the folder before the name: re-adding a folder under its own name is then told that the folder is already added, not that the name is taken. The step is then applied and recorded in the history, and answered with a FolderAdded message carrying what the scan skipped, or a FolderRefused message with its reason. The Editor keeps the prompt open with the reason for a refusal about the name; any other refusal closes it and is shown in the status line, as is the count of skipped entries.
- **The Manifest** lives in the editor's configuration directory: one JSON file per Asset Folder, named by its folder key, holding the key, the folder's path as the Author gave it, the Canonical Name, the version, and the renames (none). The version is the UTC day the folder was added. A Manifest is written through a sibling temporary file and a rename, so a reader never sees half of one. The set of folders in use on this device is the set of Manifests in that directory; forgetting a folder removes its Manifest and stops serving it. A file in that directory that is not a Manifest, or whose name is not the key inside it, is logged and skipped.
- **The folder key** is a device-local identifier minted when the folder is added. It names the Manifest, the index cache, and the `lib://` source entry, and it is never written into a Project; Projects know a folder by its Canonical Name.
- **Scanning** is a synchronous, stat-only walk through LibraryAccess: directory entries and file metadata are read, nothing is opened. Hidden entries and symbolic links are left out silently; names that are not valid Unicode, subfolders that cannot be listed, and entries whose metadata cannot be read are counted as skipped; the folder itself being unlistable is an error. The scan lists every file found, Asset or not, ordered by place, and is diffed against the folder's index cache (added, removed, changed by size or modification time), after which the cache is rewritten. _Why_ every file: the cache knows files by their metadata alone; which of them are Assets is decided afterwards by classifying them.
- **The index cache** lives in the editor's cache directory under the folder key, as JSON. It is only a cache: one that is missing or cannot be read counts as empty, and undoing Add Asset Folder leaves it in place.
- **Classification** is CatalogEngine's Classify with one built-in Indexing Rule: the image extensions, compared ignoring case, give the image Asset Kind. A further rule is another Indexing Rule, not a change to the Engine. An indexed Asset carries its name (the file stem), its place (its relative path with `/` separators, spelled as on disk), its Asset Kind, its byte size, and its modification time.
- **The `lib://` source**: every Asset is loaded through one dynamic asset source, `lib://<folder-key>/<place>`, whose reader looks the key up in a table LibraryAccess changes at runtime as folders are added and forgotten; scanning registers the folder, forgetting removes it. The reader refuses any path that is not a plain relative path below a registered folder. The Host registers the source before the asset plugin builds, because asset sources freeze then, and turns `.meta` lookups off, since Asset Folders never hold them.
- **Directories**: LibraryAccess resolves the platform's configuration and cache directories; a resource in `model` overrides either, so tests and development builds point them at other directories.
- **Startup**: LibraryManager reads every Manifest, spawns an Asset Folder entity for each with an empty index, and Refreshes it, which is the same scan diffed against its cache. A folder whose scan fails keeps its entity and its Manifest and is reported with a FolderUnavailable message, which the Editor shows in the status line. The Asset Folder Changed event is not raised; nothing listens for it.
- **In the World**, each Asset Folder is an entity carrying its Canonical Name, folder key, path, version, index of Assets, and the counts of what the last scan skipped, written only by LibraryManager. The Editor reads it to fill the browser and filters by name itself: folders are listed by Canonical Name with how many Assets match, each Asset by name with the subfolder it sits in.
- **Undo and redo**: the recorded step holds the Manifest. Undoing forgets the Manifest and despawns the entity; redoing writes the same Manifest again (same key, name, and version) and scans the folder again.
- **Ordering**: LibraryManager handles its Commands in the set `model` orders for every Manager, Commands before Undo before Redo within a frame, so a Command and the Undo sent in the same frame apply in the order the Author gave them.

## Test seams

The automated seam is a headless Bevy App of the real plugins of `model`, `history`, LibraryAccess, and LibraryManager over fixture folders in a temporary directory, with the editor's directories pointed at temporary ones, driven by messages and asserted on the World, the configuration directory, and the cache directory. `Add Asset Folder is undoable` runs in the Host's test App, which also holds AuthoringManager and ProjectManager. _Why_ two Apps: the workspace check counts development dependencies, so the LibraryManager seam must not depend on another Manager.

- **Folder name proposed**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Blank names are refused**: `crates/drs-library-manager/tests/asset_folders.rs::blank_names_are_refused`
- **Names are unique on this device**: `crates/drs-library-manager/tests/asset_folders.rs::names_are_unique_on_this_device`
- **The same folder is refused**: `crates/drs-library-manager/tests/asset_folders.rs::the_same_folder_is_refused` (the symbolic link on Unix only; the letter case on macOS only, where the file system folds it)
- **Nested folders are refused**: `crates/drs-library-manager/tests/asset_folders.rs::nested_folders_are_refused`
- **Unreadable folders are refused**: `crates/drs-library-manager/tests/asset_folders.rs::unreadable_folders_are_refused`
- **Any path works**: `crates/drs-library-manager/tests/asset_folders.rs::any_path_works` (indexing; loading is checked by hand)
- **Nothing is written into the folder**: `crates/drs-library-manager/tests/asset_folders.rs::nothing_is_written_into_the_folder`
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

## Not supported

- The editor never writes into an Asset Folder: no Manifest, cache, or sidecar file of any kind.
- The folder key never leaves the device: a Project never records it.

## Notes

- Four Rules (Folder name proposed, Cancelling records nothing, Browsed across folders, Filtered by name) have no automated test, against the requirement that every Rule has one. They are behaviour of the egui interface alone, for which no headless seam exists; the accepted deviation is verification by hand, driving the editor with the development-only input script.
- Development builds take the folder from an environment variable instead of opening the dialog, drive the editor from a script of input steps, save a screenshot on request, and keep the editor's own files under a directory of choice. That is tooling for verification, not behaviour of the capability.
