# dungeon_rs (/ˈdʌnʒənˌrʌʃ/)
[![CI](https://github.com/dungeon-rs/dungeon-rs/actions/workflows/ci.yaml/badge.svg)](https://github.com/dungeon-rs/dungeon-rs/actions/workflows/ci.yaml)
[![Release](https://github.com/dungeon-rs/dungeon-rs/actions/workflows/release.yaml/badge.svg)](https://github.com/dungeon-rs/dungeon-rs/actions/workflows/release.yaml)
[![Bevy](https://img.shields.io/deps-rs/bevy/0.16.1)]([[https://crate.io]()](https://crates.io/crates/bevy))

DungeonRS is a small map-making editor (think DungeonDraft) written in Rust and Bevy.

# Installation
1. Clone the repository
2. `cargo build` to fetch dependencies and compile (might take a while)
3. Run the editor using
    ```bash
   BEVY_ASSET_ROOT=. cargo run -p editor --features=editor/dev
   ```

If you're not interested in the debug features, you can build for release using:
```bash
cargo build --release --locked --workspace
```
