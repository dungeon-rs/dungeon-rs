# Erase and reshape strokes

**Capabilities**:
- composing: Paint, Edit Element, Remove Element
- projects: none of its Commands; saving and opening Terrain whose strokes erase

## Problem Statement

A Terrain keeps every stroke as it was laid, but the Author can only add to it: a stroke that spills over a doorway stays there unless it is undone, together with everything painted since, and a floor cannot be thinned out or cut back without repainting it. The functional baseline erases into pixels, so its erases are as fixed as its paint; and in both, a stroke laid a little too wide, too hard, or a cell to the left is there for good.

## Solution

The Paint tool erases as well as paints: `E` switches it to erasing, and a stroke drawn then takes ground away from the Layer's Terrain with the same soft round Brush. An erase is a stroke like any other: it is kept in the Terrain's strokes in the order it was laid, saved with them, and undone and redone the same way. It only ever lowers what the strokes before it show, by the strongest erase there, so going over a joint or a patch twice never takes more away than the Brush says.

Every stroke stays editable once laid. The Paint tool's third mode, Edit strokes, picks a stroke of the current Layer's Terrain by clicking it, the latest laid winning, and shows its path with a handle at each point. The Author drags a handle to move that point, drags the stroke to move it whole, changes its size, hardness, and strength in the options strip, turns it from painting to erasing or back, and removes it with Delete; each of these is one undo step, and the ground is redrawn from the strokes as they now stand, recomputing only where the stroke was and is.

## User Stories

### Erasing

1. As an Author, I can press `E` to choose the Paint tool erasing, so that taking ground away is one key from any tool.
2. As an Author, I can choose Erase in the Paint tool's options, so that I can switch to erasing without the keyboard.
3. As an Author, I can press `B` while erasing to go back to painting, so that switching between the two never needs the pointer.
4. As an Author, I can press on the Level, drag, and release while erasing to lay one erase along the path my pointer took, so that erasing is the same gesture as painting.
5. As an Author, I can press and release without moving while erasing to erase one round dab, so that a single spot is one click.
6. As an Author, I can erase with the same Brush size, hardness, and strength I paint with, so that I tune one Brush, not two.
7. As an Author, I can see the Brush's circle and the band of the erase I am drawing in a colour of their own while erasing, so that I never mistake an erase for paint before I let go.
8. As an Author, I can read in the status line that a drag erases and Escape stops, so that I know what my next drag will do.
9. As an Author, I can see the ground gone along an erase once I release, the Brush's soft edge fading the erase out as it fades paint in, so that an erase blends as paint does.
10. As an Author, I can erase with less than full strength and see the ground thinned rather than gone, never below what the strength leaves, so that worn patches are one setting.
11. As an Author, I can rely on a half-strength erase over half-strength ground leaving it as it was, so that an erase says how much may remain, not how much it takes away.
12. As an Author, I can rely on an erase never taking more away where it turns sharply, crosses itself, or loops back, so that an erase looks as even as the Brush I set.
13. As an Author, I can rely on erasing the same ground twice never taking more away than the strongest of the two erases, so that I can erase without counting how often I passed.
14. As an Author, I can paint again over ground I erased and see the new stroke there as on bare ground, so that an erase never stops me repainting.
15. As an Author, I can erase across the edge of a Terrain and beyond it with nothing happening where nothing was painted, so that an erase never adds anything.
16. As an Author, I am told in the status line that there is nothing to erase when I press while erasing on a Layer with no Terrain, and nothing is laid, so that an erase never leaves an invisible Element.
17. As an Author, I can erase from a Layer's Terrain whatever Asset is chosen in the browser, so that erasing never asks for an image.
18. As an Author, I can rely on an erase being one undo step, undo bringing the ground back exactly and redo taking it away again, so that erasing is as reversible as painting.
19. As an Author, I can press Escape while drawing an erase to throw it away, so that an erase started by accident costs nothing.
20. As an Author, I can rely on a stroke staying a paint or an erase as it was when I pressed, whatever I switch to before I release, so that a stroke is never half of each.

### Picking a stroke

21. As an Author, I can choose Edit strokes in the Paint tool's options, so that my clicks pick strokes instead of painting.
22. As an Author, I can read in the status line that a click picks a stroke, a drag reshapes it, and Delete removes it, so that I know what Edit strokes does.
23. As an Author, I can click on a stroke of the current Layer's Terrain to select it, so that any stroke I laid can be changed.
24. As an Author, I can rely on the latest laid stroke under the pointer being the one selected where strokes overlap, so that I get the one painted on top.
25. As an Author, I can click inside a patch I erased and get the erase, so that an erase is as easy to pick as paint, though it shows no ground.
26. As an Author, I can pick a narrow stroke at a low zoom with a click a few pixels off its path, so that zooming out never makes a stroke unpickable.
27. As an Author, I can click where no stroke lies to let the selected stroke go, so that I can always start afresh.
28. As an Author, I can see the selected stroke's band as wide as its Brush, its path as a thin line, and a handle at each point of its path, so that I know what I picked and what I can drag.
29. As an Author, I can rely on selecting a stroke never being an undo step, so that looking at my work costs me nothing.

### Reshaping a stroke

30. As an Author, I can drag a handle of the selected stroke to move that point of its path, every other point staying where it was, so that a stroke that strayed is pulled back where I meant it.
31. As an Author, I can drag a selected stroke, or press on another stroke and drag, to move the whole stroke, keeping its shape, so that a stroke a cell off is put right in one gesture.
32. As an Author, I can see the ground follow the stroke while I drag it, so that I see the result before I let go.
33. As an Author, I can rely on a drag of a point or of a stroke being a single undo step however long it is, so that undo puts the stroke back where the drag began.
34. As an Author, I can drag a stroke or its points beyond the Bounds, so that the Bounds never get in the way of composing.
35. As an Author, I can rely on moving a stroke keeping its place among the Terrain's strokes, so that an erase laid after it still takes from it, and one laid before it still does not.

### A stroke's Brush settings

36. As an Author, I can see the selected stroke's size, hardness, and strength in the options strip, so that I know what it was laid with.
37. As an Author, I can change the selected stroke's size, hardness, or strength in the options strip and see its ground redrawn, each change one undo step, a whole drag of the value one step, so that a stroke laid with the wrong Brush is fixed without repainting it.
38. As an Author, I am refused a size of zero or less typed for a stroke, with the reason and nothing changed, so that a stroke never vanishes through a typo.
39. As an Author, I can rely on changing a selected stroke's settings leaving the Brush as it was, so that fixing an old stroke never changes the next one.
40. As an Author, I can turn the selected stroke from painting to erasing, or back, in the options strip, as one undo step, so that a stroke laid in the wrong mode costs no redrawing.
41. As an Author, I can rely on the options strip showing the Brush again when no stroke is selected, so that the strip always says what my next stroke will be.

### Removing a stroke

42. As an Author, I can press Delete with a stroke selected to remove it, and see the ground as if it had never been laid, so that one bad stroke goes without undoing everything after it.
43. As an Author, I can rely on removing a stroke leaving every other stroke and their order as they were, so that removing one never reshuffles the rest.
44. As an Author, I can remove an erase and see the ground it took away come back, so that an erase is never final.
45. As an Author, I can rely on removing the last stroke of a Terrain removing the Terrain, as one undo step that brings both back, so that a Layer is never left holding a Terrain with no strokes.
46. As an Author, I can undo a removal and get the stroke back at its place in the order, exactly as it was, so that undo never loses a stroke.

### History, the view, and the selection

47. As an Author, I can undo and redo every stroke edit with the usual shortcuts, in the order I took them among my other steps, so that history stays one line.
48. As an Author, I can rely on the ground looking exactly as it did before an edit once I undo it, so that undo never leaves a trace.
49. As an Author, I can rely on the ground being the same whether I reached its strokes by painting, erasing, editing, removing, undoing, or opening a file, so that what I see is always what the strokes say.
50. As an Author, I can rely on undo and redo waiting while I drag a point or a stroke or hold one of its options while it changes, so that a step is never taken back while it is still being made.
51. As an Author, I can rely on the selected stroke being let go when I undo or redo, leave the Paint tool, switch to painting or erasing, or the stroke goes, so that the strip and the handles never show a stroke that is not the one I picked.
52. As an Author, I can rely on editing one stroke of a large Terrain redrawing only the ground that stroke covered and covers, so that changing one stroke stays quick however much I painted.

### Saving and exporting

53. As an Author, I can save a Project with erases and reopen it with every erase where it was and the ground drawn the same, so that my erases survive closing the editor.
54. As an Author, I can open a Project saved before strokes could erase and find every stroke painting as before, and save it, so that my older maps keep working.
55. As an Author sharing a Project with a collaborator whose editor predates erasing, I can rely on their editor refusing it as newer, naming the version, rather than opening it with my erases painting, so that a round trip through an older editor never turns holes into ground.
56. As an Author, I can export a Level and see every erase and every edited stroke in the image exactly as the editor shows them, so that the ground I see is the ground I export.

## Rules

### Erasing

**Erasing caps what remains**: a stroke that erases lowers a Terrain's coverage at each point to one minus the erase's own coverage there (Shaped by a soft round Brush) where that is lower, and leaves it where it is already as low: a full-strength erase leaves nothing within its hardness, and an erase of strength *s* leaves at most 1 − *s* there.
_Why_: an erase says how much of the Material may remain, not how much it takes away, so it is exact in one pass wherever its segments overlap.

**No build-up along an erase**: an erase lowers the coverage at a point by the same however many of its segments pass near the point, at a joint, a self-crossing, or a loop alike, and passing over ground again with an erase never lowers it below what the strongest erase there leaves.

**An erase takes from what lies before it**: an erase lowers only the coverage the strokes laid before it give; a stroke laid after it paints over the erased ground as over bare ground, and an erase where no earlier stroke covers anything changes nothing.

**Nothing to erase**: a Paint that erases on a Layer with no Terrain is answered with the reason, makes no Terrain, and records no history step.

**An erase needs no image**: a Paint that erases adds its stroke to the topmost Terrain on its Layer whatever image it names, or none.

### Editing strokes

**Strokes are numbered**: a Terrain's first-laid stroke is its first, and so on; moving a stroke or a point of its path, changing its Brush settings, and turning it to painting or erasing change no stroke's number or place in the order, and removing a stroke numbers each later stroke one lower. Follows from: Nothing is fixed at creation.

**A stroke's point moves alone**: an Edit Element moving a point of a stroke's path changes that point and nothing else: every other point, the stroke's Brush settings, whether it erases, and every other stroke stay as they were. Follows from: Nothing is fixed at creation.

**Moving a stroke moves its path**: an Edit Element moving a stroke to a position moves every point of its path by the difference between that position and the centre of the smallest box around its points, keeping its Brush settings, whether it erases, and its place in the order. Follows from: Nothing is fixed at creation.

**A stroke's Brush stays editable**: an Edit Element setting a stroke's Brush settings changes its size, hardness, and strength and nothing else of it or of any other stroke. Follows from: Nothing is fixed at creation.

**Painting or erasing stays editable**: an Edit Element turning a stroke to erasing, or an erase to painting, changes that and nothing else of it or of any other stroke; one naming what the stroke already does changes nothing and records no step. Follows from: Nothing is fixed at creation.

**A stroke edit is one step**: moving a point, moving a stroke, setting its Brush settings, turning it to painting or erasing, and removing it are each one history step outside a gesture, and a gesture of moves or of Brush settings from its beginning to its end is one step however long, which undo returns to where the gesture began. Follows from: Every Command can be undone.

**Removing a stroke keeps the rest**: an Edit Element removing a stroke takes it out of its Terrain's strokes, leaving every other stroke and their order as they were, as one history step that undo returns to the same place in the order, exactly as it was. Follows from: Every Command can be undone.

**The last stroke takes its Terrain**: removing the only stroke of a Terrain removes the Terrain instead, as one history step that undo returns with its ElementId, its stroke, and its place in the stacking order.

**Malformed stroke edits are refused**: an Edit Element naming a stroke its Terrain does not have or a point its stroke's path does not have, moving a point or a stroke where it is not finite, or setting Brush settings that are not a Brush's (a size not above zero or not finite, a hardness outside 0 to 1, or a strength not above 0 or above 1), and a stroke change sent for an Element that is not a Terrain, are answered with the reason, change nothing, and record no history step.

### The Paint tool

**The selected stroke**: with the Paint tool editing strokes, a click on the Level selects the latest laid stroke of the current Layer's topmost Terrain whose path lies no farther from the pointer than its radius or four screen pixels, whichever is more, an erase as much as a paint, and a click where none does lets the selection go; the selected stroke is shown as a translucent band as wide as its Brush along its path, in the erase colour when it erases, its path as a thin line, and a handle at each point of its path; selecting it is never a history step.

**Dragging a stroke**: with the Paint tool editing strokes, a press on a handle of the selected stroke and a drag move that point, a press anywhere else on a stroke selects it and a drag from there moves the whole stroke, each from press to release as one gesture; a press where no stroke lies paints and erases nothing.

**Stroke options follow the selection**: with a stroke selected, the options strip shows its size, its hardness and strength as percentages, and whether it paints or erases, and a change to any of them is sent as an Edit Element of that stroke, one dragged or held while it changes as one gesture and a typed one as one step, a typed size of zero or less as typed, so that it is refused with the reason; the Brush stays as it was; a stroke whose settings lie outside the ranges the Brush offers shows its own and sends nothing until changed; with no stroke selected, they are the Brush's.

**Delete removes the selected stroke**: with the Paint tool editing strokes and a stroke selected, Delete, and Backspace on macOS too, sends an Edit Element removing it.

**The selected stroke is let go**: the selected stroke is let go when the Paint tool is left, when it switches to painting or erasing, on every undo and redo, and when its Terrain no longer has it.

## Changes to existing behaviour

- composing — **Terrain is its strokes**: modified to "a Terrain holds one Material, which shows an image Asset, and an ordered list of strokes, each a path of one or more points in Grid cells with the Brush settings it was laid with (a size in cells above zero, the diameter the Brush covers; a hardness from 0 to 1; and a strength above 0 up to 1) and whether it paints or erases", because an erase is a stroke like any other.
- composing — **Strokes composite by the strongest**: modified to "a Terrain's coverage at a point starts at none and is composited from its strokes in the order they were laid: a stroke that paints raises it to the stroke's own coverage there where that is higher, so a later, weaker stroke never lowers it and passing over ground again never raises it past the strongest stroke, and an erase lowers it (Erasing caps what remains)", because the order now matters wherever an erase lies.
- composing — **Painted with its Material**: modified to "a Paint that paints and names no image paints with its Terrain's Material and is refused on a Layer with no Terrain; a Paint that paints and names an image other than the one its Terrain's Material shows is refused, with the reason naming both; the image a Paint that erases names, if any, is not looked at (An erase needs no image)", because an erase takes away whatever the Terrain shows.
- composing — **Undo leaves no trace**: modified to "after undoing a stroke, or an edit or the removal of one, every point of its Terrain has the coverage it had before the step was taken", because strokes are now edited after they are laid.
- composing — **Coverage is the strokes alone**: modified to "a Terrain's coverage is the same however its strokes came to be, laid one by one, edited, removed, undone and redone, or opened from a file", because edits and removals are new ways for strokes to come to be.
- composing — **Terrain changes only its Material**: replaced by **Terrain changes only its Material and its strokes**: "an Edit Element of a Terrain that changes anything but its Material or one of its strokes, its position and every change only a Wall, a Room, or a Portal has included, and an Edit Element setting the Material or changing a stroke of an Element that is not a Terrain, are answered with the reason, change nothing, and record no history step", because a stroke's path and Brush settings are now edited through Edit Element.
- composing — **Undo waits for the step being made**: modified to "undo and redo, from the keys or the menu, wait while a drag, a Wall, a Room's outline, or a stroke being drawn, or an option of the tool strip held while it changes is under way, the Brush's options aside when they change the Brush, which is never a step", because a held option of a selected stroke is part of a step.
- composing — **Painting with the Paint tool**: modified to "with the Paint tool chosen, from the tool strip or with `B` or `E`, it paints, erases, or edits strokes, as Paint, Erase, and Edit strokes in its options choose: `B` chooses it painting and `E` erasing, whatever tool or mode was chosen before, and choosing it from the tool strip chooses it painting; while painting or erasing, a press on the Level starts a stroke at the pointer that paints or erases as the tool did at the press, moving adds the pointer's path to it, and the release sends one Paint onto the current Layer with the path, the Brush's settings, and whether it erases, a stroke that paints naming the chosen Asset or, with none chosen, no image, and an erase naming none; with no Asset chosen and no Terrain on the current Layer a press while painting paints nothing and the status line asks for an Asset, and with no Terrain on the current Layer a press while erasing erases nothing and the status line says there is nothing to erase; the tool stays chosen in the mode it is in; undo and redo wait while a stroke is being drawn", because the tool now erases and edits strokes.
- composing — **Seeing the stroke**: modified to "with the Paint tool painting or erasing, a circle as large as the Brush's size follows the pointer over the Level, and a stroke being drawn is shown as a translucent band as wide as the Brush along its path, until the release, both in a colour of their own while erasing; while it edits strokes, no circle is shown", because an erase must not look like paint before it is laid, and Edit strokes lays nothing.
- composing — **Brush options**: modified to "with the Paint tool chosen, the tool's options show Paint, Erase, and Edit strokes, the one it is in chosen, then the Brush's size in cells, its hardness and its strength as percentages, or the selected stroke's (Stroke options follow the selection), and the name of the image it paints with, the chosen Asset's or the current Layer's Terrain's, or, with neither, ask for an Asset to paint with; the status line says that a drag paints, naming the chosen Asset, when there is one, as the one painted with, that a drag erases, or that a click picks a stroke, a drag reshapes it, and Delete removes it, each with Escape stopping; the Brush starts at a size of two cells, a hardness of 50 %, and a strength of 100 %; the size offered goes from a tenth of a cell to sixty-four cells, the strength from 1 % to 100 %; the Brush's settings, and whether the tool paints, erases, or edits strokes, are the Editor's, never a history step and never saved; and, when an Asset is chosen and the current Layer's Terrain shows another image, a button there sends the Edit Element that makes the Terrain show the chosen Asset", because the options choose the mode and show the selected stroke.
- composing — the Notes' acceptance that a Terrain's strokes cannot be changed once laid, against the reading of Nothing is fixed at creation that the shape of anything painted stays editable, is removed, because every stroke's path and Brush settings are now editable; and the note that "stroke" is used in its plain sense goes, as Stroke is now a domain term ("coverage" stays defined by the Terrain Rules themselves).
- projects — **Saved as its strokes**: modified to "a saved Terrain holds its image's Asset Reference and every stroke in order with its points, size, hardness, strength, and whether it erases, and reopens the same, with the same coverage; an editor that does not know the Terrain kind keeps it as a placeholder of its box and writes it back unchanged; a Terrain saved before strokes could erase opens with every stroke painting", because whether a stroke erases is part of the stroke.

## Implementation Decisions

The technology the architecture fixes (an erase as a stroke, strokes compositing in order with coverage the maximum over segments, an erase lowering the mask to the minimum of its value and one minus the erase's coverage; the CPU rasterizer as the golden reference; the tiled pixel cache opaque to all but PaintEngine, held by AuthoringManager and published as the derived coverage; the per-component versions and migrations of the Project format; the generic field-setting and reflection-snapshot history commands) is used as written there, and the composing spec's decisions for Terrain (the Terrain component, the derived coverage and its once-per-frame deriving, Rasterize's pixel rules, the Paint step that keeps only its own stroke) are extended, not restated. GPU rasterization, the mask tile bands, and blend weights are not part of this change.

- **The stroke**: `model`'s stroke gains whether it erases, beside its points and its Brush settings, so a Paint carries it in its stroke and the Paint message's shape is otherwise unchanged. The Terrain component's own check refuses the same strokes as before; whether a stroke erases is never malformed.
- **The Terrain component at version two**: the Terrain is a serialisable component at version two under the stable name `terrain`, each stroke holding whether it erases under `erase`, written for every stroke. Reading version one, the shape a Terrain had before strokes could erase, gives every stroke `erase` false, and the next save writes version two; a version above two is refused as newer, as every component's is. It still checks itself after reading, so a file is refused for the same malformed strokes as a Paint. The serialisation registry's other components stay at version one.
- **Rasterize**: composites the strokes in order onto a region that starts empty: a stroke that paints raises a pixel to its own value where that is higher, its value its coverage times 255 rounded half up as before, and an erase lowers it to one minus the erase's coverage, times 255 rounded half up, where that is lower, so an erase is exact in one pass, its maximum over segments taken before it is applied, and a half-strength erase over half-strength paint leaves the very byte it found. Only the pixels inside each stroke's box grown by its radius are visited, an erase's included, so an erase never touches what lies beyond its reach. It stays one operation serving the cache's tiles and the Export's alike, so the Export shows every erase and every edit with no change to ProjectManager, RenderEngine, or OutputAccess.
- **ApplyStroke**: brings the cache up to the Terrain's strokes by comparing them with the strokes it holds from both ends: the strokes the two lists share from the first onwards and from the last backwards are kept; when only new strokes follow the shared start, they are composited onto the tiles they touch, an erase as much as a paint; otherwise only the tiles touched by the strokes between the shared ends, as they were and as they are, are rasterized again from every stroke whose box reaches each tile. So moving a point or a whole stroke, changing its Brush settings or whether it erases, removing it, and undoing any of these recompute only the tiles the stroke touched before and touches after, however many strokes the Terrain holds. A tile left with no coverage is dropped; a tile whose pixels come out as they were keeps its revision, so RenderEngine uploads nothing for it.
- **Paint that erases**: AuthoringManager checks the stroke as before, then looks for the topmost Terrain on the Layer. With none, a Paint that erases is a CommandFailed saying there is nothing to erase, whatever image it names, and nothing is resolved or recorded. With one, it is appended by the same step a painting stroke is, whatever image it names, with no lookup of the image.
- **Edit Element of a stroke**: the Element change gains five changes, each naming the stroke by its number: a point of its path moved (the point's number and its new position), the stroke moved (its new position, the centre of the smallest box around its points), its Brush settings set (all three together, as one value), its erasing set (whether it erases), and the stroke removed. AuthoringManager refuses each for an Element that is not a Terrain, builds the Terrain each would leave and refuses it for the reason the Terrain component gives, and refuses a stroke or point number the Terrain lacks and a position that is not finite, before anything is recorded. Moving a point, setting the Brush settings, and setting the erasing set that field of that stroke through the generic field-setting command; moving the stroke sets the stroke's whole path through it, every point translated by the difference between the new position and the current centre; so these join a gesture's history Group as a Wall's point drag does, and a single one is a step of its own. Setting the erasing to what the stroke already does records nothing. Removing a stroke, the only change that renumbers strokes, is a reversible step of its own that keeps the stroke and its number and puts it back there on revert, closing any gesture group left open first; removing the only stroke records the Terrain's Remove Element instead, through the generic reflection snapshot, so undo restores the Terrain with its ElementId, its stroke, and its place among the Layer's children, and its coverage is derived afresh. A Terrain's position change stays a CommandFailed.
- **Deriving**: unchanged in when it runs: every Terrain whose component changed, by a Paint, a stroke edit, a gesture's step, an undo, a redo, or an opened file, has its coverage brought up to its strokes through ApplyStroke and its box set around them before anything draws it, so a dragged stroke's ground follows the drag frame by frame. The box still holds every stroke, erases included.
- **The Paint tool's modes**: the Paint tool's state, in its own Editor module, gains its mode (painting, erasing, or editing strokes), the selected stroke (the Terrain's ElementId and the stroke's number), the drag under way of a handle or of the stroke, and the option gesture of the tool-strip guideline for the selected stroke's settings, whose being in progress the Editor's step-under-way check includes. A stroke being drawn records whether it erases at its press. `E` joins the keyboard bindings, stated once so that the menu and the tooltips show exactly the keys that work; it does nothing while a text field has the keyboard, as every key. The Brush's circle and band are drawn in a warm red while erasing, beside the blue of painting.
- **Picking and dragging strokes** are the Editor's hit-testing, as a Wall's are: it maps the pointer to cells through the Viewport and tests the selected stroke's handles first, the nearest within a few pixels, then the current Layer's topmost Terrain's strokes from the last laid back, a stroke being under the pointer when the distance to the nearest segment of its path, or to its single point, is no more than its radius or four screen pixels converted to cells, whichever is more. A drag of a handle sends point moves at the pointer, and a drag of a stroke sends its new position, the centre it had at the press plus the pointer's travel, from press to release as one gesture, the end sent at the pointer's last position, as a Wall's drag does; a press that moves less than a few pixels is a click. The selected stroke is drawn by the Editor over the viewport after the last Manager set, as the stroke being drawn is, and checked against the Terrain each frame, so it is let go as soon as the Terrain no longer has it.
- **Stroke options**: drawn through the tool-strip option guideline, with the selected stroke's Brush settings sent as one change, the floor for dragging the size applied after a drag as for the Brush, and the gesture ended when the stroke stops being shown; Paint and Erase for the selected stroke are a pair of buttons, each a single step. Delete, with Backspace on macOS, sends the removal marked single.
- **The development-only input script**: its `describe` step logs the Paint tool's mode, the selected stroke and the handle being dragged, and for every Terrain each stroke's number, whether it erases, its Brush settings, and its number of points, so a script can aim at a stroke's points and check an edit.

## Testing

- **Composing seam**: the existing headless App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, and AuthoringManager over the fixture folder of two texture images and a Prop image, with no window and no RenderEngine, driven by Apply, Undo, and Redo messages and asserted on the Terrain and Element components, the derived coverage read from its tiles at chosen cells and whole tiles compared, tile revisions, the Layer's children, the answers, and the history. A new file holds the erase and stroke-editing tests; the Terrain tests gain the cases the pinned spec left untested.
  - **Erasing caps what remains**: `crates/drs-app/tests/strokes.rs::erasing_caps_what_remains` (a full-strength erase over full paint leaving nothing within its hardness; a half-strength erase over full paint leaving half; a half-strength erase over half-strength paint leaving it as it was; in the soft edges, each pixel the smaller of the paint's byte and the byte of one minus the erase's coverage)
  - **No build-up along an erase**: `crates/drs-app/tests/strokes.rs::no_build_up_along_an_erase` (at a sharp joint and a self-crossing of a half-strength erase, the value a single segment leaves; two overlapping half-strength erases leaving half)
  - **An erase takes from what lies before it**: `crates/drs-app/tests/strokes.rs::an_erase_takes_from_what_lies_before_it` (a stroke laid after an erase at full coverage on its path; an erase over unpainted ground and beyond the Terrain adding no tile)
  - **Nothing to erase**: `crates/drs-app/tests/strokes.rs::nothing_to_erase` (with and without an Asset named: no Terrain, no Asset Reference, no step)
  - **An erase needs no image**: `crates/drs-app/tests/strokes.rs::an_erase_needs_no_image` (an erase naming the other texture, and one naming none, both appended)
  - **Strokes are numbered**: `crates/drs-app/tests/strokes.rs::strokes_are_numbered`
  - **A stroke's point moves alone**: `crates/drs-app/tests/strokes.rs::a_strokes_point_moves_alone`
  - **Moving a stroke moves its path**: `crates/drs-app/tests/strokes.rs::moving_a_stroke_moves_its_path` (and the coverage at its old and new place; an erase moved keeps taking from a stroke laid before it)
  - **A stroke's Brush stays editable**: `crates/drs-app/tests/strokes.rs::a_strokes_brush_stays_editable`
  - **Painting or erasing stays editable**: `crates/drs-app/tests/strokes.rs::painting_or_erasing_stays_editable` (and naming what it already does records no step)
  - **A stroke edit is one step**: `crates/drs-app/tests/strokes.rs::a_stroke_edit_is_one_step` (a gesture of point moves, one of stroke moves, and one of Brush settings each undoing to where it began; single changes each a step)
  - **Removing a stroke keeps the rest**: `crates/drs-app/tests/strokes.rs::removing_a_stroke_keeps_the_rest` (a middle stroke and an erase removed, undone back to their places)
  - **The last stroke takes its Terrain**: `crates/drs-app/tests/strokes.rs::the_last_stroke_takes_its_terrain` (undo restoring the Terrain's ElementId, stroke, place among the Layer's children, and coverage)
  - **Malformed stroke edits are refused**: `crates/drs-app/tests/strokes.rs::malformed_stroke_edits_are_refused`
  - The new and modified composing Rules this change extends: **Terrain is its strokes** `crates/drs-app/tests/strokes.rs::an_erase_is_a_stroke` (a Paint that erases appended with whether it erases, the stroke and its place kept through undo and redo), **Strokes composite by the strongest** by the pinned `crates/drs-app/tests/terrain.rs::strokes_composite_by_the_strongest` and `crates/drs-app/tests/strokes.rs::an_erase_takes_from_what_lies_before_it`, **Painted with its Material** by `crates/drs-app/tests/strokes.rs::an_erase_needs_no_image`, **A stroke is one step** by `crates/drs-app/tests/strokes.rs::an_erase_is_a_stroke`, **Undo leaves no trace** `crates/drs-app/tests/strokes.rs::an_edit_leaves_no_trace` (every tile byte for byte as before after undoing a point move, a stroke move, a Brush change, an erasing change, and a removal of a stroke overlapping others), **Coverage is the strokes alone** `crates/drs-app/tests/strokes.rs::edited_coverage_is_the_strokes_alone` (strokes painted, erased, edited, and removed giving the same tiles as the strokes they end as painted afresh, and the same after saving and opening), **Redo repeats exactly** `crates/drs-app/tests/strokes.rs::stroke_edits_redo_exactly`, **One history** `crates/drs-app/tests/strokes.rs::stroke_edits_share_the_history`, **Terrain changes only its Material and its strokes** `crates/drs-app/tests/terrain.rs::terrain_changes_only_its_material_and_strokes` (the position and every change only a Wall or a Portal has, the Portal's width, rotation, mirroring, side, and place along its Wall included, refused for a Terrain; the Material and every stroke change refused for a Prop), **A failed Command is reported** by `crates/drs-app/tests/strokes.rs::nothing_to_erase` and `crates/drs-app/tests/strokes.rs::malformed_stroke_edits_are_refused`.
  - The pinned Rules whose Terrain cases had no test: **Removal is reversible in place** `crates/drs-app/tests/terrain.rs::terrain_removal_is_reversible_in_place` (a Remove Element of a Terrain undone: its ElementId, its strokes, its place among the Layer's children, and its coverage derived afresh equal to before), **Shaped by a soft round Brush** `crates/drs-app/tests/terrain.rs::shaped_by_a_soft_round_brush` (extended: a hard Brush's pixel exactly at its radius showing its full strength, and the next one beyond showing nothing).
- **Offscreen export seam**: the existing headless App with RenderEngine under Bevy's default plugins without a window, exporting to a temporary PNG and asserting pixels.
  - **Erasing caps what remains**: `crates/drs-app/tests/export.rs::an_erase_shows_what_lies_below` (a full-strength erase across a painted stroke: the background on the erase's path, the texture on the stroke beyond the erase's radius)
  - **No build-up along an erase**: `crates/drs-app/tests/export.rs::an_erase_leaves_no_build_up` (a half-strength erase with a sharp joint over full paint: the same colour at the joint as on its straight stretch, and as a half-strength stroke painted alone over the background)
  - **Moving a stroke moves its path** and **Painting or erasing stays editable**, as exported: `crates/drs-app/tests/export.rs::an_edited_stroke_exports_as_edited` (a moved stroke's texture at its new place and the background at its old one; an erase turned to painting showing its texture)
- **Projects seam**: the existing headless App saving and reopening Projects in temporary directories.
  - **Saved as its strokes**: `crates/drs-app/tests/projects.rs::terrain_is_saved_as_its_strokes` (extended with an erase: whether each stroke erases, written at version two, and the coverage after reopening), `crates/drs-app/tests/projects.rs::an_older_terrain_opens` (a file holding a Terrain at version one opening with every stroke painting and the coverage of those strokes, then saved with the Terrain at version two and every stroke not erasing), `crates/drs-app/tests/projects.rs::unknown_terrain_round_trips` (unchanged, with a stroke that erases among those passed through)
  - **A newer file is refused**, the projects Rule a collaborator's older editor follows: `crates/drs-app/tests/projects.rs::a_newer_file_is_refused` (extended with a Terrain at version three, refused naming it and its version)
- **By hand**: **The selected stroke**, **Dragging a stroke**, **Stroke options follow the selection**, **Delete removes the selected stroke**, and **The selected stroke is let go**, with the modified **Painting with the Paint tool**, **Seeing the stroke**, **Brush options**, and **Undo waits for the step being made**: no automated seam for the egui interface, the accepted deviation of the composing spec; verified by driving the editor with the development-only input script, whose `describe` step logs the Paint tool's mode, the selected stroke, and each Terrain's strokes, and which also confirms that an erase and an edited stroke are drawn in the viewport as the coverage says, in the frame after the release or during a drag.
- PaintEngine's own unit tests check what the Rules above rest on, as functions of the Engine alone, and are the coverage of no Rule: an erase taking the minimum against its formula, appending an erase equal to rasterizing every stroke, an edit of one stroke rasterizing only the tiles its old and new reach touch, a removal and its undo rasterizing only the removed stroke's tiles, and every edit's result equal to a full Rasterize (`crates/drs-paint-engine/src/rasterize.rs::tests::an_erase_takes_the_minimum`, `crates/drs-paint-engine/src/cache.rs::tests::appending_an_erase_equals_rasterizing`, `crates/drs-paint-engine/src/cache.rs::tests::an_edit_touches_only_its_tiles`, `crates/drs-paint-engine/src/cache.rs::tests::a_removal_touches_only_its_tiles`, `crates/drs-paint-engine/src/cache.rs::tests::an_edit_equals_rasterizing`).

## Out of Scope

- Rasterizing strokes on the GPU, the mask tile bands above the resident base, and a dragged stroke's ground following it at the speed of the GPU path: the next change. Here every edit rasterizes on the CPU at 32 pixels per cell.
- More than one Material on a Terrain, blend weights, and erasing one Material and not another: the change after.
- Changing the order of a Terrain's strokes, such as bringing an erase forward over a stroke laid after it; an erase acts on the strokes before it, and a new erase is laid to take from later paint.
- Adding or removing points of a stroke's path, smoothing or simplifying a path, and splitting or joining strokes.
- Picking a stroke hidden under later ones at the same place, picking strokes of a Terrain other than the current Layer's topmost, and selecting several strokes at once.
- A key of its own for Edit strokes; the mode is chosen in the options strip.
- Moving, restacking, or removing a whole Terrain from the editor; Terrain is still not picked by the Select tool.
- Snapping a stroke's points to the Grid.
- A Brush for erasing with settings of its own, Brush Presets, textured or shaped Brush tips, and pressure.
- An eraser that removes strokes whole where it passes.

## Further Notes

- **Architecture check**: no new component, contract operation, dependency direction, or restricted crate is needed. ApplyStroke and Rasterize keep their shape and gain the erase and the two-ended comparison; the stroke's new field and the Terrain's second version are `model`'s; hit-testing a stroke is the Editor's, as hit-testing a Wall is; and every change to a stroke is an Edit Element or a Paint that AuthoringManager already owns.
- An erase that outlives the paint it took from, once that paint is moved or removed, takes from nothing and shows nothing; it stays in the Terrain's strokes and its box until it is removed, so moving the paint back brings the hole back with it.
- A Terrain whose only strokes left are erases shows nothing and stays a Terrain until its last stroke is removed; the Layer's next stroke that paints adds to it and is erased by none of them, since they were laid before it.
- Editing a stroke on dense Terrain recomputes every stroke reaching the tiles that stroke touches, on the CPU, every frame of a drag. Measured on one Apple M4 Max with 200 strokes of 40 points laid densely, one in five an erase, each frame of a drag moving one of them takes about 20 ms in a release build and 24 ms in a development build, and so does its undo; the release of the drag, which sends the last change again, takes under 0.1 ms, since the strokes are as they were and nothing is rasterized again. That is the order of cost until strokes rasterize on the GPU.
- A stroke laid while the tool erased and later turned to painting paints with the Terrain's image, like every stroke of that Terrain, since a Terrain has one Material.
