//! Search: the Assets of every Asset Folder whose name, or path in the library, holds every word
//! of a text, ranked by whether the words begin words of the name.

use crate::fold::fold;
use drs_model::{CanonicalName, FolderKey, IndexedAsset};
use memchr::memmem::Finder;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::path::Path;
use unicode_normalization::char::canonical_combining_class;

/// The byte that ends every row of a [`Rows`] text: no name, path, or word holds it.
const END: u8 = 0;

/// How long a [`Rows`] text may grow, so that every offset into it fits in a `u32`.
const ROOM: usize = u32::MAX as usize;

/// How many rows after the last one hit are looked at one by one for the next hit before the
/// rest are searched by halves: the hits of a broad search are dense, so the next hit is most
/// often within a few rows.
const NEARBY_ROWS: usize = 8;

/// One text per row, laid out back to back in one contiguous buffer, each followed by [`END`],
/// so that a substring scan runs over the whole buffer at once.
#[derive(Debug, Clone, Default)]
struct Rows {
    /// The rows, each followed by [`END`].
    text: String,
    /// Where each row starts in `text`, and after the last, where the text ends.
    starts: Vec<u32>,
}

impl Rows {
    /// Rows out of `texts`, in order, as far as they fit in [`ROOM`] bytes.
    fn of<'a>(texts: impl IntoIterator<Item = &'a str>) -> Self {
        let mut rows = Self {
            text: String::new(),
            starts: vec![0],
        };
        for text in texts {
            let Ok(end) = u32::try_from(rows.text.len() + text.len() + 1) else {
                break;
            };
            rows.text.push_str(text);
            rows.text.push(char::from(END));
            rows.starts.push(end);
        }
        // A search is kept for as long as its folder is added, so it keeps no spare room.
        rows.text.shrink_to_fit();
        rows.starts.shrink_to_fit();
        rows
    }

    /// How many rows there are.
    fn len(&self) -> usize {
        self.starts.len() - 1
    }

    /// Where `row` starts in the text.
    fn start(&self, row: usize) -> usize {
        self.starts[row] as usize
    }

    /// The text of `row`.
    fn row(&self, row: usize) -> &str {
        &self.text[self.start(row)..self.start(row + 1) - 1]
    }

    /// The row the byte at `at` lies in, looked for from `from` onwards.
    fn row_at(&self, at: usize, from: usize) -> usize {
        (from..self.len())
            .take(NEARBY_ROWS)
            .find(|&row| self.start(row + 1) > at)
            .unwrap_or_else(|| {
                from + self.starts[from + 1..].partition_point(|&start| start as usize <= at)
            })
    }
}

/// One Asset Folder's search: its Assets in search order (by name, then by place, each compared
/// folded first and then as spelled), with each Asset's name and its path in the library (the
/// folder's Canonical Name, a `/`, and its place without the extension), both folded, laid out
/// as two contiguous texts. It holds nothing a search does not read: the names as spelled, which
/// only order two folders' Assets whose names fold alike, are read from the folder's index.
#[derive(Debug, Clone)]
struct FolderSearch {
    /// The folder's key.
    key: FolderKey,
    /// The folder's Canonical Name, folded.
    folded_name: String,
    /// The folder's Canonical Name as spelled.
    name: String,
    /// Each Asset's name, folded, in search order.
    names: Rows,
    /// Each Asset's path in the library, folded, in search order.
    paths: Rows,
    /// Each Asset's position in the folder's index, in search order.
    positions: Vec<u32>,
}

/// An Asset of a folder being built into its search.
struct Built<'a> {
    /// Its position in the index.
    position: u32,
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
    /// The search of the Asset Folder with `key`, named `name`, whose index holds `assets`. A
    /// folder whose folded names or paths run past 4 GiB is searched as far as they fit.
    fn build(key: &FolderKey, name: &CanonicalName, assets: &[IndexedAsset]) -> Self {
        let folded_name = fold(name.as_str());
        let mut built: Vec<Built> = assets
            .iter()
            .zip(0..=u32::MAX)
            .map(|(asset, position)| {
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
        let (mut names, mut paths) = (0, 0);
        let fitting = built
            .iter()
            .take_while(|built| {
                names += built.name.len() + 1;
                paths += folded_name.len() + built.stem + 2;
                names <= ROOM && paths <= ROOM
            })
            .count();
        built.truncate(fitting);
        let paths: Vec<String> = built
            .iter()
            .map(|built| format!("{folded_name}/{}", &built.place[..built.stem]))
            .collect();
        Self {
            key: key.clone(),
            names: Rows::of(built.iter().map(|built| built.name.as_str())),
            paths: Rows::of(paths.iter().map(String::as_str)),
            positions: built.iter().map(|built| built.position).collect(),
            name: name.as_str().to_owned(),
            folded_name,
        }
    }

    /// How many Assets the folder's search holds.
    fn len(&self) -> usize {
        self.positions.len()
    }

    /// How this folder's Canonical Name orders against `other`'s: folded, then as spelled.
    fn by_name(&self, other: &Self) -> Ordering {
        self.folded_name
            .cmp(&other.folded_name)
            .then_with(|| self.name.cmp(&other.name))
            .then_with(|| self.key.cmp(&other.key))
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

/// The search of every added Asset Folder: each folder's search, built from its Canonical Name
/// and index, in the order of their Canonical Names, and the order of all their Assets taken
/// together, worked out at the first search that needs it after the folders change, so that a
/// search merges the matches of every folder without comparing a name.
#[derive(Debug, Clone, Default)]
pub struct LibrarySearch {
    /// Each folder's search, by Canonical Name folded and then as spelled.
    folders: Vec<FolderSearch>,
    /// For each folder, where each of its Assets, in its search order, comes among those of
    /// every folder; `None` until a search needs it after the folders changed.
    order: Option<Vec<Vec<u32>>>,
}

impl LibrarySearch {
    /// Build: the search of the Asset Folder with `key`, named `name`, whose index holds
    /// `assets`, in place of the one it had.
    pub fn build(&mut self, key: &FolderKey, name: &CanonicalName, assets: &[IndexedAsset]) {
        self.remove(key);
        let built = FolderSearch::build(key, name, assets);
        let at = self
            .folders
            .partition_point(|folder| folder.by_name(&built) == Ordering::Less);
        self.folders.insert(at, built);
        self.order = None;
    }

    /// Drops the search of the Asset Folder with `key`, if it has one.
    pub fn remove(&mut self, key: &FolderKey) {
        let before = self.folders.len();
        self.folders.retain(|folder| folder.key != *key);
        if self.folders.len() != before {
            self.order = None;
        }
    }

    /// Search: the Assets that match `text`, in order, and how many match per folder;
    /// `indexes` gives each folder's index by key, as its search was built from it.
    ///
    /// The text is split at whitespace into words, each folded as names are. An Asset matches
    /// when its name holds every word without a `/`, and its path in the library every word with
    /// one, each on whole letters: an occurrence followed by a combining mark does not count. It
    /// ranks first when each word occurs at least once where a word of the name, or of the path,
    /// begins. A text without words matches every Asset, and no match is listed. The order
    /// depends on nothing but the text and the folders' Canonical Names and indexes.
    #[must_use]
    pub fn search<'a>(
        &mut self,
        text: &str,
        indexes: impl Fn(&FolderKey) -> Option<&'a [IndexedAsset]>,
    ) -> Matches {
        let counted = |counts: &mut dyn Iterator<Item = usize>| {
            self.folders
                .iter()
                .zip(counts)
                .map(|(folder, count)| (folder.key.clone(), count))
                .collect()
        };
        let mut folded: Vec<(String, bool)> = text
            .split_whitespace()
            .map(|word| (fold(word), word.contains('/')))
            .collect();
        if folded.is_empty() {
            return Matches {
                has_words: false,
                folders: counted(&mut self.folders.iter().map(FolderSearch::len)),
                assets: Vec::new(),
            };
        }
        if folded
            .iter()
            .any(|(word, _)| word.as_bytes().contains(&END))
        {
            return Matches {
                has_words: true,
                folders: counted(&mut std::iter::repeat(0)),
                assets: Vec::new(),
            };
        }
        // The longest word is the rarest, as a rule, so it is the one scanned for.
        folded.sort_by_key(|(word, _)| Reverse(word.len()));
        let words: Vec<Word> = folded
            .iter()
            .map(|(word, in_path)| Word {
                finder: Finder::new(word.as_bytes()),
                in_path: *in_path,
            })
            .collect();

        let ranked: Vec<[Vec<u32>; 2]> = self
            .folders
            .iter()
            .map(|folder| search_folder(folder, &words))
            .collect();
        let mut assets = Vec::with_capacity(ranked.iter().flatten().map(Vec::len).sum());
        for rank in 0..2 {
            let lists: Vec<(usize, &[u32])> = ranked
                .iter()
                .enumerate()
                .map(|(folder, ranks)| (folder, ranks[rank].as_slice()))
                .filter(|(_, rows)| !rows.is_empty())
                .collect();
            if let [(folder, rows)] = lists.as_slice() {
                let search = &self.folders[*folder];
                assets.extend(rows.iter().map(|&row| Match {
                    folder: folder_index(*folder),
                    position: search.positions[row as usize],
                }));
            } else if !lists.is_empty() {
                let order = self
                    .order
                    .get_or_insert_with(|| order_of(&self.folders, &indexes));
                merge(&self.folders, order, &lists, &mut assets);
            }
        }
        Matches {
            has_words: true,
            folders: counted(
                &mut ranked
                    .iter()
                    .map(|[first, second]| first.len() + second.len()),
            ),
            assets,
        }
    }
}

/// The position of a folder among those searched, as a [`Match`] names it.
fn folder_index(folder: usize) -> u32 {
    u32::try_from(folder).unwrap_or(u32::MAX)
}

/// For each of `folders`, where each of its Assets comes among those of every folder: by name
/// folded, then as spelled, read from the folder's index, then by the folders' Canonical Names,
/// then in the folder's search order. The folders' lists, each already in order, are merged
/// through a heap of their heads.
fn order_of<'a>(
    folders: &[FolderSearch],
    indexes: &impl Fn(&FolderKey) -> Option<&'a [IndexedAsset]>,
) -> Vec<Vec<u32>> {
    let spelled: Vec<&[IndexedAsset]> = folders
        .iter()
        .map(|folder| indexes(&folder.key).unwrap_or_default())
        .collect();
    let head = |folder: usize, row: usize| {
        let search = &folders[folder];
        let name = usize::try_from(search.positions[row])
            .ok()
            .and_then(|position| spelled[folder].get(position))
            .map_or("", |asset| asset.name.as_str());
        Reverse((search.names.row(row), name, folder, row))
    };
    let mut heads: BinaryHeap<_> = (0..folders.len())
        .filter(|&folder| folders[folder].len() > 0)
        .map(|folder| head(folder, 0))
        .collect();
    let mut places: Vec<Vec<u32>> = folders
        .iter()
        .map(|folder| Vec::with_capacity(folder.len()))
        .collect();
    let mut place: u32 = 0;
    while let Some(Reverse((_, _, folder, row))) = heads.pop() {
        places[folder].push(place);
        place = place.saturating_add(1);
        if row + 1 < folders[folder].len() {
            heads.push(head(folder, row + 1));
        }
    }
    places
}

/// Appends to `assets` the matches of one rank of several folders, each list in its folder's
/// search order, merged into `order` through a heap of their heads.
fn merge(
    folders: &[FolderSearch],
    order: &[Vec<u32>],
    lists: &[(usize, &[u32])],
    assets: &mut Vec<Match>,
) {
    let head = |list: usize, at: usize| {
        let (folder, rows) = lists[list];
        let row = rows[at] as usize;
        Reverse((order[folder][row], list, at))
    };
    let mut heads: BinaryHeap<_> = (0..lists.len()).map(|list| head(list, 0)).collect();
    while let Some(Reverse((_, list, at))) = heads.pop() {
        let (folder, rows) = lists[list];
        assets.push(Match {
            folder: folder_index(folder),
            position: folders[folder].positions[rows[at] as usize],
        });
        if at + 1 < rows.len() {
            heads.push(head(list, at + 1));
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
fn search_folder(folder: &FolderSearch, words: &[Word]) -> [Vec<u32>; 2] {
    let mut ranked = [Vec::new(), Vec::new()];
    let Some((scanned, others)) = words.split_first() else {
        return ranked;
    };
    let rows = folder.rows_for(scanned);
    let text = rows.text.as_bytes();
    let mut from = 0;
    let mut row = 0;
    while let Some(hit) = scanned.finder.find(&text[from..]) {
        let hit = from + hit;
        row = rows.row_at(hit, row);
        let start = rows.start(row);
        from = rows.start(row + 1);
        let mut rank = occurs(scanned, rows.row(row), Some(hit - start));
        for word in others {
            let Some(found) = rank else { break };
            rank = occurs(word, folder.rows_for(word).row(row), None).map(|other| found.max(other));
        }
        let Ok(matched) = u32::try_from(row) else {
            break;
        };
        match rank {
            Some(Rank::WordStarts) => ranked[0].push(matched),
            Some(Rank::Inside) => ranked[1].push(matched),
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
    /// Which folder it sits in, as the folder's position in [`Matches::folders`].
    pub folder: u32,
    /// Its position in that folder's index.
    pub position: u32,
}

/// What a text matches over every folder searched.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Matches {
    /// Whether the text holds a word. A text without one matches every Asset, and `assets`
    /// lists none of them.
    pub has_words: bool,
    /// Every folder searched, by its key, with how many of its Assets match, in the order of
    /// their Canonical Names, folded and then as spelled.
    pub folders: Vec<(FolderKey, usize)>,
    /// The Assets that match, in order: by rank, then by name, then by their folder's Canonical
    /// Name, then by place, each compared folded first and then as spelled.
    pub assets: Vec<Match>,
}
