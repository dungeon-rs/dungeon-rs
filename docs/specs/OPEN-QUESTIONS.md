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
- Dungeondraft compatibility and parity: import of existing `.dungeondraft_map` and `.dungeondraft_pack` libraries; recolouring of colourable Assets (red mask); tags and tag sets (merged by name) driving search; a Scatter brush (random rotation, scale, colour, spread; area scatter); alignment guides beyond snapping; map generation (Map Wizard).
- Embedded Assets and vendor licences (warn when embedding a vendor Asset?).
- Export formats beyond a plain image: Universal VTT, separate roof images, etc.
- Grid presentation beyond faint lines in the viewport: dots, hiding the Grid, a colour the Author chooses, and the Grid drawn into an Export.

## From the architecture

- Show a broken Shader's error to the Author (today it only reaches the log and the surface disappears); validate with naga and keep the last good version.
- WebP exports are limited to 16,383 px per side by the format; how the export UI communicates this.
- 100k fully visible Props is borderline: level-of-detail or static batching.
- Lighting under tiled export (single camera per config; per-tile light maps).
- Export: screen-space effects (soft shadows, light maps, blur) need tile gutters or seams appear; JPEG at that size needs a scanline or streaming encoder; lossy WebP needs libwebp.
- Label crispness across zoom and export (atlas per size, memory at high export scales).
- Project format: Embedded Assets need a container (for example a zip with `project.json` and `assets/<blake3>.<ext>`, stored once).
- Memory: dropping the last `Handle` frees an asset, but RSS stays at its high-water mark because the allocator keeps freed pages; consider mimalloc or jemalloc with purging, and `RenderAssetUsages::RENDER_WORLD` to drop the CPU copy (GPU free on drop unmeasured).
- Release and packaging: build for Linux, Windows, and macOS with Bundled Files, locales, and licences; macOS `.app` with icon and dmg; Windows icon and no console window in release; release notes from git-cliff. Undecided: universal macOS binary, notarisation, Windows signing, a Linux package, cargo-dist or cargo-packager.
- Layer-specific disallowed APIs could turn "review" rules into enforced ones: per-crate `clippy.toml` `disallowed-methods` (`std::fs` and `std::net` in Engines; `HashMap` in serialised `model` types). Clippy does not merge nested `clippy.toml` files with the root, so each repeats the root settings.
- Replacing a file atomically (write beside, flush, rename) belongs to the ResourceAccess crates alone; a per-crate `clippy.toml` could disallow `std::fs::rename` everywhere else.
- Environment variables are read only in the Client and the Host under the `dev` feature, and in the diagnostics Utility for `RUST_LOG` and for whether a crash dialog can be shown; a per-crate `clippy.toml` could disallow `std::env::var` and `std::env::var_os` everywhere else.
- Only the Client ends the application: `AppExit` is written in the Editor alone, so the unsaved-changes question is never bypassed; the `ci` tool could look for `AppExit` outside `drs-editor`.
- An ownership check for `model` components: the `ci` tool could read a table of which crate owns (writes) each `model` component type and look for writes elsewhere, turning the "each service owns its business objects" row from review into enforced.
- A profile override in `Cargo.toml` (`[profile.<name>.package.<crate>]`) is named in the architecture's Profiles bullet with its reason; the `ci` tool could compare the two.
- Engines keep no mutable statics, so that everything an Engine remembers is held by the value its caller owns; the `ci` tool could look for `static` items holding an atomic, a lock, or a cell in the Engine crates' sources.
- Markdown line width: READMEs and documents are wrapped by hand at widths from 80 to 100 columns; a formatter or a `ci` check with one width would keep them alike.
- Angle constructors outside the Editor: glam's `Quat::from_rotation_z` and `Rot2::radians` may call the platform's `sin` and `cos`, so an Export built with them could differ between machines; a `disallowed-methods` entry would enforce building turns through `ops::sin_cos`, though it may be redundant where glam is built with `libm`.
- Headless rendering tests: the offscreen Export seam runs RenderEngine without a window but needs a GPU adapter and fails without one; whether every hosted CI runner has one, whether a software adapter (Mesa lavapipe) is the fallback, and whether that seam or a render-less check of the sprites can cover the viewport's rendering Rules, which are still checked by hand.
- `ShownAsset` completeness: every kind that shows an image Asset must add its component to `ShownAsset`, or counting Missing Assets and placeholders silently leaves it out; the `ci` tool or a model test could check that every component holding an `AssetReferenceRow` appears there.
- Engine contracts in code: an Engine's public API should be its contract verbs alone; `cargo public-api` per Engine crate, compared against the Contract column of the architecture, would catch a helper made public by accident.
- Doc-comment line length: rustfmt does not wrap comments, so a doc line can run past the 100 columns the code keeps; a `ci` check or `comment_width` with `wrap_comments` (nightly rustfmt) would hold them to it.
- Routing Element changes on purpose: clippy's `wildcard_enum_match_arm` does not see inside `matches!` or an if-let with an `else`, so a fallback over `ElementChange` in the authoring Manager slips past it; the `ci` tool could look for either there.
- Migrating every older version: the `ci` tool could check that each `Serialisable` whose `VERSION` is above 1 has a `read` arm for every version below it, so a version is never read as another.
- Writers of the `Pointer` and the `SnappedPoint`: the Editor alone writes the one and the authoring Manager alone the other; the `ci` tool could look for `ResMut` or `resource_mut` of either outside its owner's non-test sources.
- Loops that step a float by adding to it stop advancing far from zero and never end; the `ci` tool could look for a float `+= 1.0` inside a loop, so loops over cells count with integers.
- Every commit of a branch passing the lints: `just check` lints the tip alone, so a commit that fails clippy on its own lands unnoticed; CI could run clippy on each commit of a pull request.
- A WGSL Shader's constants and structs mirror Rust ones by hand (the stroke Shader's tile side and `COVERAGE_TILE_PIXELS`, its `Tile` and `Segment` and the paint Engine's `TileData` and `SegmentData`); generating them from the Rust side, or a check comparing them, would keep the two from drifting.
- An Engine's public items beyond its contract verbs: PaintEngine exposes its cache type, its rasterizer system parameter, its plugin, and a hidden function that readies a World to rasterize on the GPU for its timed tests beside ApplyStroke and Rasterize; whether the contract names such types, or a `cargo public-api` check allows them by kind.
