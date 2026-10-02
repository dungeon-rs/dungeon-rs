# drs-model

The Model: the components of the `ECS` World that make up a Project, and the
messages the Editor and the Managers exchange.

A Project is an entity carrying [`Project`](crate::Project), its
[`Grid`](crate::Grid), its [`Bounds`](crate::Bounds), its
[`AssetReferences`](crate::AssetReferences), and the
[`ResolutionTable`](crate::ResolutionTable) that says where each Asset Reference
loads from on this device, or why it is a Missing Asset. Its Levels are its
children, each Level's Layers are that Level's children, and each Layer's
Elements are that Layer's children in stacking order: the first child is drawn
first. An Element is addressed by its [`ElementId`](crate::ElementId), never by
its entity handle.

A Prop carries [`Prop`](crate::Prop) beside its [`Element`](crate::Element); a
Wall carries [`Wall`](crate::Wall): its points in Grid cells, one
[`Segment`](crate::Segment) between each point and the next, straight or curved
by a control point, its thickness, and its [`Colour`](crate::Colour). The
segments are numbered from the first point on, and only adding or removing a
point renumbers them. A Wall's [`WallShape`](crate::WallShape) is derived from
it and never saved: its line flattened into chords, each point tagged with its
segment and the parameter along it, the [`Stretch`](crate::Stretch) of it each
Portal set into it covers, and the stroke it is drawn with, left out along those
stretches. The authoring Manager writes it; whoever draws or picks a Wall reads
it.

A Portal carries [`Portal`](crate::Portal): the Asset Reference row of its image,
its width in Grid cells, its rotation, whether it is mirrored, and, when it is
set into a Wall, its [`PortalAnchor`](crate::PortalAnchor): the identity of the
Element it is set into, the `index` of a part of it (a Wall's segment, a Room's
edge), a parameter along that part, and the [`Side`](crate::Side) it faces. The
anchor names what the Portal is set into as `host` and its part as `index`,
never by kind. A set Portal's position, rotation, and mirroring are kept
equal to what its anchor gives, so freeing it is clearing the anchor. Whoever
needs the Asset an Element shows, a Prop's or a Portal's, queries
[`ShownAsset`](crate::ShownAsset) and reads its row.

A [`ProjectSnapshot`](crate::ProjectSnapshot) holds every component of every
entity of the Project as an envelope of a version and data under a stable name,
in no file's shape: how a snapshot is laid out in a Project file is
`ProjectAccess`'s business. The
[`SerialisationRegistry`](crate::SerialisationRegistry) knows, for each
[`Serialisable`](crate::Serialisable) component, how to write the current version
and read every version it has had, and the [`Tier`](crate::Tier) of entity it
belongs on, so an envelope under another tier is refused as malformed; each
crate registers the components it owns when its plugin is built. An envelope no entry knows stays on its entity in
[`UnknownComponents`](crate::UnknownComponents) and is written back unchanged.
[`SavedMark`](crate::SavedMark) remembers the Project's file and where the
history stood at the last save or open, and
[`PROJECT_EXTENSION`](crate::PROJECT_EXTENSION) is the extension Project files
carry.

The [`Viewport`](crate::Viewport) is where the Author is looking: the cell at
the centre of the view, the zoom, and the area of the window the Level is shown
in. The Editor steers it and the render Engine follows it; its conversions
between cells and screen points are the ones picking and drawing share.

Each added Asset Folder is an entity carrying
[`AssetFolder`](crate::AssetFolder) with its index of Assets, and
[`Thumbnails`](crate::Thumbnails) with where each Asset's thumbnail stands:
pending, ready at its pixel size, or broken. A thumbnail is read through the
asset source [`THUMBNAIL_SOURCE`](crate::THUMBNAIL_SOURCE), as
`thumb://<folder-key>/<place>`; the Editor sends the text typed in its search
field and names the Assets it shows with [`Browse`](crate::Browse), the library
Manager answers the text in [`SearchMatches`](crate::SearchMatches), and
[`ThumbnailsUnavailable`](crate::ThumbnailsUnavailable) says thumbnails cannot
be kept. [`EditorDirectories`](crate::EditorDirectories) overrides where the
editor keeps its own files, so tests point them at temporary directories, and
resolves the platform's directories where nothing overrides them, so every crate
that writes the editor's own files asks it.
[`CaughtPanics`](crate::CaughtPanics) is how a background thread that catches
its own panics tells the crash handler so, which the Host puts in.

Every component type here is written only by the systems of the crate that owns
it; everyone else reads. The [`ModelPlugin`](crate::ModelPlugin) registers the
types for reflection, the messages, and the Element kind registry with Prop,
Wall, and Portal as its kinds, and orders the Managers' handling through
[`ManagerSystems`](crate::ManagerSystems): Commands before Undo before Redo.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
