# Element kind

**Use when**: the editor gains a kind of Element beside the Prop, the Wall, the Portal, the Terrain, and the Room (a Light). **Not when**: the new thing is a property of an existing kind (a field of its component, edited through `ElementChange`), or it is not an Element at all (a Layer, a Level, device state).
**Exemplar**: `crates/drs-model/src/wall.rs`

## Rules

- `drs-model` holds the kind: a `pub const` `ElementKindName` with its stable name, one component deriving `Component`, `Reflect` with `#[reflect(Component)]`, `Serialize`, and `Deserialize` that holds only what the Author edits, in Grid cells, registered with `register_type` in `ModelPlugin`, and a descriptor in `ElementKindRegistry::default` whose `drawn_as` says how it is drawn. The kind's box and name stay on the common `Element`, which the authoring Manager sets. _Why_ `register_type`: Remove Element snapshots and `SetField` paths go through reflection, so an unregistered component is silently lost on undo.
- The component has one `malformation()` that says why a value is not one of its kind; the placement, every edit (through the `well_formed` of the authoring Manager's `outline.rs` or `portal.rs`), and reading a file (the checked component of the serialisable-component guideline) all refuse through it, so they never disagree.
- A kind drawn from an editable outline that Portals are set into places, edits, and derives through the outline-host guideline instead of the next two rules' code of its own.
- Placing is a `Placement` arm in `messages.rs`, matched in `place_element`: the arm builds the component, checks it, and records a step that spawns it with a fresh `ElementId` through `place::spawn_on_top` and reverts through `place::take_off`. A kind that another Command brings into being (a Terrain, made by the first Paint on a Layer) records a step of its own in the kind's module of the authoring Manager that spawns it the same way, through `place::spawn_beneath` when it goes under every Element on its Layer, and reverts through `place::take_off`. A kind that shows an image Asset adds its component to `ShownAsset` in the model's `element.rs`, so counting Missing Assets, counting placeholders, and drawing find its Asset without naming the kind.
- Editing is one `ElementChange` arm per property, applied to a clone, checked on the clone, and recorded as `SetField` at the property's reflect path; `Position` on a kind whose box follows its points translates the whole component at the path `""`. A Wall's and a Room's changes are handled once, in `outline.rs`; a kind with changes of its own has `edit.rs` route them, from one arm, to the kind's module of the authoring Manager (`portal.rs`, `terrain.rs`). Several changes of one part of a kind are one arm naming the part, with an enum of the part's changes (`ElementChange::Stroke { stroke, change: StrokeChange }`). Every match over `ElementChange` names each variant, with no `_` arm, `matches!`, or if-let fallback, so a new change is routed on purpose. A change that renumbers what other Elements anchor to (adding or removing a Wall's point, removing a stroke) is a reversible command of its own.
- Drawing dispatches on the descriptor's `drawn_as`, never on the component's presence: RenderEngine's `drawn_as` picks the sprite path, the stroke path, or the floor-and-stroke path, and a kind the registry lacks falls to the placeholder sprite. What a kind is drawn and picked from, when it is computed, is a derived model component.
- The Editor picks the kind in `LevelView::topmost_at` with its own hit test against the derived component (a kind drawn as a painted surface is never picked, which `topmost_at` reads from its `drawn_as` too), keeps the kind's tool state in a module of its own (`walls.rs`, `portals.rs`, `rooms.rs`), its handles through the editor-handles guideline, and adds its tool as a `Tool` variant on `EditorState`.
- Tests: the composing seam per Rule in `crates/drs-app/tests/<kind>s.rs`, the export seam for its pixels, and the projects seam for a save and reopen plus a round trip through an editor whose registries `remove` the kind.

## Example

```rust
/// The Wall kind.
pub const WALL: ElementKindName = ElementKindName::new("wall");

/// A boundary along a line: an ordered list of two or more points in Grid cells with a segment
/// between each point and the next, drawn at a thickness in an opaque colour.
///
/// The segment from the first point to the second is the first segment, and so on: what is set
/// into a Wall is anchored by segment number, and only adding or removing a point renumbers the
/// segments.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Wall {
    /// The points, in order.
    pub points: Vec<Vec2>,
    /// One entry per segment, one fewer than there are points.
    pub segments: Vec<Segment>,
    /// How wide the Wall is drawn, in Grid cells.
    pub thickness: f32,
    /// The colour the Wall is drawn in.
    pub colour: Colour,
}

impl Wall {
    /// Why the Wall is not one, if it is not: fewer than two points, a segment count that does
    /// not match the points, a thickness not above zero, or a coordinate that is not finite.
    #[must_use]
    pub fn malformation(&self) -> Option<String> {
    }
}

impl Serialisable for Wall {
    const NAME: &'static str = "wall";
    const VERSION: u32 = 1;
    const TIER: Tier = Tier::Element;

    fn read(version: u32, data: &RawValue) -> Result<Self, SerialisationError> {
        let wall: Self = read_current_version(version, data)?;
        match wall.malformation() {
            Some(reason) => Err(SerialisationError::Malformed {
                component: Self::NAME.to_owned(),
                reason,
            }),
            None => Ok(wall),
        }
    }
}
```

## Pitfalls

- Matching on `Has<Wall>` (or any component) to choose how to draw: the next kind needs a new query in every drawing system, and an unknown kind loses its placeholder.
- Checking a value again with wording of its own in an edit or a placement: the file, the Command, and the edit then refuse different things.
- Writing the kind's derived data in the Command: an undo, a redo, or an Open bypasses the Command, and the derived data goes stale.
