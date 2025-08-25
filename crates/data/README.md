# `DungeonRS data`

Core data structures shared throughout the application.
This crate is kept lightweight and dependency-free so other crates can use these types without pulling in Bevy or other heavy dependencies.

Contains the fundamental types:
- `Project` - Top-level marker component that defines the boundary of saveable map content
- `Element` - Individual map elements and components
- `Layer` - Map layer organisation and structure
- `Level` - Multi-floor/level support and metadata

The `Project` component serves as a crucial boundary marker in the ECS hierarchy.
Only entities beneath a `Project` component are considered during save/load and export operations, which prevents editor tools, gizmos, and temporary objects from being inadvertently included in persisted data.

To make working with these data structures easier, the `data` crate provides a `Query` implementation for each type.
The fundamental types and their corresponding `Query` implementations are:
- `Project` - `ProjectQuery`
- `Level` - `LevelQuery`
- `Layer` - `LayerQuery`
- `Element` - `ElementQuery`

There is also the [`DungeonQueries`] system parameter for fetching all the above types in a single query.