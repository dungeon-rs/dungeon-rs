# `DungeonRS config`

Configuration management for the application.
This crate handles loading and validation of configuration files during early startup, before Bevy initialisation, allowing configuration of the entire bootstrap process.

The two main configuration types are:
- `Configuration` - Application settings including language preferences, recent files, and asset directory paths
- `LogConfiguration` - Logging system configuration with output levels and destinations

Configuration files are loaded from platform-appropriate directories and validated at startup.
