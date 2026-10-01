# drs-catalog-engine

The Engine that classifies, searches, and resolves Assets.

[`Classifier::classify`](crate::Classifier::classify) decides whether a file in
an Asset Folder is an Asset and of what Asset Kind, by running the
[`IndexingRule`](crate::IndexingRule)s it holds in order until one answers. The
one built-in rule makes files with an image extension image Assets; further
rules are added to a `Classifier` without restructuring anything. Nothing here
touches the `ECS`: the Engine is plain Rust over paths and kinds.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
