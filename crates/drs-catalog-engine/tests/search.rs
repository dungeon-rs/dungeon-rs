//! Search over searches built from synthetic Asset Folders: what a text matches, and in what
//! order.
#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    reason = "a test and its fixtures stop at the first thing that is not as expected"
)]

use drs_catalog_engine::{FolderSearch, SearchOrder, search};
use drs_model::{AssetKind, CanonicalName, IndexedAsset};
use std::path::Path;
use std::time::{Duration, Instant};

/// An Asset Folder as the search sees it: its Canonical Name and its index.
struct Folder {
    /// The Canonical Name.
    name: CanonicalName,
    /// The index, ordered by place.
    assets: Vec<IndexedAsset>,
}

/// An image Asset at `place`, named after its file name without the extension.
fn asset(place: &str) -> IndexedAsset {
    let name = Path::new(place)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
        .to_owned();
    IndexedAsset {
        name,
        place: place.to_owned(),
        kind: AssetKind::IMAGE,
        byte_size: 1,
        modified: Duration::ZERO,
    }
}

/// The Asset Folder named `name` holding an image at each of `places`.
fn folder(name: &str, places: &[&str]) -> Folder {
    let mut assets: Vec<IndexedAsset> = places.iter().map(|place| asset(place)).collect();
    assets.sort_by(|a, b| a.place.cmp(&b.place));
    Folder {
        name: CanonicalName(name.to_owned()),
        assets,
    }
}

/// What `text` matches over `folders`, in order, each as its folder's Canonical Name and its
/// place, with how many match in each folder.
fn matches(text: &str, folders: &[&Folder]) -> (Vec<(String, String)>, Vec<usize>) {
    let built: Vec<FolderSearch> = folders
        .iter()
        .map(|folder| FolderSearch::build(&folder.name, &folder.assets))
        .collect();
    let searched: Vec<&FolderSearch> = built.iter().collect();
    let found = search(text, &searched, &SearchOrder::of(&searched));
    let assets = found
        .assets
        .iter()
        .map(|found| {
            let folder = folders[found.folder];
            (
                folder.name.0.clone(),
                folder.assets[found.position].place.clone(),
            )
        })
        .collect();
    (assets, found.counts)
}

/// The places `text` matches in `folder`, in order.
fn places(text: &str, folder: &Folder) -> Vec<String> {
    matches(text, &[folder])
        .0
        .into_iter()
        .map(|(_, place)| place)
        .collect()
}

/// The typed text is split at whitespace into words; whitespace before, between, and after
/// them is ignored.
#[test]
fn words_of_the_text() {
    let props = folder(
        "Props",
        &["Table_Oak_Round.png", "Table_Pine.png", "Barrel.png"],
    );

    assert_eq!(places("  barrel \t", &props), places("barrel", &props));
    assert_eq!(places("barrel", &props), vec!["Barrel.png"]);
    assert_eq!(
        places(" oak \t\n table ", &props),
        vec!["Table_Oak_Round.png"]
    );
    assert_eq!(
        places("oak table", &props),
        places("oak\u{3000}table", &props)
    );
}

/// An Asset matches a word without `/` when its name contains the word, both compared
/// Unicode-normalised and fully case-folded, so that `barrel` finds `Old_BARREL` and `STRASSE`
/// finds `Straße`.
#[test]
fn matched_by_part_of_a_name() {
    let props = folder(
        "Props",
        &[
            "Old_BARREL.png",
            "barrels/Crate.png",
            "Straße.png",
            "Table.png",
        ],
    );

    assert_eq!(places("barrel", &props), vec!["Old_BARREL.png"]);
    assert_eq!(places("RRE", &props), vec!["Old_BARREL.png"]);
    assert_eq!(places("STRASSE", &props), vec!["Straße.png"]);
    assert_eq!(places("straße", &props), vec!["Straße.png"]);
    assert!(
        places("png", &props).is_empty(),
        "the extension is not part of the name"
    );
}

/// A word spelled with composed accents matches a name spelled with decomposed ones and the
/// reverse; an unaccented letter does not match an accented one.
#[test]
fn accents_however_stored() {
    // `é` and `É` are written composed here; `e\u{301}` is an `e` and a combining acute.
    let props = folder(
        "Props",
        &["Café_Sign.png", "Cafe\u{301}_Table.png", "Cafeteria.png"],
    );

    let accented = vec!["Café_Sign.png", "Cafe\u{301}_Table.png"];
    assert_eq!(places("café", &props), accented);
    assert_eq!(places("cafe\u{301}", &props), accented);
    assert_eq!(places("CAFÉ", &props), accented);
    assert_eq!(places("cafe", &props), vec!["Cafeteria.png"]);
}

/// With several words, an Asset matches only when it matches each of them, in any order; two
/// words may match overlapping parts of the name.
#[test]
fn every_word_must_match() {
    let props = folder(
        "Props",
        &[
            "Table_Oak_Round.png",
            "Table_Pine.png",
            "Oak_Chair.png",
            "Tablet.png",
        ],
    );

    assert_eq!(places("table oak", &props), vec!["Table_Oak_Round.png"]);
    assert_eq!(places("oak table", &props), vec!["Table_Oak_Round.png"]);
    assert_eq!(
        places("tab able", &props),
        vec!["Table_Oak_Round.png", "Table_Pine.png", "Tablet.png"]
    );
    assert!(places("table chair", &props).is_empty());
}

/// A word that contains `/` is matched, as a name is, against the Asset's path in the library
/// instead of its name: its folder's Canonical Name, a `/`, and its place without the
/// extension.
#[test]
fn a_word_with_a_slash_matches_the_path() {
    let library = folder(
        "Forgotten Adventures",
        &[
            "Furniture/Table_Oak.png",
            "Furniture/Chair.png",
            "Props/Table_Small.png",
        ],
    );

    assert_eq!(
        places("furniture/table", &library),
        vec!["Furniture/Table_Oak.png"]
    );
    assert_eq!(
        places("adventures/furniture", &library),
        vec!["Furniture/Chair.png", "Furniture/Table_Oak.png"]
    );
    assert_eq!(
        places("adventures/ table", &library),
        vec!["Furniture/Table_Oak.png", "Props/Table_Small.png"]
    );
    assert_eq!(places("oak/", &library), Vec::<String>::new());
    assert!(
        places("furniture/table_oak.png", &library).is_empty(),
        "the extension is not part of the path"
    );
    assert!(
        places("furniture", &library).is_empty(),
        "a word without a slash is matched against the name only"
    );
}

/// A text with no words matches every Asset of every added Asset Folder.
#[test]
fn an_empty_text_matches_everything() {
    let props = folder("Props", &["Barrel.png", "Table.png"]);
    let maps = folder("Maps", &["Cave.png"]);

    for text in ["", "   \t "] {
        let (found, counts) = matches(text, &[&props, &maps]);
        assert_eq!(found.len(), 3, "{text:?}");
        assert_eq!(counts, vec![2, 1]);
    }
}

/// An Asset in which every word occurs at least once where a word of the name (or of the path,
/// for a word with `/`) begins ranks above an Asset in which some word occurs only inside a
/// word; a word of the name begins at its start or after a character that is neither a letter
/// nor a digit.
#[test]
fn word_starts_rank_first() {
    let props = folder(
        "Props",
        &[
            "Flowerbed.png",
            "Bed_Double.png",
            "Room2bed.png",
            "Old-Bed.png",
            "Bedbug_Flowerbed.png",
            "Flowerbed_Bedside.png",
        ],
    );

    assert_eq!(
        places("bed", &props),
        vec![
            "Bed_Double.png",
            "Bedbug_Flowerbed.png",
            "Flowerbed_Bedside.png",
            "Old-Bed.png",
            "Flowerbed.png",
            "Room2bed.png",
        ]
    );
    assert_eq!(
        places("flower bed", &props),
        vec![
            "Bedbug_Flowerbed.png",
            "Flowerbed_Bedside.png",
            "Flowerbed.png"
        ]
    );

    let library = folder(
        "Library",
        &["old_furniture/Bed.png", "oldfurniture/Bed.png"],
    );
    assert_eq!(
        places("furniture/bed", &library),
        vec!["old_furniture/Bed.png", "oldfurniture/Bed.png"]
    );
}

/// Matches are ordered by rank, then by name, then by their folder's Canonical Name, then by
/// place, each compared Unicode-normalised and case-folded first and as spelled to break a tie;
/// the order depends on nothing but the text and the folders' Canonical Names and indexes, so it
/// is the same on every device.
#[test]
fn a_deterministic_order() {
    let vendor = folder(
        "Vendor",
        &[
            "props/barrel.png",
            "Barrel.png",
            "b/Barrel.png",
            "Old_Barrel.png",
        ],
    );
    let maps = folder("maps", &["Barrel.png", "Barrels.png"]);

    let (found, counts) = matches("barrel", &[&vendor, &maps]);
    let expected: Vec<(String, String)> = [
        ("maps", "Barrel.png"),
        ("Vendor", "b/Barrel.png"),
        ("Vendor", "Barrel.png"),
        ("Vendor", "props/barrel.png"),
        ("maps", "Barrels.png"),
        ("Vendor", "Old_Barrel.png"),
    ]
    .iter()
    .map(|(folder, place)| ((*folder).to_owned(), (*place).to_owned()))
    .collect();
    assert_eq!(found, expected);
    assert_eq!(counts, vec![4, 2]);

    let (reversed, counts) = matches("barrel", &[&maps, &vendor]);
    assert_eq!(reversed, expected);
    assert_eq!(counts, vec![2, 4]);
}

/// The words vendor-shaped names are made of.
const WORDS: [&str; 24] = [
    "table",
    "oak",
    "barrel",
    "door",
    "chair",
    "bed",
    "flowerbed",
    "crate",
    "wall",
    "stone",
    "torch",
    "rug",
    "chest",
    "shelf",
    "bench",
    "pine",
    "round",
    "iron",
    "rope",
    "lamp",
    "skull",
    "cart",
    "well",
    "tree",
];

/// 400,000 vendor-shaped Assets in two Asset Folders of 200,000 each:
/// `<word>_pack_NN/<Word>_<Word>_NNNN.png`.
fn vendor_library() -> [Folder; 2] {
    let make = |name: &str, seed: usize| {
        let mut assets: Vec<IndexedAsset> = (0..200_000)
            .map(|index: usize| {
                let n = index.wrapping_mul(2_654_435_761).wrapping_add(seed);
                let pack = WORDS[n % WORDS.len()];
                let first = WORDS[(n / 7) % WORDS.len()];
                let second = WORDS[(n / 131) % WORDS.len()];
                let capital = |word: &str| {
                    let mut letters = word.chars();
                    letters
                        .next()
                        .map(|first| first.to_uppercase().chain(letters).collect::<String>())
                        .unwrap_or_default()
                };
                asset(&format!(
                    "{pack}_pack_{:02}/{}_{}_{:04}.png",
                    index % 40,
                    capital(first),
                    capital(second),
                    index % 10_000
                ))
            })
            .collect();
        assets.sort_by(|a, b| a.place.cmp(&b.place));
        assets.dedup_by(|a, b| a.place == b.place);
        Folder {
            name: CanonicalName(name.to_owned()),
            assets,
        }
    };
    [make("Forgotten Adventures", 0), make("Tom Cartos", 17)]
}

/// The fastest of `runs` runs of `work`, and what the last run gave.
fn fastest<T>(runs: usize, mut work: impl FnMut() -> T) -> (Duration, T) {
    let mut best = Duration::MAX;
    let mut last = None;
    for _ in 0..runs {
        let start = Instant::now();
        let outcome = work();
        best = best.min(start.elapsed());
        last = Some(outcome);
    }
    (best, last.expect("at least one run"))
}

/// Over 400,000 Assets in the test build, building the search takes under 1 second and
/// answering a text takes under 50 milliseconds.
#[test]
fn answered_within_a_keystroke() {
    let library = vendor_library();
    let total: usize = library.iter().map(|folder| folder.assets.len()).sum();
    assert!(total >= 399_000, "{total} Assets");

    let (built, folders) = fastest(3, || {
        library
            .iter()
            .map(|folder| FolderSearch::build(&folder.name, &folder.assets))
            .collect::<Vec<_>>()
    });
    let searched: Vec<&FolderSearch> = folders.iter().collect();
    let (ordered, order) = fastest(3, || SearchOrder::of(&searched));
    eprintln!("built the search of {total} Assets in {built:?} and ordered it in {ordered:?}");
    assert!(
        built + ordered < Duration::from_secs(1),
        "built in {built:?} and ordered in {ordered:?}"
    );

    let narrow = library[0].assets[1234].name.to_lowercase();
    for text in [
        narrow.as_str(),
        "a",
        "oak table",
        "adventures/well_pack chair",
    ] {
        let (answered, found) = fastest(5, || search(text, &searched, &order));
        eprintln!(
            "{text:?} matched {} Assets in {answered:?}",
            found.assets.len()
        );
        assert!(!found.assets.is_empty(), "{text:?} matches something");
        assert!(
            answered < Duration::from_millis(50),
            "{text:?} answered in {answered:?}"
        );
    }
}
