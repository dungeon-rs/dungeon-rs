# drs-model

The Model: the components of the `ECS` World that make up a Project, and the
messages the Editor and the Managers exchange.

A Project is an entity carrying [`Project`](crate::Project), its
[`Grid`](crate::Grid), its [`Bounds`](crate::Bounds), and its
[`AssetReferences`](crate::AssetReferences). Its Levels are its children, each
Level's Layers are that Level's children, and each Layer's Elements are that
Layer's children in stacking order: the first child is drawn first. An Element
is addressed by its [`ElementId`](crate::ElementId), never by its entity handle.

Each added Asset Folder is an entity carrying [`AssetFolder`](crate::AssetFolder)
with its index of Assets. [`EditorDirectories`](crate::EditorDirectories)
overrides where the editor keeps its own files, so tests point them at
temporary directories.

Every component type here is written only by the systems of the crate that owns
it; everyone else reads. The [`ModelPlugin`](crate::ModelPlugin) registers the
types for reflection, the messages, and the Element kind registry with Prop as
its first kind.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
