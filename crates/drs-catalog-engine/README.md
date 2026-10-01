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
then the file at a recorded place, or the one file that differs from a recorded
place only in letter case or Unicode normalisation. Anything less certain is a
Missing Asset with its reason; two files that would each fit are never chosen
between.

Nothing here touches the `ECS`: the Engine is plain Rust over names, paths, and
kinds.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
