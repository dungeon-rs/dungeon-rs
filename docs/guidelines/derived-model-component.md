# Derived model component

**Use when**: a Client or an Engine needs something computed from the model that it may not compute itself, because the computing Engine is not among its dependencies (a Wall's shape from ShapeEngine, a Project's resolution table from CatalogEngine). **Not when**: the value is saved (a serialisable component), or only the crate that computes it reads it (keep it private to that crate).
**Exemplar**: `crates/drs-authoring-manager/src/wall.rs`

## Rules

- The component lives in `drs-model` beside what it is derived from (`WallShape` in `wall.rs`, `ResolutionTable` in `resolution.rs`), documented as never saved and naming the Manager that writes it. It implements no `Serialisable`, so a Save leaves it out; `WallShape` derives no `Reflect` either, so a Remove Element snapshot leaves it out and the restored Wall is derived again.
- Exactly one Manager writes it, calling the Engine directly, in a system added `.after(ManagerSystems::Redo)` in its plugin, so every Command, Undo, Redo, and Open of the frame is in before it runs and readers in `PostUpdate` see the result that frame.
- The system is driven by change detection on the source (`Changed<Wall>`, `Changed<AssetReferences>`): it inserts through `Commands` the first time and assigns in place after.
- It writes only when the result can differ: it remembers what it last derived from in a component private to the Manager (`DerivedFrom`) or compares before assigning (`write_table`), so a reader's `Ref::is_changed` means a real change and a recolour does not rebuild a mesh.
- Readers only read it and treat its absence as not yet: RenderEngine skips the Element that frame and the Editor's hit test misses it.

## Example

```rust
/// What a Wall's shape was last derived from: its points, segments, and thickness. A change that
/// leaves them as they were, a new colour, keeps the shape.
#[derive(Component, Debug, Clone, PartialEq)]
pub(crate) struct DerivedFrom {
    /// The points the shape was derived from.
    points: Vec<Vec2>,
    /// The segments it was derived from.
    segments: Vec<Segment>,
    /// The thickness it was derived at.
    thickness: f32,
}

impl DerivedFrom {
    /// What `wall`'s shape is derived from.
    fn of(wall: &Wall) -> Self {
        Self {
            points: wall.points.clone(),
            segments: wall.segments.clone(),
            thickness: wall.thickness,
        }
    }
}

/// Derives the shape of every Wall whose points, segments, or thickness changed since the last
/// frame through the shape Engine, and sets its Element's box around its points.
#[expect(
    clippy::type_complexity,
    reason = "a Bevy query is spelled out by the components it reads and writes"
)]
pub(crate) fn derive_shapes(
    mut commands: Commands,
    mut walls: Query<
        (
            Entity,
            &Wall,
            &mut Element,
            Option<&mut WallShape>,
            Option<&DerivedFrom>,
        ),
        Changed<Wall>,
    >,
) {
    for (entity, wall, mut element, shape, derived_from) in &mut walls {
        let geometry = DerivedFrom::of(wall);
        match shape {
            Some(_) if derived_from == Some(&geometry) => continue,
            Some(mut shape) => {
                *shape = generate_walls(wall, &[]);
                commands.entity(entity).insert(geometry);
            }
            None => {
                commands
                    .entity(entity)
                    .insert((generate_walls(wall, &[]), geometry));
            }
        }
        let footprint = wall.element_box();
        if element.position != footprint.center() {
            element.position = footprint.center();
        }
        if element.size != footprint.size() {
            element.size = footprint.size();
        }
    }
}
```

## Pitfalls

- Deriving inside the Command: Undo, Redo, and Open change the source without the Command, and the derived component goes stale.
- Writing the component unconditionally each run: every reader's change detection fires every frame, and RenderEngine replaces meshes that did not change.
