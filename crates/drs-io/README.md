# `DungeonRS io`

Handles persistence operations for `DungeonRS` projects.
This crate manages the serialisation of ECS world state to disk and deserialization back into the running application using Bevy's event system for non-blocking operations.

Core functionality includes project loading from disk into ECS, project saving to files, and internal document representation for serialisation purposes.

The main events are:
- `LoadProjectEvent` - Loads a project from a specified file path
- `SaveProjectEvent` - Saves the current project state
- `SaveProjectCompleteEvent` - Signals completion of save operations

This crate integrates with the serialisation system to support multiple file formats and respects the `Project` component hierarchy to ensure only relevant entities are persisted, excluding editor tools and temporary objects.
