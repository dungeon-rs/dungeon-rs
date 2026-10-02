# Roadmap

## I have a working editor: I can place Props from my own Asset Folder, undo, save, reopen, and export an image

- **[Walking skeleton: save, reopen, export](changes/walking-skeleton-save-reopen-export.md)**: the Author saves the Project, reopens it with its Asset References resolved (Missing Assets kept as placeholders and reported), and exports the Level as a PNG at a chosen resolution within fixed Bounds. Commands: Export Level.
- **[Diagnostics](changes/diagnostics.md)**: logs go to a daily-rolling file, a crash shows a dialog and leaves a crash report, and Bundled Files are found on every platform. Commands: none.

## I can build and export a simple dungeon: Walls, doors, Rooms, painted Terrain, and Layers, from a library I can actually browse

- **Browse the library at scale**: thumbnails appear as they are generated in the background, and search finds any of hundreds of thousands of Assets instantly. Commands: none.
- **Walls**: the Author draws straight or curved Walls and edits their points at any time. Commands: none.
- **Portals**: doors and windows set into Walls stay anchored through every Wall edit, or stand free. Commands: Set Portal into Wall, Free Portal.
- **Rooms**: Room outlines generate their Walls, and outlines combine and cut. Commands: none.
- **Paint Terrain with one Material**: the Author paints Terrain with a Brush, and every stroke stays editable. Commands: Paint.
- **Blend any number of Materials**: Terrain blends as many Materials as the Author paints. Commands: none.
- **Levels and Layers**: the Author adds, removes, and orders Levels and Layers, and restacks Elements. Commands: Add Level, Remove Level, Reorder Levels, Add Layer, Remove Layer, Reorder Layers, Restack.
- **Layer compositing**: Layers and Layer Groups can be hidden, locked, faded, and blended. Commands: Edit Layer, Group Layers.
- **Bounds**: the Author resizes the Bounds, which are always visible in the editor. Commands: Resize Bounds.

## My Projects travel

Relink with reports that name the version gap, Embedded Assets, managing Asset Folders, and Asset Packs. Commands: Relink, Embed Asset, Remove Asset Folder, Rename Canonical Name, Install Asset Pack.

## Richer maps

Caves, Water, Patches, Paths, Pattern Shapes, Roofs, Labels, custom Materials and Shaders, and Trace Images. Commands: none.

## Reusable arrangements

Prefabs placed as linked instances that follow, detach from, or sync with their Prefab. Commands: Place Prefab, Update Prefab, Sync Prefab Instance, Detach Prefab Instance.

## Light and extension

2D lighting with Ambient Light, Plugins, and VTT export. Commands: Set Ambient Light.
