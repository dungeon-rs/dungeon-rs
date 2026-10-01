# drs-library-access

The `ResourceAccess` that reads Asset Folders and their Assets.

Its contract: the Manifests ([`read_manifests`](crate::read_manifests) to find
every folder at startup, [`write_manifest`](crate::write_manifest) to remember a
folder, and [`forget_manifest`](crate::forget_manifest) to drop one),
[`scan_folder`](crate::scan_folder), and [`load_asset`](crate::load_asset).

The Manifest of an Asset Folder lives in the editor's configuration directory
and its index cache in the editor's cache directory, both named by the folder
key; nothing is ever written into the folder itself. A scan reads directory
entries and file metadata only, skips hidden entries and symbolic links, counts
names that are not valid Unicode, and reports what changed since the cache was
written. Loading an Asset reads the file once for its byte size, its pixel size,
and its BLAKE3 fingerprint.

Assets are addressed through one dynamic asset source, `lib://<folder-key>/<place>`,
whose reader looks the key up in a [`LibraryTable`](crate::LibraryTable) that
changes at runtime. The Host registers it with
[`register_library_source`](crate::register_library_source) before Bevy's
`AssetPlugin` is added, because asset sources freeze when that plugin builds.

[`LibraryDirectories`](crate::LibraryDirectories) resolves the platform's
configuration and cache directories unless the model's directory resource
overrides them.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
