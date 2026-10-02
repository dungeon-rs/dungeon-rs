# Serialisable component

**Use when**: a crate adds a component that a Project file must hold: anything a saved Project is made of. **Not when**: the component is device or session state (the `ResolutionTable`, the `Viewport`, an `AssetFolder`), which the Project lifecycle rebuilds or the device keeps.
**Exemplar**: `crates/drs-model/src/serialisation.rs`

## Rules

- `Serialisable::NAME` is the domain term in snake_case (`asset_references`), chosen once: a file written with it exists from then on, so the name never changes, whatever the type is renamed to. The `SerialisationRegistry` refuses a second component under one name in debug builds.
- `VERSION` starts at 1 and goes up by one whenever the data's shape changes; `read` then matches every version the component has had and migrates the old ones. A component that has had one version delegates to `read_only_version`, which refuses a newer version and one that never was. A changed field is a new version, never a `#[serde(rename)]` or `alias` on the current one.
- `TIER` is the entity the component belongs on (`Project`, `Level`, `Layer`, `Element`); a component another one `#[require]`s has an entry of its own, since the file holds it on its own.
- The data derives `Serialize` and `Deserialize`; a field the file gives rather than holds is `#[serde(skip)]` (the Project's name is its file's). Collections in the data are `Vec` or `BTreeMap`, so the same World writes the same bytes; the `iter_over_hash_type` lint catches iteration, not what serde walks.
- The components of one crate are listed in one `macro_rules!` invocation that both implements the trait for them and emits `register_all`, which the crate's plugin calls in `build` on the `SerialisationRegistry` it inserts (the model) or that the model already inserted. _Why_: a component implemented but not registered is skipped on Save without a word.

## Example

```rust
macro_rules! serialisable_at_version_one {
    ($($component:ty => $name:literal on $tier:expr),* $(,)?) => {
        $(
            impl Serialisable for $component {
                const NAME: &'static str = $name;
                const VERSION: u32 = 1;
                const TIER: Tier = $tier;

                fn read(version: u32, data: &RawValue) -> Result<Self, SerialisationError> {
                    read_only_version(version, data)
                }
            }
        )*

        /// Registers every component the model owns.
        pub(crate) fn register_all(registry: &mut SerialisationRegistry) {
            $(registry.register::<$component>();)*
        }
    };
}

serialisable_at_version_one! {
    Project => "project" on Tier::Project,
    Grid => "grid" on Tier::Project,
    Bounds => "bounds" on Tier::Project,
    AssetReferences => "asset_references" on Tier::Project,
    Level => "level" on Tier::Level,
    Layer => "layer" on Tier::Layer,
    Element => "element" on Tier::Element,
    Prop => "prop" on Tier::Element,
}
```

## Pitfalls

- Reading the envelope instead of its data: `read` is handed the `data` of an `Envelope` whose `version` is the other argument, so deserialising `Envelope` inside `read` fails on every file.
- Changing a field and bumping nothing: an old file then reads as malformed instead of migrating, and the Author is told the file is broken when it is the editor that moved on.
