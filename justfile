# https://just.systems
# Some recipes are duplicated as they use the 'fast' profile which doesn't work under Windows due to linker limits.

# Run every check the CI will run
check: format lint typos dependencies workspace commits test msrv docs

# Check the workspace rules: required, documented and propagated features, the architecture's dependency tables, that guideline examples come from their exemplars, and that every WGSL Shader validates
[working-directory('tools/ci')]
workspace:
    cargo run all

# Check if code is formatted correctly
format:
    cargo fmt --check
    cd tools/ci && cargo fmt --check
    taplo fmt --check

# Run the tests of the tool that checks the workspace rules
[working-directory('tools/ci')]
test-tools:
    cargo test

# Run unit tests
[linux, macos]
test: test-tools
    cargo nextest run --no-tests=pass --all-features --cargo-profile=fast
    cargo nextest run --no-tests=pass --all-features --cargo-profile=fast --benches
    cargo test --workspace --profile=fast --doc
[windows]
test: test-tools
    cargo nextest run --no-tests=pass --all-features
    cargo nextest run --no-tests=pass --all-features --benches
    cargo test --workspace --doc

# Run linters
[linux, macos]
lint:
    cargo check --profile=fast
    cargo clippy --all-targets --all-features -- -D warnings
    cd tools/ci && cargo clippy --all-targets -- -D warnings
[windows]
lint:
    cargo check
    cargo clippy --all-targets --all-features -- -D warnings
    cd tools/ci && cargo clippy --all-targets -- -D warnings

# Check for typos
typos:
    typos

# Check dependencies and licensing
dependencies:
    cargo machete
    cargo deny check
    cargo about generate -o THIRD-PARTY-LICENSES.html -m . about.hbs

# Check commit messages made since origin/master (or since the first commit while there is no remote)
commits:
    committed "$(git rev-parse --verify --quiet origin/master || git rev-list --max-parents=0 HEAD)"..HEAD

# Check that the workspace still builds on the minimum supported Rust version
msrv:
    cargo +1.96 check --workspace --all-features

# Check that the documentation builds without warnings
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features

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
    cargo run -p drs-app --features=drs-app/dev


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
    cargo install cargo-cache
    cargo install cargo-machete
    rustup toolchain install 1.96 --profile minimal
    cargo fetch --locked
