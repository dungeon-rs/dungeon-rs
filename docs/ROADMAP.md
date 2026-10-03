# Roadmap

## I can build and export a simple dungeon: Walls, doors, Rooms, painted Terrain, and Layers, from a library I can actually browse

- **[Rooms combine and cut](changes/rooms-combine-and-cut.md)**: overlapping Room outlines combine into one, a Room cuts another, their Walls follow, and Portals set into them stay anchored. Commands: none.
- **[Blend any number of Materials](changes/blend-materials.md)**: Terrain blends as many Materials as the Author paints. Commands: none.
- **[Layers](changes/layers.md)**: the Author adds, removes, and orders the Layers of a Level, picks the current one, and restacks Elements within and across them. Commands: Add Layer, Remove Layer, Reorder Layers, Restack.
- **Layer compositing**: Layers can be hidden, locked, faded, and blended, hidden Layers leave the Export, and the points of hidden or locked Layers leave snapping's reach. Commands: Edit Layer.
- **Layer Groups**: Layers nest in Layer Groups whose visibility, lock, opacity, and blend mode apply to everything inside. Commands: Group Layers.
- **[Bounds](changes/bounds.md)**: the Author resizes the Bounds, which are always visible in the editor. Commands: Resize Bounds.
- **Levels**: the Author adds, removes, and orders Levels, switches between them, and exports the one chosen. Commands: Add Level, Remove Level, Reorder Levels.

## My Projects travel: a Project opened on another device works, or says in plain terms what is missing, and I can relink, embed, and manage the Asset Folders it depends on

- **Relink**: the Author confirms which Asset a Missing Asset means, which adds a place to its Asset Reference, so that the Project resolves again for everyone it worked for before. Commands: Relink.
- **Manage Asset Folders**: the Author removes an Asset Folder or renames its Canonical Name, and open Projects re-resolve. Commands: Remove Asset Folder, Rename Canonical Name.
- **Resolve by content**: an Asset moved or renamed inside its folder, or shipped in another image format, resolves by identical bytes or as a format twin without asking; weaker matches are suggestions for Relink. Commands: none.
- **Embed Asset**: a one-off Asset is stored inside the Project once and travels with it. Commands: Embed Asset.
- **Install Asset Pack**: an Asset Pack unpacks into a new Asset Folder under its Canonical Name. Commands: Install Asset Pack.

## Richer maps

Caves, Water, Patches, Paths, Pattern Shapes, Roofs, Labels, custom Materials and Shaders, and Trace Images. Commands: none.

## Reusable arrangements

Prefabs placed as linked instances that follow, detach from, or sync with their Prefab. Commands: Place Prefab, Update Prefab, Sync Prefab Instance, Detach Prefab Instance.

## Light and extension

2D lighting with Ambient Light, Plugins, and VTT export. Commands: Set Ambient Light.
