# Composing

**Commands**: Place Element, Edit Element, Remove Element, Restack, Paint, Set Portal into Wall, Free Portal

## Purpose

The Author builds a Level by putting Elements on its Layers. This capability lets the Author choose an Asset and place it as a Prop where they click, at the size the vendor meant, move it, remove it, and take every step back and forward again, so that composing is a line of small, reversible gestures. The editor opens on a Project to compose on at once.

## User Stories

1. As an Author, I can open the editor on a new, unsaved Project with one Level and one Layer, so that I can place something at once.
2. As an Author, I can choose an Asset and click on the Level to place a Prop centred where I clicked, so that placing is one gesture.
3. As an Author, I can see a placed Prop at its natural size in Grid cells, so that a vendor's table is as big as the vendor meant.
4. As an Author, I can rely on each new Prop landing on top of the Props already on the Layer, so that what I place last is what I see.
5. As an Author, I can place the same Asset many times, so that a room gets as many barrels as it needs.
6. As an Author, I can place a Prop outside the Bounds, so that the Bounds never get in the way of composing.
7. As an Author, I can press Escape to stop placing and go back to selecting, so that I never place by accident.
8. As an Author, I can click a Prop to select it, with the topmost one winning when they overlap, so that I always get the one I see.
9. As an Author, I can drag a selected Prop to move it and have the whole drag be a single undo step, so that undo takes the Prop back to where the drag began, not one pixel back.
10. As an Author, I can press Delete to remove the selected Prop, so that removing is one key.
11. As an Author, I can undo and redo with the usual shortcuts, so that I never need a menu to take a step back.
12. As an Author, I can rely on an undone removal bringing the Prop back exactly where it was, including its place in the stacking order, so that undo never reshuffles my Level.
13. As an Author, I can rely on a new action after undoing discarding the undone steps, so that history stays a single line I can reason about.
14. As an Author, I can pan and zoom the viewport without that showing up in undo, so that looking around never costs me a step.
15. As an Author, I can see a Prop whose image cannot be loaded as a placeholder of the right size while the editor keeps running, so that a deleted or broken file never takes the editor down.
16. As an Author, I am told in the status line when a Command could not be carried out, with nothing changed, so that a failure costs me a retry and never a crash.

## Rules

**A Project to start with**: the editor opens with a new, unsaved Project holding one Level named `Level 1` with one Layer named `Layer 1`.

**Placed where clicked**: with an Asset chosen, a click on the Level places a Prop of that Asset on the current Layer, centred on the clicked point.

**Natural size**: a Prop's size in Grid cells is its image's pixel size divided by the Grid's pixels per cell, which is 256.
_Why_: the convention of the functional baseline, so its libraries place at the size their vendors meant.

**Placed on top**: a new Prop is placed above every Element already on its Layer.

**Placement records a reference**: placing a Prop records in the Project an Asset Reference holding the Asset's name, the Canonical Name of its Asset Folder, the place in that folder it was placed from as the first place it is known to sit, its byte size, its pixel size, and its content fingerprint; placing a second Prop of the same Asset adds no second Asset Reference. Follows from: A Project is device-independent.

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

**A failed load is a placeholder**: an Element whose Asset cannot be loaded or decoded, an Element whose Asset is Missing, and an Element of a kind this editor does not know are drawn as the same placeholder of their recorded size, stay on their Layer, and the editor keeps running.

**A failed Command is reported**: a Place Element, Edit Element, or Remove Element that cannot be carried out is answered with the reason, places or changes nothing, and records no history step.

## Implementation Decisions

- **The new Project** is created at startup by ProjectManager, the owner of the Project lifecycle: named `Untitled`, with one Level, one Layer, a Grid of 256 pixels per cell, and default Bounds of thirty by thirty cells from the origin, which are neither drawn nor edited.
- **Model**: the Project is an entity carrying its Grid, its Bounds, its Asset Reference table, and the resolution table that says where each Asset Reference loads from on this device, which ProjectManager alone writes and no file holds; its Levels are its children, each Level's Layers are that Level's children, and each Layer's Elements are that Layer's children in stacking order, the first drawn first. Every Element carries its kind, its position (its centre in Grid cells, `x` to the right and `y` upwards from the Level's origin), its size in cells, and an ElementId: a stable identity Commands and the history address it by, never the entity handle, which changes whenever an Element is respawned. Prop is the first descriptor in the Element kind registry; its default Material shows its image, and it refers to its Asset by the Project-local row of the Asset Reference table.
- **Asset Reference rows** hold the folder's Canonical Name, every place the Asset is known to sit (Unicode-normalised, with `/` separators, the first being where it was placed), the name as shown, the Asset Kind, the content fingerprint (`blake3:` followed by the hex digest), the byte size, and the pixel size; one row per Asset Folder holds its Canonical Name and version. A row is found by any of its places. Rows are never removed by composing: undoing a placement leaves them, and placing the Asset again reuses them.
- **Place Element**: the Editor sends AuthoringManager Apply(Place Element) with the Layer, the position, and the chosen Asset (folder key and place). AuthoringManager finds the Asset in the added folder's index, asks LibraryAccess to LoadAsset, which reads the file once for its byte size, pixel size, and fingerprint, records the rows, and spawns the Element with a fresh ElementId as the last child of the Layer; ProjectManager resolves the table's new row in the same frame, before anything draws it. Undo despawns it; redo spawns it again with the same identity, appended, which is on top because every later step was undone first.
- **Edit Element**: the Editor sends a drag as a sequence of Apply(Edit Element) messages for the position, marked as the beginning, continuation, and end of one gesture from press to release, the end sent at the pointer's last position; AuthoringManager records them as one history Group of the generic field-setting command, so the step undoes to the position at press. A change marked as single is a step of its own. The position is the only property an Edit Element carries. A press that moves the pointer less than a few pixels is a click, not a drag. Undo and redo, from the keys or the menu, wait while a drag is under way. _Why_: the drag is one step that is still being recorded.
- **Remove Element**: the generic reflection-snapshot command, extended with the Element's index among its Layer's children; undo restores the entity and rebuilds the Layer's whole order with it at that index, so the Elements above it keep their places.
- **Steps of their own**: Place Element, Remove Element, and Add Asset Folder each close any gesture group left open before recording, so none joins a drag.
- **Failures**: an authoring Command that cannot be carried out (a chosen Asset in no added folder or not in its index, a file that cannot be read or is not an image, a target that is not a Layer or an Element) is answered with a CommandFailed message carrying the reason, and nothing is recorded. An undo or redo that fails is answered with a HistoryFailed message, and the step stays where it was in the history. The Editor shows either in its status line.
- **Undo and redo** are handled by AuthoringManager for the one history, whichever Manager recorded the step: an Undo or Redo message takes back or repeats the most recent step, be it a placement or an added folder.
- **Ordering**: `model` orders every Manager's handling within a frame, Commands before Undo before Redo, so a Command and the Undo sent in the same frame apply in the order the Author gave them whichever Manager handles each.
- **The Viewport** (the cell at the centre of the view, the zoom in logical pixels per cell between 4 and 1024, and the area of the window the Level is shown in) is presentation state in `model`, written only by the Editor and followed by RenderEngine's projection; it starts centred on the origin at 64 pixels per cell; its conversions between cells and screen points are the ones picking and drawing share. _Why_ it lives in `model`: the Editor may not depend on RenderEngine, so the type both use sits with the other shared contracts. Panning is the middle button, Space with the left button, or a plain trackpad scroll; zooming is the wheel, a pinch, or a scroll with Command or Control held, around the pointer.
- **Rendering**: RenderEngine keeps one sprite per Element through change detection, placed one unit apart along the camera's axis in stacking order through every Level, so a later Element is drawn over an earlier one. A Prop's image is loaded through the `lib://` handle from the folder key and the place, spelled as on disk, that the Project's resolution table gives for its Asset Reference, at the Element's size and position; RenderEngine never looks a folder up by Canonical Name itself. An image that fails to load, a Missing Asset, a row not yet resolved, and an Element of a kind this editor does not know give the same flat coloured placeholder of the Element's size. The projection is a 2D camera with one cell per world unit that follows the Viewport.
- **Picking and selection** are the Editor's: it maps pointer positions to cells through the Viewport and hit-tests the Props' rectangles from the topmost Layer down and the last-drawn Element back. The selection lives in the Editor, is outlined on the Level, is dropped when its Prop is gone or when an Asset is chosen for placing, and is never in the history. The current Layer is the Project's only Layer.
- **Keyboard**: Escape leaves placing; Delete, and Backspace on macOS too, removes the selection; the platform's standard undo and redo shortcuts drive history, stated once so that the menu shows exactly the keys that work. Nothing happens while a text field has the keyboard.

## Test seams

The automated seam is a headless Bevy App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, and AuthoringManager over one fixture Asset Folder of images with known pixel sizes, driven by messages and asserted on the World; it has no window and no RenderEngine. `A Project to start with` runs ProjectManager alone over `model`.

- **A Project to start with**: `crates/drs-project-manager/tests/new_project.rs::a_project_to_start_with`
- **Placed where clicked**: `crates/drs-app/tests/composing.rs::placed_where_clicked` (the Command's centring; the mapping from the click to cells is checked by hand)
- **Natural size**: `crates/drs-app/tests/composing.rs::natural_size`
- **Placed on top**: `crates/drs-app/tests/composing.rs::placed_on_top`
- **Placement records a reference**: `crates/drs-app/tests/composing.rs::placement_records_a_reference`
- **Placement records the folder**: `crates/drs-app/tests/composing.rs::placement_records_the_folder`
- **Anywhere on the Level**: `crates/drs-app/tests/composing.rs::anywhere_on_the_level`
- **Many of the same**: `crates/drs-app/tests/composing.rs::many_of_the_same`
- **Escape stops placing**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **Topmost is selected**: by hand: no automated seam for the egui UI; verified by driving the editor with the dev-only input script
- **A drag is one step**: `crates/drs-app/tests/composing.rs::a_drag_is_one_step`
- **Removal is reversible in place**: `crates/drs-app/tests/composing.rs::removal_is_reversible_in_place`
- **Identity survives undo**: `crates/drs-app/tests/composing.rs::identity_survives_undo`
- **Redo repeats exactly**: `crates/drs-app/tests/composing.rs::redo_repeats_exactly`
- **A new step clears redo**: `crates/drs-app/tests/composing.rs::a_new_step_clears_redo`
- **One history**: `crates/drs-app/tests/composing.rs::one_history` (Add Asset Folder and Place Element; Edit Element and Remove Element are covered as steps by `a_drag_is_one_step` and `removal_is_reversible_in_place`)
- **View is not a step**: `crates/drs-app/tests/composing.rs::view_is_not_a_step` (a changed Viewport records nothing; the mapping from the pointer and the wheel to the Viewport is checked by hand)
- **A failed load is a placeholder**: `crates/drs-app/tests/projects.rs::missing_assets_stay`, `crates/drs-app/tests/projects.rs::unknown_kinds_are_kept` (the Element staying on its Layer; the drawing is by hand: no headless seam renders the viewport; verified by driving the editor with the dev-only input script)
- **A failed Command is reported**: `crates/drs-app/tests/composing.rs::a_failed_command_is_reported` (Place Element; Edit Element and Remove Element on an unknown Element are checked by hand)

## Not supported

- Panning, zooming, and selecting are never Commands and never history steps.
- An Element is never addressed by its entity handle across a Command or a history step; only its ElementId is stable.

## Notes

- `A Project to start with` lives here because it is the Project the Author composes on; saving and reopening a Project are the projects capability's.
- Two Rules (Escape stops placing, Topmost is selected) have no automated test, against the requirement that every Rule has one, and the drawing of A failed load is a placeholder is checked only by hand. They are behaviour of the egui interface and of the viewport's rendering, for which no headless seam exists. The accepted deviation is verification by hand, driving the editor with the development-only input script, which also confirms that a placed Prop is drawn where and as large as the model says.
- Add Asset Folder changes device state, not Project state, yet it sits in the same history as composing steps because every Manager records into the one history. Undo therefore always reaches a Prop before the folder it came from.
