# Contributing to DungeonRS

Setting up a local development environment requires having [Rust](https://rust-lang.org) installed.
DungeonRS provides a `rust-toolchain.toml` file, so after cloning the repository, you can simply run:
```bash
rustup show
```
This will cause `rustup` to automatically install the version and targets required.

In the meanwhile, you can install the helper tools DungeonRS uses.
We provide a `justfile` which is essentially a `Makefile` (you can read more about them [here](https://just.systems)).

To install `just` run:
```bash
cargo install just
```
Then, you can use `just` to install all other dependencies using:
```bash
just setup
```

If you want to see which `just` commands are available, you can run `just --list`.
Some handy commands you'll probably use often are:
- linting: `just lint`
- testing: `just test`
- run: `just run`

If you are about to make a commit and want to run (almost) the same checks as the CI will run on your PR, you can simply
run `just` (which will automatically run `just ci`).

### Improving compile times
By default DungeonRS has sensible defaults that work on most platforms to provide reasonable compile times.
However, Rust isn't known for it's fast compile times, so depending on your platform there's a few tricks you can try to
improve the compile times.
Check out `.cargo/config_fast_build.toml` for ways to do so.

## AI assisted contribution
AI contributions are welcome, but they are subjected to the same rules as regular contributions.
Do not make low-effort slop contributions, as they will be rejected without comment.

Direct your AI to [AGENTS.md](./AGENTS.md) for instructions on how to contribute.

### Using Claude Code on Windows
If you are using [Claude Code](https://www.anthropic.com/claude-code) on Windows you'll probably run into the issue that
Windows doesn't support symlinks (by default), which is what we use to link the `CLAUDE.md` file to `AGENTS.md`.

To resolve this issue, when (re)installing Git for Windows, make sure to check the option to enable symlinks.
You'll also need to enable the developer mode in Windows settings.

For detailed information see [this StackOverflow answer](https://stackoverflow.com/questions/5917249/git-symbolic-links-in-windows/59761201#59761201).
