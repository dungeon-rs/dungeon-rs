# drs-catalog-engine

The Engine that classifies, searches, and resolves Assets.

[`Classifier::classify`](crate::Classifier::classify) decides whether a file in
an Asset Folder is an Asset and of what Asset Kind, by running the
[`IndexingRule`](crate::IndexingRule)s it holds in order until one answers. The
one built-in rule makes files with an image extension image Assets; a further
rule is another `IndexingRule`, not a change to the Engine.

[`resolve`](crate::resolve) decides which Asset on this device an Asset
Reference means: the Asset Folder whose Canonical Name is the recorded one,
compared the way names are compared for uniqueness ([`same_name`](crate::same_name)),
then the one file at a recorded place, compared Unicode-normalised, or the one
file that differs from a recorded place only in letter case or Unicode
normalisation. Anything less certain is a Missing Asset with its reason; two
files that would each fit are never chosen between, even when one is spelled
byte for byte as recorded.

[`LibrarySearch::build`](crate::LibrarySearch::build) turns one Asset Folder's
Canonical Name and index into that folder's search, kept in the search of the
whole library, and [`LibrarySearch::search`](crate::LibrarySearch::search)
answers a text over every folder: the matching Assets in order and how many
match per folder. The text is split at whitespace into words; an Asset matches
when its name holds every word, and its path in the library (the folder's
Canonical Name, a `/`, and its place without the extension) every word that
holds a `/`, all compared Unicode-normalised and fully case-folded, on whole
letters. Assets in which every word begins a word of the name come first; within
each rank they are ordered by name, then by Canonical Name, then by place. A
text without words matches every Asset and lists none. Each folder's names and
paths are laid out folded in two contiguous texts, so a word is found by one
SIMD substring scan of each; the order of every folder's Assets taken together,
worked out at the first search after the folders change, lets a search merge
the folders' matches without comparing names. The names as spelled, which order
two folders' Assets whose names fold alike, are read from the folders' indexes
rather than kept.

Nothing here touches the `ECS`: the Engine is plain Rust over names, paths, and
kinds.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
