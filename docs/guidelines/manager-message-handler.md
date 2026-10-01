# Message handler of a Manager

**Use when**: a Manager handles a new Command or request message. **Not when**: a system reacts to a component change, or an Engine or ResourceAccess is needed (those are called directly, never through a message).
**Exemplar**: `crates/drs-library-manager/src/lib.rs`

## Rules

- The handler is an exclusive system, `fn handle_<request>(world: &mut World, requests: &mut SystemState<MessageReader<T>>)`, added to `Update` in the `ManagerSystems` set its message belongs to (`Commands`, `Undo`, or `Redo`) with `.in_set(..)` in the plugin's `build`.
- Read first, then act: collect the messages into a `Vec` with `reader.read().cloned().collect()` (`.count()` for a message with no fields), returning on `Err(_)`, and only then take the `World` for each. _Why_: the reader borrows the World for as long as it reads.
- The work is a `pub(crate) fn(world: &mut World, ..) -> Result<Report, Error>` in a module of its own (`add_folder.rs`, `place.rs`); the handler only turns each outcome into a message with `world.write_message(..)`: the report on `Ok` where the request has one, the failure with `reason: error.to_string()` on `Err`. It never logs, panics, or stops at a failed request.
- The error is the Manager's own `thiserror` enum, wrapping the ResourceAccess errors with `#[from]`; the failure message echoes the request's fields (`path`, `name`, the `command`), so the Editor can tell which request it answers.
- The request and its answers are `Message` types in `drs-model`, next to each other; the handler's doc comment names them.

## Example

```rust
impl Plugin for LibraryManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, restore_folders)
            .add_systems(Update, handle_add_folder.in_set(ManagerSystems::Commands));
    }
}

/// Carries out every [`AddFolder`] request, answering each with [`drs_model::FolderAdded`] or
/// [`FolderRefused`].
fn handle_add_folder(world: &mut World, requests: &mut SystemState<MessageReader<AddFolder>>) {
    let requests: Vec<AddFolder> = match requests.get_mut(world) {
        Ok(mut reader) => reader.read().cloned().collect(),
        Err(_) => return,
    };
    for AddFolder { path, name } in requests {
        match add_folder(world, path.clone(), &name) {
            Ok(added) => {
                world.write_message(added);
            }
            Err(reason) => {
                world.write_message(FolderRefused { path, name, reason });
            }
        }
    }
}
```

## Pitfalls

- Writing the answer while the `MessageReader` is still borrowed does not compile, and the tempting way out, `Commands` in an ordinary system, defers the answer to the next frame and loses the `ManagerSystems` ordering with Undo and Redo.
- Returning from the loop on the first failure: the requests queued after it are dropped without an answer.
