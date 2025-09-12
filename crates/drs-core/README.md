# `DungeonRS core`

Core functionality of the editor that isn't tied specifically to the UI.
This crate centralises shared functionality between the `ui` and `cli` crates, preventing code duplication and providing
a consistent event-driven architecture.

The core crate implements an event-driven system where:
- Events are exposed for triggering core functionality
- Event listeners handle the actual implementation
- Other crates can dispatch events to trigger functionality without duplicating logic

The core crate serves as the foundation for:
- Map manipulation and processing algorithms
- Asset management operations
- Core editing operations and transformations
- Event-driven business logic shared between UI and CLI implementations

By centralising this logic with an event-based approach, functionality can be easily shared between different frontends (UI, CLI, automation tools).