//! Adding Asset Folders through the headless editor: the real plugins over fixture folders in
//! temporary directories, with the editor's own directories pointed at temporary ones too.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

use bevy_app::App;
use bevy_ecs::message::Messages;
use drs_history::{History, HistoryPlugin};
use drs_library_access::LibraryAccessPlugin;
use drs_library_manager::LibraryManagerPlugin;
use drs_model::{
    AddFolder, AssetFolder, AssetKind, CanonicalName, EditorDirectories, FolderAdded,
    FolderRefusal, FolderRefused, ModelPlugin,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

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

/// The places of a folder's Assets.
fn places(folder: &AssetFolder) -> Vec<&str> {
    folder
        .assets
        .iter()
        .map(|asset| asset.place.as_str())
        .collect()
}

/// The Manifest files in the configuration directory under `root`.
fn manifests(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(root.join("configuration")) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    files.sort();
    files
}

/// Writes a small file at `place` under `folder`, creating folders on the way. A scan reads
/// metadata only, so the bytes need not be an image.
fn file(folder: &Path, place: &str) -> PathBuf {
    let path = folder.join(place);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture folder");
    }
    fs::write(&path, b"fixture").expect("fixture file");
    path
}

/// A fixture Asset Folder named `name` under `root`.
fn folder(root: &Path, name: &str) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(&path).expect("fixture folder");
    path
}

/// Every entry of a folder, recursively, with its size and modification time.
fn listing(folder: &Path) -> Vec<(PathBuf, u64, std::time::SystemTime)> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(folder).expect("listable fixture") {
        let entry = entry.expect("entry");
        let metadata = entry.metadata().expect("metadata");
        entries.push((
            entry.path(),
            metadata.len(),
            metadata.modified().expect("modification time"),
        ));
        if metadata.is_dir() {
            entries.extend(listing(&entry.path()));
        }
    }
    entries.sort();
    entries
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

/// A Canonical Name already in use, compared ignoring case and Unicode normalisation, is refused
/// with the folder that holds it named.
#[test]
fn names_are_unique_on_this_device() {
    let root = TempDir::new().expect("temporary root");
    let first = folder(root.path(), "first");
    let second = folder(root.path(), "second");
    let third = folder(root.path(), "third");
    let mut app = editor(root.path());

    add(&mut app, &first, "Café Forest").expect("the first folder is added");

    let same_but_for_case = add(&mut app, &second, "café forest");
    assert_eq!(
        same_but_for_case,
        Err(FolderRefusal::NameInUse {
            name: CanonicalName("Café Forest".to_owned()),
            path: first.clone(),
        })
    );
    let same_but_for_normalisation = add(&mut app, &third, "Cafe\u{301} Forest");
    assert!(matches!(
        same_but_for_normalisation,
        Err(FolderRefusal::NameInUse { .. })
    ));
    assert_eq!(folders(&mut app).len(), 1);
    assert_eq!(manifests(root.path()).len(), 1);
}

/// A folder already added, however its path is spelled, is refused with the name it already has.
#[test]
fn the_same_folder_is_refused() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    let mut app = editor(root.path());
    add(&mut app, &maps, "Maps").expect("the folder is added");

    let with_trailing_separator = PathBuf::from(format!("{}/", maps.display()));
    assert_eq!(
        add(&mut app, &with_trailing_separator, "Again"),
        Err(FolderRefusal::AlreadyAdded {
            name: CanonicalName("Maps".to_owned()),
        })
    );
    let through_dot = maps.join(".");
    assert_eq!(
        add(&mut app, &through_dot, "Again"),
        Err(FolderRefusal::AlreadyAdded {
            name: CanonicalName("Maps".to_owned()),
        })
    );
    #[cfg(unix)]
    {
        let link = root.path().join("link-to-maps");
        std::os::unix::fs::symlink(&maps, &link).expect("symbolic link");
        assert_eq!(
            add(&mut app, &link, "Again"),
            Err(FolderRefusal::AlreadyAdded {
                name: CanonicalName("Maps".to_owned()),
            })
        );
    }
    assert_eq!(folders(&mut app).len(), 1);
    assert_eq!(manifests(root.path()).len(), 1);
}

/// A folder inside an added Asset Folder, or one containing it, is refused with the reason.
#[test]
fn nested_folders_are_refused() {
    let root = TempDir::new().expect("temporary root");
    let library = folder(root.path(), "library");
    let inner = folder(root.path(), "library/vendor");
    let deeper = folder(root.path(), "library/vendor/props");
    let mut app = editor(root.path());
    add(&mut app, &inner, "Vendor").expect("the folder is added");

    assert_eq!(
        add(&mut app, &library, "Library"),
        Err(FolderRefusal::ContainsAdded {
            name: CanonicalName("Vendor".to_owned()),
            path: inner.clone(),
        })
    );
    assert_eq!(
        add(&mut app, &deeper, "Props"),
        Err(FolderRefusal::InsideAdded {
            name: CanonicalName("Vendor".to_owned()),
            path: inner.clone(),
        })
    );
    assert_eq!(folders(&mut app).len(), 1);
    assert_eq!(manifests(root.path()).len(), 1);
}

/// A folder that does not exist or cannot be listed is refused, and nothing is recorded.
#[test]
fn unreadable_folders_are_refused() {
    let root = TempDir::new().expect("temporary root");
    let mut app = editor(root.path());

    let missing = root.path().join("nowhere");
    assert!(matches!(
        add(&mut app, &missing, "Nowhere"),
        Err(FolderRefusal::Unreadable { .. })
    ));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let sealed = folder(root.path(), "sealed");
        fs::set_permissions(&sealed, fs::Permissions::from_mode(0o000)).expect("permissions");
        let outcome = add(&mut app, &sealed, "Sealed");
        fs::set_permissions(&sealed, fs::Permissions::from_mode(0o755)).expect("permissions");
        assert!(matches!(outcome, Err(FolderRefusal::Unreadable { .. })));
    }

    assert!(folders(&mut app).is_empty());
    assert!(manifests(root.path()).is_empty());
    assert!(!app.world().resource::<History>().can_undo());
}

/// A folder whose path holds spaces, quotes, non-ASCII letters, or symbols is added and indexed.
#[test]
fn any_path_works() {
    let root = TempDir::new().expect("temporary root");
    let odd = folder(root.path(), "Tom's \"Maps\" – café 地図 #1 (v2) & more!");
    file(&odd, "s\u{e9}ance room/B\u{e4}r 🐻.png");
    let mut app = editor(root.path());

    let added = add(&mut app, &odd, "Odd").expect("the folder is added");

    assert_eq!(added.name, CanonicalName("Odd".to_owned()));
    let folders = folders(&mut app);
    assert_eq!(folders[0].path, odd);
    assert_eq!(
        places(&folders[0]),
        vec!["s\u{e9}ance room/B\u{e4}r 🐻.png"]
    );
    assert_eq!(folders[0].assets[0].name, "B\u{e4}r 🐻");
    assert_eq!(manifests(root.path()).len(), 1);
}

/// Adding a folder creates or changes no file inside it, so a read-only folder is added too.
#[test]
fn nothing_is_written_into_the_folder() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    file(&maps, "props/table.png");
    file(&maps, "readme.txt");
    let before = listing(&maps);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(maps.join("props"), fs::Permissions::from_mode(0o555))
            .expect("permissions");
        fs::set_permissions(&maps, fs::Permissions::from_mode(0o555)).expect("permissions");
    }
    let mut app = editor(root.path());

    let outcome = add(&mut app, &maps, "Maps");
    let after = listing(&maps);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&maps, fs::Permissions::from_mode(0o755)).expect("permissions");
        fs::set_permissions(maps.join("props"), fs::Permissions::from_mode(0o755))
            .expect("permissions");
    }

    assert!(outcome.is_ok(), "{outcome:?}");
    assert_eq!(before, after);
    assert_eq!(places(&folders(&mut app)[0]), vec!["props/table.png"]);
    assert_eq!(manifests(root.path()).len(), 1);
    assert!(
        root.path().join("cache").is_dir(),
        "the index cache lives in the editor's own directory"
    );
}

/// Adding a folder reads directory entries and file metadata only, so a file the editor may not
/// read is indexed like any other.
#[cfg(unix)]
#[test]
fn scanning_opens_no_file() {
    use std::os::unix::fs::PermissionsExt;
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    let sealed = file(&maps, "sealed.png");
    fs::set_permissions(&sealed, fs::Permissions::from_mode(0o000)).expect("permissions");
    let mut app = editor(root.path());

    add(&mut app, &maps, "Maps").expect("the folder is added");

    let folders = folders(&mut app);
    assert_eq!(places(&folders[0]), vec!["sealed.png"]);
    assert_eq!(folders[0].assets[0].byte_size, "fixture".len() as u64);
}

/// A file with a `png`, `webp`, `jpg`, or `jpeg` extension in any letter case is an image Asset;
/// every other file is not an Asset.
#[test]
fn images_are_assets() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    for name in [
        "a.png", "B.PNG", "c.webp", "d.JPG", "e.jpeg", "f.txt", "g.psd", "h",
    ] {
        file(&maps, name);
    }
    let mut app = editor(root.path());

    add(&mut app, &maps, "Maps").expect("the folder is added");

    let folders = folders(&mut app);
    assert_eq!(
        places(&folders[0]),
        vec!["B.PNG", "a.png", "c.webp", "d.JPG", "e.jpeg"]
    );
    assert!(
        folders[0]
            .assets
            .iter()
            .all(|asset| asset.kind == AssetKind::IMAGE)
    );
    assert_eq!(folders[0].assets[0].name, "B");
}

/// Files and folders whose name begins with a dot are not scanned.
#[test]
fn hidden_entries_are_skipped() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    file(&maps, ".hidden.png");
    file(&maps, ".cache/inside.png");
    file(&maps, "visible.png");
    let mut app = editor(root.path());

    add(&mut app, &maps, "Maps").expect("the folder is added");

    assert_eq!(places(&folders(&mut app)[0]), vec!["visible.png"]);
}

/// A symbolic link inside the folder is neither indexed nor descended into.
#[cfg(unix)]
#[test]
fn links_are_not_followed() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    file(&maps, "own.png");
    let elsewhere = folder(root.path(), "elsewhere");
    let outside = file(&elsewhere, "outside.png");
    std::os::unix::fs::symlink(&outside, maps.join("linked.png")).expect("file link");
    std::os::unix::fs::symlink(&elsewhere, maps.join("linked-folder")).expect("folder link");
    std::os::unix::fs::symlink(&maps, maps.join("loop")).expect("looping link");
    let mut app = editor(root.path());

    add(&mut app, &maps, "Maps").expect("the folder is added");

    assert_eq!(places(&folders(&mut app)[0]), vec!["own.png"]);
}

/// A file whose name is not valid Unicode is not an Asset, and the rest of the folder is indexed.
///
/// Linux only: Apple's file system refuses such a name, and Windows paths are always Unicode.
#[cfg(target_os = "linux")]
#[test]
fn unreadable_names_are_skipped() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    file(&maps, "fine.png");
    fs::write(maps.join(OsStr::from_bytes(b"bad\xff.png")), b"fixture").expect("odd file");
    let mut app = editor(root.path());

    add(&mut app, &maps, "Maps").expect("the folder is added");

    assert_eq!(places(&folders(&mut app)[0]), vec!["fine.png"]);
}

/// An Asset's name is its file name without the extension; two files with the same name in
/// different subfolders are two Assets, told apart by their place.
#[test]
fn known_by_name_and_place() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    file(&maps, "kitchen/table.png");
    file(&maps, "tavern/table.png");
    let mut app = editor(root.path());

    add(&mut app, &maps, "Maps").expect("the folder is added");

    let folders = folders(&mut app);
    let names: Vec<&str> = folders[0]
        .assets
        .iter()
        .map(|asset| asset.name.as_str())
        .collect();
    assert_eq!(names, vec!["table", "table"]);
    assert_eq!(
        places(&folders[0]),
        vec!["kitchen/table.png", "tavern/table.png"]
    );
}

/// A folder holding no Assets is added with its Manifest and shown as holding none.
#[test]
fn an_empty_folder_is_added() {
    let root = TempDir::new().expect("temporary root");
    let empty = folder(root.path(), "empty");
    let mut app = editor(root.path());

    let added = add(&mut app, &empty, "Empty").expect("the folder is added");

    let folders = folders(&mut app);
    assert_eq!(folders.len(), 1);
    assert_eq!(folders[0].name, added.name);
    assert!(folders[0].assets.is_empty());
    assert_eq!(manifests(root.path()).len(), 1);
}

/// A folder added on this device is available at the next start without being added again.
#[test]
fn remembered_across_starts() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    file(&maps, "table.png");
    let added = {
        let mut app = editor(root.path());
        add(&mut app, &maps, "Maps").expect("the folder is added")
    };

    let mut again = editor(root.path());

    let folders = folders(&mut again);
    assert_eq!(folders.len(), 1);
    assert_eq!(folders[0].name, CanonicalName("Maps".to_owned()));
    assert_eq!(folders[0].key, added.key);
    assert_eq!(folders[0].path, maps);
    assert_eq!(places(&folders[0]), vec!["table.png"]);
}

/// Assets added to or removed from a remembered folder while the editor was closed appear in or
/// disappear from the index at the next start.
#[test]
fn current_at_start() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    let old = file(&maps, "old.png");
    {
        let mut app = editor(root.path());
        add(&mut app, &maps, "Maps").expect("the folder is added");
        assert_eq!(places(&folders(&mut app)[0]), vec!["old.png"]);
    }
    fs::remove_file(old).expect("remove the old Asset");
    file(&maps, "new.png");

    let mut again = editor(root.path());

    assert_eq!(places(&folders(&mut again)[0]), vec!["new.png"]);
}
