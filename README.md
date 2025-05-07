# dungeon_rs (/ˈdʌnʒənˌrʌʃ/)

DungeonRS is a small map-making editor (think DungeonDraft) written in Rust and Bevy.

# Installation
1. Clone the repository
2. `cargo build` to fetch dependencies and compile (might take a while)
3. Run the editor using
    ```bash
   BEVY_ASSET_ROOT=. cargo run -p dungeonrs_editor
   ```

If you're not interested in the debug features, you can build for release using:
```bash
cargo build --release --locked --workspace --no-default-features --features='dungeonrs_editor/<windows/linux/macos>'
```
