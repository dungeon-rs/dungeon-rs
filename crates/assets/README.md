# DungeonRS assets

Handles management of the asset library (and containing asset packs), indexing, thumbnail generation and provides an
interface for searching and resolving assets from this library.

The primary feature is to abstract the exact location on the device away so that the same assets can be located on
different
devices, even if they are in different locations on each device.
Each asset is assigned a stable identifier that is the same on each device, independent for the specific location in the
file system, and maps refer to these assets by their assigned IDs. When resolving an asset, this crate is responsible
for
finding the actual location in the filesystem and providing it to Bevy's asset system.

## Features

| Feature | Description                                          | default |
|---------|------------------------------------------------------|:--------|
| jpeg    | Enables the crate to process and generate JPEG files | ❌       |
| png     | Enables the crate to process and generate PNG files  | ✅       |
| webp    | Enables the crate to process and generate WEBP files | ❌       |
