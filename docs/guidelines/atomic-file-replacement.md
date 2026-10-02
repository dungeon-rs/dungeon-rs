# Atomic file replacement

**Use when**: a ResourceAccess writes a file that must never be found half-written: a saved Project, an Export, a Manifest or an index cache (`drs-library-access` writes both through one `write_atomically`). **Not when**: the file is read, not written, or is appended to (a log).
**Exemplar**: `crates/drs-project-access/src/lib.rs`

## Rules

- The temporary file comes from one private `fn` (`file_beside`, `partial_file`) that builds it with `tempfile::Builder`: the target's file name and a `.` as `prefix`, `.part` as `suffix`, created with `tempfile_in` in the target's parent (`Path::new(".")` when the path has none), and under `#[cfg(unix)]` with `permissions(std::fs::Permissions::from_mode(0o666))`. _Why_: `tempfile` creates files `0o600`, private to the owner, and the rename keeps that mode; `0o666` is what the umask narrows for every other new file. The `disallowed_methods` lint refuses the `NamedTempFile::new*` constructors, which cannot set the mode.
- The bytes go to the temporary file only; nothing opens, truncates, or removes the target. Then `sync_all` on the file, then `persist(path)` renames it over the target, in that order.
- A writer that hands its handle to something else (the PNG encoder) splits the `NamedTempFile` with `into_parts()`, keeps the `TempPath` in an `Option<TempPath>` field, and flushes once the handle is closed with `File::open(&partial).and_then(|file| file.sync_all())` before `persist`.
- Every step maps its error to the crate's `Io { action, path, source }` with the target's path, never the temporary one, and one of the actions `"create a file beside"`, `"write"`, `"flush"`, `"rename"`; a `persist` failure maps `error.error`, the `std::io::Error` inside the `PersistError`.
- The temporary file is removed by dropping the `NamedTempFile` or `TempPath`, which every `?` does; a writer that lives across calls sets its `Option<TempPath>` to `None` in `Drop` and on every failed finish (`spend`). Nothing calls `remove_file`.

## Example

```rust
pub fn write_project(path: &Path, snapshot: &ProjectSnapshot) -> Result<(), ProjectAccessError> {
    let file = ProjectFile::from(snapshot);
    let mut text =
        serde_json::to_string_pretty(&file).map_err(|error| ProjectAccessError::NotAProject {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;
    text.push('\n');
    let io = |action: &'static str| {
        move |source: std::io::Error| ProjectAccessError::Io {
            action,
            path: path.to_path_buf(),
            source,
        }
    };
    let mut temporary = file_beside(path).map_err(io("create a file beside"))?;
    temporary.write_all(text.as_bytes()).map_err(io("write"))?;
    temporary.as_file().sync_all().map_err(io("flush"))?;
    temporary
        .persist(path)
        .map_err(|error| io("rename")(error.error))?;
    Ok(())
}

/// The temporary file a Project is written to until it is complete: the path's file name, a
/// random infix, and a `.part` suffix, in the same directory so the final rename never crosses a
/// file system; removed when dropped unless it is persisted.
fn file_beside(path: &Path) -> std::io::Result<NamedTempFile> {
    let name = path.file_name().map_or_else(
        || std::ffi::OsString::from("project"),
        std::ffi::OsStr::to_os_string,
    );
    let beside = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut prefix = name;
    prefix.push(".");
    let mut builder = Builder::new();
    builder.prefix(&prefix).suffix(".part");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        builder.permissions(std::fs::Permissions::from_mode(0o666));
    }
    builder.tempfile_in(beside)
}
```

## Pitfalls

- Creating the temporary file in `std::env::temp_dir()`: the rename crosses a file system, where it fails or copies instead of replacing in one step.
- Persisting without `sync_all`: the rename can reach the disk before the bytes, and a crash leaves a short file under the target's name, which is exactly what the temporary file was for.
- Mapping the whole `PersistError`: it carries the `NamedTempFile` too, so the crate's error does not take it; `error.error` is the `std::io::Error`.
