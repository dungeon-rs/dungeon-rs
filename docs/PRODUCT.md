# Product

A fast map editor for tabletop role-playing games that exposes the full power of a game engine — Materials, Shaders, brushes, precise placement — to authors, with the quality-of-life features that make it just as comfortable for casual users.

## Principles

- **Gets out of your way.** The common path is short; power is there when you go looking for it.
- **Simple view, advanced view.** Everything offers a simple view with what a regular author needs, and an advanced view with every property we reasonably expose. Each view is tailored to its use case.
- **Fast at any library size.** Opening the editor and browsing Assets stays fast with hundreds of thousands of Assets.
- **Assets work as they are.** A vendor's release is usable the day it ships, without editor-specific packaging.
- **Maps travel.** A map moved to another device either works, or says in plain terms what is missing and what it affects.

## Baseline

Dungeondraft is the functional baseline: its tools cover what authors need. This product differs in how those tools work.

Pain points that matter:
- Stutter on busy Layers.
- Only 4–8 texture slots when blending Terrain.
- Single-line text only.
- 10–30 minute loads for large libraries, and 25+ GB of memory.
- Exports capped at 300 PPI, and no headless export.
- Portals dropped or corrupted when their Wall is edited.
- Path-based Asset references, and silent drops on save.
- Properties frozen after placement, and gaps in undo (resizing, deleting a Level).
- Vendor packs must be repackaged and need a restart to refresh.
- Special characters in paths, cloud-synced folders, and deleting a loaded pack break or crash the app.

## Non-goals

- **Live co-editing.** Authors share work by exchanging Project files, never by editing the same Project at the same time.
