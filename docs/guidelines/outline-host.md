# Outline host

**Use when**: the editor gains a kind of Element drawn from an editable outline of points and straight or curved parts that Portals can be set into (the Wall's open line, the Room's closed outline, a Cave). **Not when**: the kind has no points to edit (a Prop, a Portal), or it is a surface painted rather than drawn from an outline (a Terrain); a new kind starts from the element-kind guideline either way.
**Exemplar**: `crates/drs-authoring-manager/src/room.rs` (a closed outline with a floor), beside `crates/drs-authoring-manager/src/wall.rs` (an open one)

## Rules

- The kind implements `OutlineHost` (the authoring Manager's `outline.rs`) in a module of its own, and nothing else in the Manager: its `KIND`, the `OutlineKind` variant errors name it by, `FEWEST_POINTS`, the reflect paths of its colours and of a part's control point, its derived `Shape`, and the reading of its component as the shape Engine's `Path` and back (`path`, `with_path`, with `closed` set for an outline that runs back to its first point). `element_box` and `malformation` forward to the component's own, so the model's check stays the only one.
- Placing (`place_outline`), every edit (`outline_edit`: position, point, control point, thickness, colours), adding and removing a point with the Portals it carries (`Reshape`, `anchor_portals_through`), removal with its Portals (`remove_with_portals`), and deriving (`derive::reshape`) are written once over the trait; a kind never copies them. A property only some kinds have is a trait constant that is `None` for the others (`FLOOR_COLOUR`), refused with the reason where it is `None`.
- Wiring a kind is one line in each dispatch: its `Placement` arm calls `place_outline`, `edit_element` tries `outline_edit::<Kind>`, `portal::host_path` chains `path_of::<Kind>` so Portals, removal, and anchors find it, `derive_shapes` gains an `Outlines<Kind>` query, its `parts_of`, its `reshape`, and `Changed<Kind>`, and `derive_snapped_point` gains a `SnapSources<Kind>` query chained in its `points_of`, `Changed<Kind>`, and `RemovedComponents<Kind>`, so its points are within reach of snapping.
- The shape Engine is only ever handed the `Path`: `combine_outlines` gives the line and, for a closed outline, the floor, and `generate_walls` strokes that line; `shape` only assembles the kind's derived component from the two. Whether the outline closes decides joins, caps, wrapping stretches, and the remap of anchors inside the Engine; the Manager reads `closed` only where it changes the `Path` itself, as `without_point` joins a removed point's parts.
- The Editor sees the kind through `handles::Outline` (an `of_<kind>` constructor, per the editor-handles guideline), lists its line in `LevelView::lines_in_order` and `shape_of` so the Portal tool snaps to it and slides along it, picks it in `topmost_at`, and has its tool snap in `snapping::what_snaps`; RenderEngine draws it by its descriptor's `drawn_as` (`StrokedPath`, `FilledOutline`).

## Example

```rust
impl OutlineHost for Room {
    const KIND: ElementKindName = ROOM;
    const OUTLINE: OutlineKind = OutlineKind::Room;
    const FEWEST_POINTS: usize = 3;
    const COLOUR: &'static str = "wall_colour";
    const FLOOR_COLOUR: Option<&'static str> = Some("floor_colour");

    type Shape = RoomShape;

    fn control_path(part: usize) -> String {
        format!("edges[{part}].control")
    }

    fn path(&self) -> Path {
        Path::of_room(self)
    }

    fn with_path(&self, path: Path) -> Self {
        Self {
            points: path.points,
            edges: path
                .controls
                .into_iter()
                .map(|control| Edge { control })
                .collect(),
            ..self.clone()
        }
    }

    fn element_box(&self) -> Rect {
        Self::element_box(self)
    }

    fn malformation(&self) -> Option<String> {
        Self::malformation(self)
    }

    fn shape(walls: WallShape, floor: FillMesh) -> RoomShape {
        RoomShape { walls, floor }
    }
}
```

## Pitfalls

- Matching on the kind inside an operation (a `Wall` arm beside a `Room` arm, or telling the kind from whether its outline closes): the next kind has to find every such place, and the copies drift apart.
- Converting a kind's component into another kind to reuse its code (a Room read as a Wall): the parts are counted, numbered, and wrapped differently, and the Engine's closed-outline rules never run.
- Building a second check of a value in `with_path` or an edit: the file, the Command, and the edit then refuse different things; `well_formed` asks the component's own `malformation`.
