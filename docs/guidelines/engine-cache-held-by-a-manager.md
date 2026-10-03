# Engine cache held by a Manager

**Use when**: an Engine keeps work between calls so the next call redoes only what changed (the paint Engine's tiled coverage of a Terrain), and what it keeps is drawn or read by others. **Not when**: the Engine computes its result afresh on every call (a Wall's shape, a resolution table): that is a derived model component alone. Nor when nothing outside the Manager reads the result: the cache then needs no published component.
**Exemplar**: `crates/drs-authoring-manager/src/terrain.rs`

## Rules

- The Engine owns the cache's type, a `pub struct` with private fields, so only the Engine's own verbs look inside it (`PaintCache` in the paint Engine's `cache.rs`); an Engine keeps no state of its own, so the value lives with its caller, as the architecture's Painting bullet says. A cache that works one way or another depending on what the App holds is made by a constructor that settles the way once, from what the verb is passed (`PaintCache::new` from the `StrokeRasterizer`), so the verb never switches a live cache over; one that needs nothing derives `Default`.
- The Engine's verb brings the cache up to the model in place and says whether anything a reader sees changed (`apply_stroke` returns `bool`); the cache hands out only the model's derived component (`PaintCache::coverage`), sharing its buffers through `Arc` rather than copying them, never its own parts. Work the verb hands to the render world goes through a system parameter the Engine defines (`StrokeRasterizer`), which the Manager's system takes and passes on, never through an event.
- The Manager that calls the verb holds the cache in a `pub(crate)` component of its own on the Element (`Painted`), never in `drs-model`, never reflected or serialised, so a Save and a Remove Element snapshot leave it out and a restored Element builds it again.
- The system that drives it is the derived model component's: added `.after(ManagerSystems::Redo)`, driven by change detection on the source and on whatever else the verb takes (`Ref<Terrain>` and the `Viewport`), inserting the published component and the cache together through `Commands` the first time, and assigning the published component only when the verb says something changed.
- Each unit of the published component carries a revision that changes only when its contents do (`CoverageTile::revision`), so a reader uploads only what changed.
- The Engine's unit tests check the cache against the Engine's stateless verb over the same input (`appending_equals_rasterizing`); the seams check the published component, never the cache.

## Example

```rust
/// A Terrain's tiled coverage as the paint Engine keeps it, private to the Manager: the cache the
/// published [`TerrainCoverage`] shares its tiles with.
#[derive(Component, Debug)]
pub(crate) struct Painted(PaintCache);

pub(crate) fn derive_coverage(
    mut commands: Commands,
    mut terrains: Query<(
        Entity,
        Ref<Terrain>,
        &mut Element,
        Option<&mut TerrainCoverage>,
        Option<&mut Painted>,
    )>,
    viewport: Option<Res<Viewport>>,
    mut rasterizer: StrokeRasterizer,
) {
    let looked_elsewhere = viewport.as_ref().is_some_and(DetectChanges::is_changed);
    let viewport = viewport.map_or_else(Viewport::default, |viewport| *viewport);
    for (entity, terrain, mut element, coverage, painted) in &mut terrains {
        let repainted = terrain.is_changed();
        if !repainted && !looked_elsewhere {
            continue;
        }
        match (coverage, painted) {
            (Some(mut coverage), Some(mut painted)) => {
                if apply_stroke(&mut painted.0, &terrain.strokes, &viewport, &mut rasterizer) {
                    *coverage = painted.0.coverage();
                }
            }
            (_, painted) => {
                let fresh = PaintCache::new(&rasterizer);
                let mut cache = match painted {
                    Some(mut painted) => std::mem::replace(&mut painted.0, fresh),
                    None => fresh,
                };
                apply_stroke(&mut cache, &terrain.strokes, &viewport, &mut rasterizer);
                commands
                    .entity(entity)
                    .insert((cache.coverage(), Painted(cache)));
            }
        }
    }
}
```

## Pitfalls

- Comparing the published component with the cache in the Manager to decide whether to publish: the Manager then reads the cache's insides, and the comparison drifts from what the verb changed.
- Keeping the cache in a resource keyed by entity: an undone removal respawns the Element under a new entity, and the cache of the old one is never dropped.
- Cloning the source into the cache on every run: the cache keeps what it last saw so it can tell what changed, and only what differs needs copying (`truncate` and `extend_from_slice`).
