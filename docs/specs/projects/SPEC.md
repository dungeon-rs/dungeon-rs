# Projects

**Commands**: Relink, Embed Asset

## Purpose

The Author's work must outlive the editor and travel between devices. This capability saves the Project to a single file wherever the Author likes and reopens it later, on this device or another, with every Element where it was and every Asset found again through the Canonical Name of its Asset Folder, however that folder is located here. What cannot be found stays in the Project as a placeholder and is explained in plain terms, and saving keeps it, so a collaborator who lacks a folder never damages the Project.

Relink and Embed Asset are owned here and not implemented; Save and Open are lifecycle operations of the Project, not domain Commands.

## User Stories

### Saving

1. As an Author, I can save the Project to a file I pick in my operating system's save dialog, so that my work survives closing the editor.
2. As an Author, I can type a name without the Project file extension and have it added, so that the file is recognised later without me remembering the extension.
3. As an Author, I can press Save and have the Project written to the file it was last saved to or opened from without being asked again, so that saving is one key.
4. As an Author, I can press Save on a Project that has never been saved and be asked where, so that I am never asked for a location twice nor saved somewhere I did not choose.
5. As an Author, I can Save As under a new name and continue working in that file, so that I can keep versions.
6. As an Author, I can read the Project's name and whether it has unsaved changes in the editor's title, so that I know at a glance whether I can close it.
7. As an Author, I am told why a save to a read-only or vanished location failed, with my work and the previous file left as they were, so that a bad location costs me a retry, not a map.
8. As an Author, I can rely on a save that fails halfway leaving the previous file intact, so that a full disk never leaves me with half a Project.
9. As an Author, I can save and open a Project file whose path holds spaces, quotes, accents, non-Latin letters, or symbols like any other, so that my folder layout is never a problem.
10. As an Author, I can rely on the file holding nothing about my device, such as where my Asset Folders are, so that the file works on a collaborator's device.
11. As an Author, I can rely on saving keeping every Asset Reference and every Element, including those whose Asset is missing on this device, so that opening a Project on the wrong device never loses anything.
12. As an Author, I can save the same Project twice and get the same file, so that a version control tool shows only real changes.
13. As an Author, I can rely on saving changing nothing on screen and adding no undo step, so that saving is never something I have to undo around.
14. As an Author, I can undo back to the state I saved and have that count as no unsaved changes, so that the unsaved marker tells the truth.

### Reopening

15. As an Author, I can open a Project file through my operating system's open dialog, filtered to Project files, so that I find my Projects quickly.
16. As an Author, I can rely on the opened Project replacing what I was working on, with every Prop where it was, at its size, in its stacking order, so that I continue exactly where I left off.
17. As an Author, I am asked whether to save, discard, or cancel when I open a file while I have unsaved changes, so that I never lose work by opening another Project.
18. As an Author, I am asked the same question when I quit with unsaved changes, so that closing the window never loses work.
19. As an Author, I can cancel at that question, or cancel the save it leads to, and be left exactly where I was, so that a wrong click costs nothing.
20. As an Author, I can rely on an opened Project starting with an empty undo history, so that undo never reaches into a session that is gone.
21. As an Author, I am refused a file that is not a Project, is damaged, or cannot be read, with the reason and my current Project untouched, so that a bad file never takes down what I have open.
22. As an Author, I am refused a Project saved by a newer version of the editor than mine, with a plain explanation rather than a Project opened with parts missing, so that I never unknowingly save a Project with parts of it dropped.
23. As an Author, I can open a Project holding data this editor does not know, such as a property added by another version or a Plugin, and save it with that data unchanged, so that passing a file through my editor never strips it.
24. As an Author, I can rely on an Element of a kind this editor does not know staying in the Project as a placeholder, being mentioned when the Project opens, and being saved back untouched, so that a collaborator's Plugin content survives a round trip through my editor.
25. As an Author whose Asset Folder sits at a different path than on the device the Project was saved on, I can rely on every Asset being found through the folder's Canonical Name, so that the Project never cares where my library is.
26. As an Author on a device that lacks one of the Project's Asset Folders, I can see the Props from it as placeholders of the right size and am told which Assets are missing, from which Asset Folder and version, and how many Elements use each, so that I know what to install.
27. As an Author whose copy of an Asset Folder is an older version than the Project was saved against, I am told, for an Asset my version lacks, the version the Project recorded and the version I have, so that I know the gap is a version, not a mistake.
28. As an Author whose copy of an Asset Folder is a different version, I can rely on the Assets present in my copy being used without comment, so that a version difference alone is never noise.
29. As an Author whose operating system spells an Asset's path differently in letter case or Unicode form than the device the Project was saved on, I can rely on the Asset being found anyway, so that moving between operating systems never produces false Missing Assets.
30. As an Author who touched up an image without renaming it, I can rely on the Project using the image at its place as it is now, so that an edit to my own art is picked up.
31. As an Author, I can read the report of Missing Assets and dismiss it, keeping the placeholders, so that I can work on the rest of the map meanwhile.
32. As an Author who opened a Project with Missing Assets, I can add the Asset Folder under its Canonical Name and see them appear without reopening the Project, so that fixing a missing library is one step.
33. As an Author sharing a Project with a collaborator who lacks one of my Asset Folders, I can rely on the Project coming back from them with every Asset Reference and Element intact, so that their saving never damages my map.
34. As an Author, I can open a Project on a device with no Asset Folders at all, with every Prop a placeholder, so that I can at least look at the map and see what it needs.
35. As an Author, I can rely on the Project remembering the file it was opened from, so that Save after Open goes to that file.

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

## Implementation Decisions

The technology the architecture fixes (the Project format principles of stable component names, per-component versions, and unknown data passed through; the asset identity table; the resolution order; the `lib://` source; deterministic saves) is used as written there and not restated.

### The Project file

- **Shape**: one JSON document, pretty-printed in a fixed key order and ending in a newline. It holds the format version, which is 1; the Project entity's envelopes (the Project, the Grid, the Bounds, and the Asset Reference table); the Levels in order, each with its own envelopes and its Layers in order, each Layer with its envelopes and the ElementIds of its Elements in stacking order; and the Elements, keyed by ElementId written as a decimal string and sorted by it. Every component is an envelope of a version and that component's data under its stable name (`project`, `grid`, `bounds`, `asset_references`, `level`, `layer`, `element`, `prop`). The extension is `.dungeon`, a fact of the product that `model` states; the file's shape and its version live in ProjectAccess and nowhere else.
- **The snapshot**: `model` holds a format-agnostic snapshot of a whole Project (the Project's envelopes, its Levels and Layers in order, its Elements by identity). ProjectManager gathers it from the World on Save and materialises it into the World on Open; ProjectAccess's ReadProject and WriteProject map it to and from the file's shape. _Why_: the snapshot is the model's view of a Project and the file is ProjectAccess's volatility, so a new format version changes one crate and no Manager.
- **Serialisation registry**: `model` holds a registry of serialisable components, each with its stable name, the version it writes, the tier of entity it belongs on (Project, Level, Layer, or Element), and how to read every version it has had; each crate registers the components it owns when its plugin is built, in one declaration that both implements and registers them, so a component is never implemented but forgotten. All components of a Project are `model`'s today, each at version 1. The Project's name is not written: it is the file's. Reading checks the format version first, then every known envelope's version, so a newer file is refused by name and version before anything is built. An envelope whose name no entry knows is kept verbatim on its entity, under a component that holds unknown envelopes, and written back unchanged. A known envelope under another tier than its component's is malformed and refused. _Why_: a Level's envelope on the Project entity would otherwise read as a second Level.
- **Atomic writes**: WriteProject writes to a temporary file beside the target, flushes it to the disk, and renames it over the target once complete, so a failed write or a crash leaves the previous file untouched and never a partial one; the temporary file is removed on failure. It is created as readable as any file the Author makes, not private to the owner as temporary files are. _Why_: a saved Project is for sharing.
- **Not in the file**: the history, the selection, the Viewport, folder keys, paths, the Project's name, and the resolution table.

### Save, Open, and quit

- **Requests**: the Editor sends ProjectManager a Save message with a path for Save As or none for Save, and an Open message with a path; ProjectManager answers with a message that says the Project was saved or opened, or why the request was refused, which the Editor shows in its status line. The Editor runs the platform's native save and open dialogs on the main thread, filtered to Project files, the save dialog proposing the Project's name with the extension.
- **Save**: ProjectManager gathers the Project from the World through the registry, asks ProjectAccess to WriteProject, and on success names the Project after the file and records the path and the history's position as the saved mark. Save without a path and without a remembered file is refused, which is how the Editor knows to ask where. A refusal changes nothing.
- **Open**: ProjectManager asks ProjectAccess to ReadProject, then materialises the whole tree beside the current Project; a failure anywhere despawns what was built and refuses the file, so nothing of the current Project changes until the opened one stands complete. It then despawns the current Project, clears the history, records the file and the history's position as the saved mark, resolves every Asset Reference, abandons any Export in progress, and reports. Materialising a saved Project is lifecycle, not composing, so ProjectManager writes every component here through the registry, the one exception to each crate writing only its own components. A Level without its component, a Layer without its component, an Element without its common component, an Element listed on no Layer or on two, or a listed identity the file does not hold is malformed and refused with the reason.
- **The saved mark**: a resource in `model` holding the Project's file, or none, and the history's position at the last save or open, written only by ProjectManager. `history` exposes its position as a comparable value that never repeats a value an earlier step had, so undoing back to the mark compares equal and a new step never does; clearing the history puts it back at the position of an empty one. The Project has unsaved changes exactly when the two differ; the mark also names the Project (`Untitled` without a file), one source for the title and the dialogs.
- **The question**: the Editor asks whether to save, discard, or cancel whenever Open or Quit is chosen while the mark says the Project has unsaved changes, in a modal dialog that names the Project and the action. Save in it runs Save (or Save As, without a file) and keeps the dialog open as awaiting until the Manager answers: a success lets the action go ahead, a refusal keeps the dialog open with the reason, and cancelling the save dialog closes the question. Discard goes ahead; Cancel, Escape, or a click outside closes the question and nothing else happens. Quitting is the Editor answering the window's close request itself: the Editor describes its own window, which the Host sets on Bevy's defaults, and that window is not closed on request, so the question is asked first; a close request during a question asked before opening another Project turns it into the question before quitting. Only the Editor ends the application.
- **Shortcuts**: Open, Save, Save As, and Export Level take the platform's usual shortcuts (Command or Control with O, S, Shift-S, and E); Quit takes Command-Q on macOS and Linux and has no shortcut of the editor's own on Windows, where the window's own Alt-F4 is the way. The shortcuts wait while a dialog of the Editor's own is open or a text field has the keyboard.
- **Resolution**: ProjectManager asks CatalogEngine to Resolve each Asset Reference against the Asset Folders in the World, ordered by folder key so that the outcome never depends on entity order. CatalogEngine finds the folder whose Canonical Name is the recorded one, compared as uniqueness compares them (Unicode-normalised, ignoring letter case), and takes the first two steps of the architecture's order: the first recorded place at which an Asset sits exactly, then the first recorded place that exactly one Asset's place matches ignoring letter case and Unicode normalisation (fully case-folded, so `Straße` and `STRASSE` match); two or more such Assets make the reference Missing as ambiguous. Nothing is read from disk, and neither versions nor sizes nor fingerprints are compared. The outcome is the resolution table, a component of the Project entity that is never saved, written only by ProjectManager, with one row per Asset Reference row: the folder key and the place as spelled on disk that loads it, or Missing with its reason (the folder absent; the Asset absent from a folder of a named version; or ambiguous, with the candidates and the folder's version). RenderEngine reads the table to load through `lib://` or draw a placeholder, and no longer looks folders up by Canonical Name itself; the Editor reads it to count placeholders.
- **When resolution runs**: once per frame, after every Manager has handled its Commands, Undo, and Redo. The whole Project is resolved again whenever its Asset Reference table changes, so a row a placement has just added is resolved before anything draws it; the rows recorded against one Canonical Name are resolved again whenever LibraryManager announces Asset Folder Changed for that name, which it does after Add Asset Folder is applied, after it is undone or redone, and at startup for each remembered folder. _Why_ after the last Manager set: every set can change what resolution depends on, and the folder's Manager announces the change in the same frame, so resolving once after them all lets the sprites drawn later in the frame see the rows resolved in the frame they appeared.
- **The report**: ProjectManager's answer to Open carries, in the order of the Asset Reference table, each Missing Asset with its name, its folder's Canonical Name, the version the Project recorded for that folder, how many Elements use it, and why it is Missing; and each Element kind this editor does not know, ordered by name, with how many Elements have it. The Editor summarises the counts in the status line and, when either list is non-empty, shows the report in a modal dialog the Author dismisses, each Missing Asset on one line in plain terms, with the folder's version on this device beside the recorded one when the folder is present.
- **Unknown kinds**: an Element whose kind is not in the Element kind registry is spawned with its identity, its common Element component, and its unknown envelopes, and no kind component; RenderEngine draws it as the placeholder; the report counts it; Save writes it back as read.

## Test seams

The automated seam is a headless Bevy App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, and AuthoringManager (ProjectAccess and CatalogEngine are called by the Managers), with no window and no RenderEngine, over fixture Asset Folders and Project files in temporary directories. Each App is one device with editor directories of its own, driven by Save, Open, Apply, Undo, Redo, and Add Asset Folder messages, and asserted on the World, the files written, and the messages answered; a second App over other directories plays the other device, a folder with a file fewer or none at all plays a device that lacks the Asset or the folder, and a Manifest written before start plays a remembered folder of a given version. The read-only half of `A failed save leaves the old file` runs on Unix only and skips itself where the process may write into a read-only folder. The ambiguity of two spellings runs as a unit test of CatalogEngine over in-memory Asset Folders, because a case-insensitive file system cannot hold both.

- **Saved as one file**: `crates/drs-app/tests/projects.rs::saved_as_one_file`
- **Save remembers its file**: `crates/drs-app/tests/projects.rs::save_remembers_its_file`
- **Save without a file asks**: `crates/drs-app/tests/projects.rs::save_without_a_file_asks` (the refusal without a remembered file; the dialog is checked by hand)
- **Save As moves the Project**: `crates/drs-app/tests/projects.rs::save_as_moves_the_project`
- **Named by its file**: by hand: no automated seam for the window title; verified by driving the editor with the dev-only input script
- **Everything the Project is**: `crates/drs-app/tests/projects.rs::everything_the_project_is`
- **Nothing of this device**: `crates/drs-app/tests/projects.rs::nothing_of_this_device`
- **History is not saved**: `crates/drs-app/tests/projects.rs::history_is_not_saved`
- **Saving keeps every reference**: `crates/drs-app/tests/projects.rs::saving_keeps_every_reference`
- **Saving is a fixed point**: `crates/drs-app/tests/projects.rs::saving_is_a_fixed_point`
- **A failed save leaves the old file**: `crates/drs-app/tests/projects.rs::a_failed_save_leaves_the_old_file` (a vanished location everywhere; a read-only one on Unix)
- **Any path works for Projects**: `crates/drs-app/tests/projects.rs::any_path_works_for_projects`
- **Saving is not a step**: `crates/drs-app/tests/projects.rs::saving_is_not_a_step`
- **Unsaved means a step since the save**: `crates/drs-app/tests/projects.rs::unsaved_means_a_step_since_the_save`
- **Opened as saved**: `crates/drs-app/tests/projects.rs::opened_as_saved`
- **Open replaces the Project**: `crates/drs-app/tests/projects.rs::open_replaces_the_project`
- **Unsaved changes are asked about**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script, whose `close` step presses the window's close button
- **A bad file is refused**: `crates/drs-app/tests/projects.rs::a_bad_file_is_refused`, `crates/drs-app/tests/projects.rs::a_misplaced_envelope_is_refused`
- **A newer file is refused**: `crates/drs-app/tests/projects.rs::a_newer_file_is_refused`
- **Unknown components round-trip**: `crates/drs-app/tests/projects.rs::unknown_components_round_trip`
- **Unknown kinds are kept**: `crates/drs-app/tests/projects.rs::unknown_kinds_are_kept` (all but the drawing, which is checked by hand)
- **Resolved by Canonical Name and place**: `crates/drs-app/tests/projects.rs::resolved_by_canonical_name_and_place`, `crates/drs-app/tests/projects.rs::any_path_works_for_assets`
- **Spelling differences resolve**: `crates/drs-app/tests/projects.rs::spelling_differences_resolve`, `crates/drs-catalog-engine/src/resolve.rs::tests::two_spellings_are_ambiguous`
- **Same place, changed content**: `crates/drs-app/tests/projects.rs::same_place_changed_content`
- **Versions do not block**: `crates/drs-app/tests/projects.rs::versions_do_not_block`
- **Missing Assets stay**: `crates/drs-app/tests/projects.rs::missing_assets_stay` (all but the drawing, which is checked by hand)
- **Missing Assets are explained**: `crates/drs-app/tests/projects.rs::missing_assets_are_explained` (the facts reported; the dialog is checked by hand)
- **Resolved when a folder arrives**: `crates/drs-app/tests/projects.rs::resolved_when_a_folder_arrives`
- **Recorded versions stay**: `crates/drs-app/tests/projects.rs::recorded_versions_stay`
- **Opened without folders**: `crates/drs-app/tests/projects.rs::opened_without_folders`
- **Identity survives the file**: `crates/drs-app/tests/projects.rs::identity_survives_the_file`

## Not supported

- The file never holds a path, a folder key, or anything else of one device.
- Opening a file never drops data: a newer file is refused whole rather than opened in part, and unknown data is carried, never stripped.
- Saving never prunes: Asset References no Element uses, Missing Assets, and unknown Elements are written as they are.
- Filename-only matches are never accepted, and two files differing only in spelling are never chosen between.

## Notes

- Two Rules (Named by its file, Unsaved changes are asked about) have no automated test, against the requirement that every Rule has one, and three more (Save without a file asks, Missing Assets are explained, Missing Assets stay with Unknown kinds are kept) are tested up to the dialog or the drawing. They are behaviour of the window, the egui interface, and rendering, for which the headless seam has no window and no renderer. The accepted deviation is verification by hand, driving the editor with the development-only input script, which also stands in for the file dialogs through environment variables.
- Unsaved means a step since the save counts Add Asset Folder, which changes device state and not the Project, as a step: adding a folder and quitting asks about saving. Accepted as the conservative side.
- Relink and Embed Asset are owned here and not implemented; the content-hash, Manifest-redirect, and format-twin steps of resolution, suggestions the Author confirms, updating the recorded Asset Folder versions, migrations between format versions, a New Project command, recent files, autosave, and more than one Project open at a time do not exist.
