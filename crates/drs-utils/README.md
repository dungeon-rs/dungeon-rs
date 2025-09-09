# `DungeonRS utils`

Common utilities shared across the codebase.
This includes platform-specific directory resolution, async ECS helpers, path manipulation utilities, hashing functions, build version information, and re-exported macros from `utils_macros`.

The directory handling follows each platform's recommended conventions:
- Windows: Known Folder API
- Linux: XDG Base Directory specifications
- macOS: Standard Directories

This implementation is based on the approach used by the `directories` crate, though we maintain our own version due to licensing considerations.
