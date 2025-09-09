# dungeon_rs (/ˈdʌnʒənˌrʌʃ/)
[![CI](https://github.com/dungeon-rs/dungeon-rs/actions/workflows/ci.yaml/badge.svg)](https://github.com/dungeon-rs/dungeon-rs/actions/workflows/ci.yaml)
[![Release](https://github.com/dungeon-rs/dungeon-rs/actions/workflows/release.yaml/badge.svg)](https://github.com/dungeon-rs/dungeon-rs/actions/workflows/release.yaml)
[![Bevy](https://img.shields.io/deps-rs/bevy/0.16.1)]([[https://crate.io]()](https://crates.io/crates/bevy))

DungeonRS is a cross-platform map editor for tabletop RPG sessions, inspired by DungeonDraft. Built with Rust and the Bevy game engine, it provides an efficient and responsive tool for creating detailed dungeon maps with asset management, layered editing, and project serialisation capabilities.

## Features

- **Visual Map Editor**: Intuitive interface for creating and editing dungeon maps
- **Asset Management**: Organised system for managing tiles, textures, and map elements
- **Layer-based Editing**: Multiple layers for organising different map components
- **Cross-platform**: Native support for Windows, macOS, and Linux
- **Modular Architecture**: Plugin-based system with separate crates for different functionality

## Requirements

- Rust 1.89.0 or later (managed via `rust-toolchain.toml`)
- System graphics drivers supporting OpenGL or Vulkan
- Developer mode enabled (Windows only, see [CONTRIBUTING](./CONTRIBUTING.md#using-claude-code-on-windows)).

## Installation

### Development Build

1. Clone the repository:
   ```bash
   git clone https://github.com/dungeon-rs/dungeon-rs.git
   cd dungeon-rs
   ```

2. Install Rust toolchain:
   ```bash
   rustup show
   ```

3. Build and run the development editor:
   ```bash
   BEVY_ASSET_ROOT=. cargo run -p drs-editor -F drs-editor/dev
   ```

### Release Build

For production use without debug features:
```bash
cargo build --release --locked --workspace
```

### Using Just

The project includes a `justfile` for common development tasks:
```bash
cargo install just
just setup    # Install development dependencies
just run      # Run the development editor
just test     # Run test suite
just lint     # Run linting checks
just ci       # Run all checks a PR would run on GitHub Actions
```

## Project Structure

DungeonRS uses a workspace-based architecture with the following crates:

- **`drs-assets`**: Asset loading, management, and scripting system
- **`drs-cli`**: Command-line interface for project management and asset management
- **`drs-config`**: Configuration management and settings
- **`drs-data`**: Core data structures for projects, levels, and layers
- **`drs-editor`**: Main application binary and UI integration
- **`i18n`**: Internationalisation support
- **`io`**: Project serialisation and file operations
- **`logging`**: Logging and error handling
- **`serialization`**: Serialisation and deserialization of project data
- **`ui`**: User interface components and layout management
- **`utils`**: Shared utilities and helper functions

## Configuration

DungeonRS supports custom configuration files. Specify a config file when launching:
```bash
cargo run -p drs-editor -- --config-file path/to/config.toml
```

Configuration options include logging levels, language settings, and editor preferences.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING](CONTRIBUTING.md) for development setup instructions and guidelines.

## License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file.
