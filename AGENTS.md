# DungeonRS

DungeonRS is a cross-platform map editor for tabletop RPG sessions, inspired by DungeonDraft. Built with Rust and the Bevy game engine, it provides an efficient and responsive tool for creating detailed dungeon maps with asset management, layered editing, and project serialisation capabilities.

**Current state**: early development stages

## Project structure
- [.github](./.github) - Contains workflows, issue templates and code owners files.
- [.vscode](./.vscode) - Visual Studio Code configuration files
- [assets](./assets) - Contains 'assets' (images, branding, fonts, ...) for the project. does **not** contain code
- [crates](./crates) - Contains the source code for the project, unless otherwise specified you'll search for code here.
- [docs](./docs) - Contains the source code for (end) user facing documentation, builds with [mdBook](https://rust-lang.github.io/mdBook/)
- [locales](./locales) - Contains the translations for the project. Structures like `locales/{LANGUAGE ID}/{domain}.ftl`
- [tools](./tools) - Contains tools for the project, such as build scripts, CI, etc. This is not included in distribution

Each crate has their own `README.md` file with more information.

## Languages, framework, infrastructure
- Rust - Rust 2024 edition, MSRV is defined in `rust-toolchain.toml`
- [Bevy](https://bevyengine.org/) - Game engine used for rendering and logic
- [mdBook](https://rust-lang.github.io/mdBook/) - Documentation generator
- [Fluent](https://projectfluent.org/) - used for translations (see `locales` folder)

# Relevant Commands
*note*: on Windows, each command should be ran as `just --shell pwsh.exe --shell-arg -c <command>`
**policy**: use commands below over `cargo` commands where possible (eg, `just format` over `cargo fmt`). 

- `just ci` - Run all checks a PR would run on GitHub Actions
- `just format` - Format the code
- `just lint` - Run linters
- `just setup` - Install all tools used for this repo's CI and other tools
- `just test` - Run tests
- `just typos` - Check for typos

## Code style

- **Formatting**: Uses `rustfmt` with Unix line endings and field init shorthand enabled
- **Linting**: Enforces strict clippy rules:
  - `pedantic`, `suspicious`, and `complexity` warnings enabled
  - `unsafe_code` forbidden workspace-wide
  - `missing_docs` and `missing_docs_in_private_items` warnings required
- **Documentation**: 
  - All public and private items must have documentation comments
  - Module-level docs using `#![doc = include_str!("../README.md")]` pattern
  - Comprehensive examples in doc comments where appropriate
- **Error handling**: Uses `anyhow::Error` and `thiserror` for structured error handling
- **Imports**: Organised by external crates first, then local modules
- **Naming**: 
  - Snake_case for functions, variables, and modules
  - PascalCase for types, structs, and enums
  - Module names avoid repetition (flagged by clippy)
- **Dependencies**: All workspace dependencies centralised in root `Cargo.toml`
- **imports**: Prefer `bevy::prelude` over direct imports (eg. `bevy::prelude::info` over `bevy::log::info`)
- **Spelling**: Prefer Oxford English (as defined in `typos.toml`)

# AI contribution guidelines
- Code must pass `just ci` before being pushed (see [commands](#relevant-commands))
- Provide unit tests for new code
- Request information from human if there are multiple ways to handle a problem and the desired solution was not specified
- Provide a detailed description of the problem and proposed solution
