//! Search: the Assets of every Asset Folder whose name, or path in the library, holds every word
//! of a text, ranked by whether the words begin words of the name.

use crate::fold::fold;
use drs_model::{CanonicalName, IndexedAsset};
use memchr::memmem::Finder;
use std::cmp::Ordering;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use unicode_normalization::char::canonical_combining_class;

/// The identity of the next search built; an empty search made by `Default` has the identity
/// zero, which no built one has.
static BUILT: AtomicU64 = AtomicU64::new(1);

/// The byte that ends every row of a [`Rows`] text: no name, path, or word holds it.
const END: u8 = 0;

/// One text per row, laid out back to back in one contiguous buffer, each followed by [`END`],
/// so that a substring scan runs over the whole buffer at once.
#[derive(Debug, Clone, Default)]
struct Rows {
    /// The rows, each followed by [`END`].
    text: String,
    /// Where each row starts in `text`, and after the last, where the text ends.
    starts: Vec<usize>,
}

impl Rows {
    /// Rows out of `texts`, in order.
    fn from<'a>(texts: impl IntoIterator<Item = &'a str>) -> Self {
        let mut rows = Self {
            text: String::new(),
            starts: vec![0],
        };
        for text in texts {
            rows.text.push_str(text);
            rows.text.push(char::from(END));
            rows.starts.push(rows.text.len());
        }
        rows
    }

    /// The text of `row`.
    fn row(&self, row: usize) -> &str {
        &self.text[self.starts[row]..self.starts[row + 1] - 1]
    }

    /// The row the byte at `at` lies in, looked for from `from` onwards.
    fn row_at(&self, at: usize, from: usize) -> usize {
        // The hits of a broad search are dense, so the next few rows are looked at first.
        (from..self.starts.len() - 1)
            .take(8)
            .find(|&row| self.starts[row + 1] > at)
            .unwrap_or_else(|| from + self.starts[from + 1..].partition_point(|&start| start <= at))
    }
}

/// One Asset Folder's search: its Assets in search order (by name, then by place, each compared
/// folded first and then as spelled), with each Asset's name and its path in the library (the
/// folder's Canonical Name, a `/`, and its place without the extension), both folded, laid out
/// as two contiguous texts. Built from the folder's index; it holds nothing a search does not
/// read.
#[derive(Debug, Clone, Default)]
pub struct FolderSearch {
    /// What tells this search from every other built in the session, so that a
    /// [`SearchOrder`] knows which searches it orders.
    identity: u64,
    /// The folder's Canonical Name, folded.
    folded_name: String,
    /// The folder's Canonical Name as spelled.
    name: String,
    /// Each Asset's name, folded, in search order.
    names: Rows,
    /// Each Asset's path in the library, folded, in search order.
    paths: Rows,
    /// Each Asset's name as spelled, in search order, to order two Assets of different folders
    /// whose names fold alike.
    spelled: Rows,
    /// Each Asset's position in the folder's index, in search order.
    positions: Vec<usize>,
    /// The first eight bytes of each Asset's folded name, in search order, as a number that
    /// orders as the names do as far as it goes, so that most comparisons between the matches
    /// of two folders compare two numbers.
    keys: Vec<u64>,
}

/// The first eight bytes of `name`, big-endian, padded with zeros, which no folded name holds.
fn key_of(name: &str) -> u64 {
    let mut bytes = [0; 8];
    for (byte, from) in bytes.iter_mut().zip(name.as_bytes()) {
        *byte = *from;
    }
    u64::from_be_bytes(bytes)
}

/// An Asset of a folder being built into its search.
struct Built<'a> {
    /// Its position in the index.
    position: usize,
    /// The Asset.
    asset: &'a IndexedAsset,
    /// Its name, folded.
    name: String,
    /// Its place, folded.
    place: String,
    /// How long the folded place is without its extension.
    stem: usize,
}

/// The length of `place` without its extension, if it has one.
fn without_extension(place: &str) -> usize {
    Path::new(place)
        .extension()
        .map_or(place.len(), |extension| place.len() - extension.len() - 1)
}

impl FolderSearch {
    /// Build: the search of the Asset Folder named `name` whose index holds `assets`.
    #[must_use]
    pub fn build(name: &CanonicalName, assets: &[IndexedAsset]) -> Self {
        let folded_name = fold(name.as_str());
        let mut built: Vec<Built> = assets
            .iter()
            .enumerate()
            .map(|(position, asset)| {
                let (stem, extension) = asset.place.split_at(without_extension(&asset.place));
                // Folding goes letter by letter and nothing composes across a `.`, so the folded
                // place is its folded stem and extension put together.
                let mut place = fold(stem);
                let stem = place.len();
                place.push_str(&fold(extension));
                Built {
                    position,
                    asset,
                    name: fold(&asset.name),
                    place,
                    stem,
                }
            })
            .collect();
        built.sort_unstable_by(|a, b| {
            a.name
                .cmp(&b.name)
                .then_with(|| a.asset.name.cmp(&b.asset.name))
                .then_with(|| a.place.cmp(&b.place))
                .then_with(|| a.asset.place.cmp(&b.asset.place))
        });
        let paths: Vec<String> = built
            .iter()
            .map(|built| format!("{folded_name}/{}", &built.place[..built.stem]))
            .collect();
        Self {
            identity: BUILT.fetch_add(1, AtomicOrdering::Relaxed),
            names: Rows::from(built.iter().map(|built| built.name.as_str())),
            paths: Rows::from(paths.iter().map(String::as_str)),
            spelled: Rows::from(built.iter().map(|built| built.asset.name.as_str())),
            positions: built.iter().map(|built| built.position).collect(),
            keys: built.iter().map(|built| key_of(&built.name)).collect(),
            name: name.as_str().to_owned(),
            folded_name,
        }
    }

    /// How many Assets the folder's search holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Whether the folder's search holds no Asset.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// The text an Asset of the folder is matched against for `word`.
    fn rows_for(&self, word: &Word) -> &Rows {
        if word.in_path {
            &self.paths
        } else {
            &self.names
        }
    }
}

/// One word of the text, folded as names are.
struct Word<'a> {
    /// What finds it.
    finder: Finder<'a>,
    /// Whether it is matched against the path rather than the name: it holds a `/`.
    in_path: bool,
}

/// How well an Asset matches every word of the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Rank {
    /// Every word occurs at least once where a word of the name, or of the path, begins.
    WordStarts,
    /// Some word occurs only inside a word.
    Inside,
}

/// Whether the byte at `at` of `text` begins a word: it is the start, or follows a character
/// that is neither a letter nor a digit.
fn begins_word(text: &str, at: usize) -> bool {
    match text.as_bytes()[..at].last() {
        None => true,
        Some(byte) if byte.is_ascii() => !byte.is_ascii_alphanumeric(),
        Some(_) => text[..at]
            .chars()
            .next_back()
            .is_none_or(|before| !before.is_alphanumeric()),
    }
}

/// Whether the byte at `at` of `text` lies inside a letter: it starts a combining mark, which
/// belongs with the character before it, as the acute of an `x` with an acute does.
fn inside_a_letter(text: &str, at: usize) -> bool {
    match text.as_bytes().get(at) {
        None => false,
        Some(byte) if byte.is_ascii() => false,
        Some(_) => text
            .get(at..)
            .and_then(|rest| rest.chars().next())
            .is_some_and(|next| canonical_combining_class(next) != 0),
    }
}

/// Whether `word` occurs in `text`, and if so whether at least once where a word begins; `first`
/// is where it is already known to occur first, if it is. An occurrence followed by a combining
/// mark ends inside a letter and does not count.
fn occurs(word: &Word, text: &str, first: Option<usize>) -> Option<Rank> {
    let mut at = match first {
        Some(at) => at,
        None => word.finder.find(text.as_bytes())?,
    };
    let length = word.finder.needle().len();
    let mut found = None;
    loop {
        if !inside_a_letter(text, at + length) {
            if begins_word(text, at) {
                return Some(Rank::WordStarts);
            }
            found = Some(Rank::Inside);
        }
        match word.finder.find(&text.as_bytes()[at + 1..]) {
            Some(next) => at += 1 + next,
            None => return found,
        }
    }
}

/// The rows of `folder` that match every word, by rank, each in search order; the first word is
/// the one scanned for, the others are checked within each row it occurs in.
fn search_folder(folder: &FolderSearch, words: &[Word]) -> [Vec<usize>; 2] {
    let mut ranked = [Vec::new(), Vec::new()];
    let Some((scanned, others)) = words.split_first() else {
        ranked[0] = (0..folder.len()).collect();
        return ranked;
    };
    let rows = folder.rows_for(scanned);
    let text = rows.text.as_bytes();
    let mut from = 0;
    let mut row = 0;
    while let Some(hit) = scanned.finder.find(&text[from..]) {
        let hit = from + hit;
        row = rows.row_at(hit, row);
        let start = rows.starts[row];
        from = rows.starts[row + 1];
        let mut rank = occurs(scanned, rows.row(row), Some(hit - start));
        for word in others {
            let Some(found) = rank else { break };
            rank = occurs(word, folder.rows_for(word).row(row), None).map(|other| found.max(other));
        }
        match rank {
            Some(Rank::WordStarts) => ranked[0].push(row),
            Some(Rank::Inside) => ranked[1].push(row),
            None => {}
        }
        if from >= text.len() {
            break;
        }
    }
    ranked
}

/// One Asset that matches a text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Match {
    /// Which of the folders searched it sits in, as their position in the list searched.
    pub folder: usize,
    /// Its position in that folder's index.
    pub position: usize,
}

/// What a text matches over every folder searched.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Matches {
    /// The Assets that match, in order: by rank, then by name, then by their folder's Canonical
    /// Name, then by place, each compared folded first and then as spelled.
    pub assets: Vec<Match>,
    /// How many Assets of each folder match, in the order the folders were given.
    pub counts: Vec<usize>,
}

/// The order of every Asset of several folders' searches taken together: by name folded, then
/// as spelled, then by their folder's Canonical Name folded and then as spelled, then in the
/// folder's search order. Worked out when the folders change, so that a search merges the
/// matches of every folder without comparing a name.
#[derive(Debug, Clone, Default)]
pub struct SearchOrder {
    /// The identities of the searches it orders, in the order they were given.
    searches: Vec<u64>,
    /// For each search given, where each of its rows comes among all of them.
    places: Vec<Vec<usize>>,
    /// For each place among all of them, the search, as its position in the order given, and
    /// its row there.
    rows: Vec<(usize, usize)>,
}

impl SearchOrder {
    /// The order of every Asset of `folders`.
    #[must_use]
    pub fn of(folders: &[&FolderSearch]) -> Self {
        let order = by_canonical_name(folders);
        let mut places: Vec<Vec<usize>> = folders
            .iter()
            .map(|folder| Vec::with_capacity(folder.len()))
            .collect();
        let mut rows = Vec::with_capacity(folders.iter().map(|folder| folder.len()).sum());
        // A library has a handful of folders, so the next row is the first of their heads.
        let mut next = vec![0; folders.len()];
        loop {
            let mut first: Option<(usize, usize)> = None;
            for (folder, search) in folders.iter().enumerate() {
                let row = next[folder];
                if row < search.len()
                    && first.is_none_or(|earlier| before(folders, &order, (folder, row), earlier))
                {
                    first = Some((folder, row));
                }
            }
            let Some((folder, row)) = first else {
                break;
            };
            places[folder].push(rows.len());
            rows.push((folder, row));
            next[folder] += 1;
        }
        Self {
            searches: folders.iter().map(|folder| folder.identity).collect(),
            places,
            rows,
        }
    }

    /// Whether it orders exactly `folders`, given in this order.
    fn orders(&self, folders: &[&FolderSearch]) -> bool {
        self.searches.len() == folders.len()
            && self
                .searches
                .iter()
                .zip(folders)
                .all(|(identity, folder)| *identity == folder.identity)
    }
}

/// Where each folder comes by Canonical Name, folded and then as spelled, among `folders`.
fn by_canonical_name(folders: &[&FolderSearch]) -> Vec<usize> {
    let mut by_name: Vec<usize> = (0..folders.len()).collect();
    by_name.sort_by(|&a, &b| {
        let (a, b) = (folders[a], folders[b]);
        a.folded_name
            .cmp(&b.folded_name)
            .then_with(|| a.name.cmp(&b.name))
    });
    let mut order = vec![0; folders.len()];
    for (place, &folder) in by_name.iter().enumerate() {
        order[folder] = place;
    }
    order
}

/// Whether the row `a` of one folder comes before the row `b` of another: by name folded, then
/// as spelled, then by the folders' `order` by Canonical Name.
fn before(
    folders: &[&FolderSearch],
    order: &[usize],
    a: (usize, usize),
    b: (usize, usize),
) -> bool {
    let (first, second) = (folders[a.0], folders[b.0]);
    first.keys[a.1]
        .cmp(&second.keys[b.1])
        .then_with(|| first.names.row(a.1).cmp(second.names.row(b.1)))
        .then_with(|| first.spelled.row(a.1).cmp(second.spelled.row(b.1)))
        .then_with(|| order[a.0].cmp(&order[b.0]))
        == Ordering::Less
}

/// Search: the Assets of `folders` that match `text`, in order, and how many match per folder;
/// `order` is the [`SearchOrder`] of the same folders given in the same order, and is worked out
/// afresh when it is not.
///
/// The text is split at whitespace into words, each folded as names are. An Asset matches when
/// its name holds every word without a `/`, and its path in the library every word with one;
/// it ranks first when each word occurs at least once where a word of the name, or of the path,
/// begins. A text without words matches every Asset. The order depends on nothing but the text
/// and the folders' Canonical Names and indexes, not on the order the folders are given in.
#[must_use]
pub fn search(text: &str, folders: &[&FolderSearch], order: &SearchOrder) -> Matches {
    let mut folded: Vec<(String, bool)> = text
        .split_whitespace()
        .map(|word| (fold(word), word.contains('/')))
        .collect();
    if folded
        .iter()
        .any(|(word, _)| word.as_bytes().contains(&END))
    {
        return Matches {
            assets: Vec::new(),
            counts: vec![0; folders.len()],
        };
    }
    // The longest word is the rarest, as a rule, so it is the one scanned for.
    folded.sort_by_key(|(word, _)| std::cmp::Reverse(word.len()));
    let words: Vec<Word> = folded
        .iter()
        .map(|(word, in_path)| Word {
            finder: Finder::new(word.as_bytes()),
            in_path: *in_path,
        })
        .collect();

    let ranked: Vec<[Vec<usize>; 2]> = folders
        .iter()
        .map(|folder| search_folder(folder, &words))
        .collect();
    let fresh;
    let order = if order.orders(folders) {
        order
    } else {
        fresh = SearchOrder::of(folders);
        &fresh
    };
    let mut assets = Vec::with_capacity(ranked.iter().flatten().map(Vec::len).sum());
    for rank in 0..2 {
        merge(folders, &ranked, rank, order, &mut assets);
    }
    Matches {
        assets,
        counts: ranked
            .iter()
            .map(|[first, second]| first.len() + second.len())
            .collect(),
    }
}

/// Appends to `assets` the matches of one rank of every folder, merged into `order`: each is
/// marked at its place among all the folders' Assets, and the marks are read back in order.
fn merge(
    folders: &[&FolderSearch],
    ranked: &[[Vec<usize>; 2]],
    rank: usize,
    order: &SearchOrder,
    assets: &mut Vec<Match>,
) {
    let at = |folder: usize, row: usize| Match {
        folder,
        position: folders[folder].positions[row],
    };
    let mut lists = ranked
        .iter()
        .enumerate()
        .filter(|(_, ranks)| !ranks[rank].is_empty());
    match (lists.next(), lists.next()) {
        (None, _) => return,
        (Some((only, ranks)), None) => {
            assets.extend(ranks[rank].iter().map(|&row| at(only, row)));
            return;
        }
        (Some(_), Some(_)) => {}
    }
    let mut marks = vec![0_u64; order.rows.len().div_ceil(64)];
    for (folder, ranks) in ranked.iter().enumerate() {
        for &row in &ranks[rank] {
            let place = order.places[folder][row];
            marks[place / 64] |= 1 << (place % 64);
        }
    }
    for (word, mut bits) in marks.into_iter().enumerate() {
        while bits != 0 {
            let place = word * 64 + bits.trailing_zeros() as usize;
            bits &= bits - 1;
            let (folder, row) = order.rows[place];
            assets.push(at(folder, row));
        }
    }
}
