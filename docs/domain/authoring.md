# Authoring

Composing Projects out of Levels, Layers, and Elements. Everything the author builds lives here; the Assets it is built from live in the [Asset Library](./asset-library.md).

## Language

### Structure

**Author**:
A person who builds Projects with the editor.
_Avoid_: user, creator, GM

**Project**:
The author's saved work: one or more Levels, the Grid, the Bounds, its Asset References with the version of each Asset Folder they come from, the Plugins its Element kinds come from with their versions, and other metadata that applies to all Levels.
_Avoid_: save file, document, map

**Level**:
A complete, independent map within a Project, such as one floor of a building. Each Level has its own Ambient Light.
_Avoid_: floor, page, map

**Grid**:
The Project's grid of square cells. It sets the scale of every Level, and Assets have a natural size in cells.
_Avoid_: tile (when meaning a cell)

**Bounds**:
The rectangle, measured in Grid cells and shared by all Levels of a Project, that decides what gets exported. The author can grow or shrink it at any time.
_Avoid_: map size, canvas

**Layer**:
A named slice of a Level that holds Elements, as in an image editor. A Layer can be hidden and locked, and has an opacity and a blend mode (industry-standard meaning).

**Layer Group**:
A named group of Layers and other Layer Groups. It has the same properties as a Layer, and they apply to everything inside it.
_Avoid_: folder

**Element**:
Anything placed on a Layer: a Wall, a patch of Terrain, a Prop, a Light, and so on.
_Avoid_: object, node, entity

**Stacking order**:
The order in which Elements are drawn: by the position of their Layer among the Level's Layers and Layer Groups, then by position within the Layer.
_Avoid_: z-index, depth

**Ambient Light**:
The light that fills a whole Level before any Light is placed, such as the dark of a cellar or the daylight of a courtyard.

**Brush**:
A way of painting onto a Level. Its settings can be saved as a [Brush Preset](./asset-library.md#language).

**Stroke**:
One pass of a Brush along a path, laid by one Paint, that adds to a painted Element or, as an erase, removes from it. A painted Element's shape is its strokes, in order.

**Prefab Instance**:
A group of Elements placed from a [Prefab](./asset-library.md#language) and linked to it until detached. A linked Prefab Instance may lag behind its Prefab until it is synced.

### Element kinds

**Terrain**:
Painted ground made of several blended Materials.

**Patch**:
A painted area of a Material with a soft, automatic border, such as mud or moss on a floor.
_Avoid_: material (for this Element kind)

**Water**:
A painted body of water whose depth follows the distance from its shore.

**Cave**:
A painted cave area with Walls generated around it.

**Room**:
A floor area with Walls generated around its outline.
_Avoid_: building, floor shape

**Wall**:
A boundary along a line that blocks movement and sight.

**Portal**:
An opening such as a door or window, either set into a Wall or freestanding.

**Path**:
An image repeated along a line, such as a road, a cliff edge, or a fence.

**Pattern Shape**:
An area filled with a repeating Material.

**Roof**:
A roof generated over an outline.

**Prop**:
A single placed image, such as a table, a barrel, or a tree.
_Avoid_: object

**Light**:
A light source. Walls, closed Portals, and Elements marked as blocking light cast shadows from it.

**Label**:
Text placed on a Level.
_Avoid_: text

**Trace Image**:
A reference image to trace over. Never exported.

## Invariants

**Nothing is fixed at creation**: every property of an Element stays editable after it is placed, including the shape of anything painted.

**Every Element belongs to exactly one Layer, and every Layer to exactly one Level.**

**Levels are independent**: nothing on one Level depends on another Level.

**No Element kind is limited to one per Level.**

**Every Element that shows a surface is drawn with a Material**: Terrain blends several; a Prop's default Material simply shows its image.

**Bounds only decide what is exported**: Elements may lie outside the Bounds, and changing the Bounds never adds, removes, or changes an Element.

**A Portal set into a Wall moves with it**: it keeps its position along the Wall through every edit, and exists only while the part of the Wall it sits in exists.

**A Project is device-independent**: it holds nothing specific to one device or operating system, such as an absolute path.

**References are never dropped**: opening and saving a Project on a device that lacks some Assets or Plugins keeps every Asset Reference, every Element that uses a Missing Asset, and every Element with a [Missing Element Kind](./extension.md#language).

## Commands

**Add Level**, **Remove Level**, **Reorder Levels**.
**Add Layer**, **Remove Layer**, **Reorder Layers**, **Group Layers**.
**Edit Layer**: change a Layer's or Layer Group's name, visibility, lock, opacity, or blend mode.
**Resize Bounds**: grow or shrink the Bounds on any side.
**Set Ambient Light**: change a Level's Ambient Light.
**Place Element**: add an Element to a Layer.
**Edit Element**: change any property of an Element.
**Remove Element**: take an Element off its Layer.
**Restack**: change an Element's place in the stacking order.
**Paint**: add to or remove from a painted Element with a Brush.
**Set Portal into Wall**: anchor a Portal to a position along a Wall.
**Free Portal**: make a Portal freestanding where it stands.
**Place Prefab**: add a Prefab Instance to a Layer.
**Sync Prefab Instance**: bring a linked Prefab Instance up to date with its Prefab.
**Detach Prefab Instance**: turn a Prefab Instance into plain Elements.

## Workflows

### Compose a Level
1. The Author picks a Layer of a Level.
2. The Author places, paints, or edits Elements on it.
3. Walls, Portals, and Prefab Instances stay consistent through every edit.
4. Every Command can be undone.
