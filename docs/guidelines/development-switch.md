# Development switch

**Use when**: adding something the editor needs only in development or for a verification agent driving it from a terminal: a screenshot, a script of input steps, a folder in place of a dialog, a forced crash, a root for its directories. **Not when**: a release build honours the variable too (`DRS_NO_DIALOGS`, `CI`): that is part of a component's contract, read without a feature gate and documented where the contract is; or the behaviour is something an Author would want, which is a setting, not a switch.
**Exemplar**: `crates/drs-editor/src/screenshot.rs`

## Rules

- The variable is `DRS_<WHAT>`, read with `std::env::var_os` (`std::env::var` when the value is matched as text), and only under `#[cfg(feature = "dev")]`: the module that reads it is declared under the attribute (`mod screenshot;`) and wired in the `#[cfg(feature = "dev")]` block of the plugin's `build`; a read inside a function the release build keeps carries the attribute itself (`choose_folder`, `directories`, `dialogs_possible`).
- Read once, while the plugins build: a `pub(crate) fn from_environment() -> Option<Self>` on a `Resource` holding what was read, inserted together with the system that uses it (`if let Some(path) = screenshot::ScreenshotPath::from_environment() {`), so an unset switch adds no system. The system reads the resource (`Res<ScreenshotPath>`), never the environment.
- A switch that stands in for a dialog is read where the dialog would open (`DRS_PICK_FOLDER` in `choose_folder`), and an empty value stands for a cancelled dialog: `(!path.is_empty()).then(|| std::path::PathBuf::from(path))`. Each dialog has a variable of its own, so one script can drive every dialog; a dialog that gains its own after sharing one keeps the shared one as its fallback (`DRS_EXPORT_FILE`, then `DRS_SAVE_FILE`, in `choose_export_file`).
- A value that does not parse forces nothing and is logged with the variable's name (`DRS_CRASH_TEST names no crash to force: {other}`); the editor starts anyway.
- A system that acts on a later frame counts in a `Local<u32>` with `saturating_add` against a named `const FRAME: u32` and returns until that frame.
- The module doc opens with `Development tooling:` and names the variable; the `dev` bullet under `## Features` in the crate's README says what the variable does, as a sentence starting `With \`DRS_<WHAT>\` set`. `just workspace` checks that the feature is documented, not that the variable is.

## Example

```rust
//! Development tooling: a screenshot of the window, saved where `DRS_SCREENSHOT` points a moment
//! after start, so the editor can be looked at from a terminal that cannot see the screen.

/// The frame the screenshot is taken on: long enough for the first layout to settle.
const FRAME: u32 = 90;

/// Where the screenshot is saved, as `DRS_SCREENSHOT` named it when the plugins built.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScreenshotPath(PathBuf);

impl ScreenshotPath {
    /// Reads `DRS_SCREENSHOT` once, while the plugins build; `None` when it is unset.
    pub(crate) fn from_environment() -> Option<Self> {
        let path = std::env::var_os("DRS_SCREENSHOT")?;
        Some(Self(PathBuf::from(path)))
    }
}

/// Takes the screenshot once, on the frame the first layout has settled.
pub(crate) fn screenshot(
    mut commands: Commands,
    path: Res<ScreenshotPath>,
    mut frames: Local<u32>,
) {
    *frames = frames.saturating_add(1);
    if *frames != FRAME {
        return;
    }
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path.0.clone()));
}
```

## Pitfalls

- Reading the variable inside the system: it then runs every frame whether the switch is set or not, and nothing but the module's gate keeps it out of a release build.
- Gating the module but not a read in a function the release build keeps: the release editor then honours a variable no README mentions.
- Treating an empty value as set where the switch stands in for a dialog: the dialog's cancel path can no longer be driven.
