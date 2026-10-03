# Derived model component

**Use when**: a Client or an Engine needs something computed from the model that it may not compute itself, because the computing Engine is not among its dependencies (a Wall's shape from ShapeEngine, a Project's resolution table from CatalogEngine). **Not when**: the value is saved (a serialisable component), or only the crate that computes it reads it (keep it private to that crate).
**Exemplar**: `crates/drs-authoring-manager/src/derive.rs`

## Rules

- The component lives in `drs-model` beside what it is derived from (`WallShape` in `wall.rs`, `ResolutionTable` in `resolution.rs`), documented as never saved and naming the Manager that writes it. It implements no `Serialisable`, so a Save leaves it out; `WallShape` derives no `Reflect` either, so a Remove Element snapshot leaves it out and the restored Wall is derived again.
- Exactly one Manager writes it, calling the Engine directly, in a system added `.after(ManagerSystems::Redo)` in its plugin, so every Command, Undo, Redo, and Open of the frame is in before it runs and readers in `PostUpdate` see the result that frame.
- The system is driven by change detection on the source (`Changed<Wall>`, a Portal changed or removed, `Changed<AssetReferences>`): it inserts through `Commands` the first time and assigns in place after.
- It writes only when the result can differ: it remembers what it last derived from in a component private to the Manager (`DerivedFrom`, kept for a batch that is derived together, such as the Rooms of a Layer, on the Layer) and compares before assigning (`write_table`, each shape of a batch), so a reader's `Ref::is_changed` means a real change and a recolour does not rebuild a mesh.
- Readers only read it and treat its absence as not yet: RenderEngine skips the Element that frame and the Editor's hit test misses it.

## Example

```rust
/// What the shapes of a batch of Walls or Rooms were last derived from: a Wall alone, or the
/// Rooms of a Layer together, each one's identity, outline, thickness, and whether it cuts, in
/// stacking order, and where the Portals set into them are and how wide. A change that leaves
/// them as they were, a new colour, keeps the shapes. It is kept on the Wall, or on the Layer of
/// the Rooms.
#[derive(Component, Debug, Clone, PartialEq)]
pub(crate) struct DerivedFrom<H: OutlineHost> {
    /// The outlines the shapes were derived from, in stacking order.
    outlines: Vec<(ElementId, Path, f32, bool)>,
    /// The Portals set into them, each with the outline's number in the batch.
    portals: Vec<(ElementId, PortalSetting)>,
    /// The kind of the outlines.
    kind: PhantomData<fn() -> H>,
}

pub(crate) fn derive_shapes(
    mut commands: Commands,
    mut edits: OutlineChanges,
    mut removed_portals: RemovedComponents<Portal>,
    mut walls: Outlines<Wall>,
    mut rooms: Outlines<Room>,
    derived: (Query<&DerivedFrom<Wall>>, Query<&DerivedFrom<Room>>),
    layers: Query<&Children, With<Layer>>,
    mut boxes: Boxes,
    mut portals: Portals,
) {
    let removed = removed_portals.read().count() > 0;
    // Looking at whether a Portal changed through the query that writes them marks nothing.
    if !edits.any() && !portal_changed && !removed {
        return;
    }
}

fn reshape<H: OutlineHost>(
    commands: &mut Commands,
    outlines: &mut Outlines<H>,
    derived: &Query<&DerivedFrom<H>>,
    layers: &Query<&Children, With<Layer>>,
    boxes: &mut Boxes,
    set: &BTreeMap<ElementId, Vec<Anchored>>,
) -> BTreeMap<ElementId, Option<Standing>> {
    let mut standings = BTreeMap::new();
    for (key, members) in batches(outlines, layers) {
        let geometry = DerivedFrom::of(&members, set);
        if shaped && derived.get(key).is_ok_and(|last| *last == geometry) {
            continue;
        }
        for (index, (entity, _, outline)) in members.iter().enumerate() {
            let shape = H::shape(combined, mesh, stretches[index].clone(), drawn_at);
            if let Ok((_, _, _, current, _)) = outlines.get_mut(*entity) {
                match current {
                    Some(mut current) => {
                        if *current != shape {
                            *current = shape;
                        }
                    }
                    None => {
                        commands.entity(*entity).insert(shape);
                    }
                }
            }
        }
        commands.entity(key).insert(geometry);
    }
    standings
}
```

## Pitfalls

- Deriving inside the Command: Undo, Redo, and Open change the source without the Command, and the derived component goes stale.
- Writing the component unconditionally each run: every reader's change detection fires every frame, and RenderEngine replaces meshes that did not change.
