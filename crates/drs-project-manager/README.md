# drs-project-manager

The Manager that saves, opens, and exports Projects.

At startup it creates the new, unsaved Project the editor opens on, with one
Level named `Level 1` holding one Layer named `Layer 1`, a Grid of 256 pixels per
cell, and default Bounds.

Save ([`SaveProject`](drs_model::SaveProject)) gathers the whole Project from
the World through the model's serialisation registry and asks `ProjectAccess` to
write it, to the path given (Save As, with the Project extension added when it
lacks it) or to the file the Project was last saved to or opened from; a Project
that has no file yet is refused, so the Editor asks where. Open
([`OpenProject`](drs_model::OpenProject)) asks `ProjectAccess` to read the file
and, only once the whole Project has been materialised from it, replaces the
current Project, clears the history, and resolves every Asset Reference; a file
that cannot be read, is not a Project, is newer than this editor, or is
malformed is refused with the reason and nothing changes. Both record the file
and the history's position in the [`SavedMark`](drs_model::SavedMark), which
says whether the Project has unsaved changes. Answers are
[`ProjectSaved`](drs_model::ProjectSaved),
[`ProjectOpened`](drs_model::ProjectOpened) with its report of Missing Assets
and unknown Element kinds, and [`ProjectRefused`](drs_model::ProjectRefused).

Materialising a saved Project is lifecycle, not composing, so this Manager
writes every component of the Project here, through the registry; an Element of
a kind this editor does not know is spawned with its common component and its
unknown envelopes, and no kind component.

The Manager alone writes the Project's
[`ResolutionTable`](drs_model::ResolutionTable): it asks the catalog Engine to
resolve each Asset Reference against the Asset Folders in the World when a
Project is opened, when the Asset Reference table gains a row, and when the
library Manager announces [`AssetFolderChanged`](drs_model::AssetFolderChanged)
for a Canonical Name.

It also exports a Level: on an Export Level request it checks the resolution
against the limits the request names, opens a PNG through `OutputAccess` as many
pixels as the Bounds are cells times the resolution, has the render Engine draw
the Bounds tile by tile offscreen, writes each tile as its pixels come back, and
closes the image, answering with the image written or the reason it was
refused. An Export takes a few frames per tile and is advanced every frame; a
failure removes the partial file, and nothing is recorded in the history.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
