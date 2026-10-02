# drs-library-manager

The Manager that adds, indexes, and browses Asset Folders.

It handles the Add Asset Folder Command: an [`AddFolder`](drs_model::AddFolder)
message is checked (the folder is readable, not already added under any spelling
of its path, and neither inside nor around an added folder; the Canonical Name is
not blank and not already in use on this device, ignoring case and Unicode
normalisation), then its Manifest is written, the folder scanned, its files
classified, and an [`AssetFolder`](drs_model::AssetFolder) entity written into the
World, along with how many entries the scan skipped so the Editor can say so.
The step is recorded in the history: undoing it forgets the folder,
redoing it adds the folder again under the same name without asking. A refusal
goes back to the Editor as a [`FolderRefused`](drs_model::FolderRefused) with
its reason.

At startup every Manifest is read and each remembered folder is refreshed, so
Assets added to or removed from a folder while the editor was closed appear or
disappear. [`refresh`](crate::refresh) is part of the contract and rescans one
folder on demand.

Whenever a folder arrives or goes, at startup, on Add Asset Folder, and on its
undo and redo, the Manager announces Asset Folder Changed
([`AssetFolderChanged`](drs_model::AssetFolderChanged)) with the folder's
Canonical Name, so that the project Manager resolves again whatever a Project
records against that name.

Every folder indexed (on Add Asset Folder, its redo, and the refresh at startup)
has the thumbnail of each Asset looked up in the thumbnail cache, which is
opened at startup before the remembered folders are restored. The outcome is
written as [`Thumbnails`](drs_model::Thumbnails) on the folder's entity in the
index's order (pending, ready with its size, or broken), and the pending ones
are enqueued in place order for the background generator. Each frame, before any
Command is handled, what the generator finished is written into those states,
and the Assets the browser last named with [`Browse`](drs_model::Browse) that
are still pending go to the front of the queue. A folder that is undone has its
waiting Assets withdrawn and keeps its thumbnails, so a redo finds them again.
When the cache cannot be opened or written, or its generator cannot start, a
[`ThumbnailsUnavailable`](drs_model::ThumbnailsUnavailable) says so once and
nothing more is generated in this session; a cache that opened still serves the
thumbnails it holds. Generation stops as soon as the
editor is asked to quit, keeping what was finished.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
