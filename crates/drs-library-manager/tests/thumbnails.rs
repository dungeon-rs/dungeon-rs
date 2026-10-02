//! Thumbnails through the headless editor: the real plugins over fixture folders of images
//! written by the test, with the editor's own directories pointed at temporary ones, run until
//! every thumbnail state has settled.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::disallowed_methods,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

use bevy_app::App;
use bevy_ecs::message::Messages;
use drs_history::HistoryPlugin;
use drs_library_access::{LibraryAccessPlugin, ThumbnailTable};
use drs_library_manager::LibraryManagerPlugin;
use drs_model::{
    AddFolder, AssetFolder, CanonicalName, EditorDirectories, FolderAdded, FolderRefused,
    ModelPlugin, ThumbnailState, Thumbnails, ThumbnailsUnavailable,
};
use image::{DynamicImage, Rgba, RgbaImage};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// How long a test waits for thumbnails to settle before it gives up.
const PATIENCE: Duration = Duration::from_secs(120);

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
fn add(app: &mut App, path: &Path, name: &str) -> FolderAdded {
    app.world_mut().write_message(AddFolder {
        path: path.to_path_buf(),
        name: CanonicalName(name.to_owned()),
    });
    app.update();
    let world = app.world_mut();
    let refused: Vec<FolderRefused> = world
        .resource_mut::<Messages<FolderRefused>>()
        .drain()
        .collect();
    assert!(
        refused.is_empty(),
        "the fixture folder was refused: {refused:?}"
    );
    world
        .resource_mut::<Messages<FolderAdded>>()
        .drain()
        .next()
        .expect("the fixture folder is added")
}

/// The thumbnail state of every Asset of every folder, by place.
fn states(app: &mut App) -> Vec<(String, ThumbnailState)> {
    let world = app.world_mut();
    let mut states: Vec<(String, ThumbnailState)> = world
        .query::<(&AssetFolder, Option<&Thumbnails>)>()
        .iter(world)
        .flat_map(|(folder, thumbnails)| {
            folder.assets.iter().enumerate().map(move |(index, asset)| {
                let state = thumbnails
                    .and_then(|thumbnails| thumbnails.states.get(index).copied())
                    .unwrap_or_default();
                (asset.place.clone(), state)
            })
        })
        .collect();
    states.sort_by(|a, b| a.0.cmp(&b.0));
    states
}

/// The state of the Asset at `place`.
fn state(app: &mut App, place: &str) -> ThumbnailState {
    states(app)
        .into_iter()
        .find(|(known, _)| known == place)
        .map(|(_, state)| state)
        .expect("the fixture is an Asset")
}

/// Runs the editor until no Asset is pending but those at `waiting`, and returns the states.
fn settle_but(app: &mut App, waiting: &[&str]) -> Vec<(String, ThumbnailState)> {
    let start = Instant::now();
    loop {
        app.update();
        let states = states(app);
        let settled = states.iter().all(|(place, state)| {
            *state != ThumbnailState::Pending || waiting.contains(&place.as_str())
        });
        if settled {
            return states;
        }
        assert!(
            start.elapsed() < PATIENCE,
            "the thumbnails did not settle: {states:?}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Runs the editor until no Asset is pending, and returns the states.
fn settle(app: &mut App) -> Vec<(String, ThumbnailState)> {
    settle_but(app, &[])
}

/// The pixel size of every Asset's thumbnail, by place, `None` for one that is not ready.
fn sizes(states: &[(String, ThumbnailState)]) -> Vec<(&str, Option<[u32; 2]>)> {
    states
        .iter()
        .map(|(place, state)| {
            let size = match state {
                ThumbnailState::Ready(size) => Some(size.to_array()),
                ThumbnailState::Pending | ThumbnailState::Broken => None,
            };
            (place.as_str(), size)
        })
        .collect()
}

/// Whether every Asset is ready.
fn all_ready(states: &[(String, ThumbnailState)]) -> bool {
    states
        .iter()
        .all(|(_, state)| matches!(state, ThumbnailState::Ready(_)))
}

/// The thumbnail the editor serves for the Asset at `place`, read back through its own table
/// of what the `thumb://` source serves, and decoded.
fn thumbnail(app: &mut App, place: &str) -> DynamicImage {
    let world = app.world_mut();
    let folder = world
        .query::<&AssetFolder>()
        .iter(world)
        .find(|folder| folder.assets.iter().any(|asset| asset.place == place))
        .map(|folder| folder.key.clone())
        .expect("the fixture is an Asset");
    let bytes = world
        .get_resource::<ThumbnailTable>()
        .expect("the thumbnail table")
        .read(&folder, place)
        .expect("the pack is read")
        .expect("the Asset has a thumbnail");
    image::load_from_memory(&bytes).expect("the thumbnail decodes")
}

/// A fixture Asset Folder named `name` under `root`.
fn folder(root: &Path, name: &str) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(&path).expect("fixture folder");
    path
}

/// Writes an image of one colour at `place` under `folder`, in the format its extension names.
fn image_file(folder: &Path, place: &str, width: u32, height: u32, colour: [u8; 4]) -> PathBuf {
    let path = folder.join(place);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture folder");
    }
    let image = RgbaImage::from_pixel(width, height, Rgba(colour));
    let opaque = colour[3] == u8::MAX;
    let is_jpeg = path
        .extension()
        .is_some_and(|extension| extension == "jpg" || extension == "jpeg");
    if is_jpeg || opaque {
        DynamicImage::ImageRgba8(image)
            .to_rgb8()
            .save(&path)
            .expect("fixture image");
    } else {
        image.save(&path).expect("fixture image");
    }
    path
}

/// Writes a file at `place` under `folder` with an image extension and bytes that are no image.
fn not_an_image(folder: &Path, place: &str) -> PathBuf {
    let path = folder.join(place);
    fs::write(&path, b"this is not an image").expect("fixture file");
    path
}

/// The thumbnail directory under `root`.
fn thumbnail_directory(root: &Path) -> PathBuf {
    root.join("cache").join("thumbnails")
}

/// The pack and its index under `root`, as bytes.
fn pack_and_index(root: &Path) -> (Vec<u8>, Vec<u8>) {
    let directory = thumbnail_directory(root);
    (
        fs::read(directory.join("thumbnails.pack")).expect("the pack"),
        fs::read(directory.join("thumbnails.index")).expect("the index"),
    )
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

/// Every message logged, with the name of the thread it was logged on.
static LOGGED: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

/// Keeps every message logged in [`LOGGED`].
struct Capture;

impl log::Log for Capture {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        let thread = std::thread::current()
            .name()
            .unwrap_or("unnamed")
            .to_owned();
        LOGGED
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((thread, record.args().to_string()));
    }

    fn flush(&self) {}
}

/// Generating thumbnails never runs on the main thread: the frame that indexes a folder returns
/// with every thumbnail pending, later frames turn them ready, and the files were decoded on the
/// generator's own threads.
#[test]
fn generated_in_the_background() {
    // Another logger already set, as when the tests share a process, keeps this one out; the
    // thread check is then left out.
    let capturing = log::set_logger(&Capture).is_ok();
    log::set_max_level(log::LevelFilter::Debug);
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    for index in 0..8 {
        image_file(
            &maps,
            &format!("tile_{index}.png"),
            64,
            64,
            [40, 90, 160, 255],
        );
    }
    not_an_image(&maps, "not_a_tile.png");
    let mut app = editor(root.path());

    add(&mut app, &maps, "Maps");

    let first = states(&mut app);
    assert_eq!(first.len(), 9);
    assert!(
        first
            .iter()
            .all(|(_, state)| *state == ThumbnailState::Pending)
    );
    let settled = settle(&mut app);
    assert_eq!(
        settled[0],
        ("not_a_tile.png".to_owned(), ThumbnailState::Broken)
    );
    assert!(all_ready(&settled[1..]), "{settled:?}");
    if capturing {
        let logged = LOGGED.lock().unwrap_or_else(PoisonError::into_inner);
        let (thread, _) = logged
            .iter()
            .find(|(_, message)| message.contains("not_a_tile.png"))
            .expect("the file that is not an image is logged as it is decoded");
        assert!(thread.starts_with("thumbnails-"), "decoded on {thread}");
    }
}

/// A PNG, JPEG, or WebP Asset gets a thumbnail.
#[test]
fn every_image_format() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    image_file(&maps, "crate.png", 48, 48, [200, 120, 40, 255]);
    image_file(&maps, "barrel.jpg", 48, 48, [90, 60, 30, 255]);
    image_file(&maps, "tree.webp", 48, 48, [30, 140, 50, 255]);
    let mut app = editor(root.path());

    add(&mut app, &maps, "Maps");
    let states = settle(&mut app);

    assert_eq!(
        sizes(&states),
        vec![
            ("barrel.jpg", Some([48, 48])),
            ("crate.png", Some([48, 48])),
            ("tree.webp", Some([48, 48])),
        ]
    );
    for place in ["barrel.jpg", "crate.png", "tree.webp"] {
        let decoded = thumbnail(&mut app, place);
        assert_eq!((decoded.width(), decoded.height()), (48, 48));
    }
}

/// A thumbnail of an image with pixels that are not fully opaque keeps those pixels'
/// transparency.
#[test]
fn transparency_is_kept() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    let mut cut_out = RgbaImage::from_pixel(64, 64, Rgba([0, 0, 0, 0]));
    for y in 16..48 {
        for x in 16..48 {
            cut_out.put_pixel(x, y, Rgba([220, 30, 30, 255]));
        }
    }
    cut_out
        .save(maps.join("cut_out.png"))
        .expect("fixture image");
    let mut app = editor(root.path());

    add(&mut app, &maps, "Maps");
    settle(&mut app);

    let decoded = thumbnail(&mut app, "cut_out.png").to_rgba8();
    assert_eq!(
        decoded.get_pixel(2, 2).0[3],
        0,
        "the corner stays transparent"
    );
    assert_eq!(decoded.get_pixel(32, 32).0, [220, 30, 30, 255]);
}

/// A thumbnail keeps the image's proportions, is at most 128 pixels on its longer side, and is
/// never larger than the image itself.
#[test]
fn fitted_never_enlarged() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    image_file(&maps, "fence.png", 100, 400, [120, 100, 80, 255]);
    image_file(&maps, "token.png", 16, 8, [10, 10, 200, 255]);
    let mut app = editor(root.path());

    add(&mut app, &maps, "Maps");
    let states = settle(&mut app);

    assert_eq!(
        sizes(&states),
        vec![("fence.png", Some([32, 128])), ("token.png", Some([16, 8]))]
    );
    let fence = thumbnail(&mut app, "fence.png");
    assert_eq!((fence.width(), fence.height()), (32, 128));
    let token = thumbnail(&mut app, "token.png");
    assert_eq!((token.width(), token.height()), (16, 8));
}

/// An animated image's thumbnail is of its first frame.
#[test]
fn the_first_frame() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    let file = fs::File::create(maps.join("torch.png")).expect("fixture file");
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), 8, 8);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_animated(2, 0).expect("an animation");
    let mut writer = encoder.write_header().expect("a header");
    writer
        .write_image_data(&[230, 20, 20].repeat(64))
        .expect("the first frame");
    writer
        .write_image_data(&[20, 20, 230].repeat(64))
        .expect("the second frame");
    writer.finish().expect("the end of the animation");
    let mut app = editor(root.path());

    add(&mut app, &maps, "Maps");
    settle(&mut app);

    let pixel = thumbnail(&mut app, "torch.png").to_rgb8().get_pixel(4, 4).0;
    assert!(pixel[0] > 180 && pixel[2] < 80, "red, not blue: {pixel:?}");
}

/// An Asset whose file cannot be decoded as an image shows a broken placeholder, the editor keeps
/// running, and the remaining thumbnails are still generated.
#[test]
fn broken_is_a_placeholder() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    not_an_image(&maps, "a_broken.png");
    image_file(&maps, "b_crate.png", 32, 32, [200, 120, 40, 255]);
    image_file(&maps, "c_barrel.jpg", 32, 32, [90, 60, 30, 255]);
    let mut app = editor(root.path());

    add(&mut app, &maps, "Maps");
    let states = settle(&mut app);

    assert_eq!(
        states[0],
        ("a_broken.png".to_owned(), ThumbnailState::Broken)
    );
    assert!(all_ready(&states[1..]));
}

/// A file that could not be decoded is recorded as broken and not tried again until its size or
/// modification time changes; a file that could not be read is not recorded and is tried again at
/// the next start.
#[test]
fn broken_is_remembered() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    not_an_image(&maps, "broken.png");
    image_file(&maps, "crate.png", 32, 32, [200, 120, 40, 255]);
    #[cfg(unix)]
    let sealed = {
        use std::os::unix::fs::PermissionsExt;
        let sealed = image_file(&maps, "a_sealed.png", 32, 32, [10, 200, 10, 255]);
        fs::set_permissions(&sealed, fs::Permissions::from_mode(0o000)).expect("permissions");
        fs::read(&sealed).is_err().then_some(sealed)
    };
    #[cfg(not(unix))]
    let sealed: Option<PathBuf> = None;
    {
        let mut app = editor(root.path());
        add(&mut app, &maps, "Maps");
        let waiting: Vec<&str> = sealed.iter().map(|_| "a_sealed.png").collect();
        settle_but(&mut app, &waiting);
        for _ in 0..20 {
            app.update();
        }
        assert_eq!(state(&mut app, "broken.png"), ThumbnailState::Broken);
        if sealed.is_some() {
            assert_eq!(state(&mut app, "a_sealed.png"), ThumbnailState::Pending);
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(maps.join("broken.png"), fs::Permissions::from_mode(0o000))
            .expect("permissions");
        if let Some(sealed) = &sealed {
            fs::set_permissions(sealed, fs::Permissions::from_mode(0o644)).expect("permissions");
        }
    }

    let mut again = editor(root.path());

    assert_eq!(state(&mut again, "broken.png"), ThumbnailState::Broken);
    if sealed.is_some() {
        assert_eq!(state(&mut again, "a_sealed.png"), ThumbnailState::Pending);
    }
    assert!(all_ready(
        &settle_but(&mut again, &[])
            .into_iter()
            .filter(|(place, _)| place != "broken.png")
            .collect::<Vec<_>>()
    ));
    assert_eq!(state(&mut again, "broken.png"), ThumbnailState::Broken);
}

/// A thumbnail generated once is found at the next start and not generated again.
#[test]
fn kept_across_starts() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    for index in 0..4 {
        image_file(
            &maps,
            &format!("tile_{index}.png"),
            40,
            40,
            [40, 90, 160, 255],
        );
    }
    {
        let mut app = editor(root.path());
        add(&mut app, &maps, "Maps");
        assert!(all_ready(&settle(&mut app)));
    }
    let (_, index) = pack_and_index(root.path());

    let mut again = editor(root.path());

    assert!(all_ready(&states(&mut again)), "ready at the first frame");
    for _ in 0..20 {
        again.update();
    }
    assert_eq!(
        pack_and_index(root.path()).1,
        index,
        "nothing generated again"
    );
}

/// An Asset whose byte size or modification time differs from when its thumbnail was generated is
/// generated again, and the old thumbnail is no longer served for it.
#[test]
fn a_changed_file_gets_a_new_thumbnail() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    let tile = image_file(&maps, "tile.png", 40, 40, [230, 20, 20, 255]);
    {
        let mut app = editor(root.path());
        add(&mut app, &maps, "Maps");
        settle(&mut app);
    }
    image_file(&maps, "tile.png", 60, 30, [20, 20, 230, 255]);
    fs::File::options()
        .write(true)
        .open(&tile)
        .and_then(|file| file.set_modified(std::time::SystemTime::now() + Duration::from_secs(10)))
        .expect("a later modification time");

    let mut again = editor(root.path());

    assert_eq!(state(&mut again, "tile.png"), ThumbnailState::Pending);
    settle(&mut again);
    assert_eq!(
        sizes(&states(&mut again)),
        vec![("tile.png", Some([60, 30]))]
    );
    let pixel = thumbnail(&mut again, "tile.png")
        .to_rgb8()
        .get_pixel(5, 5)
        .0;
    assert!(pixel[2] > 180 && pixel[0] < 80, "blue, not red: {pixel:?}");
}

/// A remembered folder that cannot be scanned keeps its thumbnails, and they are served again
/// when it can.
#[test]
fn kept_while_a_folder_is_away() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    for index in 0..3 {
        image_file(
            &maps,
            &format!("tile_{index}.png"),
            40,
            40,
            [40, 90, 160, 255],
        );
    }
    {
        let mut app = editor(root.path());
        add(&mut app, &maps, "Maps");
        settle(&mut app);
    }
    let (_, index) = pack_and_index(root.path());
    let away = root.path().join("unplugged");
    fs::rename(&maps, &away).expect("unplug the folder");
    {
        let mut app = editor(root.path());
        assert!(states(&mut app).is_empty());
        for _ in 0..10 {
            app.update();
        }
    }
    fs::rename(&away, &maps).expect("plug the folder back in");

    let mut again = editor(root.path());

    let states = states(&mut again);
    assert_eq!(states.len(), 3);
    assert!(all_ready(&states));
    assert_eq!(pack_and_index(root.path()).1, index);
}

/// The pack and its index live in the editor's cache directory, and generating thumbnails
/// creates or changes no file inside an Asset Folder.
#[test]
fn kept_in_the_cache_directory() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    image_file(&maps, "crate.png", 32, 32, [200, 120, 40, 255]);
    image_file(&maps, "props/barrel.jpg", 32, 32, [90, 60, 30, 255]);
    not_an_image(&maps, "broken.webp");
    let before = listing(&maps);
    let mut app = editor(root.path());

    add(&mut app, &maps, "Maps");
    settle(&mut app);

    assert_eq!(listing(&maps), before);
    let (pack, index) = pack_and_index(root.path());
    assert!(!pack.is_empty());
    assert!(!index.is_empty());
}

/// Generating a thumbnail only appends to the pack and its index; no entry is removed or
/// rewritten.
#[test]
fn append_only() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    image_file(&maps, "crate.png", 32, 32, [200, 120, 40, 255]);
    let tile = image_file(&maps, "tile.png", 40, 40, [230, 20, 20, 255]);
    {
        let mut app = editor(root.path());
        add(&mut app, &maps, "Maps");
        settle(&mut app);
    }
    let (pack, index) = pack_and_index(root.path());
    image_file(&maps, "barrel.jpg", 32, 32, [90, 60, 30, 255]);
    image_file(&maps, "tile.png", 50, 40, [20, 20, 230, 255]);
    fs::File::options()
        .write(true)
        .open(&tile)
        .and_then(|file| file.set_modified(std::time::SystemTime::now() + Duration::from_secs(10)))
        .expect("a later modification time");

    let mut again = editor(root.path());
    assert!(all_ready(&settle(&mut again)));

    let (grown_pack, grown_index) = pack_and_index(root.path());
    assert!(grown_pack.len() > pack.len() && grown_pack.starts_with(&pack));
    assert!(grown_index.len() > index.len() && grown_index.starts_with(&index));
}

/// Quitting while thumbnails are generating ends the editor without waiting for the queue, and
/// every thumbnail finished before then is kept.
#[test]
fn quitting_stops_generation() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    let mut noise = RgbaImage::new(1024, 1024);
    for (x, y, pixel) in noise.enumerate_pixels_mut() {
        let [red, green, blue, _] = (x.wrapping_mul(2_654_435_761) ^ y.wrapping_mul(40_503))
            .rotate_right(7)
            .to_le_bytes();
        *pixel = Rgba([red, green, blue, 255]);
    }
    let first = maps.join("tile_0000.png");
    noise.save(&first).expect("fixture image");
    // Links to one file are as many Assets as there are places, at no cost in disk space.
    for index in 1..1000 {
        fs::hard_link(&first, maps.join(format!("tile_{index:04}.png"))).expect("fixture link");
    }
    let mut app = editor(root.path());
    add(&mut app, &maps, "Maps");
    let start = Instant::now();
    while !states(&mut app)
        .iter()
        .any(|(_, state)| matches!(state, ThumbnailState::Ready(_)))
    {
        assert!(start.elapsed() < PATIENCE, "no thumbnail was generated");
        app.update();
        std::thread::sleep(Duration::from_millis(2));
    }

    drop(app);

    let mut again = editor(root.path());
    let states = states(&mut again);
    let ready: Vec<&str> = states
        .iter()
        .filter(|(_, state)| matches!(state, ThumbnailState::Ready(_)))
        .map(|(place, _)| place.as_str())
        .collect();
    assert!(!ready.is_empty(), "what was finished is kept");
    assert!(ready.len() < states.len(), "the queue was not waited for");
    for place in ready {
        let decoded = thumbnail(&mut again, place);
        assert_eq!((decoded.width(), decoded.height()), (128, 128));
    }
}

/// An index record that is incomplete or points beyond the end of the pack is skipped when the
/// cache is opened, every complete record is served, and the skipped Asset is generated again.
#[test]
fn a_torn_record_is_skipped() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    for index in 0..3 {
        image_file(
            &maps,
            &format!("tile_{index}.png"),
            40,
            40,
            [40, 90, 160, 255],
        );
    }
    {
        let mut app = editor(root.path());
        add(&mut app, &maps, "Maps");
        settle(&mut app);
    }
    let (pack, mut index) = pack_and_index(root.path());
    // The index is a 16-byte header and 32-byte records; a record's offset is at bytes 16..24.
    assert_eq!(index.len(), 16 + 3 * 32);
    let beyond = (pack.len() as u64 + 1_000).to_le_bytes();
    index[16 + 16..16 + 24].copy_from_slice(&beyond);
    index.truncate(index.len() - 10);
    fs::write(
        thumbnail_directory(root.path()).join("thumbnails.index"),
        &index,
    )
    .expect("tear the index");

    let mut again = editor(root.path());

    let first = states(&mut again);
    let pending = first
        .iter()
        .filter(|(_, state)| *state == ThumbnailState::Pending)
        .count();
    assert_eq!(pending, 2, "the two torn records are skipped: {first:?}");
    assert!(all_ready(&settle(&mut again)));
}

/// A pack or index the editor cannot make sense of is replaced by an empty one, every thumbnail
/// is generated again, and nothing crashes.
#[test]
fn an_unreadable_cache_starts_afresh() {
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    for index in 0..3 {
        image_file(
            &maps,
            &format!("tile_{index}.png"),
            40,
            40,
            [40, 90, 160, 255],
        );
    }
    {
        let mut app = editor(root.path());
        add(&mut app, &maps, "Maps");
        settle(&mut app);
    }
    fs::write(
        thumbnail_directory(root.path()).join("thumbnails.index"),
        b"not a thumbnail index at all",
    )
    .expect("damage the index");

    let mut again = editor(root.path());

    assert!(
        states(&mut again)
            .iter()
            .all(|(_, state)| *state == ThumbnailState::Pending)
    );
    assert!(
        again
            .world_mut()
            .resource_mut::<Messages<ThumbnailsUnavailable>>()
            .drain()
            .next()
            .is_none(),
        "a damaged cache is not reported"
    );
    assert!(all_ready(&settle(&mut again)));
    let decoded = thumbnail(&mut again, "tile_0.png");
    assert_eq!((decoded.width(), decoded.height()), (40, 40));
}

/// When the pack cannot be opened or written, the Author is told once, the browser shows
/// placeholders, and the editor runs on.
#[cfg(unix)]
#[test]
fn an_unwritable_cache_is_reported() {
    use std::os::unix::fs::PermissionsExt;
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    image_file(&maps, "crate.png", 32, 32, [200, 120, 40, 255]);
    let sealed = thumbnail_directory(root.path());
    fs::create_dir_all(&sealed).expect("the thumbnail directory");
    fs::set_permissions(&sealed, fs::Permissions::from_mode(0o555)).expect("permissions");
    if fs::write(sealed.join("probe"), b"").is_ok() {
        eprintln!(
            "skipped: this process may write a sealed folder, so the check would prove nothing"
        );
        return;
    }
    let mut app = editor(root.path());

    let reported: Vec<ThumbnailsUnavailable> = app
        .world_mut()
        .resource_mut::<Messages<ThumbnailsUnavailable>>()
        .drain()
        .collect();
    assert_eq!(reported.len(), 1);
    add(&mut app, &maps, "Maps");
    for _ in 0..20 {
        app.update();
    }
    assert_eq!(state(&mut app, "crate.png"), ThumbnailState::Pending);
    assert!(
        app.world_mut()
            .resource_mut::<Messages<ThumbnailsUnavailable>>()
            .drain()
            .next()
            .is_none(),
        "told once"
    );
    fs::set_permissions(&sealed, fs::Permissions::from_mode(0o755)).expect("permissions");
}

/// What tells the test that it runs as the child of [`a_failed_write_is_reported_once`], and
/// where the fixture root is.
#[cfg(unix)]
const LIMITED_CHILD: &str = "DRS_TEST_LIMITED_ROOT";

/// When the pack cannot be written after it was opened, as on a full disk, the Author is told
/// once, the browser shows placeholders, and the editor runs on.
#[cfg(unix)]
#[test]
fn a_failed_write_is_reported_once() {
    if let Some(root) = std::env::var_os(LIMITED_CHILD) {
        failed_write_in_this_process(Path::new(&root));
        return;
    }
    let root = TempDir::new().expect("temporary root");
    let maps = folder(root.path(), "maps");
    for (index, place) in ["barrel.png", "crate.png"].into_iter().enumerate() {
        let mut noise = RgbaImage::new(128, 128);
        for (x, y, pixel) in noise.enumerate_pixels_mut() {
            let [red, green, blue, _] = (x.wrapping_mul(2_654_435_761)
                ^ y.wrapping_mul(40_503)
                ^ u32::try_from(index).expect("a small index"))
            .rotate_right(7)
            .to_le_bytes();
            *pixel = Rgba([red, green, blue, 255]);
        }
        DynamicImage::ImageRgba8(noise)
            .to_rgb8()
            .save(maps.join(place))
            .expect("fixture image");
    }
    let test = std::env::current_exe().expect("the test executable");

    // The editor runs in a child whose files may not grow past a few kilobytes: opening the
    // cache and adding the folder fit, the first thumbnail written to the pack does not. With
    // the signal ignored, the write fails as it does on a full disk instead of ending the child.
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(r#"trap '' XFSZ; ulimit -f 8; exec "$0" "$@""#)
        .arg(test)
        .args(["a_failed_write_is_reported_once", "--exact", "--nocapture"])
        .env(LIMITED_CHILD, root.path())
        .status()
        .expect("the child runs");

    assert!(status.success(), "the child failed: {status}");
}

/// The child's half of [`a_failed_write_is_reported_once`].
#[cfg(unix)]
fn failed_write_in_this_process(root: &Path) {
    let mut app = editor(root);
    let mut reported: Vec<ThumbnailsUnavailable> = app
        .world_mut()
        .resource_mut::<Messages<ThumbnailsUnavailable>>()
        .drain()
        .collect();
    assert!(reported.is_empty(), "the cache opens: {reported:?}");
    add(&mut app, &root.join("maps"), "Maps");

    let start = Instant::now();
    while reported.is_empty() {
        assert!(
            start.elapsed() < PATIENCE,
            "the failed write was not reported"
        );
        app.update();
        reported.extend(
            app.world_mut()
                .resource_mut::<Messages<ThumbnailsUnavailable>>()
                .drain(),
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    for _ in 0..20 {
        app.update();
        reported.extend(
            app.world_mut()
                .resource_mut::<Messages<ThumbnailsUnavailable>>()
                .drain(),
        );
    }

    assert_eq!(reported.len(), 1, "told once: {reported:?}");
    assert!(
        reported[0].reason.contains("thumbnails.pack"),
        "{reported:?}"
    );
    assert!(
        states(&mut app)
            .iter()
            .all(|(_, state)| *state == ThumbnailState::Pending)
    );
}
