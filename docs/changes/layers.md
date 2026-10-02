# Layers

**Capabilities**:
- levels-and-layers: Add Layer, Remove Layer, Reorder Layers
- composing: Restack
- export: Export Level (Layers drawn in order; no Rule changes)
- projects: none of its Commands; saving and opening several Layers

## Problem Statement

A Level has exactly one Layer, and everything the Author places lands on top of everything placed before it. Floors cannot be kept apart from furniture, a building cannot be kept apart from the ground around it, and an Element placed too late cannot be put underneath the one it should sit below: the only way to fix the order is to remove Elements and place them again in the right sequence. The functional baseline, like every image editor, lets the Author slice a map into Layers, order them, and move things up and down within and between them.

## Solution

A Layers panel, docked beside the viewport, lists the Level's Layers topmost first and highlights the current one. Clicking a Layer makes it current, and every tool places on the current Layer: Props, Walls, Rooms, Portals, and the strokes of the Paint tool, which makes one Terrain per Layer. A plus button adds a Layer named `Layer 2`, `Layer 3`, and so on directly above the current one and makes it current; a minus button removes the current Layer with everything on it; dragging a Layer in the list reorders the Layers, and everything on a Layer is drawn over the Layers below it and under those above it, in the viewport and in the Export alike. A selected Element can be brought forward, sent backward, brought to the front, or sent to the back of its Layer, from the options strip or with the usual shortcuts, and moved to another Layer from a Layer menu in the options strip. Every one of these is one undo step; undoing the removal of a Layer brings back the Layer and every Element that was on it exactly, and choosing the current Layer is never a step at all. Saving keeps every Layer with its name, in order. Portals stay set into their Walls and Rooms whatever Layers they and their hosts are moved to, and Rooms combine with the Rooms of their own Layer as before, so a Room moved to another Layer leaves its old combination and joins the new one.

## User Stories

### The Layers panel

1. As an Author, I can see a Layers panel docked beside the viewport listing the Level's Layers by name, topmost first, so that the list reads like the stack I see on the Level.
2. As an Author, I can see the current Layer highlighted in the list, so that I always know where my next Element goes.
3. As an Author, I can click a Layer in the list to make it the current Layer, so that switching where I work is one click.
4. As an Author, I can rely on choosing the current Layer never being an undo step and never counting as an unsaved change, so that looking at another Layer costs me nothing.
5. As an Author, I can start a new Project with its one Layer current, so that I can place something at once.
6. As an Author, I can open a Project and find its topmost Layer current, so that what I place first is not hidden under the rest.
7. As an Author, I can rely on the panel waiting while I am drawing a Wall or a Room, dragging, painting a stroke, or holding a tool-strip option, so that a Wall I started on one Layer never ends up on another.
8. As an Author, I can rely on the panel's buttons and dragging waiting while an Export runs, so that the image I export is the Level as it was when I started.

### Placing on the current Layer

9. As an Author, I can place a Prop, draw a Wall or a Room, and place a Portal on the current Layer, so that I choose where an Element goes by choosing the Layer first.
10. As an Author, I can paint onto the current Layer and have its own Terrain made by the first stroke on it, so that each Layer can carry its own ground.
11. As an Author, I can see the Brush options name the image of the current Layer's Terrain, so that I know what painting there will show.
12. As an Author, I can select, move, and edit an Element on any Layer, the topmost under the pointer winning whatever its Layer, so that I never need to switch Layers to fix something I see.
13. As an Author, I can rely on selecting an Element leaving the current Layer as it was, so that fixing a Prop on another Layer never moves where I place next.
14. As an Author, I am told in the status line that the Level has no Layer when I try to place or paint on a Level without one, and nothing is placed, so that a click never silently does nothing.

### Adding a Layer

15. As an Author, I can press the plus button to add a new, empty Layer directly above the current Layer, so that a new Layer appears where I am working.
16. As an Author, I can see the new Layer become the current Layer, so that what I place next goes onto it.
17. As an Author, I can rely on a new Layer being named `Layer` and one more than the highest number among the Level's Layers named that way, so that names never repeat as I add Layers.
18. As an Author, I can undo adding a Layer to take it away and redo it to bring it back at the same place with the same name, so that adding a Layer is as reversible as placing a Prop.

### Removing a Layer

19. As an Author, I can press the minus button to remove the current Layer with every Element on it as one undo step, so that clearing out a whole slice of the map is one click.
20. As an Author, I can undo removing a Layer and get it back at its place in the list with its name and every Element on it exactly as it was, Props, Walls, Rooms, Portals, and Terrain with every stroke, in the same order, so that removing a Layer is never a loss.
21. As an Author, I can rely on a Portal on another Layer that is set into a Wall or a Room on the removed Layer being removed with it, and on being told how many went, so that no door is left standing in no wall.
22. As an Author, I can undo that removal and see those Portals set back where they were, so that undo restores the doors with their walls.
23. As an Author, I can remove a Layer holding Portals set into Walls on other Layers and see those Walls close where the Portals stood, so that a removed door leaves a whole wall.
24. As an Author, I am refused removing the only Layer of the Level, and the minus button is disabled while there is one, so that a Level always has somewhere to place.
25. As an Author, I can see the Layer directly below the removed one become the current Layer, or the one directly above when it was the lowest, so that I land on a neighbour of what I removed.
26. As an Author, I can rely on my selection being dropped when the Element it held went with its Layer, so that Delete and the options strip never act on something that is gone.
27. As an Author, I can undo, after undoing a Layer's removal, the steps I took on that Layer before removing it, and redo them, so that history stays one line through a removal and its undo.
28. As an Author, I can rely on undoing a Layer's removal bringing back Elements of a kind my editor does not know, and data it does not know on the Layer and its Elements, so that removing and restoring a Layer never strips a collaborator's content.

### Reordering Layers

29. As an Author, I can drag a Layer in the list and drop it between two others or at either end to move it there, so that ordering Layers is direct.
30. As an Author, I can see everything on a Layer drawn over every Layer below it and under every Layer above it, in the viewport and in the Export, so that the list's order is the order I see.
31. As an Author, I can undo a reorder and see the Layer back at its place, so that trying out an order costs nothing.
32. As an Author, I can drop a Layer where it already was and have nothing recorded, so that a fumbled drag leaves no step to undo.
33. As an Author, I can reorder Layers and see the Rooms on each stay combined exactly as before, so that moving a building's Layer up never changes its Walls.
34. As an Author, I can rely on the current Layer staying current through a reorder, so that moving a Layer never changes where I place next.

### Restacking within a Layer

35. As an Author, I can bring a selected Element forward one place within its Layer from the options strip or with Command or Control and `]`, so that it moves over the Element just above it.
36. As an Author, I can send a selected Element backward one place from the options strip or with Command or Control and `[`, so that it moves under the Element just below it.
37. As an Author, I can bring a selected Element to the front of its Layer with Shift added to the forward shortcut, or send it to the back with Shift added to the backward one, or from the options strip, so that the extremes are one gesture.
38. As an Author, I can rely on each restack being one undo step that undo returns exactly, so that restacking is as reversible as moving.
39. As an Author, I can see the forward and front controls disabled for the topmost Element of its Layer, and the backward and back controls for the lowest, and a shortcut there doing nothing and recording nothing, so that a restack that cannot move records no step.
40. As an Author, I can restack a Portal over or under its Wall and see it still standing in its gap, so that a door's place in the stacking order never pulls it out of its wall.
41. As an Author, I can restack a Room among the Rooms of its Layer and see the combination follow the new order, a cut moved below the Room it cut no longer taking that Room's floor, so that the stacking order of Rooms means the same through every step.
42. As an Author, I can rely on a Portal whose Wall a Room's restack takes away being removed in the same step, and on being told how many went, so that a door never floats over open floor.
43. As an Author, I can rely on the restack shortcuts waiting, as undo does, while a step is being made and while an Export runs, so that a step is never reordered while it is still being made.

### Moving to another Layer

44. As an Author, I can see the selected Element's Layer in a Layer menu in the options strip, so that I can tell where an Element lives without looking it up.
45. As an Author, I can choose another Layer in that menu to move the selected Element onto the top of that Layer, as one undo step, so that sorting Elements into Layers needs no removing and placing again.
46. As an Author, I can undo that move and see the Element back on its Layer at its place, so that moving between Layers never reshuffles a Layer.
47. As an Author, I can move a Wall or a Room to another Layer and see the Portals set into it stay set exactly where they were, and move a Portal to another Layer and see it stay set into its Wall, so that Layers never break a door from its wall.
48. As an Author, I can move a Room to another Layer and see it leave the combination of its old Layer and combine with the Rooms of its new one, and any Portal whose Wall that takes away removed in the same step with a word in the status line, so that a building on its own Layer keeps its own Walls.
49. As an Author, I can rely on the moved Element keeping its ElementId and every property, so that a moved Element is the same Element.

### Seeing, saving, and exporting

50. As an Author, I can save a Project with several Layers and reopen it with every Layer, its name, and its place in the order, and every Element on its Layer in its order, so that my Layers survive closing the editor.
51. As an Author, I can export a Level of several Layers and see them drawn in their order, an Element on a higher Layer over one on a lower Layer whichever was placed first, so that the Export is the Level I see.
52. As an Author, I can rely on every Layer Command and every restack joining the one history with every other step, in the order I took them, so that undo walks back through my work as I did it.

## Rules

### Layers (levels-and-layers capability)

**Layers in order**: a Level's Layers are in an order, and the stacking order of the Level's Elements is every Element of its lowest Layer in that Layer's order, then every Element of the Layer above it, and so on up to the topmost Layer. Follows from: Stacking order.

**Added where asked**: an Add Layer puts on the given Level a new Layer with no Elements at the given place among its Layers, counted from the lowest at 0 and at most one past the topmost, the Layers at and above that place each moving one place up, as one history step that undo takes away and redo puts back at the same place with the same name. Follows from: Every Command can be undone.

**Named in turn**: a Layer added by Add Layer is named `Layer` followed by a space and the number one more than the largest whole number *n* for which a Layer of its Level is named `Layer n`, or `Layer 1` when none is.

**Removed with its Elements**: a Remove Layer takes the Layer off its Level with every Element on it, as one history step; undo puts the Layer back at its place among the Level's Layers with its name and every Element that was on it, in the same order, each with its ElementId and every property, Elements of a kind this editor does not know and data this editor does not know on the Layer and on its Elements included; redo removes them again. Follows from: Every Command can be undone; References are never dropped.

**Gone with the Layer's Walls**: a Remove Layer also removes, in the same history step, every Portal on another Layer of the Level that is set into a Wall or a Room on the removed Layer, and the Author is told in the status line how many; undo sets each back where it was; a Portal whose anchor names a part its host does not have stays standing where it is. Follows from: A Portal set into a Wall moves with it.

**The last Layer stays**: a Remove Layer of the only Layer of its Level is refused with the reason, changes nothing, and records no history step.
_Why_: a Level without a Layer has nowhere to place anything, and every tool would have to say so.

**Reordered to a place**: a Reorder Layers moves one Layer to the given place among its Level's Layers, counted from the lowest at 0, the Layers it passes each moving one place to fill the gap, every Layer keeping its Elements in their order, as one history step that undo returns; a Reorder Layers to the place the Layer already holds changes nothing and records no history step. Follows from: Every Command can be undone.

**Reordering keeps Rooms**: a Reorder Layers, its undo, and its redo change no Room's floor or Walls and remove no Portal.

**Layer Commands are refused when malformed**: an Add Layer naming an entity that is not a Level or a place more than one past its topmost Layer, a Remove Layer or a Reorder Layers naming no Layer of the Project, and a Reorder Layers to a place past the topmost Layer are answered with the reason, change nothing, and record no history step.

**History follows a restored Layer**: a step recorded on a Layer, or on an Element of it, before the Layer was removed undoes and redoes on that Layer once its removal is undone, exactly as it would had the Layer never been removed. Follows from: Every Command can be undone.

### Restack (composing capability)

**Restacked to a place**: a Restack moves an Element to the given place among the Elements of the given Layer of its own Level, counted from the bottom at 0 as the Layer stands once the Element has left its own place, the Elements it passes each shifting one place, as one history step that undo returns to the Layer and the place it had; the Element keeps its ElementId and every property; a Restack to the place it already holds changes nothing and records no history step. Follows from: Stacking order; Every Command can be undone.

**Restack is refused when malformed**: a Restack naming no Element, an entity that is not a Layer, a Layer of another Level than the Element's, or a place past the last Element of its own Layer or more than one past the last Element of another Layer is answered with the reason, changes nothing, and records no history step. Follows from: Levels are independent.

**Anchors cross Layers**: restacking a Portal set into a Wall or a Room, or the Wall or Room it is set into, within a Layer or onto another Layer, changes none of the Portal's anchor, centre, rotation, or mirroring, and its host gives way along it as before. Follows from: A Portal set into a Wall moves with it.

**Restacked Rooms recombine**: after a Restack of a Room, its undo, and its redo, every Room of the Layer it left and of the Layer it joined has the floor and Walls the Rooms of that Layer now give, and every Portal set into a Room of either Layer that had a Wall at its centre before the Restack and has none after it is removed in the same history step, which undo restores whole; the Author is told in the status line how many Portals were removed. Follows from: A Portal set into a Wall moves with it.

### The current Layer and the panel (Editor)

**The current Layer**: the Editor holds one current Layer of the Level: the only Layer of a new Project, the topmost Layer of an opened Project's Level, the Layer an Add Layer the Author sent adds once it is added, and the Layer the Author clicks in the Layers panel; choosing it is never a history step, is never saved, and never gives the Project unsaved changes; selecting an Element never changes it.

**Placed on the current Layer**: every Place Element the Prop, Wall, Room, and Portal tools send, and every Paint the Paint tool sends, names the current Layer as it is when the Command is sent.

**The current Layer moves on**: when the current Layer stops existing, by a Remove Layer or by an undo or a redo, the current Layer becomes the Layer that was directly below it, or the one directly above it when it was the lowest.

**No Layer, nothing placed**: on a Level with no Layer, as an opened file may hold, there is no current Layer, a click or press with a tool places and paints nothing, and the status line says the Level has no Layer to place on; Add Layer adds one at the bottom and makes it current.

**The Layers panel**: a dock tab named Layers lists the Level's Layers topmost first, each by its name, with the current Layer highlighted; a click on a row makes its Layer current; the plus button sends one Add Layer at the place directly above the current Layer; the minus button sends one Remove Layer of the current Layer and is disabled while the Level has one Layer; dragging a row and dropping it between two rows or at either end of the list sends one Reorder Layers to the place the drop gives, and a drop where the row was sends nothing.

**The panel waits**: the Layers panel's rows, buttons, and dragging do nothing while a drag, a Wall or a Room being drawn, a stroke being painted, or an option of the tool strip held while it changes is under way, and its buttons and dragging do nothing while an Export runs.

**Restack controls**: with an Element selected, the options strip shows Forward, Backward, To front, and To back, each disabled where it would change nothing, and a Layer menu listing the Level's Layers topmost first with the Element's Layer chosen; Forward and Command or Control with `]` send a Restack one place up, Backward and Command or Control with `[` one place down, To front and Command or Control with Shift and `]` to the top of its Layer, and To back and Command or Control with Shift and `[` to the bottom; choosing another Layer in the menu sends a Restack to the top of that Layer; the shortcuts wait as undo does, and the controls and shortcuts do nothing while an Export runs.

## Changes to existing behaviour

The composing Rules named here are those composing holds once Rooms, Rooms combine and cut, and Paint Terrain with one Material have landed, which come before this change on the roadmap; the clauses below are added to what those Rules then say.

- composing — **One history**: modified to add Add Layer, Remove Layer, Reorder Layers, and Restack to the Commands that are each one undo step in the one history, because Layers and restacking join the history.
- composing — **Redo repeats exactly**: modified to add Add Layer, Remove Layer, Reorder Layers, and Restack, for the same reason.
- composing — **A failed Command is reported**: modified to add Add Layer, Remove Layer, Reorder Layers, and Restack, for the same reason.
- composing — **Gone with its part of the Wall**: modified to add "and every Portal set into a Wall on a Layer being removed by Remove Layer, whatever Layer the Portal is on (Gone with the Layer's Walls)", because removing a Layer removes its Walls.
- composing — **Gone with its part of the Room**: modified to add "and every Portal set into a Room on a Layer being removed by Remove Layer, whatever Layer the Portal is on", for the same reason.
- composing — **Gone with its Wall**: modified to "a Place Element, Edit Element, Remove Element, or Restack of a Room that leaves no Wall at the centre of a Portal set into a Room of its Layer, or, for a Restack onto another Layer, of the Layer it left or the Layer it joined, where one ran before it, removes that Portal, whichever Room it is set into, in the same history step, which undo restores whole with the Portal set where it was; the Author is told in the status line how many Portals were removed", because the order of a Layer's Rooms and which Layer a Room is on decide its Walls.
- composing — **Combined after every step**: modified to "after every Place Element, Edit Element, Remove Element, or Restack of a Room, every Remove Layer, every undo and redo of one, and every opening of a Project, every Room of each Layer the step changed has the floor and Walls the Rules above give for that Layer's Rooms as they now are, and every Room of every other Layer is unchanged", because a Restack can change two Layers and a Remove Layer takes a whole Layer's Rooms away.
- composing — **Other Rooms never move a Portal**: modified to add restacking any Room other than the one a Portal is set into, onto any Layer, to the steps that change none of the Portal's anchor, centre, rotation, or mirroring, because a Restack is a step like any other on Rooms.

## Implementation Decisions

The technology the architecture fixes (the history's command objects and its generic reflection-snapshot command, the serialisation registry and the Project snapshot, the Derived model component pattern, ShapeEngine's CombineOutlines and AnchorPortals, egui_dock and egui_ltreeview in the Editor) is used as written there and not restated. Layer compositing, one ping-pong pass per Layer, is the next change: this one draws every Layer through the existing draw path in one stacking order.

- **Layer identity**: `model` gains a Layer identity, a component every Layer carries, minted as an ElementId is when a Layer is spawned: by ProjectManager for the new Project's Layer and for each Layer it materialises on Open, and by AuthoringManager for an Add Layer; undo and redo keep it. It is never saved: the file and its format version stay as they are. Every Command names a Layer by its identity (Place Element's Layer, Paint's Layer, the Layer Commands, and Restack), and every history step that remembers a Layer remembers its identity, never its entity, so a Layer restored by an undo is the Layer earlier steps name (History follows a restored Layer); the placement and Remove Element steps change from the Layer's entity to its identity. _Why_ not saved: the history and the current Layer, the only things that need it, are never saved either.
- **Ownership**: the Layer component and the Layer identity are AuthoringManager's to write; ProjectManager writes them only through the Project lifecycle's exception, for the new Project and on Open. A Level is named by its entity, which no Command of this change respawns; the change that adds and removes Levels gives Levels an identity as this one gives Layers.
- **Apply** gains Add Layer (the Level and the place), Remove Layer (the Layer), Reorder Layers (the Layer and the place), and Restack (the ElementId, the Layer, and the place), each handled by AuthoringManager, each a step of its own that closes any gesture group left open, and each refused before anything is recorded with a CommandFailed carrying the reason.
- **Add Layer** is a reversible command of its own remembering the Level, the place, the name it gave, and the identity it minted: applying spawns the Layer at that place among the Level's children; reverting despawns it, which finds it empty because every later step, every placement and restack onto it included, has been undone first. AuthoringManager answers the Command, not its redo, with a LayerAdded message naming the new Layer's identity, which the Editor makes current.
- **Remove Layer** is a reversible command of its own: it takes the Layer's identity, every reflected component of the Layer (unknown envelopes included), its place among the Level's children, and, for each Element on it in order, the snapshot the generic Remove Element command takes; reverting respawns the Layer at its place and its Elements as its children in order with their ElementIds, and deriving rebuilds their Wall and Room shapes and Terrain coverage after the frame's Managers, as it does after Open. It follows Edit that carries anchored Elements: the Portals on other Layers set into a Wall or a Room on the Layer are found as that guideline finds them, and one history group records the Remove Element of each, then the Layer's removal, so undo restores the Layer before the Portals; a lost Portal is never among them. AuthoringManager answers with the existing PortalsRemoved message naming them, and the Editor writes how many to the status line. The last Layer is refused before anything is recorded.
- **Reorder Layers** is a reversible command of its own remembering the Layer's identity and its place before and after; it moves the Layer among the Level's children and respawns nothing.
- **Restack** is a reversible command of its own remembering the ElementId, the Layer and place before, and the Layer and place after; it moves the Element's entity among the Layers' children, re-parenting it for another Layer, so its entity, its components, and its derived shape stay as they are. For a Room, AuthoringManager works out through ShapeEngine the combination of the Layer it leaves and of the Layer it joins (one Layer within a Layer) before and after the Restack, and records in the same history group the Remove Element of every Portal set into a Room of either Layer with a Wall at its centre before and none after, as the Commands that take Walls away do; the deriving of a Layer whose Elements' order changed or that a Room joined or left already re-derives both Layers.
- **Rendering** changes nothing: RenderEngine's stacking order already walks each Level's Layers in the order of its children and each Layer's Elements in theirs, one depth unit apart, and follows the Levels' and Layers' children through change detection, so a reorder, a restack, a removal, and their undoing are drawn in the frame they happen; RenderRegion draws through the same depths, so the Export follows.
- **Picking** changes nothing: it already goes from the topmost Layer down and each Layer's last-drawn Element back, across every Layer of the Level.
- **The Layers panel** is an egui_dock tab named Layers, docked to the right of the viewport at a fifth of the window's width in the default layout, holding an egui_ltreeview tree with one leaf per Layer keyed by its identity, topmost first, the current Layer as the tree's selection, and drag and drop between rows on; a drop gives the place among the Level's Layers counted from the lowest, the list being the Level's order reversed. The plus and minus buttons sit above the list. Layer Groups, which the tree will nest, are a later change.
- **The current Layer** is Editor state holding a Layer identity, checked every frame against the Level's Layers: when it names none, the Editor moves it as The current Layer moves on says, from the order it saw the frame before, and on Open, which the Editor learns from ProjectManager's answer, it becomes the topmost Layer. Every tool reads it instead of the Project's first Layer. The Paint tool's Brush options and painting follow it as the Paint tool already follows the current Layer.
- **Restack controls**: the options strip shows them for any selected Element, beside the options its kind already has; the bindings gain the four shortcuts, stated once so that the menu shows exactly the keys that work. The Editor computes the place each sends from the Element's place among its Layer's children; the Layer menu sends the target Layer's number of Elements.
- **Waiting**: the Layers panel and the restack shortcuts wait under the conditions undo already waits under, and the panel's buttons and dragging, the restack controls, and the shortcuts under the Export's waiting as well.
- **The development-only input script**: its `describe` step logs the Level's Layers topmost first with their names and identities, which is current, and each Layer's Elements by ElementId in order, and lists the panel's rows and buttons and the restack controls among the clickable widgets with their rectangles, so a script can aim at them.

## Testing

- **Layers seam**: a headless App of the real plugins of `model`, `history`, LibraryAccess, LibraryManager, ProjectManager, and AuthoringManager, with no window and no RenderEngine, over a fixture Asset Folder holding images of known pixel size, a door image among them, and a texture, driven by Apply, Undo, and Redo messages, and by Open and Save over fixture Project files in temporary directories, and asserted on the Level's Layers (their names, their order, and their Elements by ElementId in order), the Element, Wall, Room, Portal, and Terrain components, the derived shapes and coverage, the answers, and the history.
  - **Layers in order**: `crates/drs-app/tests/layers.rs::layers_in_order` (the Level's Layers and their Elements in order after placing on two Layers in turn)
  - **Added where asked**: `crates/drs-app/tests/layers.rs::added_where_asked` (at the bottom, in the middle, at the top; undo and redo)
  - **Named in turn**: `crates/drs-app/tests/layers.rs::named_in_turn` (after `Layer 1`, after a removed highest Layer, and beside a Layer named otherwise)
  - **Removed with its Elements**: `crates/drs-app/tests/layers.rs::removed_with_its_elements` (a Layer holding a Prop, a Wall with a Portal, a Room, and a Terrain of several strokes; after undo every ElementId, component, and place as before, the Terrain's coverage tile for tile as before; redo), `crates/drs-app/tests/layers.rs::a_removed_layer_keeps_unknown_data` (an opened Project with an Element of an unknown kind and an unknown envelope on the Layer and on an Element; saved after the removal is undone, byte for byte the file opened)
  - **Gone with the Layer's Walls**: `crates/drs-app/tests/layers.rs::gone_with_the_layers_walls` (a Portal on one Layer set into a Wall and another into a Room on the removed Layer; the answer naming both; undo setting both back; a lost Portal left standing), `crates/drs-app/tests/layers.rs::portals_go_with_their_layer` (a Portal on the removed Layer set into a Wall on another: the Wall's stretches without it, and back after undo)
  - **The last Layer stays**: `crates/drs-app/tests/layers.rs::the_last_layer_stays`
  - **Reordered to a place**: `crates/drs-app/tests/layers.rs::reordered_to_a_place` (down, up, to either end, to its own place with no step; undo and redo)
  - **Reordering keeps Rooms**: `crates/drs-app/tests/layers.rs::reordering_keeps_rooms`
  - **Layer Commands are refused when malformed**: `crates/drs-app/tests/layers.rs::layer_commands_are_refused_when_malformed`
  - **History follows a restored Layer**: `crates/drs-app/tests/layers.rs::history_follows_a_restored_layer` (a Prop placed and a Wall edited on a Layer, the Layer removed, then undo back past the placement and redo forward again)
  - **Restacked to a place**: `crates/drs-app/tests/layers.rs::restacked_to_a_place` (one place up and down, to the top and the bottom, onto the top of another Layer, to its own place with no step; undo and redo, ElementId and components unchanged)
  - **Restack is refused when malformed**: `crates/drs-app/tests/layers.rs::restack_is_refused_when_malformed`
  - **Anchors cross Layers**: `crates/drs-app/tests/layers.rs::anchors_cross_layers` (a set Portal restacked under its Wall, moved to another Layer, and its Wall moved to a third: the anchor, centre, and the Wall's stretches unchanged)
  - **Restacked Rooms recombine**: `crates/drs-app/tests/layers.rs::restacked_rooms_recombine` (a cut moved below the Room it cut, its Walls round the hole gone and a Portal set into them removed and named; a Room moved onto another Layer leaving its combination walled whole on its own and joining the other's; undo restoring both Layers and the Portal)
  - The modified composing Rules: **One history** `crates/drs-app/tests/layers.rs::layer_commands_share_the_history`, **Redo repeats exactly** `crates/drs-app/tests/layers.rs::layer_commands_redo_exactly`, **A failed Command is reported** by `layer_commands_are_refused_when_malformed`, `the_last_layer_stays`, and `restack_is_refused_when_malformed` above, **Gone with its part of the Wall** and **Gone with its part of the Room** by `gone_with_the_layers_walls` above, **Gone with its Wall**, **Combined after every step**, and **Other Rooms never move a Portal** by `restacked_rooms_recombine` and `reordering_keeps_rooms` above, with `crates/drs-app/tests/layers.rs::restacking_another_room_moves_no_portal`.
  - The existing composing Rule **One Terrain per Layer** across Layers: `crates/drs-app/tests/layers.rs::each_layer_paints_its_own_terrain` (a Paint on each of two Layers makes two Terrains, and a later Paint adds to its own Layer's).
- **Offscreen export seam**: the existing headless App with RenderEngine under Bevy's default plugins without a window, exporting to a temporary PNG and asserting pixels.
  - **Layers in order** as drawn, and the existing export Rule **Drawn as in the editor** across Layers: `crates/drs-app/tests/export.rs::layers_stack_in_order` (a Prop on the upper Layer, placed first, drawn over an overlapping Prop on the lower Layer placed after it), `crates/drs-app/tests/export.rs::a_reordered_layer_is_drawn_in_its_place` (the same two after a Reorder Layers, the other's colour on top)
  - **Restacked to a place** as drawn: `crates/drs-app/tests/export.rs::a_restacked_element_is_drawn_in_its_place` (two overlapping Props, the lower one's colour on top after a Restack to the front)
  - **Removed with its Elements** as drawn: `crates/drs-app/tests/export.rs::a_removed_layer_leaves_the_export` (the background where the removed Layer's Prop stood, its colour back after undo)
- **Projects seam**: the existing headless App saving and reopening a Project of three Layers with Elements on each, reordered and restacked.
  - The existing projects Rules **Everything the Project is**, **Opened as saved**, and **Saving is a fixed point** across Layers: `crates/drs-app/tests/projects.rs::layers_are_saved_in_order` (every Layer's name and place, every Element on its Layer in order with its ElementId, and the second save byte for byte the first)
- **By hand**: **The current Layer**, **Placed on the current Layer**, **The current Layer moves on**, **No Layer, nothing placed**, **The Layers panel**, **The panel waits**, and **Restack controls**: no automated seam for the egui interface, the accepted deviation of the composing spec; verified by driving the editor with the development-only input script, whose `describe` step shows the Layers, the current one, and each Layer's Elements, and which also confirms that the viewport draws Layers in their order and follows a reorder and a restack in the frame they happen.

## Out of Scope

- Renaming a Layer, and hiding, locking, fading, and blending it: Edit Layer, in Layer compositing; until then a Layer keeps the name Add Layer gave it.
- Drawing each Layer into its own pass: Layer compositing; here every Layer is drawn through the one stacking order.
- Layer Groups and nesting Layers in the tree: Group Layers.
- Several Levels, switching between them, and moving an Element to another Level: Levels.
- Restacking or moving a Terrain from the editor: Terrain is not picked, so the Restack controls never show for one; the Command accepts it.
- Restacking or moving several Elements at once, dragging an Element onto a Layer's row, duplicating or merging Layers, a context menu on a row, thumbnails or Element counts in the panel, and keyboard navigation of the list.
- Restricting picking to the current Layer.
- A confirmation before removing a Layer that holds Elements: the removal is one undo step.

## Further Notes

- **Architecture check**: within the architecture. The Layer Commands and Restack are AuthoringManager's, sent by the Editor as Apply; the Layers panel uses egui_dock and egui_ltreeview, confined to the Editor; drawing and picking already follow the Layers' order; ShapeEngine's CombineOutlines and AnchorPortals serve a restacked Room as they serve an edited one. The Layer identity is a new `model` type beside ElementId, with no new component, contract operation, or dependency direction. The architecture's History bullet and its rule that Elements are addressed by ElementId speak of Elements only; this change addresses Layers the same way, which the architecture may want to state.
- **The capability**: levels-and-layers is new with this change. Its pinned spec, written when this change lands, owns Add Level, Remove Level, Reorder Levels, Add Layer, Remove Layer, Reorder Layers, Group Layers, Edit Layer, Resize Bounds, and Set Ambient Light, every one defined by the domain, owned by no other capability, and on the roadmap; all but the three Layer Commands here stay for the changes that build them.
- **Restack spans Layers**: moving an Element to another Layer is a Restack, not an Edit Element, because the domain's stacking order is ordered by Layer first, so changing an Element's Layer is changing its place in the stacking order; the Layer an Element belongs to is not one of its properties.
- **Removing a Layer removes the Portals set into its Walls** rather than leaving them standing as lost Portals: the editor removing the Layer knows those Portals and can restore them on undo, as Remove Element of a Wall does; a lost Portal stands only where an editor that did not know Portals removed their Wall. A Portal set into a Wall on another Layer is otherwise ordinary: Refused anchors refuses only a Wall on another Level.
- **The current Layer** is the Editor's form of the domain workflow's first step, "The Author picks a Layer of a Level"; it is presentation state, never part of the Project.
- A Portal set into a Room and stacked below it, which only setting an older freestanding Portal into a newer Room produced, can now be brought forward over the Room's floor with Restack.
