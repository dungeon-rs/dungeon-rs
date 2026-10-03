# Component migration

**Use when**: the data of a serialisable component changes shape, so files written before the change hold an older version of it (a Terrain's strokes gaining whether they erase). **Not when**: the component is new (the serialisable-component guideline, at version 1), or only its meaning in the editor changes and the data it writes stays the same.
**Exemplar**: `crates/drs-model/src/terrain.rs`

## Rules

- `VERSION` goes up by one, and the shape every older version wrote is kept beside the type as private structs named for it (`TerrainVersionOne`, with `StrokeVersionOne` for a part whose shape changed too) that derive only `Deserialize` and hold the fields exactly as that version wrote them. The current type gains no `#[serde(default)]`, `rename`, or `alias` to read old data.
- Each older version becomes the current type through `From<TypeVersionN> for Type`, which fills what the old data lacks with what it meant then (every stroke painting); the migration lives in the `From` alone.
- `read` is a `match version` with one arm per older version, `n => parse_version::<Self, TypeVersionN>(data)?.into()`, and a last arm `_ => read_current_version(version, data)?` that reads the current version and refuses a newer one and one that never was. A checked component then runs its own check on whatever came out, so an old file is refused for the same reasons, in the same words, as a new one. Saving always writes the current version.
- `parse_version` names the component in `SerialisationError::Malformed`; no arm maps a serde error itself.
- Tests, at the projects seam: one test per older version (`an_older_terrain_opens`) takes a file saved now, rewrites the component's envelope in the JSON to that version and its shape, opens it, compares the World with what the migration should give, and saves it again at the current version; the malformed-file test opens each case at every version; and `a_newer_file_is_refused` covers the version after the current one.

## Example

```rust
/// A Terrain as version one of its data holds it, before strokes could erase.
#[derive(Deserialize)]
struct TerrainVersionOne {
    /// The row of its image.
    image: AssetReferenceRow,
    /// The strokes, every one painting.
    strokes: Vec<StrokeVersionOne>,
}

/// A stroke as version one of a Terrain's data holds it: a path and Brush settings.
#[derive(Deserialize)]
struct StrokeVersionOne {
    /// The path.
    points: Vec<Vec2>,
    /// The settings it was laid with.
    brush: BrushSettings,
}

impl From<TerrainVersionOne> for Terrain {
    /// The same Terrain with every stroke painting.
    fn from(old: TerrainVersionOne) -> Self {
        Self {
            image: old.image,
            strokes: old
                .strokes
                .into_iter()
                .map(|stroke| Stroke {
                    points: stroke.points,
                    brush: stroke.brush,
                    erase: false,
                })
                .collect(),
        }
    }
}

impl Serialisable for Terrain {
    const NAME: &'static str = "terrain";
    const VERSION: u32 = 2;
    const TIER: Tier = Tier::Element;

    fn read(version: u32, data: &RawValue) -> Result<Self, SerialisationError> {
        let terrain: Self = match version {
            1 => parse_version::<Self, TerrainVersionOne>(data)?.into(),
            _ => read_current_version(version, data)?,
        };
        match terrain.malformation() {
            Some(reason) => Err(SerialisationError::Malformed {
                component: Self::NAME.to_owned(),
                reason,
            }),
            None => Ok(terrain),
        }
    }
}
```

## Pitfalls

- Making the new field `#[serde(default)]` on the current type instead: the file's version then says nothing about its shape, and the next change cannot tell an old file from a damaged one.
- Running the component's check only on the current version's arm: a malformed old file opens, and the editor holds what it would refuse to make.
- Migrating with a catch-all arm for every version below the current one: a version that never existed (zero) reads as the oldest instead of being refused.
