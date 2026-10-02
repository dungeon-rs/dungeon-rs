# Headless seam test

**Use when**: writing a test at a Manager seam: a message goes in, and the World and the answers that come back are asserted. **Not when**: the behaviour is a function of one Engine or ResourceAccess (a unit test next to it), or the test drives the Editor's panels.
**Exemplar**: `crates/drs-library-manager/tests/asset_folders.rs`

## Rules

- One `editor(root)` helper builds an `App`, inserts `EditorDirectories::under(root)`, adds the real plugins (the model, the history, the ResourceAccess the Manager uses, and the Manager; in `drs-app`, every Manager), and runs one `update()` so the `Startup` systems have run. Nothing is mocked; every file the test needs lives under a `TempDir` that outlives the `App`.
- The Bevy crates are the ones the test's crate already depends on: `bevy_app` and `bevy_ecs` in a Manager, the `bevy` facade in `drs-app`.
- In `drs-app`, whose seams span every Manager, the test files share `tests/support/mod.rs` through `mod support;`: the headless `editor` of every Manager, `png` fixtures, `add_folder`, `apply`, `try_apply`, `edit`, `undo`, `redo`, the readers of the Layer's `order`, an `entity`, and the `history`, and the geometry tolerance `CLOSE` with `quadratic`, `assert_near`, and `assert_close`. Each file's `Fixture` wraps them for its own kind; a helper a second file needs moves into the module rather than being copied. The module expects its lints like any test file and allows only `dead_code`, since each file uses a different part of it.
- One helper per request sends it with `world_mut().write_message(..)`, runs one `update()`, and drains the answers from `Messages<T>` (`resource_mut::<Messages<T>>().drain()`): the refusal first, then the report. A test file that exercises refusals returns `Result<Report, Refusal>`; one that does not asserts the refusal is absent and `expect`s the report.
- The World is read through helpers that clone what they query (`folders`, `places`); no borrow of the World is held across an `update()`. Assertions compare domain values (`CanonicalName`, places, keys), never entities.
- The file opens with `#![expect(clippy::missing_panics_doc, clippy::expect_used, clippy::disallowed_methods, reason = "a test and its fixtures stop at the first thing that is not as expected")]`, which lets a test read the `History` through `World::resource`.
- Each test's doc comment is the Rule it covers, in the spec's words; the spec links to it as `path/to/file.rs::test_fn`.

## Example

```rust
/// A headless editor whose configuration and cache directories live under `root`, started once.
fn editor(root: &Path) -> App {
    let mut app = App::new();
    app.insert_resource(EditorDirectories::under(root));
    app.add_plugins((
        ModelPlugin,
        HistoryPlugin,
        LibraryAccessPlugin,
        LibraryManagerPlugin,
    ));
    app.update();
    app
}

/// Sends Add Asset Folder and returns what came back.
///
/// # Errors
///
/// The refusal, when the folder was refused.
fn add(app: &mut App, path: &Path, name: &str) -> Result<FolderAdded, FolderRefusal> {
    app.world_mut().write_message(AddFolder {
        path: path.to_path_buf(),
        name: CanonicalName(name.to_owned()),
    });
    app.update();
    let world = app.world_mut();
    if let Some(refused) = world
        .resource_mut::<Messages<FolderRefused>>()
        .drain()
        .next()
    {
        return Err(refused.reason);
    }
    let added = world
        .resource_mut::<Messages<FolderAdded>>()
        .drain()
        .next()
        .expect("an Add Asset Folder is either added or refused");
    Ok(added)
}

/// Every Asset Folder in the World, by Canonical Name.
fn folders(app: &mut App) -> Vec<AssetFolder> {
    let world = app.world_mut();
    let mut folders: Vec<AssetFolder> =
        world.query::<&AssetFolder>().iter(world).cloned().collect();
    folders.sort_by(|a, b| a.name.cmp(&b.name));
    folders
}

/// A Canonical Name that is empty once trimmed is refused, and nothing is recorded.
#[test]
fn blank_names_are_refused() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    let mut app = editor(root.path());

    let outcome = add(&mut app, &maps, "   ");

    assert_eq!(outcome, Err(FolderRefusal::BlankName));
    assert!(folders(&mut app).is_empty());
    assert!(manifests(root.path()).is_empty());
    assert!(!app.world().resource::<History>().can_undo());
}
```

## Pitfalls

- Draining the report before the refusal: a refused request writes no report, so the `expect` panics without saying why the request was refused.
- Dropping the `TempDir` before the `App`: the directories vanish under the Manager, and a later `update()` fails for a reason unrelated to the test.
- Leaving out the `update()` at the end of `editor`: the `Startup` systems (the remembered folders, the new Project) have not run when the first request goes in.
