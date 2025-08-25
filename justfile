# https://just.systems
# Some recipes are duplicated as they use the 'fast' profile which doesn't work under Windows due to linker limits.

# Run (almost) every check the CI will run
ci: format features lint typos dependencies commits test

# Check if code is formatted correctly
format:
    cargo fmt --check

# Run more advanced CI tests on feature sets
[working-directory('tools/ci')]
features:
    cargo run all

# Run unit tests
[linux, macos]
test:
    cargo nextest run --all-features --cargo-profile=fast
    cargo nextest run --all-features --cargo-profile=fast --benches
    cargo test --workspace --profile=fast --doc
[windows]
test:
    cargo nextest run --all-features
    cargo nextest run --all-features --benches
    cargo test --workspace --doc

# Run linters
[linux, macos]
lint:
    cargo check --profile=fast
    cargo clippy --all-targets --all-features -- -D warnings
[windows]
lint:
    cargo check
    cargo clippy --all-targets --all-features -- -D warnings

# Check for typos
typos:
    typos

# Check dependencies and licensing
dependencies:
    cargo machete
    cargo deny check

# Check commit messages
commits:
    committed origin/master..HEAD

[linux, macos]
[working-directory: 'docs']
docs:
    mdbook build
    cargo doc --profile=fast --locked --workspace --all-features --document-private-items --no-deps
[windows]
[working-directory: 'docs']
docs:
    mdbook build
    cargo doc --locked --workspace --all-features --document-private-items --no-deps

# Attempt an automated fix of various lint errors
fix:
    cargo clippy --fix --allow-dirty
    cargo fmt
    typos -w

# Run the editor in development mode
run:
    cargo run -p editor --features=editor/dev

# Run the CLI in development mode
cli:
    cargo run -p cli --features=cli/dev

# Install all tools used for this repo's CI and other tools
setup:
    cargo install cargo-deny
    cargo install typos-cli
    cargo install committed
    cargo install git-cliff
    cargo install cargo-nextest
    cargo install mdbook
    cargo fetch --locked
