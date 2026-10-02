# Export-only content through render layers

**Use when**: RenderEngine draws one thing two ways, a stand-in in the viewport and an exact version the Export computes for the region it captures (a Terrain's coverage tiles at 32 pixels per cell, and its coverage rasterized at the Export's resolution). **Not when**: the content is the Editor's own overlay (the selection outline, a Wall's handles, the Portal marker): the Editor's systems stop drawing it while `EditorState::exporting`, as `outline_selection` does. Nor when both cameras draw the same thing (sprites, Wall meshes): it stays on the default layer with no `RenderLayers`.
**Exemplar**: `crates/drs-render-engine/src/region.rs`

## Rules

- The two layers are `VIEWPORT_LAYER` and `EXPORT_LAYER` in `projection.rs`; the viewport camera sees `RenderLayers::from_layers(&[0, VIEWPORT_LAYER])` and the export camera `RenderLayers::from_layers(&[0, EXPORT_LAYER])`, so whatever has no `RenderLayers` both draw.
- The viewport's stand-in is spawned with `RenderLayers::layer(VIEWPORT_LAYER)` (the coverage tiles in `terrain.rs`), so the Export never shows magnified pixels.
- The exact version reaches the Engine with the region request, computed by the Manager over that region at its resolution (`RegionCoverage` through `request_region`), and is moved into `Offscreen` rather than copied; a system in the Engine's `PostUpdate` chain spawns it with `RenderLayers::layer(EXPORT_LAYER)` at the depth and look the viewport uses, despawning the previous region's first.
- The capture waits until the exact version is spawned and a frame has passed, so it is on the GPU: `gate_regions` holds the capture while `Offscreen::terrains` is set and sets `fresh` once it is drawn; `release_regions` despawns what is left.
- Its pixels fall on the region's pixels one to one, so it is sampled with `ImageSampler::nearest()`, never filtered.

## Example

```rust
pub(crate) fn draw_region_terrains(
    mut commands: Commands,
    mut offscreen: ResMut<Offscreen>,
    mut drawings: ResMut<TerrainDrawings>,
    mut assets: TerrainAssets,
) {
    let Some(terrains) = offscreen.terrains.take() else {
        return;
    };
    for quad in std::mem::take(&mut offscreen.quads) {
        commands.entity(quad).despawn();
    }
    let mesh = drawings.quad(&mut assets.meshes);
    let extent = Vec2::splat(terrains.cells);
    for coverage in terrains.coverages {
        let Some((look, depth)) = drawings.look_of(coverage.terrain) else {
            continue;
        };
        let image = assets.images.add(coverage_image(
            coverage.pixels,
            terrains.size,
            ImageSampler::nearest(),
        ));
        let material = assets
            .materials
            .add(look.material(terrains.bottom_left, extent, image));
        let quad = commands
            .spawn((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(material),
                quad_transform(terrains.bottom_left, extent, depth),
                RenderLayers::layer(EXPORT_LAYER),
            ))
            .id();
        offscreen.quads.push(quad);
    }
    if !offscreen.quads.is_empty() {
        offscreen.fresh = true;
    }
}

pub(crate) fn gate_regions(
    mut offscreen: ResMut<Offscreen>,
    loading: Query<(), With<Loading>>,
    drawings: Res<TerrainDrawings>,
) {
    let fresh = std::mem::take(&mut offscreen.fresh);
    let capture = offscreen.pending.is_some()
        && !fresh
        && loading.is_empty()
        && offscreen.terrains.is_none()
        && !drawings.loading();
}
```

## Pitfalls

- Drawing the stand-in on the default layer: the Export then shows the viewport's coarse version under, or instead of, the exact one.
- Capturing in the frame the exact version is spawned: its mesh and Material are not on the GPU yet, and the region comes back without it.
- Keeping the previous region's quads: two tiles' worth of export-only content overlap in the next capture.
