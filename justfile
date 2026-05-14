# https://just.systems
# Some recipes are duplicated as they use the 'fast' profile which doesn't work under Windows due to linker limits.

# Run (almost) every check the CI will run
ci: format lint typos dependencies commits test

# Check if code is formatted correctly
format:
    cargo fmt --check
    taplo fmt --check

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
    cargo about generate -o THIRD-PARTY-LICENSES.html -m . about.hbs

# Check commit messages
commits:
    committed origin/main..HEAD

# Attempt an automated fix of various lint errors
fix:
    cargo clippy --fix --allow-dirty
    cargo fmt
    taplo fmt
    typos -w

# Run mutation tests
mutants:
    cargo mutants --all-features

# Run the editor in development mode
run:
    cargo run -p dungeonrs_editor --features=dungeonrs_editor/dev


# Install all tools used for this repo's CI and other tools
setup:
    cargo install cargo-deny
    cargo install typos-cli
    cargo install committed
    cargo install git-cliff
    cargo install cargo-nextest --locked
    cargo install mdbook
    cargo install --locked --features cli cargo-about
    cargo install taplo-cli --locked
    cargo install cargo-mutants --locked
    cargo fetch --locked
