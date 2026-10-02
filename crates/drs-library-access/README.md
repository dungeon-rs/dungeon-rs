# drs-library-access

The `ResourceAccess` that reads Asset Folders and their Assets.

Its contract: the Manifests ([`read_manifests`](crate::read_manifests) to find
every folder at startup, [`write_manifest`](crate::write_manifest) to remember a
folder, and [`forget_manifest`](crate::forget_manifest) to drop one),
[`scan_folder`](crate::scan_folder), [`load_asset`](crate::load_asset), and the
Thumbnail: [`ThumbnailCache`](crate::ThumbnailCache),
[`ThumbnailGenerator`](crate::ThumbnailGenerator), and
[`ThumbnailTable::read`](crate::ThumbnailTable::read) for the bytes of the
thumbnail served for an Asset.

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

Thumbnails are kept in one append-only pack of encoded thumbnails and its index
of fixed 32-byte records, in a `thumbnails` directory under the cache directory,
shared by every folder. A record names its thumbnail by a digest of the Asset's
key (folder key, place, byte size, and modification time), so a changed file
misses and gets a new thumbnail; a record of length zero says the file could not
be decoded as an image. Opening the cache reads the index whole, skips and logs
a record that is incomplete or points beyond the pack, and replaces files it
cannot make sense of with empty ones. The generator decodes the file, takes its
first frame, fits it with a box filter into 128 pixels on its longer side
without ever enlarging it, and encodes it as PNG when any pixel is not fully
opaque and as JPEG otherwise, on threads of its own (half the cores, at least
one), serving the Assets last named as wanted before the rest; a file that
cannot be read is not recorded, so it is tried again at the next start, and a
panic while decoding a file is caught on the thread, which marks itself as
catching it through [`CaughtPanics`](drs_model::CaughtPanics), and the file is
recorded as broken. Writes are buffered and flushed when the queue drains, every
few hundred thumbnails or a tenth of a second after the oldest unwritten one,
and when the generator is dropped, which stops it after the thumbnails in
flight. Each flush syncs the pack before it writes the index, so no record
outlives its thumbnail, and the first write that fails is handed back once and
stops all writing.

Thumbnails are read through a second dynamic asset source,
`thumb://<folder-key>/<place>`, whose reader reads, out of a
[`ThumbnailTable`](crate::ThumbnailTable), the thumbnail that
[`ThumbnailCache::serve`](crate::ThumbnailCache::serve) or the generator last
put there for the Asset at its current size and modification time, and refuses
any path `lib://` would refuse. The Host registers it with
[`register_thumbnail_source`](crate::register_thumbnail_source) before Bevy's
`AssetPlugin` is added.

[`LibraryDirectories`](crate::LibraryDirectories) holds the configuration and
cache directories as the model's directory resource resolves them.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
