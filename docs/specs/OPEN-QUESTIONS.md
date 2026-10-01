# Open questions (specs)

Spec-level decisions parked during domain work. The spec skill picks these up; remove each once a spec settles it.

## From the domain interview

- How far automatic matching goes (format twins, recognising a folder whose Canonical Name differs).
- Using a differently named Asset Folder (e.g. the WebP library) for a Project saved against another.
- Perceptual hash per reference; opt-in background content index.
- Two installed Asset Folders claiming the same identity (PNG and WebP copies, side-by-side versions): prefer the newer, the format the Project used, or ask? Path-set overlap ("208 of 212 paths exist in X") as a way to recognise a renamed folder.
- After one confirmed relink, offer the same folder move for the other missing Assets.
- Two editors open on the same Asset Folders: lock the on-disk index and thumbnail caches (`File::try_lock`) and report "already open in another editor".
- An Asset Folder deleted while loaded: how the editor notices it and tells the Author (folder watching).
- Plugin components not registered for reflection are lost by generic Remove and undo, so registration must be enforced in the Plugin API.
- Dungeondraft compatibility and parity: import of existing `.dungeondraft_map` and `.dungeondraft_pack` libraries; recolouring of colourable Assets (red mask); tags and tag sets (merged by name) driving search; a Scatter brush (random rotation, scale, colour, spread; area scatter); alignment guides beyond snapping; map generation (Map Wizard); a maximum map size.
- Embedded Assets and vendor licences (warn when embedding a vendor Asset?).
- Export formats beyond a plain image: Universal VTT, separate roof images, etc.
- Grid presentation (lines, dots, hidden), snapping.

## From the architecture

- Show a broken Shader's error to the Author (today it only reaches the log and the surface disappears); validate with naga and keep the last good version.
- WebP exports are limited to 16,383 px per side by the format; how the export UI communicates this.
- Fast painting at high zoom needs GPU rasterization of the stroke cache.
- 100k fully visible Props is borderline: level-of-detail or static batching.
- Lighting under tiled export (single camera per config; per-tile light maps).
- Export: screen-space effects (soft shadows, light maps, blur) need tile gutters or seams appear; JPEG at that size needs a scanline or streaming encoder; lossy WebP needs libwebp.
- Label crispness across zoom and export (atlas per size, memory at high export scales).
- Thumbnails: bevy_egui logs "bindless textures not yet supported on metal"; measure when the thumbnail cache is built. Scroll offsets: `f32` layout holds whole-pixel precision only to about 16.7 M px, fine for 400k items but not multi-million libraries.
- Project format: Embedded Assets need a container (for example a zip with `project.json` and `assets/<blake3>.<ext>`, stored once).
- Memory: dropping the last `Handle` frees an asset, but RSS stays at its high-water mark because the allocator keeps freed pages; consider mimalloc or jemalloc with purging, and `RenderAssetUsages::RENDER_WORLD` to drop the CPU copy (GPU free on drop unmeasured).
- Release and packaging: build for Linux, Windows, and macOS with bundled resources, locales, and licences; macOS `.app` with icon and dmg; Windows icon and no console window in release; release notes from git-cliff. Undecided: universal macOS binary, notarisation, Windows signing, a Linux package, cargo-dist or cargo-packager.
- Diagnostics: crash reports go to the log or cache directory, not the working directory.
- Layer-specific disallowed APIs could turn "review" rules into enforced ones: per-crate `clippy.toml` `disallowed-methods` (`std::fs` and `std::net` in Engines; `HashMap` in serialised `model` types). Clippy does not merge nested `clippy.toml` files with the root, so each repeats the root settings.
