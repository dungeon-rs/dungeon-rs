# Needs

What the architecture must satisfy. Technology-agnostic. Verdicts per candidate are folded into the Technology decisions of `ARCHITECTURE.md`, not kept here.

## Must

### Authoring surface

**Editor-grade UI**: docked panels, a tree of Layers and Layer Groups, property editors with a simple and an advanced view, menus, text input, and drag and drop, usable without first spending months building widgets.
Source: previous attempt; principle "Simple view, advanced view" in [PRODUCT.md](../PRODUCT.md)

**Many Elements at interactive speed**: thousands of Props, Walls, and Lights on one Level, with smooth panning and zooming.
Source: principle "Fast at any library size"; baseline pain point (stutter on busy layers)

**Editable vector shapes**: lines, curves, and outlines whose points stay editable forever, with operations on outlines (combine, cut, split a Wall at a Portal).
Source: [Nothing is fixed at creation](../domain/authoring.md#invariants); Wall, Room, Cave, Pattern Shape, Path

**Editable painted regions**: soft-brush painting whose result stays editable (repaint, erase, reshape), never flattened into pixels.
Source: [Nothing is fixed at creation](../domain/authoring.md#invariants); Terrain, Patch, Water, Cave

**Unlimited Material blending**: Terrain blends any number of Materials, with no fixed slot count.
Source: Terrain; baseline pain point (4–8 texture slots)

**Every surface is a Material**: author-supplied Shaders load as Assets while the editor runs, and their parameters appear in the UI.
Source: [Every Element that shows a surface is drawn with a Material](../domain/authoring.md#invariants); Shader

**Layer compositing**: opacity, blend modes, hiding, and groups, in a strict stacking order.
Source: Layer; Layer Group; Stacking order

**Text on the map**: multiple lines, rotation, and fonts supplied as Assets.
Source: Label; font Asset Kind; baseline pain point (single-line text)

**Undo for every Command**, including resizing the Bounds and painting.
Source: [Every Command can be undone](../domain/DOMAIN.md#invariants-all-contexts)

### Assets

**Assets from any folder, added at runtime**: Asset Folders are chosen while the editor runs, not fixed when it is built or started, and work with any characters in their path and in cloud-synced locations.
Source: Add Asset Folder; baseline pain point (special characters in paths, cloud-synced folders, and deleting a loaded pack broke or crashed the app)

**Browsing at scale**: searching and thumbnails over 400,000+ Assets; lists render only what is visible; thumbnails are generated in the background; startup never reads every file.
Source: principle "Fast at any library size"; baseline pain point (10–30 minute loads)

**Bounded memory**: only Assets in use are loaded, and they are released again.
Source: baseline pain point (25+ GB memory use)

**Common image formats**: PNG, WebP, and JPEG are read.
Source: vendors ship PNG and WebP

**Noticing Asset Folder changes at startup**, cheaply.
Source: Asset Folder Changed

### Projects and output

**Portable, versioned Project format**: device-independent, never drops references, stores Embedded Assets once, and keeps older Projects readable.
Source: [A Project is device-independent](../domain/authoring.md#invariants); [References are never dropped](../domain/authoring.md#invariants); Embedded Asset

**Export at any resolution**, beyond the window and beyond single-texture size limits, to PNG and JPEG; WebP up to its format's limit.
Source: Export Level; baseline pain point (300 PPI cap)

**Desktop integration**: native file and folder dialogs, drag and drop from the operating system, clipboard.
Source: Add Asset Folder; Embed Asset

**Headless operation**: domain logic and rendering run without a window, for tests and batch export.
Source: METHODOLOGY (tests at agreed seams); baseline pain point (no headless export)

**Desktop platforms**: the current major release of Windows, macOS, and a mainstream Linux distribution.
Source: principle "Maps travel"; baseline platforms

## Want

**2D lighting with shadows**: Lights are blocked by Walls, closed Portals, and light-blocking Elements; each Level has an Ambient Light. Deferred: not core; re-evaluate the technology when lighting is built.
Source: Light; Ambient Light


**Noticing Asset Folder changes while running.**
Source: Asset Folder Changed

**Plugins loaded at runtime**: a Plugin is an archive of scripts and configuration, unpacked on install, adding Indexing Rules, Asset Kinds, Brushes, and Element kinds without recompiling the editor. Not needed for a first release, but the architecture must show it fits without restructuring.
Source: [Plugin](../domain/extension.md#language)
