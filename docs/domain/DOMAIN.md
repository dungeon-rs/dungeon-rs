# dungeon-rs

A map editor for tabletop role-playing games. Authors compose 2D battle maps from Assets they already own, without those Assets needing editor-specific preparation.

## Contexts

- [Authoring](./authoring.md): composing Projects out of Levels, Layers, and Elements
- [Asset Library](./asset-library.md): the Assets available to an author, where they live, and how a Project refers to them
- [Output](./output.md): turning a Level into something usable outside the editor
- [Extension](./extension.md): Plugins that add behavior to the editor

## Relationships

- **Authoring → Asset Library**: Elements use Assets only through Asset References held by the Project; the Asset Library resolves them on each device.
- **Asset Library → Authoring**: when an Asset Folder changes, Authoring re-resolves the Asset References that point into it.
- **Output → Authoring**: an Export reads one Level and the Project's Bounds.
- **Extension → Authoring, Asset Library**: Plugins contribute Brushes and Element kinds to Authoring (a Project records which Plugin each Element kind comes from), and Indexing Rules, Asset Kinds, and Assets to the Asset Library.

## Invariants (all contexts)

**Every Command can be undone**: every Command listed in any context, without exception.

**Undo history is not part of a Project**: a Project holds what is true now.
