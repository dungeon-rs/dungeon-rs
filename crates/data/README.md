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
