# DungeonRS Editor

The `DungeonRS` editor is where most users will actually be working in.
It's the binary crate that actually handles bootstrapping the application,
setting up a graphical interface and run the application.

## Features

| Feature    | Description                                                 | default |
|------------|-------------------------------------------------------------|:--------|
| jpeg       | Enables the editor to process and generate JPEG files       | ❌       |
| png        | Enables the editor to process and generate PNG files        | ✅       |
| webp       | Enables the editor to process and generate WEBP files       | ❌       |
| no_console | Disables showing the console window (only works on Windows) | ❌       |
